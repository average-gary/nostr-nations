//! Procedural map generation for deterministic world creation.
//!
//! The map generator uses a seed to create reproducible maps. This is critical
//! for the Nostr-based replay system - the same seed must always produce
//! the same map.

use crate::hex::HexCoord;
use crate::map::{Map, Tile};
use crate::terrain::{Feature, Resource, Terrain};
use crate::types::MapSize;

/// Configuration for map generation.
#[derive(Clone, Debug)]
pub struct MapGenConfig {
    /// Map size preset.
    pub size: MapSize,
    /// Percentage of map that should be water (0-100).
    pub water_percentage: u32,
    /// Number of players (affects starting position spacing).
    pub player_count: u8,
    /// Does the map wrap horizontally?
    pub wrap_x: bool,
}

impl Default for MapGenConfig {
    fn default() -> Self {
        Self {
            size: MapSize::Standard,
            water_percentage: 30,
            player_count: 2,
            wrap_x: false,
        }
    }
}

/// A deterministic random number generator using xorshift.
///
/// This simple PRNG ensures that the same seed always produces
/// the same sequence of random numbers across all platforms.
#[derive(Clone, Debug)]
pub struct SeededRng {
    state: u64,
}

impl SeededRng {
    /// Create a new RNG from a 32-byte seed.
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        // Combine seed bytes into initial state using a mixing function
        // to ensure different seeds produce different states
        let mut state: u64 = 0xcbf29ce484222325; // FNV offset basis
        for &byte in seed.iter() {
            state ^= byte as u64;
            state = state.wrapping_mul(0x100000001b3); // FNV prime
        }
        // Ensure non-zero state
        if state == 0 {
            state = 0x853c49e6748fea9b;
        }
        Self { state }
    }

    /// Generate next random u64.
    pub fn next_u64(&mut self) -> u64 {
        // xorshift64*
        self.state ^= self.state >> 12;
        self.state ^= self.state << 25;
        self.state ^= self.state >> 27;
        self.state.wrapping_mul(0x2545F4914F6CDD1D)
    }

    /// Generate a random u32.
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    /// Generate a random number in range [0, max).
    pub fn next_range(&mut self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        self.next_u32() % max
    }

    /// Generate a random float in range [0.0, 1.0).
    pub fn next_f32(&mut self) -> f32 {
        (self.next_u32() as f32) / (u32::MAX as f32)
    }

    /// Generate a boolean with given probability of true.
    pub fn chance(&mut self, probability: f32) -> bool {
        self.next_f32() < probability
    }
}

/// Generates game maps from a seed.
pub struct MapGenerator {
    rng: SeededRng,
    config: MapGenConfig,
}

impl MapGenerator {
    /// Create a new map generator with the given seed and config.
    pub fn new(seed: [u8; 32], config: MapGenConfig) -> Self {
        Self {
            rng: SeededRng::from_seed(&seed),
            config,
        }
    }

    /// Generate a complete map.
    pub fn generate(&mut self) -> Map {
        let (width, height) = self.config.size.dimensions();
        let mut map = Map::new(width, height, self.config.wrap_x);

        // Phase 1: Generate base terrain using heightmap
        self.generate_terrain(&mut map);

        // Phase 2: Add features (hills, forests, etc.)
        self.generate_features(&mut map);

        // Phase 3: Place resources
        self.place_resources(&mut map);

        // Phase 4: Add rivers
        self.generate_rivers(&mut map);

        map
    }

    /// Generate base terrain using a simple heightmap approach.
    fn generate_terrain(&mut self, map: &mut Map) {
        let (width, height) = (map.width, map.height);
        let water_threshold = self.config.water_percentage as f32 / 100.0;

        // Generate heightmap using multiple octaves of noise
        let heightmap = self.generate_heightmap(width, height);

        for q in 0..width as i32 {
            for r in 0..height as i32 {
                let coord = HexCoord::new(q, r);
                let height = heightmap[r as usize * width as usize + q as usize];

                let terrain = self.height_to_terrain(height, water_threshold, r, height as i32);
                map.set(Tile::new(coord, terrain));
            }
        }
    }

    /// Generate a simple heightmap using value noise.
    fn generate_heightmap(&mut self, width: u32, height: u32) -> Vec<f32> {
        let size = (width * height) as usize;
        let mut heightmap = vec![0.0f32; size];

        // Use multiple scales for more natural-looking terrain
        let scales = [16.0, 8.0, 4.0, 2.0];
        let weights = [0.5, 0.25, 0.15, 0.1];

        for (scale, weight) in scales.iter().zip(weights.iter()) {
            let grid_w = (width as f32 / scale).ceil() as usize + 1;
            let grid_h = (height as f32 / scale).ceil() as usize + 1;

            // Generate random grid values
            let mut grid = vec![0.0f32; grid_w * grid_h];
            for val in grid.iter_mut() {
                *val = self.rng.next_f32();
            }

            // Interpolate to full resolution
            for r in 0..height {
                for q in 0..width {
                    let x = q as f32 / scale;
                    let y = r as f32 / scale;

                    let x0 = x.floor() as usize;
                    let y0 = y.floor() as usize;
                    let x1 = (x0 + 1).min(grid_w - 1);
                    let y1 = (y0 + 1).min(grid_h - 1);

                    let fx = x - x0 as f32;
                    let fy = y - y0 as f32;

                    // Bilinear interpolation
                    let v00 = grid[y0 * grid_w + x0];
                    let v10 = grid[y0 * grid_w + x1];
                    let v01 = grid[y1 * grid_w + x0];
                    let v11 = grid[y1 * grid_w + x1];

                    let v0 = v00 * (1.0 - fx) + v10 * fx;
                    let v1 = v01 * (1.0 - fx) + v11 * fx;
                    let value = v0 * (1.0 - fy) + v1 * fy;

                    let idx = r as usize * width as usize + q as usize;
                    heightmap[idx] += value * weight;
                }
            }
        }

        // Apply latitude-based temperature gradient
        for r in 0..height {
            let latitude_factor = 1.0 - (((r as f32 / height as f32) - 0.5).abs() * 2.0);
            for q in 0..width {
                let idx = r as usize * width as usize + q as usize;
                // Blend height with latitude for temperature
                heightmap[idx] = heightmap[idx] * 0.7 + latitude_factor * 0.3;
            }
        }

        heightmap
    }

    /// Convert height value to terrain type.
    fn height_to_terrain(
        &self,
        height: f32,
        water_threshold: f32,
        row: i32,
        map_height: i32,
    ) -> Terrain {
        // Latitude affects temperature (polar regions get snow/tundra)
        let latitude = (row as f32 / map_height as f32 - 0.5).abs() * 2.0;
        let is_polar = latitude > 0.85;
        let is_cold = latitude > 0.7;

        if height < water_threshold * 0.7 {
            Terrain::Ocean
        } else if height < water_threshold {
            Terrain::Coast
        } else if is_polar {
            Terrain::Snow
        } else if is_cold {
            Terrain::Tundra
        } else if height > 0.8 {
            // High altitude = desert or plains depending on moisture
            if self.rng.clone().chance(0.4) {
                Terrain::Desert
            } else {
                Terrain::Plains
            }
        } else if height > 0.6 {
            Terrain::Plains
        } else {
            Terrain::Grassland
        }
    }

    /// Add features like hills, forests, jungles, etc.
    fn generate_features(&mut self, map: &mut Map) {
        let mut coords: Vec<HexCoord> = map.tiles.keys().cloned().collect();
        coords.sort(); // Ensure deterministic iteration order

        for coord in coords {
            if let Some(tile) = map.get(&coord).cloned() {
                if tile.terrain.is_water() {
                    continue;
                }

                let feature = self.select_feature(&tile);
                if let Some(f) = feature {
                    if let Some(tile_mut) = map.get_mut(&coord) {
                        tile_mut.feature = Some(f);
                    }
                }
            }
        }
    }

    /// Select a feature for a tile based on terrain and randomness.
    fn select_feature(&mut self, tile: &Tile) -> Option<Feature> {
        match tile.terrain {
            Terrain::Grassland => {
                if self.rng.chance(0.15) {
                    Some(Feature::Hills)
                } else if self.rng.chance(0.25) {
                    Some(Feature::Forest)
                } else if self.rng.chance(0.05) {
                    Some(Feature::Marsh)
                } else {
                    None
                }
            }
            Terrain::Plains => {
                if self.rng.chance(0.20) {
                    Some(Feature::Hills)
                } else if self.rng.chance(0.15) {
                    Some(Feature::Forest)
                } else {
                    None
                }
            }
            Terrain::Desert => {
                if self.rng.chance(0.15) {
                    Some(Feature::Hills)
                } else if self.rng.chance(0.03) {
                    Some(Feature::Oasis)
                } else if self.rng.chance(0.08) {
                    Some(Feature::FloodPlains)
                } else {
                    None
                }
            }
            Terrain::Tundra => {
                if self.rng.chance(0.20) {
                    Some(Feature::Hills)
                } else if self.rng.chance(0.10) {
                    Some(Feature::Forest)
                } else {
                    None
                }
            }
            Terrain::Snow => {
                if self.rng.chance(0.10) {
                    Some(Feature::Hills)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Place resources on the map.
    fn place_resources(&mut self, map: &mut Map) {
        let mut coords: Vec<HexCoord> = map.tiles.keys().cloned().collect();
        coords.sort(); // Ensure deterministic iteration order

        for coord in coords {
            if let Some(tile) = map.get(&coord).cloned() {
                let resource = self.select_resource(&tile);
                if let Some(r) = resource {
                    if let Some(tile_mut) = map.get_mut(&coord) {
                        tile_mut.resource = Some(r);
                    }
                }
            }
        }
    }

    /// Select a resource for a tile based on terrain and features.
    fn select_resource(&mut self, tile: &Tile) -> Option<Resource> {
        // Lower probability for more balanced distribution
        let base_chance = 0.08;

        if !self.rng.chance(base_chance) {
            return None;
        }

        match tile.terrain {
            Terrain::Grassland => {
                if tile.feature == Some(Feature::Forest) {
                    Some(Resource::Deer)
                } else if self.rng.chance(0.5) {
                    Some(Resource::Wheat)
                } else if self.rng.chance(0.5) {
                    Some(Resource::Cattle)
                } else {
                    Some(Resource::Horses)
                }
            }
            Terrain::Plains => {
                if tile.feature == Some(Feature::Hills) {
                    if self.rng.chance(0.5) {
                        Some(Resource::Iron)
                    } else {
                        Some(Resource::Copper)
                    }
                } else if self.rng.chance(0.5) {
                    Some(Resource::Wheat)
                } else {
                    Some(Resource::Horses)
                }
            }
            Terrain::Desert => {
                if tile.feature == Some(Feature::Oasis) {
                    None // Oasis is already valuable
                } else if tile.feature == Some(Feature::Hills) {
                    if self.rng.chance(0.5) {
                        Some(Resource::Gold)
                    } else {
                        Some(Resource::Silver)
                    }
                } else if self.rng.chance(0.3) {
                    Some(Resource::Oil)
                } else {
                    Some(Resource::Incense)
                }
            }
            Terrain::Tundra => {
                if self.rng.chance(0.5) {
                    Some(Resource::Furs)
                } else if self.rng.chance(0.5) {
                    Some(Resource::Deer)
                } else {
                    Some(Resource::Silver)
                }
            }
            Terrain::Snow => {
                if self.rng.chance(0.3) {
                    Some(Resource::Oil)
                } else {
                    Some(Resource::Uranium)
                }
            }
            Terrain::Coast => {
                if self.rng.chance(0.5) {
                    Some(Resource::Fish)
                } else if self.rng.chance(0.5) {
                    Some(Resource::Crabs)
                } else {
                    Some(Resource::Pearls)
                }
            }
            Terrain::Ocean => {
                if self.rng.chance(0.3) {
                    Some(Resource::Fish)
                } else if self.rng.chance(0.5) {
                    Some(Resource::Whales)
                } else {
                    None
                }
            }
        }
    }

    /// Generate rivers on the map.
    fn generate_rivers(&mut self, map: &mut Map) {
        let (width, height) = (map.width, map.height);
        let river_count = (width * height / 200) as usize;

        for _ in 0..river_count {
            // Start rivers from hills/mountains
            let start_q = self.rng.next_range(width) as i32;
            let start_r = self.rng.next_range(height) as i32;
            let start = HexCoord::new(start_q, start_r);

            if let Some(tile) = map.get(&start) {
                if tile.feature == Some(Feature::Hills) && !tile.terrain.is_water() {
                    self.trace_river(map, start);
                }
            }
        }
    }

    /// Trace a river from a starting point to water.
    fn trace_river(&mut self, map: &mut Map, start: HexCoord) {
        let mut current = start;
        let max_length = 20;

        for _ in 0..max_length {
            // Find neighbor closest to water or lowest elevation
            let neighbors = map.neighbors(&current);
            let mut best_neighbor = None;
            let mut found_water = false;

            for neighbor in &neighbors {
                if let Some(tile) = map.get(neighbor) {
                    if tile.terrain.is_water() {
                        found_water = true;
                        best_neighbor = Some(*neighbor);
                        break;
                    }
                    if best_neighbor.is_none() {
                        best_neighbor = Some(*neighbor);
                    }
                }
            }

            if let Some(next) = best_neighbor {
                // Add river edge between current and next
                let edge = self.find_edge_index(&current, &next);
                if let Some(tile) = map.get_mut(&current) {
                    if let Some(idx) = edge {
                        tile.river_edges[idx] = true;
                    }
                }

                if found_water {
                    break;
                }
                current = next;
            } else {
                break;
            }
        }
    }

    /// Find which edge index connects two adjacent hexes.
    fn find_edge_index(&self, from: &HexCoord, to: &HexCoord) -> Option<usize> {
        let neighbors = from.neighbors();
        for (i, n) in neighbors.iter().enumerate() {
            if n == to {
                return Some(i);
            }
        }
        None
    }

    /// Find suitable starting positions for players.
    pub fn find_starting_positions(&mut self, map: &Map) -> Vec<HexCoord> {
        let mut positions = Vec::new();
        let player_count = self.config.player_count as usize;

        // Minimum distance between starting positions
        let min_distance = map.width.min(map.height) / (player_count as u32 + 1);

        // Collect all valid starting tiles (land, not mountains, good yields)
        let mut candidates: Vec<(HexCoord, u32)> = Vec::new();

        for (coord, tile) in map.iter() {
            if self.is_valid_start(tile, map) {
                let score = self.rate_start_position(coord, map);
                candidates.push((*coord, score));
            }
        }

        // Sort by score (best first)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // Select positions ensuring minimum distance
        for (coord, _score) in candidates {
            let far_enough = positions
                .iter()
                .all(|p: &HexCoord| coord.distance(p) >= min_distance);

            if far_enough {
                positions.push(coord);
                if positions.len() >= player_count {
                    break;
                }
            }
        }

        positions
    }

    /// Check if a tile is valid for starting position.
    fn is_valid_start(&self, tile: &Tile, _map: &Map) -> bool {
        if tile.terrain.is_water() {
            return false;
        }
        if tile.feature == Some(Feature::Mountains) || tile.feature == Some(Feature::Ice) {
            return false;
        }
        true
    }

    /// Rate a starting position (higher = better).
    fn rate_start_position(&self, coord: &HexCoord, map: &Map) -> u32 {
        let mut score = 0u32;

        // Check tiles in radius 2
        for neighbor_coord in coord.hexes_in_radius(2) {
            if let Some(tile) = map.get(&neighbor_coord) {
                // Prefer land tiles
                if !tile.terrain.is_water() {
                    score += 2;
                }

                // Value good yields
                let yields = tile.yields();
                score += yields.food as u32 * 3;
                score += yields.production as u32 * 2;
                score += yields.gold as u32;

                // Bonus for resources
                if tile.resource.is_some() {
                    score += 5;
                }

                // Bonus for fresh water
                if tile.has_fresh_water() {
                    score += 10;
                }

                // Penalty for mountains (can't work)
                if tile.feature == Some(Feature::Mountains) {
                    score = score.saturating_sub(3);
                }
            }
        }

        score
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seeded_rng_determinism() {
        let seed = [42u8; 32];
        let mut rng1 = SeededRng::from_seed(&seed);
        let mut rng2 = SeededRng::from_seed(&seed);

        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_seeded_rng_different_seeds() {
        let mut rng1 = SeededRng::from_seed(&[1u8; 32]);
        let mut rng2 = SeededRng::from_seed(&[2u8; 32]);

        // Different seeds should produce different sequences
        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_map_generation_determinism() {
        let seed = [123u8; 32];
        let config = MapGenConfig {
            size: MapSize::Duel,
            water_percentage: 30,
            player_count: 2,
            wrap_x: false,
        };

        let mut gen1 = MapGenerator::new(seed, config.clone());
        let mut gen2 = MapGenerator::new(seed, config);

        let map1 = gen1.generate();
        let map2 = gen2.generate();

        // Same seed should produce identical maps
        assert_eq!(map1.tile_count(), map2.tile_count());

        for (coord, tile1) in map1.iter() {
            let tile2 = map2.get(coord).unwrap();
            assert_eq!(tile1.terrain, tile2.terrain);
            assert_eq!(tile1.feature, tile2.feature);
            assert_eq!(tile1.resource, tile2.resource);
        }
    }

    #[test]
    fn test_map_has_land_and_water() {
        let seed = [0u8; 32];
        let config = MapGenConfig {
            size: MapSize::Duel,
            water_percentage: 30,
            player_count: 2,
            wrap_x: false,
        };

        let mut gen = MapGenerator::new(seed, config);
        let map = gen.generate();

        let land_count = map.iter().filter(|(_, t)| !t.terrain.is_water()).count();
        let water_count = map.iter().filter(|(_, t)| t.terrain.is_water()).count();

        assert!(land_count > 0, "Map should have land");
        assert!(water_count > 0, "Map should have water");
    }

    #[test]
    fn test_starting_positions() {
        let seed = [99u8; 32];
        let config = MapGenConfig {
            size: MapSize::Small,
            water_percentage: 30,
            player_count: 4,
            wrap_x: false,
        };

        let mut gen = MapGenerator::new(seed, config.clone());
        let map = gen.generate();
        let positions = gen.find_starting_positions(&map);

        assert_eq!(positions.len(), config.player_count as usize);

        // All positions should be on land
        for pos in &positions {
            let tile = map.get(pos).unwrap();
            assert!(!tile.terrain.is_water());
        }
    }

    #[test]
    fn test_map_has_resources() {
        let seed = [7u8; 32];
        let config = MapGenConfig::default();

        let mut gen = MapGenerator::new(seed, config);
        let map = gen.generate();

        let resource_count = map.iter().filter(|(_, t)| t.resource.is_some()).count();
        assert!(resource_count > 0, "Map should have resources");
    }

    #[test]
    fn test_map_has_features() {
        let seed = [13u8; 32];
        let config = MapGenConfig::default();

        let mut gen = MapGenerator::new(seed, config);
        let map = gen.generate();

        let feature_count = map.iter().filter(|(_, t)| t.feature.is_some()).count();
        assert!(feature_count > 0, "Map should have features");
    }
}

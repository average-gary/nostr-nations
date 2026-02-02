//! Game map structure with tiles and spatial queries.

use crate::hex::HexCoord;
use crate::terrain::{Feature, Improvement, Resource, Road, Terrain};
use crate::types::{CityId, PlayerId};
use crate::yields::Yields;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The game map containing all tiles.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    /// Map width in tiles.
    pub width: u32,
    /// Map height in tiles.
    pub height: u32,
    /// All tiles indexed by coordinate.
    pub tiles: HashMap<HexCoord, Tile>,
    /// Does the map wrap horizontally?
    pub wrap_x: bool,
}

impl Map {
    /// Create a new empty map with the given dimensions.
    pub fn new(width: u32, height: u32, wrap_x: bool) -> Self {
        Self {
            width,
            height,
            tiles: HashMap::new(),
            wrap_x,
        }
    }

    /// Create a map filled with a single terrain type (useful for testing).
    pub fn filled(width: u32, height: u32, terrain: Terrain) -> Self {
        let mut map = Self::new(width, height, false);
        for q in 0..width as i32 {
            for r in 0..height as i32 {
                let coord = HexCoord::new(q, r);
                map.tiles.insert(coord, Tile::new(coord, terrain));
            }
        }
        map
    }

    /// Get a tile at the given coordinate.
    pub fn get(&self, coord: &HexCoord) -> Option<&Tile> {
        let wrapped = self.wrap_coord(coord);
        self.tiles.get(&wrapped)
    }

    /// Get a mutable reference to a tile.
    pub fn get_mut(&mut self, coord: &HexCoord) -> Option<&mut Tile> {
        let wrapped = self.wrap_coord(coord);
        self.tiles.get_mut(&wrapped)
    }

    /// Insert or replace a tile.
    pub fn set(&mut self, tile: Tile) {
        let wrapped = self.wrap_coord(&tile.coord);
        let mut tile = tile;
        tile.coord = wrapped;
        self.tiles.insert(wrapped, tile);
    }

    /// Check if a coordinate is within the map bounds.
    pub fn in_bounds(&self, coord: &HexCoord) -> bool {
        if self.wrap_x {
            coord.r >= 0 && (coord.r as u32) < self.height
        } else {
            coord.q >= 0
                && coord.r >= 0
                && (coord.q as u32) < self.width
                && (coord.r as u32) < self.height
        }
    }

    /// Wrap a coordinate if map wraps horizontally.
    pub fn wrap_coord(&self, coord: &HexCoord) -> HexCoord {
        if self.wrap_x {
            let q = ((coord.q % self.width as i32) + self.width as i32) % self.width as i32;
            HexCoord::new(q, coord.r)
        } else {
            *coord
        }
    }

    /// Get valid neighbors of a hex (respecting map boundaries).
    pub fn neighbors(&self, coord: &HexCoord) -> Vec<HexCoord> {
        coord
            .neighbors()
            .into_iter()
            .map(|c| self.wrap_coord(&c))
            .filter(|c| self.in_bounds(c))
            .collect()
    }

    /// Get all tiles within a radius of a point.
    pub fn tiles_in_radius(&self, center: &HexCoord, radius: u32) -> Vec<&Tile> {
        center
            .hexes_in_radius(radius)
            .into_iter()
            .filter_map(|c| self.get(&c))
            .collect()
    }

    /// Count total tiles in the map.
    pub fn tile_count(&self) -> usize {
        self.tiles.len()
    }

    /// Iterate over all tiles.
    pub fn iter(&self) -> impl Iterator<Item = (&HexCoord, &Tile)> {
        self.tiles.iter()
    }

    /// Iterate over all tiles mutably.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (&HexCoord, &mut Tile)> {
        self.tiles.iter_mut()
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new(80, 50, false)
    }
}

/// A single tile on the map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    /// Position on the map.
    pub coord: HexCoord,
    /// Base terrain type.
    pub terrain: Terrain,
    /// Optional feature overlay (hills, forest, etc.).
    pub feature: Option<Feature>,
    /// Optional resource on this tile.
    pub resource: Option<Resource>,
    /// Built improvement (farm, mine, etc.).
    pub improvement: Option<Improvement>,
    /// Road/railroad on this tile.
    pub road: Option<Road>,
    /// Player who owns this tile (via city borders).
    pub owner: Option<PlayerId>,
    /// City that owns this tile.
    pub city_id: Option<CityId>,
    /// Which edges have rivers (6 edges, clockwise from NE).
    pub river_edges: [bool; 6],
}

impl Tile {
    /// Create a new tile with just terrain.
    pub fn new(coord: HexCoord, terrain: Terrain) -> Self {
        Self {
            coord,
            terrain,
            feature: None,
            resource: None,
            improvement: None,
            road: None,
            owner: None,
            city_id: None,
            river_edges: [false; 6],
        }
    }

    /// Calculate the total yields from this tile.
    ///
    /// Combines base terrain yields with feature modifiers, resources, and improvements.
    pub fn yields(&self) -> Yields {
        let mut y = self.terrain.base_yields();

        if let Some(feature) = &self.feature {
            y += feature.yield_modifier();
        }

        if let Some(improvement) = &self.improvement {
            y += improvement.yield_bonus();
        }

        // Resources only provide bonus if improved
        if self.resource.is_some() && self.improvement.is_some() {
            if let Some(resource) = &self.resource {
                y += resource.yield_bonus();
            }
        }

        y.clamp_non_negative()
    }

    /// Get the movement cost to enter this tile.
    pub fn movement_cost(&self) -> u32 {
        // Feature cost takes precedence (impassable mountains, etc.)
        if let Some(feature) = &self.feature {
            let cost = feature.movement_cost();
            if cost == u32::MAX {
                return u32::MAX; // Impassable
            }
            // Road reduces cost
            if let Some(road) = &self.road {
                let multiplier = road.movement_multiplier();
                return ((cost as f32) * multiplier).ceil() as u32;
            }
            return cost;
        }

        let base = self.terrain.movement_cost();

        // Road reduces cost
        if let Some(road) = &self.road {
            let multiplier = road.movement_multiplier();
            return ((base as f32) * multiplier).ceil() as u32;
        }

        base
    }

    /// Get the defense bonus for units on this tile.
    pub fn defense_bonus(&self) -> i32 {
        self.feature.map_or(0, |f| f.defense_bonus())
    }

    /// Check if a city can be founded on this tile.
    pub fn can_found_city(&self) -> bool {
        if !self.terrain.can_found_city() {
            return false;
        }
        if let Some(feature) = &self.feature {
            if feature.blocks_city() {
                return false;
            }
        }
        // Can't found on already-owned tile with a city
        self.city_id.is_none()
    }

    /// Check if this tile is passable by land units.
    pub fn is_passable_land(&self) -> bool {
        if self.terrain.is_water() {
            return false;
        }
        self.movement_cost() != u32::MAX
    }

    /// Check if this tile is passable by naval units.
    pub fn is_passable_naval(&self) -> bool {
        self.terrain.is_water()
    }

    /// Check if this tile has a river on any edge.
    pub fn has_river(&self) -> bool {
        self.river_edges.iter().any(|&e| e)
    }

    /// Get fresh water bonus (river or lake adjacent).
    pub fn has_fresh_water(&self) -> bool {
        self.has_river()
    }
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(HexCoord::default(), Terrain::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_map_creation() {
        let map = Map::new(40, 25, false);
        assert_eq!(map.width, 40);
        assert_eq!(map.height, 25);
        assert!(!map.wrap_x);
    }

    #[test]
    fn test_map_filled() {
        let map = Map::filled(10, 10, Terrain::Grassland);
        assert_eq!(map.tile_count(), 100);

        let tile = map.get(&HexCoord::new(5, 5)).unwrap();
        assert_eq!(tile.terrain, Terrain::Grassland);
    }

    #[test]
    fn test_tile_yields() {
        let mut tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        assert_eq!(tile.yields().food, 2);

        tile.feature = Some(Feature::Hills);
        let y = tile.yields();
        assert_eq!(y.food, 2);
        assert_eq!(y.production, 1);
    }

    #[test]
    fn test_tile_movement_cost() {
        let mut tile = Tile::new(HexCoord::new(0, 0), Terrain::Plains);
        assert_eq!(tile.movement_cost(), 1);

        tile.feature = Some(Feature::Forest);
        assert_eq!(tile.movement_cost(), 2);

        tile.road = Some(Road::Road);
        assert_eq!(tile.movement_cost(), 1); // 2 * 0.5 = 1
    }

    #[test]
    fn test_tile_impassable() {
        let mut tile = Tile::new(HexCoord::new(0, 0), Terrain::Plains);
        tile.feature = Some(Feature::Mountains);
        assert_eq!(tile.movement_cost(), u32::MAX);
        assert!(!tile.is_passable_land());
    }

    #[test]
    fn test_can_found_city() {
        let land = Tile::new(HexCoord::new(0, 0), Terrain::Plains);
        assert!(land.can_found_city());

        let water = Tile::new(HexCoord::new(0, 0), Terrain::Ocean);
        assert!(!water.can_found_city());

        let mut mountain = Tile::new(HexCoord::new(0, 0), Terrain::Plains);
        mountain.feature = Some(Feature::Mountains);
        assert!(!mountain.can_found_city());
    }

    #[test]
    fn test_map_wrap() {
        let map = Map::new(10, 10, true);
        let coord = HexCoord::new(-1, 5);
        let wrapped = map.wrap_coord(&coord);
        assert_eq!(wrapped.q, 9);
        assert_eq!(wrapped.r, 5);
    }

    #[test]
    fn test_map_neighbors() {
        let map = Map::filled(10, 10, Terrain::Plains);
        let neighbors = map.neighbors(&HexCoord::new(5, 5));
        assert_eq!(neighbors.len(), 6);

        // Corner should have fewer neighbors
        let corner_neighbors = map.neighbors(&HexCoord::new(0, 0));
        assert!(corner_neighbors.len() < 6);
    }

    // =========================================================================
    // Visibility-related unit tests
    // =========================================================================

    #[test]
    fn test_tiles_in_radius_center_tile() {
        let map = Map::filled(20, 20, Terrain::Grassland);
        let center = HexCoord::new(10, 10);

        // Radius 0 should return just the center tile
        let radius_0 = map.tiles_in_radius(&center, 0);
        assert_eq!(radius_0.len(), 1);
        assert_eq!(radius_0[0].coord, center);
    }

    #[test]
    fn test_tiles_in_radius_one() {
        let map = Map::filled(20, 20, Terrain::Grassland);
        let center = HexCoord::new(10, 10);

        // Radius 1 should return center + 6 neighbors = 7 tiles
        let radius_1 = map.tiles_in_radius(&center, 1);
        assert_eq!(radius_1.len(), 7);

        // Should include center
        assert!(radius_1.iter().any(|t| t.coord == center));

        // Should include all neighbors
        for neighbor in center.neighbors() {
            assert!(
                radius_1.iter().any(|t| t.coord == neighbor),
                "Neighbor {:?} should be in radius 1",
                neighbor
            );
        }
    }

    #[test]
    fn test_tiles_in_radius_two() {
        let map = Map::filled(20, 20, Terrain::Grassland);
        let center = HexCoord::new(10, 10);

        // Radius 2 should return more tiles
        let radius_2 = map.tiles_in_radius(&center, 2);

        // Should have more tiles than radius 1
        let radius_1 = map.tiles_in_radius(&center, 1);
        assert!(radius_2.len() > radius_1.len());

        // All tiles should be within distance 2
        for tile in &radius_2 {
            assert!(
                center.distance(&tile.coord) <= 2,
                "Tile {:?} should be within distance 2",
                tile.coord
            );
        }
    }

    #[test]
    fn test_tiles_in_radius_at_edge() {
        let map = Map::filled(10, 10, Terrain::Grassland);

        // Corner position
        let corner = HexCoord::new(0, 0);
        let radius_2 = map.tiles_in_radius(&corner, 2);

        // Should have fewer tiles than center due to map boundaries
        let center = HexCoord::new(5, 5);
        let center_radius_2 = map.tiles_in_radius(&center, 2);

        assert!(
            radius_2.len() < center_radius_2.len(),
            "Edge should have fewer tiles ({}) than center ({})",
            radius_2.len(),
            center_radius_2.len()
        );
    }

    #[test]
    fn test_tiles_in_radius_respects_bounds() {
        let map = Map::filled(10, 10, Terrain::Grassland);
        let edge = HexCoord::new(0, 5);

        let radius_3 = map.tiles_in_radius(&edge, 3);

        // All returned tiles should be in bounds
        for tile in &radius_3 {
            assert!(
                map.in_bounds(&tile.coord),
                "Tile {:?} should be in bounds",
                tile.coord
            );
        }
    }

    #[test]
    fn test_tile_visibility_blocking_features() {
        // Mountains should block passage
        let mut mountain_tile = Tile::new(HexCoord::new(0, 0), Terrain::Plains);
        mountain_tile.feature = Some(Feature::Mountains);
        assert_eq!(mountain_tile.movement_cost(), u32::MAX);
        assert!(!mountain_tile.is_passable_land());

        // Hills don't block passage but affect movement
        let mut hill_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        hill_tile.feature = Some(Feature::Hills);
        assert_eq!(hill_tile.movement_cost(), 2);
        assert!(hill_tile.is_passable_land());

        // Forest slows movement
        let mut forest_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        forest_tile.feature = Some(Feature::Forest);
        assert_eq!(forest_tile.movement_cost(), 2);
        assert!(forest_tile.is_passable_land());

        // Jungle is dense and slow
        let mut jungle_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        jungle_tile.feature = Some(Feature::Jungle);
        assert_eq!(jungle_tile.movement_cost(), 2);
        assert!(jungle_tile.is_passable_land());
    }

    #[test]
    fn test_tile_defense_bonus_for_visibility() {
        // Hills provide defense bonus (+25%)
        let mut hill_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        hill_tile.feature = Some(Feature::Hills);
        assert_eq!(hill_tile.defense_bonus(), 25);

        // Forest provides defense bonus (+25%)
        let mut forest_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        forest_tile.feature = Some(Feature::Forest);
        assert_eq!(forest_tile.defense_bonus(), 25);

        // Jungle provides defense bonus (+25%)
        let mut jungle_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        jungle_tile.feature = Some(Feature::Jungle);
        assert_eq!(jungle_tile.defense_bonus(), 25);

        // Marsh provides defense penalty (-10%)
        let mut marsh_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        marsh_tile.feature = Some(Feature::Marsh);
        assert_eq!(marsh_tile.defense_bonus(), -10);

        // Open terrain has no bonus
        let open_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        assert_eq!(open_tile.defense_bonus(), 0);
    }

    #[test]
    fn test_water_tiles_for_naval_visibility() {
        // Coast is water
        let coast_tile = Tile::new(HexCoord::new(0, 0), Terrain::Coast);
        assert!(coast_tile.terrain.is_water());
        assert!(coast_tile.is_passable_naval());
        assert!(!coast_tile.is_passable_land());

        // Ocean is water
        let ocean_tile = Tile::new(HexCoord::new(0, 0), Terrain::Ocean);
        assert!(ocean_tile.terrain.is_water());
        assert!(ocean_tile.is_passable_naval());
        assert!(!ocean_tile.is_passable_land());

        // Land is not water
        let land_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        assert!(!land_tile.terrain.is_water());
        assert!(!land_tile.is_passable_naval());
        assert!(land_tile.is_passable_land());
    }

    #[test]
    fn test_map_wrap_affects_visibility_calculation() {
        let wrap_map = Map::new(10, 10, true);
        let no_wrap_map = Map::new(10, 10, false);

        // Coordinate near edge
        let edge_coord = HexCoord::new(9, 5);

        // In wrapping map, neighbor at x=10 wraps to x=0
        let wrapped_neighbors = wrap_map.neighbors(&edge_coord);

        // In non-wrapping map, neighbor at x=10 is out of bounds
        let unwrapped_neighbors = no_wrap_map.neighbors(&edge_coord);

        // Wrapping map may have more valid neighbors at edges
        // (depends on specific hex geometry)
        assert!(!wrapped_neighbors.is_empty());
        assert!(!unwrapped_neighbors.is_empty());
    }

    #[test]
    fn test_tile_ownership_affects_visibility_context() {
        let mut tile = Tile::new(HexCoord::new(5, 5), Terrain::Grassland);

        // Initially no owner
        assert!(tile.owner.is_none());
        assert!(tile.city_id.is_none());

        // Set owner
        tile.owner = Some(0);
        tile.city_id = Some(1);

        assert_eq!(tile.owner, Some(0));
        assert_eq!(tile.city_id, Some(1));
    }

    #[test]
    fn test_large_radius_visibility() {
        let map = Map::filled(50, 50, Terrain::Grassland);
        let center = HexCoord::new(25, 25);

        // Large radius (like from a high vantage point)
        let large_radius = map.tiles_in_radius(&center, 5);

        // Should have many tiles
        assert!(large_radius.len() > 50);

        // All should be valid
        for tile in &large_radius {
            assert!(map.in_bounds(&tile.coord));
        }
    }

    #[test]
    fn test_river_affects_tile_properties() {
        let mut tile = Tile::new(HexCoord::new(5, 5), Terrain::Grassland);

        // No river initially
        assert!(!tile.has_river());
        assert!(!tile.has_fresh_water());

        // Add river edge
        tile.river_edges[0] = true;

        assert!(tile.has_river());
        assert!(tile.has_fresh_water());
    }
}

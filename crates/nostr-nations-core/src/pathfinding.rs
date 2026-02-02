//! A* pathfinding on hex grids.
//!
//! This module provides efficient pathfinding for units on the game map,
//! taking into account terrain costs, unit type restrictions, and fog of war.

use crate::hex::HexCoord;
use crate::map::Map;
use crate::unit::UnitCategory;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

/// Result of a pathfinding operation.
#[derive(Clone, Debug)]
pub struct PathResult {
    /// The path from start to goal (inclusive).
    pub path: Vec<HexCoord>,
    /// Total movement cost of the path.
    pub total_cost: u32,
}

/// Configuration for pathfinding.
#[derive(Clone, Debug)]
pub struct PathConfig {
    /// Maximum movement points available.
    pub max_movement: u32,
    /// Unit category (affects terrain passability).
    pub unit_category: UnitCategory,
    /// Is the unit embarked?
    pub embarked: bool,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self {
            max_movement: 20, // 2 movement points * 10
            unit_category: UnitCategory::Melee,
            embarked: false,
        }
    }
}

/// Node in the A* priority queue.
#[derive(Clone, Eq, PartialEq)]
struct PathNode {
    coord: HexCoord,
    g_cost: u32, // Cost from start
    f_cost: u32, // g_cost + heuristic
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse order for min-heap (lowest f_cost first)
        other
            .f_cost
            .cmp(&self.f_cost)
            .then_with(|| other.g_cost.cmp(&self.g_cost))
    }
}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Find the shortest path between two hexes using A*.
///
/// Returns None if no valid path exists.
pub fn find_path(
    map: &Map,
    start: HexCoord,
    goal: HexCoord,
    config: &PathConfig,
) -> Option<PathResult> {
    if start == goal {
        return Some(PathResult {
            path: vec![start],
            total_cost: 0,
        });
    }

    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<HexCoord, HexCoord> = HashMap::new();
    let mut g_scores: HashMap<HexCoord, u32> = HashMap::new();

    g_scores.insert(start, 0);
    open_set.push(PathNode {
        coord: start,
        g_cost: 0,
        f_cost: heuristic(&start, &goal),
    });

    while let Some(current) = open_set.pop() {
        if current.coord == goal {
            // Reconstruct path
            let path = reconstruct_path(&came_from, goal, start);
            return Some(PathResult {
                path,
                total_cost: current.g_cost,
            });
        }

        let current_g = *g_scores.get(&current.coord).unwrap_or(&u32::MAX);

        for neighbor in map.neighbors(&current.coord) {
            let move_cost = get_movement_cost(map, &neighbor, config);

            // Skip impassable tiles
            if move_cost == u32::MAX {
                continue;
            }

            let tentative_g = current_g.saturating_add(move_cost);

            // Skip if we've found a better path already
            if tentative_g >= *g_scores.get(&neighbor).unwrap_or(&u32::MAX) {
                continue;
            }

            came_from.insert(neighbor, current.coord);
            g_scores.insert(neighbor, tentative_g);

            let f_cost = tentative_g + heuristic(&neighbor, &goal);
            open_set.push(PathNode {
                coord: neighbor,
                g_cost: tentative_g,
                f_cost,
            });
        }
    }

    None // No path found
}

/// Find all tiles reachable within a movement budget.
///
/// Returns a map of coordinates to their movement cost from the start.
pub fn find_reachable(map: &Map, start: HexCoord, config: &PathConfig) -> HashMap<HexCoord, u32> {
    let mut reachable: HashMap<HexCoord, u32> = HashMap::new();
    let mut frontier: BinaryHeap<PathNode> = BinaryHeap::new();

    reachable.insert(start, 0);
    frontier.push(PathNode {
        coord: start,
        g_cost: 0,
        f_cost: 0,
    });

    while let Some(current) = frontier.pop() {
        let current_cost = *reachable.get(&current.coord).unwrap_or(&u32::MAX);

        for neighbor in map.neighbors(&current.coord) {
            let move_cost = get_movement_cost(map, &neighbor, config);

            if move_cost == u32::MAX {
                continue;
            }

            let total_cost = current_cost.saturating_add(move_cost);

            // Check if within movement budget
            if total_cost > config.max_movement {
                continue;
            }

            // Check if we found a better path
            if total_cost < *reachable.get(&neighbor).unwrap_or(&u32::MAX) {
                reachable.insert(neighbor, total_cost);
                frontier.push(PathNode {
                    coord: neighbor,
                    g_cost: total_cost,
                    f_cost: total_cost,
                });
            }
        }
    }

    reachable
}

/// Find tiles that can be attacked from current position.
pub fn find_attackable(
    map: &Map,
    position: HexCoord,
    range: u32,
    _config: &PathConfig,
) -> Vec<HexCoord> {
    if range == 0 {
        // Melee: can attack adjacent tiles
        return map.neighbors(&position);
    }

    // Ranged: can attack tiles within range
    position
        .hexes_in_radius(range)
        .into_iter()
        .filter(|coord| {
            let dist = position.distance(coord);
            dist > 0 && dist <= range && map.get(coord).is_some()
        })
        .collect()
}

/// Get the movement cost to enter a tile.
fn get_movement_cost(map: &Map, coord: &HexCoord, config: &PathConfig) -> u32 {
    let tile = match map.get(coord) {
        Some(t) => t,
        None => return u32::MAX,
    };

    // Check terrain passability based on unit type
    match config.unit_category {
        UnitCategory::Naval => {
            if !tile.terrain.is_water() {
                return u32::MAX;
            }
        }
        UnitCategory::Air => {
            // Air units can fly over anything
            return 10; // Standard cost
        }
        _ => {
            // Land units
            if tile.terrain.is_water() && !config.embarked {
                return u32::MAX;
            }
        }
    }

    // Get base movement cost (x10 for precision)
    let cost = tile.movement_cost();
    if cost == u32::MAX {
        return u32::MAX;
    }

    cost * 10
}

/// Heuristic for A* (hex distance * minimum cost).
fn heuristic(a: &HexCoord, b: &HexCoord) -> u32 {
    a.distance(b) * 10 // Minimum cost is 1 * 10
}

/// Reconstruct the path from came_from map.
fn reconstruct_path(
    came_from: &HashMap<HexCoord, HexCoord>,
    goal: HexCoord,
    start: HexCoord,
) -> Vec<HexCoord> {
    let mut path = vec![goal];
    let mut current = goal;

    while current != start {
        if let Some(&prev) = came_from.get(&current) {
            path.push(prev);
            current = prev;
        } else {
            break;
        }
    }

    path.reverse();
    path
}

/// Calculate total movement cost along a path.
pub fn path_cost(map: &Map, path: &[HexCoord], config: &PathConfig) -> Option<u32> {
    if path.is_empty() {
        return Some(0);
    }

    let mut total = 0u32;
    for coord in path.iter().skip(1) {
        let cost = get_movement_cost(map, coord, config);
        if cost == u32::MAX {
            return None;
        }
        total = total.saturating_add(cost);
    }

    Some(total)
}

/// Check if a path is valid (all tiles passable and connected).
pub fn is_valid_path(map: &Map, path: &[HexCoord], config: &PathConfig) -> bool {
    if path.is_empty() {
        return true;
    }

    for window in path.windows(2) {
        let from = &window[0];
        let to = &window[1];

        // Check tiles are adjacent
        if from.distance(to) != 1 {
            return false;
        }

        // Check destination is passable
        if get_movement_cost(map, to, config) == u32::MAX {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::Terrain;

    fn create_test_map() -> Map {
        Map::filled(10, 10, Terrain::Grassland)
    }

    #[test]
    fn test_find_path_same_tile() {
        let map = create_test_map();
        let config = PathConfig::default();
        let start = HexCoord::new(5, 5);

        let result = find_path(&map, start, start, &config);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.path.len(), 1);
        assert_eq!(path.total_cost, 0);
    }

    #[test]
    fn test_find_path_adjacent() {
        let map = create_test_map();
        let config = PathConfig::default();
        let start = HexCoord::new(5, 5);
        let goal = HexCoord::new(5, 6);

        let result = find_path(&map, start, goal, &config);
        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(path.path.len(), 2);
        assert_eq!(path.path[0], start);
        assert_eq!(path.path[1], goal);
    }

    #[test]
    fn test_find_path_longer() {
        let map = create_test_map();
        let config = PathConfig::default();
        let start = HexCoord::new(0, 0);
        let goal = HexCoord::new(3, 3);

        let result = find_path(&map, start, goal, &config);
        assert!(result.is_some());
        let path = result.unwrap();

        // Path should start at start and end at goal
        assert_eq!(path.path.first(), Some(&start));
        assert_eq!(path.path.last(), Some(&goal));

        // Each step should be to an adjacent hex
        for window in path.path.windows(2) {
            assert_eq!(window[0].distance(&window[1]), 1);
        }
    }

    #[test]
    fn test_find_reachable() {
        let map = create_test_map();
        let config = PathConfig {
            max_movement: 20, // 2 moves
            ..Default::default()
        };
        let start = HexCoord::new(5, 5);

        let reachable = find_reachable(&map, start, &config);

        // Should include start
        assert!(reachable.contains_key(&start));
        assert_eq!(reachable.get(&start), Some(&0));

        // Should include neighbors (cost 10)
        for neighbor in map.neighbors(&start) {
            assert!(reachable.contains_key(&neighbor));
        }
    }

    #[test]
    fn test_naval_unit_water_only() {
        let mut map = create_test_map();

        // Create a water path
        for q in 0..5 {
            let coord = HexCoord::new(q, 5);
            if let Some(tile) = map.get_mut(&coord) {
                tile.terrain = Terrain::Coast;
            }
        }

        let config = PathConfig {
            unit_category: UnitCategory::Naval,
            ..Default::default()
        };

        // Naval unit should find path through water
        let result = find_path(&map, HexCoord::new(0, 5), HexCoord::new(4, 5), &config);
        assert!(result.is_some());

        // Naval unit should NOT find path through land
        let result2 = find_path(&map, HexCoord::new(0, 5), HexCoord::new(0, 0), &config);
        assert!(result2.is_none());
    }

    #[test]
    fn test_find_attackable_melee() {
        let map = create_test_map();
        let config = PathConfig::default();
        let position = HexCoord::new(5, 5);

        let targets = find_attackable(&map, position, 0, &config);
        assert_eq!(targets.len(), 6); // 6 adjacent hexes
    }

    #[test]
    fn test_find_attackable_ranged() {
        let map = create_test_map();
        let config = PathConfig::default();
        let position = HexCoord::new(5, 5);

        let targets = find_attackable(&map, position, 2, &config);

        // Should include hexes at distance 1 and 2, but not 0
        assert!(targets.iter().all(|t| {
            let dist = position.distance(t);
            dist > 0 && dist <= 2
        }));
    }

    #[test]
    fn test_path_cost() {
        let map = create_test_map();
        let config = PathConfig::default();
        let path = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];

        let cost = path_cost(&map, &path, &config);
        assert!(cost.is_some());
        assert_eq!(cost.unwrap(), 20); // 2 moves * 10 each
    }

    #[test]
    fn test_is_valid_path() {
        let map = create_test_map();
        let config = PathConfig::default();

        // Valid path
        let valid = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];
        assert!(is_valid_path(&map, &valid, &config));

        // Invalid path (non-adjacent hexes)
        let invalid = vec![
            HexCoord::new(0, 0),
            HexCoord::new(5, 5), // Not adjacent
        ];
        assert!(!is_valid_path(&map, &invalid, &config));
    }
}

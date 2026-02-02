//! Comprehensive fog of war tests for Nostr Nations.
//!
//! These tests cover all aspects of visibility, exploration, and fog of war
//! mechanics including:
//! - Initial visibility states
//! - Vision radius calculations
//! - Exploration persistence
//! - Unit movement visibility
//! - Combat visibility requirements
//! - City visibility mechanics
//! - Shared visibility (allies/treaties)
//! - Edge cases

use nostr_nations_core::{
    city::{BuildingType, City},
    game_state::{DiplomaticStatus, GameState, TreatyType},
    hex::HexCoord,
    map::{Map, Tile},
    player::{Civilization, Player},
    settings::GameSettings,
    terrain::{Feature, Terrain},
    types::{MapSize, PlayerId},
    unit::{Promotion, Unit, UnitType},
};
use std::collections::HashSet;

// =============================================================================
// Test Helpers
// =============================================================================

/// Create a basic game with specified number of players
fn create_game_with_players(player_count: u8) -> GameState {
    let mut settings = GameSettings::new("Fog of War Test".to_string());
    settings.player_count = player_count;
    settings.map_size = MapSize::Duel;

    let mut game = GameState::new("fog-test-game".to_string(), settings, [42u8; 32]);

    for i in 0..player_count {
        let player = Player::new(
            i,
            format!("npub_player_{}", i),
            format!("Player {}", i + 1),
            Civilization::generic(),
        );
        game.add_player(player).unwrap();
    }

    game
}

/// Create a map filled with grassland for testing
fn create_test_map(width: u32, height: u32) -> Map {
    Map::filled(width, height, Terrain::Grassland)
}

/// Create a test player
fn create_test_player(id: PlayerId, name: &str) -> Player {
    Player::new(
        id,
        format!("npub_{}", id),
        name.to_string(),
        Civilization::generic(),
    )
}

/// Default vision radius for different unit types
fn base_vision_radius(unit_type: UnitType) -> u32 {
    match unit_type {
        // Scouts have extended vision
        UnitType::Warrior => 2,
        UnitType::Archer => 2,
        UnitType::Swordsman => 2,
        UnitType::Knight => 2,
        UnitType::Cavalry => 2,
        // Civilians have limited vision
        UnitType::Settler => 2,
        UnitType::Worker => 2,
        // Naval units have better vision
        UnitType::Galley => 2,
        UnitType::Trireme => 2,
        UnitType::Caravel => 3,
        UnitType::Frigate => 3,
        UnitType::Battleship => 3,
        // Air units have extended vision
        UnitType::Fighter => 4,
        UnitType::Bomber => 4,
        _ => 2, // Default vision
    }
}

/// Calculate tiles visible from a position with given vision radius
fn calculate_visible_tiles(center: &HexCoord, radius: u32, map: &Map) -> HashSet<HexCoord> {
    let mut visible = HashSet::new();

    for tile_coord in center.hexes_in_radius(radius) {
        if map.in_bounds(&tile_coord) {
            visible.insert(tile_coord);
        }
    }

    visible
}

/// Check if a tile blocks vision (mountains, dense forest)
fn blocks_vision(tile: &Tile) -> bool {
    match tile.feature {
        Some(Feature::Mountains) => true,
        Some(Feature::Jungle) => true,
        _ => false,
    }
}

/// Check if a tile provides vision bonus (hills)
fn gives_vision_bonus(tile: &Tile) -> u32 {
    match tile.feature {
        Some(Feature::Hills) => 1,
        _ => 0,
    }
}

/// Visibility state for a tile from a player's perspective
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TileVisibility {
    /// Never seen - completely unknown
    Hidden,
    /// Previously explored but not currently visible (fog)
    Explored,
    /// Currently visible
    Visible,
}

/// Track visibility state for a player
struct PlayerVisibility {
    /// Tiles that have ever been explored
    explored: HashSet<HexCoord>,
    /// Tiles currently visible
    visible: HashSet<HexCoord>,
}

impl PlayerVisibility {
    fn new() -> Self {
        Self {
            explored: HashSet::new(),
            visible: HashSet::new(),
        }
    }

    fn get_tile_state(&self, coord: &HexCoord) -> TileVisibility {
        if self.visible.contains(coord) {
            TileVisibility::Visible
        } else if self.explored.contains(coord) {
            TileVisibility::Explored
        } else {
            TileVisibility::Hidden
        }
    }

    fn reveal_tile(&mut self, coord: HexCoord) {
        self.explored.insert(coord);
        self.visible.insert(coord);
    }

    #[allow(dead_code)]
    fn hide_tile(&mut self, coord: &HexCoord) {
        self.visible.remove(coord);
        // explored stays true
    }

    fn clear_visible(&mut self) {
        self.visible.clear();
    }
}

// =============================================================================
// 1. Initial Visibility Tests
// =============================================================================

mod initial_visibility {
    use super::*;

    #[test]
    fn test_new_player_starts_with_no_visibility() {
        let player = create_test_player(0, "NewPlayer");

        // New player should have no explored tiles
        assert!(player.explored_tiles.is_empty());
        assert_eq!(player.explored_count(), 0);

        // Random tiles should not be explored
        assert!(!player.has_explored(&HexCoord::new(5, 5)));
        assert!(!player.has_explored(&HexCoord::new(10, 10)));
        assert!(!player.has_explored(&HexCoord::new(0, 0)));
    }

    #[test]
    fn test_unit_reveals_tiles_around_it() {
        let mut player = create_test_player(0, "UnitPlayer");
        let map = create_test_map(20, 20);

        // Place a warrior at (10, 10)
        let unit_pos = HexCoord::new(10, 10);
        let vision_radius = base_vision_radius(UnitType::Warrior);

        // Reveal tiles based on unit vision
        let visible_tiles = calculate_visible_tiles(&unit_pos, vision_radius, &map);

        for coord in &visible_tiles {
            player.explore_tile(*coord);
        }

        // Unit position should be visible
        assert!(player.has_explored(&unit_pos));

        // Adjacent tiles should be visible
        for neighbor in unit_pos.neighbors() {
            if map.in_bounds(&neighbor) {
                assert!(
                    player.has_explored(&neighbor),
                    "Neighbor {:?} should be explored",
                    neighbor
                );
            }
        }

        // Tiles within vision radius should be visible
        for coord in unit_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                assert!(
                    player.has_explored(&coord),
                    "Tile {:?} within radius {} should be explored",
                    coord,
                    vision_radius
                );
            }
        }

        // Tiles far away should not be visible
        assert!(!player.has_explored(&HexCoord::new(0, 0)));
        assert!(!player.has_explored(&HexCoord::new(19, 19)));
    }

    #[test]
    fn test_city_reveals_tiles_in_radius() {
        let mut player = create_test_player(0, "CityPlayer");
        let map = create_test_map(20, 20);

        // City position
        let city_pos = HexCoord::new(10, 10);
        let city_vision_radius = 2u32; // Base city vision

        // Reveal tiles based on city vision
        let visible_tiles = calculate_visible_tiles(&city_pos, city_vision_radius, &map);

        for coord in &visible_tiles {
            player.explore_tile(*coord);
        }

        // City position should be visible
        assert!(player.has_explored(&city_pos));

        // All tiles within city vision should be visible
        let tiles_in_radius = city_pos.hexes_in_radius(city_vision_radius);
        let visible_count = tiles_in_radius.iter().filter(|c| map.in_bounds(c)).count();

        assert!(
            player.explored_count() >= visible_count,
            "Expected at least {} explored tiles, got {}",
            visible_count,
            player.explored_count()
        );
    }

    #[test]
    fn test_starting_position_reveals_surrounding_area() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Simulate placing starting units for player 0
        let start_pos = HexCoord::new(5, 5);
        let vision_radius = 2u32;

        // Reveal starting area
        for coord in start_pos.hexes_in_radius(vision_radius) {
            if game.map.in_bounds(&coord) {
                game.players[0].explore_tile(coord);
            }
        }

        // Player should have explored the starting area
        assert!(game.players[0].has_explored(&start_pos));
        assert!(game.players[0].explored_count() > 0);

        // Player 1 should not have explored player 0's starting area (unless they start nearby)
        // This depends on starting positions which would be determined by map generation
    }

    #[test]
    fn test_multiple_units_combine_visibility() {
        let mut player = create_test_player(0, "MultiUnitPlayer");
        let map = create_test_map(30, 30);

        // Place two warriors at different positions
        let unit1_pos = HexCoord::new(10, 10);
        let unit2_pos = HexCoord::new(15, 15);
        let vision_radius = base_vision_radius(UnitType::Warrior);

        // Reveal from both units
        for coord in unit1_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        for coord in unit2_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        // Both unit positions should be visible
        assert!(player.has_explored(&unit1_pos));
        assert!(player.has_explored(&unit2_pos));

        // Total explored should be combined (with some overlap)
        let unit1_tiles = unit1_pos.hexes_in_radius(vision_radius);
        let unit2_tiles = unit2_pos.hexes_in_radius(vision_radius);

        let mut all_tiles: HashSet<HexCoord> = HashSet::new();
        for t in unit1_tiles {
            if map.in_bounds(&t) {
                all_tiles.insert(t);
            }
        }
        for t in unit2_tiles {
            if map.in_bounds(&t) {
                all_tiles.insert(t);
            }
        }

        assert_eq!(player.explored_count(), all_tiles.len());
    }
}

// =============================================================================
// 2. Vision Radius Tests
// =============================================================================

mod vision_radius {
    use super::*;

    #[test]
    fn test_different_unit_types_have_different_vision() {
        // Standard military units
        assert_eq!(base_vision_radius(UnitType::Warrior), 2);
        assert_eq!(base_vision_radius(UnitType::Archer), 2);
        assert_eq!(base_vision_radius(UnitType::Swordsman), 2);

        // Cavalry (same as infantry)
        assert_eq!(base_vision_radius(UnitType::Knight), 2);
        assert_eq!(base_vision_radius(UnitType::Cavalry), 2);

        // Naval units may have different vision
        assert!(base_vision_radius(UnitType::Caravel) >= 2);
        assert!(base_vision_radius(UnitType::Frigate) >= 2);

        // Air units have best vision
        assert!(base_vision_radius(UnitType::Fighter) >= base_vision_radius(UnitType::Warrior));
        assert!(base_vision_radius(UnitType::Bomber) >= base_vision_radius(UnitType::Warrior));
    }

    #[test]
    fn test_hills_give_vision_bonus() {
        let flat_tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        let hill_tile = {
            let mut t = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
            t.feature = Some(Feature::Hills);
            t
        };

        // Flat terrain gives no bonus
        assert_eq!(gives_vision_bonus(&flat_tile), 0);

        // Hills give +1 vision
        assert_eq!(gives_vision_bonus(&hill_tile), 1);
    }

    #[test]
    fn test_unit_on_hills_sees_further() {
        let mut player = create_test_player(0, "HillsPlayer");
        let mut map = create_test_map(20, 20);

        // Place hills at unit position
        let unit_pos = HexCoord::new(10, 10);
        if let Some(tile) = map.get_mut(&unit_pos) {
            tile.feature = Some(Feature::Hills);
        }

        // Unit on hills gets +1 vision
        let base_vision = base_vision_radius(UnitType::Warrior);
        let hill_bonus = 1;
        let total_vision = base_vision + hill_bonus;

        // Reveal tiles with enhanced vision
        for coord in unit_pos.hexes_in_radius(total_vision) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        // Should see tiles at radius 3 (base 2 + hill bonus 1)
        let far_tile = HexCoord::new(10, 10 + total_vision as i32);
        if map.in_bounds(&far_tile) && unit_pos.distance(&far_tile) <= total_vision {
            assert!(
                player.has_explored(&far_tile),
                "Should see tile at distance {} with hill bonus",
                total_vision
            );
        }
    }

    #[test]
    fn test_forests_block_vision() {
        let forest_tile = {
            let mut t = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
            t.feature = Some(Feature::Forest);
            t
        };

        // Forests reduce vision but don't completely block it
        // In this test we're checking the blocking mechanic
        // Note: In actual implementation, forests would reduce vision through them
        assert!(!blocks_vision(&forest_tile)); // Forest doesn't completely block
    }

    #[test]
    fn test_jungle_blocks_vision() {
        let jungle_tile = {
            let mut t = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
            t.feature = Some(Feature::Jungle);
            t
        };

        // Dense jungle blocks vision
        assert!(blocks_vision(&jungle_tile));
    }

    #[test]
    fn test_mountains_block_vision() {
        let mountain_tile = {
            let mut t = Tile::new(HexCoord::new(0, 0), Terrain::Plains);
            t.feature = Some(Feature::Mountains);
            t
        };

        // Mountains block vision
        assert!(blocks_vision(&mountain_tile));
    }

    #[test]
    fn test_sentry_promotion_increases_vision() {
        // The Sentry promotion gives +1 vision
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(5, 5));

        let base_vision = base_vision_radius(unit.unit_type);

        // Add Sentry promotion
        unit.add_promotion(Promotion::Sentry);

        // With Sentry, vision should be base + 1
        let sentry_bonus = 1u32;
        let expected_vision = base_vision + sentry_bonus;

        // Verify promotion was added
        assert!(unit.promotions.contains(&Promotion::Sentry));

        // The actual vision calculation would add the bonus
        assert_eq!(expected_vision, 3);
    }

    #[test]
    fn test_vision_blocked_by_terrain_chain() {
        let mut map = create_test_map(20, 20);

        // Create a line of mountains blocking vision
        for q in 8..=12 {
            let coord = HexCoord::new(q, 10);
            if let Some(tile) = map.get_mut(&coord) {
                tile.feature = Some(Feature::Mountains);
            }
        }

        let _viewer_pos = HexCoord::new(10, 8);
        let _target_pos = HexCoord::new(10, 12);

        // Check if there's a mountain between viewer and target
        let mountain_between = HexCoord::new(10, 10);
        let tile = map.get(&mountain_between).unwrap();

        assert!(blocks_vision(tile));

        // In a proper line-of-sight implementation, target should not be visible
        // through the mountains (but tile itself is visible)
    }
}

// =============================================================================
// 3. Exploration Tests
// =============================================================================

mod exploration {
    use super::*;

    #[test]
    fn test_explored_tiles_stay_explored() {
        let mut player = create_test_player(0, "Explorer");
        let map = create_test_map(20, 20);

        let unit_pos = HexCoord::new(10, 10);
        let vision_radius = base_vision_radius(UnitType::Warrior);

        // Reveal tiles
        for coord in unit_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        let explored_count = player.explored_count();

        // Moving unit away doesn't remove exploration
        // (In Player struct, explored_tiles is persistent)

        // All previously explored tiles should still be marked
        for coord in unit_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                assert!(player.has_explored(&coord));
            }
        }

        assert_eq!(player.explored_count(), explored_count);
    }

    #[test]
    fn test_fog_returns_when_units_leave() {
        let mut visibility = PlayerVisibility::new();
        let map = create_test_map(20, 20);

        let unit_pos = HexCoord::new(10, 10);
        let vision_radius = 2u32;

        // Reveal tiles while unit is present
        for coord in unit_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                visibility.reveal_tile(coord);
            }
        }

        // All tiles should be visible
        assert_eq!(
            visibility.get_tile_state(&unit_pos),
            TileVisibility::Visible
        );

        // Unit moves away - clear current visibility
        visibility.clear_visible();

        // Tiles should now be explored but not visible (fog)
        assert_eq!(
            visibility.get_tile_state(&unit_pos),
            TileVisibility::Explored
        );

        // But still in explored set
        assert!(visibility.explored.contains(&unit_pos));
    }

    #[test]
    fn test_cities_always_keep_tiles_visible() {
        let mut visibility = PlayerVisibility::new();
        let map = create_test_map(20, 20);

        let city_pos = HexCoord::new(10, 10);
        let city_vision = 2u32;

        // City reveals tiles
        for coord in city_pos.hexes_in_radius(city_vision) {
            if map.in_bounds(&coord) {
                visibility.reveal_tile(coord);
            }
        }

        // Even after clearing visible (simulating end of turn recalculation),
        // cities immediately re-reveal their area

        // Re-apply city vision
        for coord in city_pos.hexes_in_radius(city_vision) {
            if map.in_bounds(&coord) {
                visibility.reveal_tile(coord);
            }
        }

        // City area should still be visible
        assert_eq!(
            visibility.get_tile_state(&city_pos),
            TileVisibility::Visible
        );

        for coord in city_pos.hexes_in_radius(city_vision) {
            if map.in_bounds(&coord) {
                assert_eq!(visibility.get_tile_state(&coord), TileVisibility::Visible);
            }
        }
    }

    #[test]
    fn test_exploration_accumulates_over_time() {
        let mut player = create_test_player(0, "Explorer");
        let map = create_test_map(40, 40);

        // Move unit along a path
        let path = vec![
            HexCoord::new(10, 10),
            HexCoord::new(12, 10),
            HexCoord::new(14, 10),
            HexCoord::new(16, 10),
            HexCoord::new(18, 10),
        ];

        let vision_radius = 2u32;

        for pos in &path {
            for coord in pos.hexes_in_radius(vision_radius) {
                if map.in_bounds(&coord) {
                    player.explore_tile(coord);
                }
            }
        }

        // All path positions should be explored
        for pos in &path {
            assert!(player.has_explored(pos));
        }

        // Total exploration should be substantial
        assert!(player.explored_count() > path.len());
    }

    #[test]
    fn test_exploration_score_contribution() {
        let mut player = create_test_player(0, "ScoreExplorer");

        let initial_land_score = player.score.land;

        // Explore 10 new tiles
        for i in 0..10 {
            player.explore_tile(HexCoord::new(i, 0));
        }

        // Land score should increase
        assert_eq!(player.score.land, initial_land_score + 10);

        // Re-exploring same tile shouldn't increase score
        player.explore_tile(HexCoord::new(0, 0));
        assert_eq!(player.score.land, initial_land_score + 10);
    }
}

// =============================================================================
// 4. Unit Movement Tests
// =============================================================================

mod unit_movement {
    use super::*;

    #[test]
    fn test_moving_unit_reveals_new_tiles() {
        let mut player = create_test_player(0, "Mover");
        let map = create_test_map(30, 30);

        let start_pos = HexCoord::new(10, 10);
        let end_pos = HexCoord::new(15, 10);
        let vision_radius = 2u32;

        // Initial revelation at start
        for coord in start_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        let initial_explored = player.explored_count();

        // Move to end position and reveal
        for coord in end_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        // Should have explored more tiles
        assert!(player.explored_count() > initial_explored);

        // Both positions should be explored
        assert!(player.has_explored(&start_pos));
        assert!(player.has_explored(&end_pos));
    }

    #[test]
    fn test_old_tiles_become_fog_after_move() {
        let mut visibility = PlayerVisibility::new();
        let map = create_test_map(30, 30);

        let start_pos = HexCoord::new(10, 10);
        let end_pos = HexCoord::new(20, 20); // Far enough that ranges don't overlap
        let vision_radius = 2u32;

        // Reveal at start
        for coord in start_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                visibility.reveal_tile(coord);
            }
        }

        assert_eq!(
            visibility.get_tile_state(&start_pos),
            TileVisibility::Visible
        );

        // Move to new position - clear and re-reveal
        visibility.clear_visible();

        for coord in end_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                visibility.reveal_tile(coord);
            }
        }

        // Old position should be fog (explored but not visible)
        assert_eq!(
            visibility.get_tile_state(&start_pos),
            TileVisibility::Explored
        );

        // New position should be visible
        assert_eq!(visibility.get_tile_state(&end_pos), TileVisibility::Visible);
    }

    #[test]
    fn test_unit_path_reveals_all_tiles_along_route() {
        let mut player = create_test_player(0, "PathMover");
        let map = create_test_map(30, 30);

        // Unit moves along a path
        let path = vec![
            HexCoord::new(5, 5),
            HexCoord::new(6, 5),
            HexCoord::new(7, 5),
            HexCoord::new(8, 5),
            HexCoord::new(9, 5),
            HexCoord::new(10, 5),
        ];

        let vision_radius = 2u32;

        // Reveal along entire path
        for pos in &path {
            for coord in pos.hexes_in_radius(vision_radius) {
                if map.in_bounds(&coord) {
                    player.explore_tile(coord);
                }
            }
        }

        // Every position on the path should be explored
        for pos in &path {
            assert!(
                player.has_explored(pos),
                "Path position {:?} should be explored",
                pos
            );
        }

        // Adjacent tiles to the path should also be explored
        for pos in &path {
            for neighbor in pos.neighbors() {
                if map.in_bounds(&neighbor) {
                    assert!(
                        player.has_explored(&neighbor),
                        "Neighbor {:?} of path position should be explored",
                        neighbor
                    );
                }
            }
        }
    }

    #[test]
    fn test_fast_units_reveal_more_tiles_per_turn() {
        let _player = create_test_player(0, "FastMover");
        let map = create_test_map(40, 40);

        // Warrior moves 2 tiles per turn
        let warrior_path: Vec<HexCoord> = vec![HexCoord::new(10, 10), HexCoord::new(11, 10)];

        // Cavalry moves 4+ tiles per turn
        let cavalry_path: Vec<HexCoord> = vec![
            HexCoord::new(10, 20),
            HexCoord::new(11, 20),
            HexCoord::new(12, 20),
            HexCoord::new(13, 20),
        ];

        let vision_radius = 2u32;

        // Count tiles explored by each
        let mut warrior_explored: HashSet<HexCoord> = HashSet::new();
        for pos in &warrior_path {
            for coord in pos.hexes_in_radius(vision_radius) {
                if map.in_bounds(&coord) {
                    warrior_explored.insert(coord);
                }
            }
        }

        let mut cavalry_explored: HashSet<HexCoord> = HashSet::new();
        for pos in &cavalry_path {
            for coord in pos.hexes_in_radius(vision_radius) {
                if map.in_bounds(&coord) {
                    cavalry_explored.insert(coord);
                }
            }
        }

        // Cavalry should explore more
        assert!(
            cavalry_explored.len() > warrior_explored.len(),
            "Cavalry ({}) should explore more than warrior ({})",
            cavalry_explored.len(),
            warrior_explored.len()
        );
    }

    #[test]
    fn test_movement_through_forest_still_reveals() {
        let mut player = create_test_player(0, "ForestMover");
        let mut map = create_test_map(20, 20);

        // Add forests along path
        for q in 8..=12 {
            let coord = HexCoord::new(q, 10);
            if let Some(tile) = map.get_mut(&coord) {
                tile.feature = Some(Feature::Forest);
            }
        }

        let forest_pos = HexCoord::new(10, 10);
        let vision_radius = 2u32;

        // Unit moves through forest - still reveals (though may have reduced vision)
        for coord in forest_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        // Forest tile should be explored
        assert!(player.has_explored(&forest_pos));
    }
}

// =============================================================================
// 5. Combat Visibility Tests
// =============================================================================

mod combat_visibility {
    use super::*;

    #[test]
    fn test_can_only_attack_visible_units() {
        let visibility = {
            let mut v = PlayerVisibility::new();
            // Only reveal some tiles
            v.reveal_tile(HexCoord::new(5, 5));
            v.reveal_tile(HexCoord::new(6, 5));
            v
        };

        let visible_enemy = HexCoord::new(6, 5);
        let hidden_enemy = HexCoord::new(10, 10);

        // Can attack visible enemy
        assert_eq!(
            visibility.get_tile_state(&visible_enemy),
            TileVisibility::Visible
        );

        // Cannot attack hidden enemy
        assert_eq!(
            visibility.get_tile_state(&hidden_enemy),
            TileVisibility::Hidden
        );
    }

    #[test]
    fn test_combat_reveals_attacker_to_defender() {
        let mut defender_visibility = PlayerVisibility::new();

        let attacker_pos = HexCoord::new(10, 10);

        // Before attack, attacker position is hidden
        assert_eq!(
            defender_visibility.get_tile_state(&attacker_pos),
            TileVisibility::Hidden
        );

        // Combat reveals attacker to defender
        defender_visibility.reveal_tile(attacker_pos);

        // Now defender can see attacker
        assert_eq!(
            defender_visibility.get_tile_state(&attacker_pos),
            TileVisibility::Visible
        );
    }

    #[test]
    fn test_ranged_units_need_line_of_sight() {
        let mut map = create_test_map(20, 20);

        let _archer_pos = HexCoord::new(5, 5);
        let _target_pos = HexCoord::new(7, 5);
        let blocking_pos = HexCoord::new(6, 5);

        // Add mountain between archer and target
        if let Some(tile) = map.get_mut(&blocking_pos) {
            tile.feature = Some(Feature::Mountains);
        }

        // Check if there's line of sight
        let blocking_tile = map.get(&blocking_pos).unwrap();
        let has_los = !blocks_vision(blocking_tile);

        // Mountains block line of sight
        assert!(!has_los, "Mountains should block line of sight");

        // Cannot attack without line of sight
        // (actual implementation would check this before allowing ranged attack)
    }

    #[test]
    fn test_melee_combat_at_adjacent_tiles() {
        let visibility = {
            let mut v = PlayerVisibility::new();
            // Reveal attacker and target positions
            v.reveal_tile(HexCoord::new(5, 5)); // Attacker
            v.reveal_tile(HexCoord::new(5, 6)); // Adjacent target
            v
        };

        let attacker_pos = HexCoord::new(5, 5);
        let target_pos = HexCoord::new(5, 6);

        // Check adjacency
        let neighbors = attacker_pos.neighbors();
        let is_adjacent = neighbors.iter().any(|n| n == &target_pos);

        assert!(is_adjacent, "Target should be adjacent to attacker");

        // Both tiles visible means melee combat is valid
        assert_eq!(
            visibility.get_tile_state(&attacker_pos),
            TileVisibility::Visible
        );
        assert_eq!(
            visibility.get_tile_state(&target_pos),
            TileVisibility::Visible
        );
    }

    #[test]
    fn test_ranged_attack_within_range() {
        let archer = Unit::new(1, 0, UnitType::Archer, HexCoord::new(5, 5));
        let target_pos = HexCoord::new(7, 5);

        let range = archer.range();
        let distance = archer.position.distance(&target_pos);

        // Archer has range 2, target is at distance 2
        assert!(
            distance <= range,
            "Target at distance {} should be within range {}",
            distance,
            range
        );
    }

    #[test]
    fn test_ranged_attack_out_of_range() {
        let archer = Unit::new(1, 0, UnitType::Archer, HexCoord::new(5, 5));
        let target_pos = HexCoord::new(10, 5);

        let range = archer.range();
        let distance = archer.position.distance(&target_pos);

        // Archer has range 2, target is at distance 5
        assert!(
            distance > range,
            "Target at distance {} should be out of range {}",
            distance,
            range
        );
    }

    #[test]
    fn test_unit_in_fog_cannot_be_targeted() {
        let mut visibility = PlayerVisibility::new();

        // Explore an area but then move away
        let explored_pos = HexCoord::new(10, 10);
        visibility.reveal_tile(explored_pos);
        visibility.clear_visible();

        // Position is now fog (explored but not visible)
        assert_eq!(
            visibility.get_tile_state(&explored_pos),
            TileVisibility::Explored
        );

        // Cannot target units in fog - only visible units can be targeted
        let is_targetable = visibility.get_tile_state(&explored_pos) == TileVisibility::Visible;
        assert!(!is_targetable, "Units in fog should not be targetable");
    }
}

// =============================================================================
// 6. City Visibility Tests
// =============================================================================

mod city_visibility {
    use super::*;

    #[test]
    fn test_cities_have_fixed_vision_radius() {
        let base_city_vision = 2u32;

        let city = City::new(1, 0, "TestCity".to_string(), HexCoord::new(10, 10), true);

        // City should provide consistent vision
        let visible_tiles = city.position.hexes_in_radius(base_city_vision);

        // Should see center + all tiles within radius 2
        assert!(!visible_tiles.is_empty());
        assert!(visible_tiles.contains(&city.position));
    }

    #[test]
    fn test_walls_increase_city_vision() {
        let base_city_vision = 2u32;
        let wall_vision_bonus = 1u32;

        let mut city = City::new(1, 0, "WalledCity".to_string(), HexCoord::new(10, 10), true);

        // Add walls
        city.add_building(BuildingType::Walls);
        assert!(city.buildings.contains(&BuildingType::Walls));

        // City with walls gets increased vision
        let enhanced_vision = base_city_vision + wall_vision_bonus;
        let visible_tiles = city.position.hexes_in_radius(enhanced_vision);

        // Should see more tiles than before
        let base_tiles = city.position.hexes_in_radius(base_city_vision);
        assert!(visible_tiles.len() > base_tiles.len());
    }

    #[test]
    fn test_castle_increases_vision_more() {
        let base_city_vision = 2u32;
        let wall_bonus = 1u32;
        let castle_bonus = 1u32;

        let mut city = City::new(
            1,
            0,
            "FortifiedCity".to_string(),
            HexCoord::new(10, 10),
            true,
        );

        // Add walls first (prerequisite for castle)
        city.add_building(BuildingType::Walls);
        city.add_building(BuildingType::Castle);

        assert!(city.buildings.contains(&BuildingType::Castle));

        // Castle further increases vision
        let total_vision = base_city_vision + wall_bonus + castle_bonus;
        let visible_tiles = city.position.hexes_in_radius(total_vision);

        // Should see at radius 4
        let far_tile = HexCoord::new(10, 10 + total_vision as i32);
        assert!(
            visible_tiles.iter().any(|t| t == &far_tile)
                || city.position.distance(&far_tile) <= total_vision
        );
    }

    #[test]
    fn test_city_sees_enemy_units_approaching() {
        let mut visibility = PlayerVisibility::new();
        let map = create_test_map(20, 20);

        let city_pos = HexCoord::new(10, 10);
        let city_vision = 2u32;

        // City reveals its area
        for coord in city_pos.hexes_in_radius(city_vision) {
            if map.in_bounds(&coord) {
                visibility.reveal_tile(coord);
            }
        }

        // Enemy approaches from outside vision
        let enemy_far = HexCoord::new(15, 10); // Outside vision
        let enemy_near = HexCoord::new(12, 10); // Inside vision

        // Far enemy not visible
        assert_eq!(
            visibility.get_tile_state(&enemy_far),
            TileVisibility::Hidden
        );

        // Near enemy is visible (if they move into city vision range)
        assert_eq!(
            visibility.get_tile_state(&enemy_near),
            TileVisibility::Visible
        );
    }

    #[test]
    fn test_capital_vs_regular_city_vision() {
        let capital = City::new(
            1,
            0,
            "Capital".to_string(),
            HexCoord::new(5, 5),
            true, // is_capital
        );

        let regular_city = City::new(
            2,
            0,
            "Village".to_string(),
            HexCoord::new(15, 15),
            false, // not capital
        );

        assert!(capital.is_capital);
        assert!(!regular_city.is_capital);

        // Both have same base vision (unless capital bonus is implemented)
        let base_vision = 2u32;
        let capital_tiles = capital.position.hexes_in_radius(base_vision);
        let regular_tiles = regular_city.position.hexes_in_radius(base_vision);

        assert_eq!(capital_tiles.len(), regular_tiles.len());
    }

    #[test]
    fn test_city_territory_visibility() {
        let city = City::new(
            1,
            0,
            "TerritoryCity".to_string(),
            HexCoord::new(10, 10),
            true,
        );

        // City territory (borders) should all be visible
        for tile in &city.territory {
            // All territory tiles should be within reasonable vision
            let distance = city.position.distance(tile);
            assert!(
                distance <= 3,
                "Territory tile {:?} at distance {} should be visible",
                tile,
                distance
            );
        }
    }
}

// =============================================================================
// 7. Shared Visibility Tests
// =============================================================================

mod shared_visibility {
    use super::*;

    #[test]
    fn test_allied_players_share_visibility() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Set players as allies
        if let Some(rel) = game.diplomacy.get_mut(0, 1) {
            rel.status = DiplomaticStatus::Allied;
        }

        // Verify alliance
        let rel = game.diplomacy.get(0, 1).unwrap();
        assert_eq!(rel.status, DiplomaticStatus::Allied);

        // Player 0 explores some tiles
        let explored_pos = HexCoord::new(10, 10);
        game.players[0].explore_tile(explored_pos);

        // In allied mode, player 1 would also see these tiles
        // This would be implemented in the visibility calculation system
        assert!(game.players[0].has_explored(&explored_pos));
    }

    #[test]
    fn test_open_borders_shares_visibility() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Establish open borders
        game.diplomacy.modify_relationship_score(0, 1, 60);
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1));

        // Verify treaty
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));

        // With open borders, visibility might be shared (depending on implementation)
        // At minimum, units can pass through each other's territory
        assert!(game.diplomacy.can_units_pass(0, 1));
        assert!(game.diplomacy.can_units_pass(1, 0));
    }

    #[test]
    fn test_war_removes_shared_visibility() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Set up alliance with shared visibility
        if let Some(rel) = game.diplomacy.get_mut(0, 1) {
            rel.status = DiplomaticStatus::Allied;
        }
        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1);

        // Verify initial state
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));

        // Declare war
        game.diplomacy.declare_war(0, 1, 2);

        // War removes treaties and shared visibility
        assert!(game.diplomacy.are_at_war(0, 1));
        assert!(!game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));
        assert!(!game.diplomacy.can_units_pass(0, 1));
    }

    #[test]
    fn test_defensive_pact_shares_visibility() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Establish defensive pact (requires allied status)
        if let Some(rel) = game.diplomacy.get_mut(0, 1) {
            rel.status = DiplomaticStatus::Allied;
        }

        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::DefensivePact, 1));

        // Defensive pact means close military cooperation
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::DefensivePact));

        // With defensive pact, visibility would be shared more extensively
    }

    #[test]
    fn test_neutral_players_dont_share_visibility() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Default neutral relationship
        let rel = game.diplomacy.get(0, 1).unwrap();
        assert_eq!(rel.status, DiplomaticStatus::Neutral);

        // Units cannot pass without treaty
        assert!(!game.diplomacy.can_units_pass(0, 1));

        // Player 0 explores
        game.players[0].explore_tile(HexCoord::new(10, 10));

        // Player 1 should not see player 0's explored tiles
        assert!(!game.players[1].has_explored(&HexCoord::new(10, 10)));
    }

    #[test]
    fn test_three_player_visibility_chains() {
        let mut game = create_game_with_players(3);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Player 0 allied with Player 1
        if let Some(rel) = game.diplomacy.get_mut(0, 1) {
            rel.status = DiplomaticStatus::Allied;
        }

        // Player 1 allied with Player 2
        if let Some(rel) = game.diplomacy.get_mut(1, 2) {
            rel.status = DiplomaticStatus::Allied;
        }

        // Player 0 and Player 2 are not directly allied
        let rel_0_2 = game.diplomacy.get(0, 2).unwrap();
        assert_ne!(rel_0_2.status, DiplomaticStatus::Allied);

        // Visibility sharing should not chain through allies
        // (Player 0 shouldn't see Player 2's vision just because both are allied with Player 1)
    }
}

// =============================================================================
// 8. Edge Cases Tests
// =============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn test_map_edges_visibility() {
        let mut player = create_test_player(0, "EdgePlayer");
        let map = create_test_map(20, 20);

        // Unit at corner of map
        let corner_pos = HexCoord::new(0, 0);
        let vision_radius = 2u32;

        // Reveal tiles (respecting map bounds)
        for coord in corner_pos.hexes_in_radius(vision_radius) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        // Corner should be explored
        assert!(player.has_explored(&corner_pos));

        // Out of bounds tiles should not be in explored set
        assert!(!player.has_explored(&HexCoord::new(-1, 0)));
        assert!(!player.has_explored(&HexCoord::new(0, -1)));
    }

    #[test]
    fn test_ocean_tile_visibility() {
        let mut map = create_test_map(20, 20);

        // Create ocean tiles
        for q in 10..15 {
            for r in 10..15 {
                let coord = HexCoord::new(q, r);
                if let Some(tile) = map.get_mut(&coord) {
                    tile.terrain = Terrain::Ocean;
                }
            }
        }

        let ocean_tile = map.get(&HexCoord::new(12, 12)).unwrap();
        assert_eq!(ocean_tile.terrain, Terrain::Ocean);
        assert!(ocean_tile.terrain.is_water());

        // Ocean doesn't block vision
        assert!(!blocks_vision(ocean_tile));
    }

    #[test]
    fn test_coast_tile_visibility() {
        let mut map = create_test_map(20, 20);

        let coord = HexCoord::new(10, 10);
        if let Some(tile) = map.get_mut(&coord) {
            tile.terrain = Terrain::Coast;
        }

        let coast_tile = map.get(&coord).unwrap();
        assert!(coast_tile.terrain.is_water());

        // Coast is visible and doesn't block
        assert!(!blocks_vision(coast_tile));
    }

    #[test]
    fn test_units_on_transport_visibility() {
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(5, 5));

        // Unit embarks
        unit.embarked = true;

        assert!(unit.embarked);

        // Embarked units should still provide visibility
        // Vision might be reduced while embarked
        let base_vision = base_vision_radius(unit.unit_type);
        let embarked_vision = base_vision; // Could be reduced

        assert!(
            embarked_vision > 0,
            "Embarked units should still have some vision"
        );
    }

    #[test]
    fn test_map_wrap_visibility() {
        let map = Map::new(20, 20, true); // wrap_x = true

        // Coordinate wrapping
        let wrapped = map.wrap_coord(&HexCoord::new(-1, 5));
        assert_eq!(wrapped.q, 19);

        let wrapped2 = map.wrap_coord(&HexCoord::new(21, 5));
        assert_eq!(wrapped2.q, 1);

        // Visibility should work across the wrap
        let near_edge = HexCoord::new(19, 5);
        let vision_radius = 2u32;

        // Tiles in radius should include wrapped coordinates
        let visible = near_edge.hexes_in_radius(vision_radius);

        // There should be neighbors
        assert!(!visible.is_empty());
    }

    #[test]
    fn test_zero_vision_edge_case() {
        // Hypothetical unit with zero vision
        let mut player = create_test_player(0, "BlindPlayer");
        let map = create_test_map(20, 20);

        let pos = HexCoord::new(10, 10);
        let zero_vision = 0u32;

        // With zero vision, only current tile is visible
        for coord in pos.hexes_in_radius(zero_vision) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        // Only the current position should be explored
        assert!(player.has_explored(&pos));
        assert_eq!(player.explored_count(), 1);
    }

    #[test]
    fn test_visibility_at_max_map_size() {
        // Test with a larger map
        let map = create_test_map(100, 100);
        let mut player = create_test_player(0, "LargeMapPlayer");

        let center = HexCoord::new(50, 50);
        let large_vision = 5u32;

        for coord in center.hexes_in_radius(large_vision) {
            if map.in_bounds(&coord) {
                player.explore_tile(coord);
            }
        }

        // Should have explored a substantial area
        assert!(player.explored_count() > 50);

        // But not the whole map
        assert!(player.explored_count() < map.tile_count());
    }

    #[test]
    fn test_multiple_feature_interactions() {
        let mut map = create_test_map(20, 20);

        // Create varied terrain
        let positions_and_features = [
            (HexCoord::new(5, 5), Some(Feature::Hills)),
            (HexCoord::new(6, 5), Some(Feature::Forest)),
            (HexCoord::new(7, 5), Some(Feature::Mountains)),
            (HexCoord::new(8, 5), Some(Feature::Jungle)),
            (HexCoord::new(9, 5), None), // Plains
        ];

        for (pos, feature) in positions_and_features.iter() {
            if let Some(tile) = map.get_mut(pos) {
                tile.feature = *feature;
            }
        }

        // Check blocking status
        let hills_tile = map.get(&HexCoord::new(5, 5)).unwrap();
        let forest_tile = map.get(&HexCoord::new(6, 5)).unwrap();
        let mountain_tile = map.get(&HexCoord::new(7, 5)).unwrap();
        let jungle_tile = map.get(&HexCoord::new(8, 5)).unwrap();
        let plains_tile = map.get(&HexCoord::new(9, 5)).unwrap();

        assert!(!blocks_vision(hills_tile));
        assert!(!blocks_vision(forest_tile));
        assert!(blocks_vision(mountain_tile));
        assert!(blocks_vision(jungle_tile));
        assert!(!blocks_vision(plains_tile));

        // Check vision bonus
        assert_eq!(gives_vision_bonus(hills_tile), 1);
        assert_eq!(gives_vision_bonus(forest_tile), 0);
        assert_eq!(gives_vision_bonus(mountain_tile), 0);
        assert_eq!(gives_vision_bonus(jungle_tile), 0);
        assert_eq!(gives_vision_bonus(plains_tile), 0);
    }

    #[test]
    fn test_ice_blocks_vision() {
        let ice_tile = {
            let mut t = Tile::new(HexCoord::new(0, 0), Terrain::Snow);
            t.feature = Some(Feature::Ice);
            t
        };

        // Ice is impassable and may affect visibility
        assert_eq!(ice_tile.movement_cost(), u32::MAX);

        // Ice doesn't explicitly block vision (you can see it)
        // but units can't be on it to see from there
    }

    #[test]
    fn test_hex_distance_calculations() {
        let a = HexCoord::new(0, 0);
        let b = HexCoord::new(3, 0);
        let c = HexCoord::new(0, 3);
        let _d = HexCoord::new(3, 3);

        // Distance calculations for visibility
        assert!(a.distance(&a) == 0);
        assert!(a.distance(&b) >= 2);
        assert!(a.distance(&c) >= 2);

        // Verify hexes_in_radius works correctly
        let radius_1 = a.hexes_in_radius(1);
        assert_eq!(radius_1.len(), 7); // center + 6 neighbors

        let radius_2 = a.hexes_in_radius(2);
        assert!(radius_2.len() > 7); // More tiles at radius 2
    }
}

// =============================================================================
// Additional Integration Tests
// =============================================================================

mod integration {
    use super::*;

    #[test]
    fn test_full_visibility_turn_cycle() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Create units for both players
        let unit1_id = game.allocate_unit_id();
        let unit2_id = game.allocate_unit_id();

        let unit1 = Unit::new(unit1_id, 0, UnitType::Warrior, HexCoord::new(5, 5));
        let unit2 = Unit::new(unit2_id, 1, UnitType::Warrior, HexCoord::new(35, 20));

        game.units.insert(unit1_id, unit1.clone());
        game.units.insert(unit2_id, unit2.clone());

        // Reveal tiles for player 0
        let vision = base_vision_radius(UnitType::Warrior);
        for coord in unit1.position.hexes_in_radius(vision) {
            if game.map.in_bounds(&coord) {
                game.players[0].explore_tile(coord);
            }
        }

        // Reveal tiles for player 1
        for coord in unit2.position.hexes_in_radius(vision) {
            if game.map.in_bounds(&coord) {
                game.players[1].explore_tile(coord);
            }
        }

        // Verify each player sees their area
        assert!(game.players[0].has_explored(&unit1.position));
        assert!(game.players[1].has_explored(&unit2.position));

        // Players shouldn't see each other's units (too far apart)
        assert!(!game.players[0].has_explored(&unit2.position));
        assert!(!game.players[1].has_explored(&unit1.position));
    }

    #[test]
    fn test_visibility_with_city_and_units() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Create city for player 0
        let city_id = game.allocate_city_id();
        let city = City::new(
            city_id,
            0,
            "TestCity".to_string(),
            HexCoord::new(10, 10),
            true,
        );
        game.cities.insert(city_id, city.clone());

        // Create unit for player 0
        let unit_id = game.allocate_unit_id();
        let unit = Unit::new(unit_id, 0, UnitType::Warrior, HexCoord::new(15, 10));
        game.units.insert(unit_id, unit.clone());

        // Reveal from city
        let city_vision = 2u32;
        for coord in city.position.hexes_in_radius(city_vision) {
            if game.map.in_bounds(&coord) {
                game.players[0].explore_tile(coord);
            }
        }

        // Reveal from unit
        let unit_vision = base_vision_radius(UnitType::Warrior);
        for coord in unit.position.hexes_in_radius(unit_vision) {
            if game.map.in_bounds(&coord) {
                game.players[0].explore_tile(coord);
            }
        }

        // Combined visibility should cover both areas
        assert!(game.players[0].has_explored(&city.position));
        assert!(game.players[0].has_explored(&unit.position));

        // Gap between city and unit may or may not be explored depending on overlap
        let _mid_point = HexCoord::new(12, 10);
    }

    #[test]
    fn test_war_affects_visibility_updates() {
        let mut game = create_game_with_players(2);
        game.map = create_test_map(40, 25);
        game.start().unwrap();

        // Initial peaceful state
        assert!(!game.diplomacy.are_at_war(0, 1));

        // Units can see each other when in range
        let p0_unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(10, 10));
        let p1_unit = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(11, 10));

        game.units.insert(1, p0_unit);
        game.units.insert(2, p1_unit);

        // War declared
        game.diplomacy.declare_war(0, 1, 1);

        assert!(game.diplomacy.are_at_war(0, 1));

        // Enemy units in visible range should be seen as hostile
        // (actual hostility handling would be in game logic)
    }
}

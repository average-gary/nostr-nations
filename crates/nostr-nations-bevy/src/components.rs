//! Bevy ECS components for Nostr Nations.
//!
//! These components wrap the core game data types and provide additional
//! ECS-specific functionality for the Bevy game engine.

use bevy::prelude::*;
use nostr_nations_core::{
    types::{CityId, PlayerId, UnitId},
    City, HexCoord, Player, Tile, Unit,
};

/// Component wrapping core Tile data for a map tile entity.
///
/// Each tile on the game map is represented as a Bevy entity with this component.
#[derive(Component, Clone, Debug, Default)]
pub struct TileComponent {
    /// The underlying tile data from the core library.
    pub tile: Tile,
}

impl TileComponent {
    /// Create a new tile component from core Tile data.
    pub fn new(tile: Tile) -> Self {
        Self { tile }
    }

    /// Get the hex coordinate of this tile.
    pub fn coord(&self) -> HexCoord {
        self.tile.coord
    }
}

impl From<Tile> for TileComponent {
    fn from(tile: Tile) -> Self {
        Self::new(tile)
    }
}

/// Component wrapping core Unit data for a unit entity.
///
/// Military and civilian units are represented as Bevy entities with this component.
#[derive(Component, Clone, Debug)]
pub struct UnitComponent {
    /// The underlying unit data from the core library.
    pub unit: Unit,
}

impl UnitComponent {
    /// Create a new unit component from core Unit data.
    pub fn new(unit: Unit) -> Self {
        Self { unit }
    }

    /// Get the unique identifier for this unit.
    pub fn id(&self) -> UnitId {
        self.unit.id
    }

    /// Get the owner player ID.
    pub fn owner(&self) -> PlayerId {
        self.unit.owner
    }

    /// Get the current position of the unit.
    pub fn position(&self) -> HexCoord {
        self.unit.position
    }

    /// Check if the unit can move this turn.
    pub fn can_move(&self) -> bool {
        self.unit.can_move()
    }

    /// Check if the unit can attack this turn.
    pub fn can_attack(&self) -> bool {
        self.unit.can_attack()
    }
}

impl From<Unit> for UnitComponent {
    fn from(unit: Unit) -> Self {
        Self::new(unit)
    }
}

/// Component wrapping core City data for a city entity.
///
/// Cities are represented as Bevy entities with this component.
#[derive(Component, Clone, Debug)]
pub struct CityComponent {
    /// The underlying city data from the core library.
    pub city: City,
}

impl CityComponent {
    /// Create a new city component from core City data.
    pub fn new(city: City) -> Self {
        Self { city }
    }

    /// Get the unique identifier for this city.
    pub fn id(&self) -> CityId {
        self.city.id
    }

    /// Get the owner player ID.
    pub fn owner(&self) -> PlayerId {
        self.city.owner
    }

    /// Get the position of the city.
    pub fn position(&self) -> HexCoord {
        self.city.position
    }

    /// Get the city's name.
    pub fn name(&self) -> &str {
        &self.city.name
    }

    /// Get the current population.
    pub fn population(&self) -> u32 {
        self.city.population
    }

    /// Check if this is the player's capital.
    pub fn is_capital(&self) -> bool {
        self.city.is_capital
    }
}

impl From<City> for CityComponent {
    fn from(city: City) -> Self {
        Self::new(city)
    }
}

/// Component wrapping core Player data for a player entity.
///
/// Each player in the game is represented as a Bevy entity with this component.
#[derive(Component, Clone, Debug)]
pub struct PlayerComponent {
    /// The underlying player data from the core library.
    pub player: Player,
}

impl PlayerComponent {
    /// Create a new player component from core Player data.
    pub fn new(player: Player) -> Self {
        Self { player }
    }

    /// Get the player's unique identifier.
    pub fn id(&self) -> PlayerId {
        self.player.id
    }

    /// Get the player's display name.
    pub fn name(&self) -> &str {
        &self.player.name
    }

    /// Get the player's current gold.
    pub fn gold(&self) -> i32 {
        self.player.gold
    }

    /// Check if the player has been eliminated.
    pub fn is_eliminated(&self) -> bool {
        self.player.eliminated
    }

    /// Check if the player has explored a given tile.
    pub fn has_explored(&self, coord: &HexCoord) -> bool {
        self.player.has_explored(coord)
    }
}

impl From<Player> for PlayerComponent {
    fn from(player: Player) -> Self {
        Self::new(player)
    }
}

/// Marker component for the currently selected entity.
///
/// Only one entity should have this component at a time.
/// Used to track which unit, city, or tile the player has selected.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct SelectionComponent {
    /// Whether this entity is the primary selection.
    pub is_primary: bool,
    /// Selection timestamp for ordering multiple selections.
    pub selected_at: f64,
}

impl SelectionComponent {
    /// Create a new primary selection component.
    pub fn primary() -> Self {
        Self {
            is_primary: true,
            selected_at: 0.0,
        }
    }

    /// Create a selection component with a timestamp.
    pub fn with_timestamp(selected_at: f64) -> Self {
        Self {
            is_primary: true,
            selected_at,
        }
    }
}

/// Component storing the hex coordinate position of an entity.
///
/// This is a convenience component that extracts just the position
/// from entities that have spatial location on the hex map.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct PositionComponent {
    /// The hex coordinate position.
    pub coord: HexCoord,
}

impl PositionComponent {
    /// Create a new position component.
    pub fn new(coord: HexCoord) -> Self {
        Self { coord }
    }

    /// Create a position at the given q, r coordinates.
    pub fn at(q: i32, r: i32) -> Self {
        Self {
            coord: HexCoord::new(q, r),
        }
    }

    /// Get the q (column) coordinate.
    pub fn q(&self) -> i32 {
        self.coord.q
    }

    /// Get the r (row) coordinate.
    pub fn r(&self) -> i32 {
        self.coord.r
    }

    /// Calculate distance to another position.
    pub fn distance(&self, other: &PositionComponent) -> u32 {
        self.coord.distance(&other.coord)
    }
}

impl From<HexCoord> for PositionComponent {
    fn from(coord: HexCoord) -> Self {
        Self::new(coord)
    }
}

/// Marker component indicating an entity is visible to the current player.
///
/// This component is added/removed based on fog of war calculations.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct VisibleComponent {
    /// Whether the tile is currently visible (in sight range).
    pub in_sight: bool,
    /// Whether the tile has been explored (seen at least once).
    pub explored: bool,
}

impl VisibleComponent {
    /// Create a visible and explored component.
    pub fn visible() -> Self {
        Self {
            in_sight: true,
            explored: true,
        }
    }

    /// Create an explored but not currently visible component.
    pub fn explored() -> Self {
        Self {
            in_sight: false,
            explored: true,
        }
    }

    /// Create a hidden (unexplored) component.
    pub fn hidden() -> Self {
        Self {
            in_sight: false,
            explored: false,
        }
    }
}

/// Marker component for entities that belong to the local player.
///
/// Used for quick filtering in queries.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct LocalPlayerOwned;

/// Marker component for entities that belong to other players.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct OtherPlayerOwned {
    /// The owner's player ID.
    pub owner: PlayerId,
}

impl OtherPlayerOwned {
    /// Create a new component with the specified owner.
    pub fn new(owner: PlayerId) -> Self {
        Self { owner }
    }
}

/// Component tracking movement animation state for units.
#[derive(Component, Clone, Debug, Default)]
pub struct MovementAnimation {
    /// Path the unit is following.
    pub path: Vec<HexCoord>,
    /// Current index in the path.
    pub current_index: usize,
    /// Progress between current and next tile (0.0 to 1.0).
    pub progress: f32,
    /// Movement speed multiplier.
    pub speed: f32,
}

impl MovementAnimation {
    /// Create a new movement animation along a path.
    pub fn new(path: Vec<HexCoord>) -> Self {
        Self {
            path,
            current_index: 0,
            progress: 0.0,
            speed: 1.0,
        }
    }

    /// Check if the animation is complete.
    pub fn is_complete(&self) -> bool {
        self.current_index >= self.path.len().saturating_sub(1) && self.progress >= 1.0
    }

    /// Get the current position along the path.
    pub fn current_position(&self) -> Option<HexCoord> {
        self.path.get(self.current_index).copied()
    }

    /// Get the next position in the path.
    pub fn next_position(&self) -> Option<HexCoord> {
        self.path.get(self.current_index + 1).copied()
    }
}

/// Component for combat animation state.
#[derive(Component, Clone, Debug)]
pub struct CombatAnimation {
    /// The attacker entity.
    pub attacker: Entity,
    /// The defender entity.
    pub defender: Entity,
    /// Animation progress (0.0 to 1.0).
    pub progress: f32,
    /// Damage dealt to defender.
    pub damage_dealt: u32,
    /// Whether the defender was destroyed.
    pub defender_destroyed: bool,
}

/// Bundle for spawning a tile entity with common components.
#[derive(Bundle, Clone, Debug)]
pub struct TileBundle {
    /// The tile data component.
    pub tile: TileComponent,
    /// The position component.
    pub position: PositionComponent,
    /// Visibility state.
    pub visible: VisibleComponent,
}

impl TileBundle {
    /// Create a new tile bundle from a core Tile.
    pub fn new(tile: Tile) -> Self {
        let coord = tile.coord;
        Self {
            tile: TileComponent::new(tile),
            position: PositionComponent::new(coord),
            visible: VisibleComponent::hidden(),
        }
    }
}

/// Bundle for spawning a unit entity with common components.
#[derive(Bundle, Clone, Debug)]
pub struct UnitBundle {
    /// The unit data component.
    pub unit: UnitComponent,
    /// The position component.
    pub position: PositionComponent,
}

impl UnitBundle {
    /// Create a new unit bundle from a core Unit.
    pub fn new(unit: Unit) -> Self {
        let position = unit.position;
        Self {
            unit: UnitComponent::new(unit),
            position: PositionComponent::new(position),
        }
    }
}

/// Bundle for spawning a city entity with common components.
#[derive(Bundle, Clone, Debug)]
pub struct CityBundle {
    /// The city data component.
    pub city: CityComponent,
    /// The position component.
    pub position: PositionComponent,
}

impl CityBundle {
    /// Create a new city bundle from a core City.
    pub fn new(city: City) -> Self {
        let position = city.position;
        Self {
            city: CityComponent::new(city),
            position: PositionComponent::new(position),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::{player::Civilization, unit::UnitType, Terrain};

    // ============================================
    // TileComponent Tests
    // ============================================

    #[test]
    fn test_tile_component_creation() {
        let tile = Tile::new(HexCoord::new(5, 5), Terrain::Grassland);
        let component = TileComponent::new(tile.clone());
        assert_eq!(component.coord(), HexCoord::new(5, 5));
        assert_eq!(component.tile.terrain, Terrain::Grassland);
    }

    #[test]
    fn test_tile_component_default() {
        let component = TileComponent::default();
        assert_eq!(component.coord(), HexCoord::new(0, 0));
    }

    #[test]
    fn test_tile_component_from_tile() {
        let tile = Tile::new(HexCoord::new(10, 20), Terrain::Desert);
        let component: TileComponent = tile.clone().into();
        assert_eq!(component.coord(), HexCoord::new(10, 20));
    }

    #[test]
    fn test_tile_component_clone() {
        let tile = Tile::new(HexCoord::new(5, 5), Terrain::Grassland);
        let component = TileComponent::new(tile);
        let cloned = component.clone();
        assert_eq!(component.coord(), cloned.coord());
    }

    // ============================================
    // UnitComponent Tests
    // ============================================

    #[test]
    fn test_unit_component_creation() {
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let component = UnitComponent::new(unit);
        assert_eq!(component.id(), 1);
        assert_eq!(component.owner(), 0);
        assert_eq!(component.position(), HexCoord::new(0, 0));
    }

    #[test]
    fn test_unit_component_can_move() {
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let component = UnitComponent::new(unit);
        assert!(component.can_move());
    }

    #[test]
    fn test_unit_component_can_attack() {
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let component = UnitComponent::new(unit);
        assert!(component.can_attack());
    }

    #[test]
    fn test_unit_component_civilian_cannot_attack() {
        let unit = Unit::new(1, 0, UnitType::Settler, HexCoord::new(0, 0));
        let component = UnitComponent::new(unit);
        assert!(!component.can_attack());
    }

    #[test]
    fn test_unit_component_from_unit() {
        let unit = Unit::new(42, 1, UnitType::Archer, HexCoord::new(3, 4));
        let component: UnitComponent = unit.into();
        assert_eq!(component.id(), 42);
        assert_eq!(component.owner(), 1);
    }

    #[test]
    fn test_unit_component_clone() {
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let component = UnitComponent::new(unit);
        let cloned = component.clone();
        assert_eq!(component.id(), cloned.id());
        assert_eq!(component.owner(), cloned.owner());
    }

    // ============================================
    // CityComponent Tests
    // ============================================

    #[test]
    fn test_city_component_creation() {
        let city = City::new(1, 0, "Rome".to_string(), HexCoord::new(5, 5), true);
        let component = CityComponent::new(city);
        assert_eq!(component.id(), 1);
        assert_eq!(component.name(), "Rome");
        assert!(component.is_capital());
        assert_eq!(component.position(), HexCoord::new(5, 5));
    }

    #[test]
    fn test_city_component_owner() {
        let city = City::new(1, 2, "Athens".to_string(), HexCoord::new(3, 3), false);
        let component = CityComponent::new(city);
        assert_eq!(component.owner(), 2);
    }

    #[test]
    fn test_city_component_population() {
        let city = City::new(1, 0, "Paris".to_string(), HexCoord::new(0, 0), false);
        let component = CityComponent::new(city);
        assert_eq!(component.population(), 1); // Default population
    }

    #[test]
    fn test_city_component_from_city() {
        let city = City::new(5, 1, "London".to_string(), HexCoord::new(7, 8), true);
        let component: CityComponent = city.into();
        assert_eq!(component.id(), 5);
        assert_eq!(component.name(), "London");
    }

    #[test]
    fn test_city_component_clone() {
        let city = City::new(1, 0, "Rome".to_string(), HexCoord::new(5, 5), true);
        let component = CityComponent::new(city);
        let cloned = component.clone();
        assert_eq!(component.id(), cloned.id());
        assert_eq!(component.name(), cloned.name());
    }

    // ============================================
    // PlayerComponent Tests
    // ============================================

    #[test]
    fn test_player_component_creation() {
        let player = Player::new(
            0,
            "npub123".to_string(),
            "TestPlayer".to_string(),
            Civilization::rome(),
        );
        let component = PlayerComponent::new(player);
        assert_eq!(component.id(), 0);
        assert_eq!(component.name(), "TestPlayer");
        assert!(!component.is_eliminated());
    }

    #[test]
    fn test_player_component_gold() {
        let mut player = Player::new(
            0,
            "npub123".to_string(),
            "TestPlayer".to_string(),
            Civilization::rome(),
        );
        player.gold = 500;
        let component = PlayerComponent::new(player);
        assert_eq!(component.gold(), 500);
    }

    #[test]
    fn test_player_component_has_explored() {
        let mut player = Player::new(
            0,
            "npub123".to_string(),
            "TestPlayer".to_string(),
            Civilization::rome(),
        );
        let coord = HexCoord::new(5, 5);
        player.explore_tile(coord);
        let component = PlayerComponent::new(player);
        assert!(component.has_explored(&coord));
        assert!(!component.has_explored(&HexCoord::new(10, 10)));
    }

    #[test]
    fn test_player_component_from_player() {
        let player = Player::new(
            1,
            "npub456".to_string(),
            "Player2".to_string(),
            Civilization::egypt(),
        );
        let component: PlayerComponent = player.into();
        assert_eq!(component.id(), 1);
        assert_eq!(component.name(), "Player2");
    }

    #[test]
    fn test_player_component_clone() {
        let player = Player::new(
            0,
            "npub123".to_string(),
            "TestPlayer".to_string(),
            Civilization::rome(),
        );
        let component = PlayerComponent::new(player);
        let cloned = component.clone();
        assert_eq!(component.id(), cloned.id());
        assert_eq!(component.name(), cloned.name());
    }

    // ============================================
    // SelectionComponent Tests
    // ============================================

    #[test]
    fn test_selection_component_primary() {
        let selection = SelectionComponent::primary();
        assert!(selection.is_primary);
        assert_eq!(selection.selected_at, 0.0);
    }

    #[test]
    fn test_selection_component_with_timestamp() {
        let selection = SelectionComponent::with_timestamp(123.456);
        assert!(selection.is_primary);
        assert_eq!(selection.selected_at, 123.456);
    }

    #[test]
    fn test_selection_component_default() {
        let selection = SelectionComponent::default();
        assert!(!selection.is_primary);
        assert_eq!(selection.selected_at, 0.0);
    }

    #[test]
    fn test_selection_component_clone() {
        let selection = SelectionComponent::with_timestamp(42.0);
        let cloned = selection;
        assert_eq!(selection.is_primary, cloned.is_primary);
        assert_eq!(selection.selected_at, cloned.selected_at);
    }

    // ============================================
    // PositionComponent Tests
    // ============================================

    #[test]
    fn test_position_component_new() {
        let coord = HexCoord::new(5, 10);
        let pos = PositionComponent::new(coord);
        assert_eq!(pos.coord, coord);
    }

    #[test]
    fn test_position_component_at() {
        let pos = PositionComponent::at(3, 7);
        assert_eq!(pos.q(), 3);
        assert_eq!(pos.r(), 7);
    }

    #[test]
    fn test_position_component_distance() {
        let pos1 = PositionComponent::at(0, 0);
        let pos2 = PositionComponent::at(1, 0);
        assert_eq!(pos1.distance(&pos2), 1);

        let pos3 = PositionComponent::at(2, 2);
        assert!(pos1.distance(&pos3) > 1);
    }

    #[test]
    fn test_position_component_default() {
        let pos = PositionComponent::default();
        assert_eq!(pos.q(), 0);
        assert_eq!(pos.r(), 0);
    }

    #[test]
    fn test_position_component_from_hexcoord() {
        let coord = HexCoord::new(15, 25);
        let pos: PositionComponent = coord.into();
        assert_eq!(pos.coord, coord);
    }

    #[test]
    fn test_position_component_equality() {
        let pos1 = PositionComponent::at(5, 5);
        let pos2 = PositionComponent::at(5, 5);
        let pos3 = PositionComponent::at(5, 6);
        assert_eq!(pos1, pos2);
        assert_ne!(pos1, pos3);
    }

    #[test]
    fn test_position_component_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PositionComponent::at(1, 2));
        set.insert(PositionComponent::at(3, 4));
        assert!(set.contains(&PositionComponent::at(1, 2)));
        assert!(!set.contains(&PositionComponent::at(5, 6)));
    }

    // ============================================
    // VisibleComponent Tests
    // ============================================

    #[test]
    fn test_visible_component_visible() {
        let visible = VisibleComponent::visible();
        assert!(visible.in_sight);
        assert!(visible.explored);
    }

    #[test]
    fn test_visible_component_explored() {
        let explored = VisibleComponent::explored();
        assert!(!explored.in_sight);
        assert!(explored.explored);
    }

    #[test]
    fn test_visible_component_hidden() {
        let hidden = VisibleComponent::hidden();
        assert!(!hidden.in_sight);
        assert!(!hidden.explored);
    }

    #[test]
    fn test_visible_component_default() {
        let default = VisibleComponent::default();
        assert!(!default.in_sight);
        assert!(!default.explored);
    }

    #[test]
    fn test_visible_component_clone() {
        let visible = VisibleComponent::visible();
        let cloned = visible;
        assert_eq!(visible.in_sight, cloned.in_sight);
        assert_eq!(visible.explored, cloned.explored);
    }

    // ============================================
    // LocalPlayerOwned / OtherPlayerOwned Tests
    // ============================================

    #[test]
    fn test_local_player_owned_default() {
        let marker = LocalPlayerOwned;
        let _ = marker; // Just verify it can be created
    }

    #[test]
    fn test_other_player_owned_new() {
        let owned = OtherPlayerOwned::new(2);
        assert_eq!(owned.owner, 2);
    }

    #[test]
    fn test_other_player_owned_default() {
        let owned = OtherPlayerOwned::default();
        assert_eq!(owned.owner, 0);
    }

    // ============================================
    // MovementAnimation Tests
    // ============================================

    #[test]
    fn test_movement_animation_new() {
        let path = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];
        let anim = MovementAnimation::new(path.clone());
        assert_eq!(anim.path, path);
        assert_eq!(anim.current_index, 0);
        assert_eq!(anim.progress, 0.0);
        assert_eq!(anim.speed, 1.0);
    }

    #[test]
    fn test_movement_animation_is_complete_false() {
        let path = vec![HexCoord::new(0, 0), HexCoord::new(1, 0)];
        let anim = MovementAnimation::new(path);
        assert!(!anim.is_complete());
    }

    #[test]
    fn test_movement_animation_is_complete_true() {
        let path = vec![HexCoord::new(0, 0), HexCoord::new(1, 0)];
        let mut anim = MovementAnimation::new(path);
        anim.current_index = 1;
        anim.progress = 1.0;
        assert!(anim.is_complete());
    }

    #[test]
    fn test_movement_animation_current_position() {
        let path = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];
        let mut anim = MovementAnimation::new(path);
        assert_eq!(anim.current_position(), Some(HexCoord::new(0, 0)));

        anim.current_index = 1;
        assert_eq!(anim.current_position(), Some(HexCoord::new(1, 0)));
    }

    #[test]
    fn test_movement_animation_next_position() {
        let path = vec![
            HexCoord::new(0, 0),
            HexCoord::new(1, 0),
            HexCoord::new(2, 0),
        ];
        let mut anim = MovementAnimation::new(path);
        assert_eq!(anim.next_position(), Some(HexCoord::new(1, 0)));

        anim.current_index = 2;
        assert_eq!(anim.next_position(), None);
    }

    #[test]
    fn test_movement_animation_empty_path() {
        let anim = MovementAnimation::new(vec![]);
        assert_eq!(anim.current_position(), None);
        assert_eq!(anim.next_position(), None);
    }

    #[test]
    fn test_movement_animation_single_position() {
        let path = vec![HexCoord::new(5, 5)];
        let mut anim = MovementAnimation::new(path);
        assert_eq!(anim.current_position(), Some(HexCoord::new(5, 5)));
        assert_eq!(anim.next_position(), None);

        // Single position path is complete when progress >= 1.0
        anim.progress = 1.0;
        assert!(anim.is_complete());
    }

    #[test]
    fn test_movement_animation_default() {
        let anim = MovementAnimation::default();
        assert!(anim.path.is_empty());
        assert_eq!(anim.current_index, 0);
        assert_eq!(anim.progress, 0.0);
        assert_eq!(anim.speed, 0.0);
    }

    // ============================================
    // CombatAnimation Tests
    // ============================================

    #[test]
    fn test_combat_animation_creation() {
        let attacker = Entity::from_raw(1);
        let defender = Entity::from_raw(2);
        let anim = CombatAnimation {
            attacker,
            defender,
            progress: 0.5,
            damage_dealt: 25,
            defender_destroyed: false,
        };
        assert_eq!(anim.attacker, attacker);
        assert_eq!(anim.defender, defender);
        assert_eq!(anim.progress, 0.5);
        assert_eq!(anim.damage_dealt, 25);
        assert!(!anim.defender_destroyed);
    }

    #[test]
    fn test_combat_animation_clone() {
        let anim = CombatAnimation {
            attacker: Entity::from_raw(1),
            defender: Entity::from_raw(2),
            progress: 0.75,
            damage_dealt: 50,
            defender_destroyed: true,
        };
        let cloned = anim.clone();
        assert_eq!(anim.attacker, cloned.attacker);
        assert_eq!(anim.defender_destroyed, cloned.defender_destroyed);
    }

    // ============================================
    // TileBundle Tests
    // ============================================

    #[test]
    fn test_tile_bundle_new() {
        let tile = Tile::new(HexCoord::new(3, 3), Terrain::Plains);
        let bundle = TileBundle::new(tile);
        assert_eq!(bundle.position.coord, HexCoord::new(3, 3));
        assert!(!bundle.visible.in_sight);
        assert!(!bundle.visible.explored);
    }

    #[test]
    fn test_tile_bundle_clone() {
        let tile = Tile::new(HexCoord::new(5, 5), Terrain::Plains);
        let bundle = TileBundle::new(tile);
        let cloned = bundle.clone();
        assert_eq!(bundle.position.coord, cloned.position.coord);
    }

    #[test]
    fn test_tile_bundle_components() {
        let tile = Tile::new(HexCoord::new(7, 8), Terrain::Ocean);
        let bundle = TileBundle::new(tile.clone());

        // Verify all components are properly initialized
        assert_eq!(bundle.tile.coord(), tile.coord);
        assert_eq!(bundle.position.coord, tile.coord);
        assert!(!bundle.visible.in_sight);
        assert!(!bundle.visible.explored);
    }

    // ============================================
    // UnitBundle Tests
    // ============================================

    #[test]
    fn test_unit_bundle_new() {
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(5, 5));
        let bundle = UnitBundle::new(unit);
        assert_eq!(bundle.position.coord, HexCoord::new(5, 5));
        assert_eq!(bundle.unit.id(), 1);
    }

    #[test]
    fn test_unit_bundle_clone() {
        let unit = Unit::new(1, 0, UnitType::Archer, HexCoord::new(3, 4));
        let bundle = UnitBundle::new(unit);
        let cloned = bundle.clone();
        assert_eq!(bundle.unit.id(), cloned.unit.id());
        assert_eq!(bundle.position.coord, cloned.position.coord);
    }

    #[test]
    fn test_unit_bundle_different_unit_types() {
        let warrior = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let settler = Unit::new(2, 0, UnitType::Settler, HexCoord::new(1, 1));

        let warrior_bundle = UnitBundle::new(warrior);
        let settler_bundle = UnitBundle::new(settler);

        assert_eq!(warrior_bundle.unit.id(), 1);
        assert_eq!(settler_bundle.unit.id(), 2);
    }

    // ============================================
    // CityBundle Tests
    // ============================================

    #[test]
    fn test_city_bundle_new() {
        let city = City::new(1, 0, "TestCity".to_string(), HexCoord::new(10, 10), true);
        let bundle = CityBundle::new(city);
        assert_eq!(bundle.position.coord, HexCoord::new(10, 10));
        assert_eq!(bundle.city.id(), 1);
        assert!(bundle.city.is_capital());
    }

    #[test]
    fn test_city_bundle_clone() {
        let city = City::new(5, 1, "Cloned".to_string(), HexCoord::new(7, 8), false);
        let bundle = CityBundle::new(city);
        let cloned = bundle.clone();
        assert_eq!(bundle.city.id(), cloned.city.id());
        assert_eq!(bundle.city.name(), cloned.city.name());
    }

    #[test]
    fn test_city_bundle_non_capital() {
        let city = City::new(2, 0, "Secondary".to_string(), HexCoord::new(15, 15), false);
        let bundle = CityBundle::new(city);
        assert!(!bundle.city.is_capital());
    }

    // ============================================
    // Integration tests with Bevy World
    // ============================================

    #[test]
    fn test_spawn_tile_bundle_in_world() {
        let mut world = World::new();
        let tile = Tile::new(HexCoord::new(0, 0), Terrain::Grassland);
        let bundle = TileBundle::new(tile);

        let entity = world.spawn(bundle).id();

        assert!(world.get::<TileComponent>(entity).is_some());
        assert!(world.get::<PositionComponent>(entity).is_some());
        assert!(world.get::<VisibleComponent>(entity).is_some());
    }

    #[test]
    fn test_spawn_unit_bundle_in_world() {
        let mut world = World::new();
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(5, 5));
        let bundle = UnitBundle::new(unit);

        let entity = world.spawn(bundle).id();

        assert!(world.get::<UnitComponent>(entity).is_some());
        assert!(world.get::<PositionComponent>(entity).is_some());
    }

    #[test]
    fn test_spawn_city_bundle_in_world() {
        let mut world = World::new();
        let city = City::new(1, 0, "Test".to_string(), HexCoord::new(3, 3), true);
        let bundle = CityBundle::new(city);

        let entity = world.spawn(bundle).id();

        assert!(world.get::<CityComponent>(entity).is_some());
        assert!(world.get::<PositionComponent>(entity).is_some());
    }

    #[test]
    fn test_query_components_in_world() {
        let mut world = World::new();

        // Spawn multiple units
        for i in 0..3 {
            let unit = Unit::new(i, 0, UnitType::Warrior, HexCoord::new(i as i32, 0));
            world.spawn(UnitBundle::new(unit));
        }

        // Query and count
        let mut query = world.query::<(&UnitComponent, &PositionComponent)>();
        let count = query.iter(&world).count();
        assert_eq!(count, 3);
    }

    #[test]
    fn test_modify_component_in_world() {
        let mut world = World::new();
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let entity = world.spawn(UnitBundle::new(unit)).id();

        // Modify position
        if let Some(mut pos) = world.get_mut::<PositionComponent>(entity) {
            pos.coord = HexCoord::new(5, 5);
        }

        // Verify change
        let pos = world.get::<PositionComponent>(entity).unwrap();
        assert_eq!(pos.coord, HexCoord::new(5, 5));
    }
}

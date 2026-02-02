//! Bevy ECS resources for Nostr Nations.
//!
//! Resources are singleton data that can be accessed by any system.
//! These resources hold game-wide state and configuration.

use bevy::prelude::*;
use nostr_nations_core::{
    settings::{Difficulty, GameSpeed},
    types::{CityId, PlayerId, UnitId},
    GameEngine, GameSettings, GameState, HexCoord,
};

/// Main game state resource holding the core GameEngine.
///
/// This is the primary interface between Bevy systems and the
/// deterministic game logic in nostr-nations-core.
#[derive(Resource)]
pub struct GameStateResource {
    /// The game engine that processes all game actions.
    pub engine: GameEngine,
}

impl GameStateResource {
    /// Create a new game state resource with the given settings.
    pub fn new(settings: GameSettings, seed: [u8; 32]) -> Self {
        Self {
            engine: GameEngine::new(settings, seed),
        }
    }

    /// Get a reference to the current game state.
    pub fn state(&self) -> &GameState {
        &self.engine.state
    }

    /// Get a mutable reference to the current game state.
    pub fn state_mut(&mut self) -> &mut GameState {
        &mut self.engine.state
    }

    /// Get the current turn number.
    pub fn turn(&self) -> u32 {
        self.engine.turn()
    }

    /// Get the current player's ID.
    pub fn current_player(&self) -> PlayerId {
        self.engine.state.current_player
    }

    /// Check if the game has ended.
    pub fn is_ended(&self) -> bool {
        self.engine.is_ended()
    }
}

impl Default for GameStateResource {
    fn default() -> Self {
        let settings = GameSettings::default();
        Self::new(settings, [0u8; 32])
    }
}

/// Resource tracking the currently selected entity.
///
/// This tracks which unit, city, or tile the player has selected
/// for actions like movement or production.
#[derive(Resource, Clone, Debug, Default)]
pub struct SelectedEntity {
    /// The selected entity, if any.
    pub entity: Option<Entity>,
    /// Type of the selected entity.
    pub selection_type: SelectionType,
    /// Hex coordinate of the selection (for tiles).
    pub coord: Option<HexCoord>,
    /// Unit ID if a unit is selected.
    pub unit_id: Option<UnitId>,
    /// City ID if a city is selected.
    pub city_id: Option<CityId>,
}

impl SelectedEntity {
    /// Create an empty selection (nothing selected).
    pub fn none() -> Self {
        Self::default()
    }

    /// Select a unit.
    pub fn unit(entity: Entity, unit_id: UnitId, coord: HexCoord) -> Self {
        Self {
            entity: Some(entity),
            selection_type: SelectionType::Unit,
            coord: Some(coord),
            unit_id: Some(unit_id),
            city_id: None,
        }
    }

    /// Select a city.
    pub fn city(entity: Entity, city_id: CityId, coord: HexCoord) -> Self {
        Self {
            entity: Some(entity),
            selection_type: SelectionType::City,
            coord: Some(coord),
            unit_id: None,
            city_id: Some(city_id),
        }
    }

    /// Select a tile.
    pub fn tile(entity: Entity, coord: HexCoord) -> Self {
        Self {
            entity: Some(entity),
            selection_type: SelectionType::Tile,
            coord: Some(coord),
            unit_id: None,
            city_id: None,
        }
    }

    /// Clear the selection.
    pub fn clear(&mut self) {
        *self = Self::none();
    }

    /// Check if anything is selected.
    pub fn has_selection(&self) -> bool {
        self.entity.is_some()
    }

    /// Check if a unit is selected.
    pub fn is_unit(&self) -> bool {
        matches!(self.selection_type, SelectionType::Unit)
    }

    /// Check if a city is selected.
    pub fn is_city(&self) -> bool {
        matches!(self.selection_type, SelectionType::City)
    }

    /// Check if a tile is selected.
    pub fn is_tile(&self) -> bool {
        matches!(self.selection_type, SelectionType::Tile)
    }
}

/// Type of entity that can be selected.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SelectionType {
    /// No selection.
    #[default]
    None,
    /// A unit is selected.
    Unit,
    /// A city is selected.
    City,
    /// A map tile is selected.
    Tile,
}

/// Resource tracking turn state and timing.
#[derive(Resource, Clone, Debug)]
pub struct CurrentTurn {
    /// Current turn number.
    pub turn: u32,
    /// Player whose turn it currently is.
    pub current_player: PlayerId,
    /// Time remaining for this turn (if timer enabled).
    pub time_remaining: Option<f32>,
    /// Whether the local player has ended their turn.
    pub turn_ended: bool,
    /// Whether we're waiting for other players.
    pub waiting_for_players: bool,
}

impl CurrentTurn {
    /// Create turn state for the beginning of a game.
    pub fn new(turn: u32, current_player: PlayerId) -> Self {
        Self {
            turn,
            current_player,
            time_remaining: None,
            turn_ended: false,
            waiting_for_players: false,
        }
    }

    /// Create turn state with a timer.
    pub fn with_timer(turn: u32, current_player: PlayerId, seconds: f32) -> Self {
        Self {
            turn,
            current_player,
            time_remaining: Some(seconds),
            turn_ended: false,
            waiting_for_players: false,
        }
    }

    /// Check if it's the specified player's turn.
    pub fn is_player_turn(&self, player_id: PlayerId) -> bool {
        self.current_player == player_id && !self.turn_ended
    }

    /// Mark the turn as ended for the local player.
    pub fn end_turn(&mut self) {
        self.turn_ended = true;
    }

    /// Advance to the next turn.
    pub fn next_turn(&mut self, turn: u32, current_player: PlayerId) {
        self.turn = turn;
        self.current_player = current_player;
        self.turn_ended = false;
        self.waiting_for_players = false;
    }

    /// Update timer (returns true if time expired).
    pub fn update_timer(&mut self, delta: f32) -> bool {
        if let Some(ref mut time) = self.time_remaining {
            *time -= delta;
            if *time <= 0.0 {
                *time = 0.0;
                return true;
            }
        }
        false
    }
}

impl Default for CurrentTurn {
    fn default() -> Self {
        Self::new(1, 0)
    }
}

/// Resource holding game configuration and settings.
///
/// These settings are configured at game creation and remain
/// constant throughout the game.
#[derive(Resource, Clone, Debug)]
pub struct GameSettingsResource {
    /// The game settings.
    pub settings: GameSettings,
    /// Local player's ID in this game.
    pub local_player_id: PlayerId,
    /// Whether this is a local (offline) or networked game.
    pub is_networked: bool,
    /// Whether fog of war is enabled.
    pub fog_of_war: bool,
    /// Game speed multiplier.
    pub game_speed: GameSpeed,
    /// Difficulty setting.
    pub difficulty: Difficulty,
}

impl GameSettingsResource {
    /// Create settings for a local game.
    pub fn local(settings: GameSettings, local_player_id: PlayerId) -> Self {
        let fog_of_war = settings.fog_of_war;
        let game_speed = settings.game_speed;
        let difficulty = settings.difficulty;
        Self {
            settings,
            local_player_id,
            is_networked: false,
            fog_of_war,
            game_speed,
            difficulty,
        }
    }

    /// Create settings for a networked game.
    pub fn networked(settings: GameSettings, local_player_id: PlayerId) -> Self {
        let fog_of_war = settings.fog_of_war;
        let game_speed = settings.game_speed;
        let difficulty = settings.difficulty;
        Self {
            settings,
            local_player_id,
            is_networked: true,
            fog_of_war,
            game_speed,
            difficulty,
        }
    }

    /// Check if fog of war should be applied.
    pub fn has_fog_of_war(&self) -> bool {
        self.fog_of_war
    }

    /// Get the production multiplier based on game speed.
    pub fn production_multiplier(&self) -> f32 {
        self.game_speed.production_multiplier()
    }

    /// Get the research multiplier based on game speed.
    pub fn research_multiplier(&self) -> f32 {
        self.game_speed.research_multiplier()
    }
}

impl Default for GameSettingsResource {
    fn default() -> Self {
        Self::local(GameSettings::default(), 0)
    }
}

/// Resource for camera and viewport state.
#[derive(Resource, Clone, Debug)]
pub struct CameraState {
    /// Current camera position (world coordinates).
    pub position: Vec2,
    /// Current zoom level.
    pub zoom: f32,
    /// Minimum zoom level.
    pub min_zoom: f32,
    /// Maximum zoom level.
    pub max_zoom: f32,
    /// Whether the camera is currently being dragged.
    pub is_dragging: bool,
    /// Target position for smooth camera movement.
    pub target_position: Option<Vec2>,
    /// Camera smoothing factor (0.0 = instant, 1.0 = very slow).
    pub smoothing: f32,
}

impl CameraState {
    /// Create a new camera state with default values.
    pub fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            min_zoom: 0.25,
            max_zoom: 4.0,
            is_dragging: false,
            target_position: None,
            smoothing: 0.1,
        }
    }

    /// Set the camera zoom level, clamped to valid range.
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Zoom in by a factor.
    pub fn zoom_in(&mut self, factor: f32) {
        self.set_zoom(self.zoom * factor);
    }

    /// Zoom out by a factor.
    pub fn zoom_out(&mut self, factor: f32) {
        self.set_zoom(self.zoom / factor);
    }

    /// Move camera to center on a position.
    pub fn center_on(&mut self, position: Vec2) {
        self.target_position = Some(position);
    }

    /// Instantly move camera to a position.
    pub fn jump_to(&mut self, position: Vec2) {
        self.position = position;
        self.target_position = None;
    }
}

impl Default for CameraState {
    fn default() -> Self {
        Self::new()
    }
}

/// Resource tracking pending actions that need confirmation.
#[derive(Resource, Clone, Debug, Default)]
pub struct PendingAction {
    /// The action waiting for confirmation.
    pub action: Option<PendingActionType>,
    /// Target coordinate for the action.
    pub target: Option<HexCoord>,
}

/// Types of actions that may require confirmation or additional input.
#[derive(Clone, Debug)]
pub enum PendingActionType {
    /// Moving a unit to a destination.
    MoveUnit {
        unit_id: UnitId,
        path: Vec<HexCoord>,
    },
    /// Attacking an enemy unit.
    AttackUnit {
        attacker_id: UnitId,
        defender_id: UnitId,
    },
    /// Founding a city.
    FoundCity { settler_id: UnitId },
    /// Attacking a city.
    AttackCity {
        attacker_id: UnitId,
        city_id: CityId,
    },
}

impl PendingAction {
    /// Clear any pending action.
    pub fn clear(&mut self) {
        self.action = None;
        self.target = None;
    }

    /// Check if there's a pending action.
    pub fn has_pending(&self) -> bool {
        self.action.is_some()
    }
}

/// Resource for UI state that persists across frames.
#[derive(Resource, Clone, Debug, Default)]
pub struct UiState {
    /// Whether the main menu is open.
    pub menu_open: bool,
    /// Whether the city production panel is open.
    pub production_panel_open: bool,
    /// Whether the tech tree is open.
    pub tech_tree_open: bool,
    /// Whether the diplomacy panel is open.
    pub diplomacy_open: bool,
    /// Currently hovered hex coordinate.
    pub hovered_hex: Option<HexCoord>,
    /// Whether we're in unit command mode.
    pub command_mode: bool,
    /// Current tooltip text.
    pub tooltip: Option<String>,
}

impl UiState {
    /// Close all panels.
    pub fn close_all(&mut self) {
        self.menu_open = false;
        self.production_panel_open = false;
        self.tech_tree_open = false;
        self.diplomacy_open = false;
    }

    /// Toggle the main menu.
    pub fn toggle_menu(&mut self) {
        self.menu_open = !self.menu_open;
        if self.menu_open {
            self.production_panel_open = false;
            self.tech_tree_open = false;
            self.diplomacy_open = false;
        }
    }
}

/// Lookup table mapping hex coordinates to tile entities.
#[derive(Resource, Clone, Debug, Default)]
pub struct TileEntityMap {
    /// Map from hex coordinate to entity.
    tiles: std::collections::HashMap<HexCoord, Entity>,
}

impl TileEntityMap {
    /// Insert a tile entity.
    pub fn insert(&mut self, coord: HexCoord, entity: Entity) {
        self.tiles.insert(coord, entity);
    }

    /// Get a tile entity by coordinate.
    pub fn get(&self, coord: &HexCoord) -> Option<Entity> {
        self.tiles.get(coord).copied()
    }

    /// Remove a tile entity.
    pub fn remove(&mut self, coord: &HexCoord) -> Option<Entity> {
        self.tiles.remove(coord)
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.tiles.clear();
    }

    /// Get the number of tiles.
    pub fn len(&self) -> usize {
        self.tiles.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }
}

/// Lookup table mapping unit IDs to entities.
#[derive(Resource, Clone, Debug, Default)]
pub struct UnitEntityMap {
    /// Map from unit ID to entity.
    pub units: std::collections::HashMap<UnitId, Entity>,
}

impl UnitEntityMap {
    /// Insert a unit entity.
    pub fn insert(&mut self, unit_id: UnitId, entity: Entity) {
        self.units.insert(unit_id, entity);
    }

    /// Get a unit entity by ID.
    pub fn get(&self, unit_id: UnitId) -> Option<Entity> {
        self.units.get(&unit_id).copied()
    }

    /// Remove a unit entity.
    pub fn remove(&mut self, unit_id: UnitId) -> Option<Entity> {
        self.units.remove(&unit_id)
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.units.clear();
    }
}

/// Lookup table mapping city IDs to entities.
#[derive(Resource, Clone, Debug, Default)]
pub struct CityEntityMap {
    /// Map from city ID to entity.
    pub cities: std::collections::HashMap<CityId, Entity>,
}

impl CityEntityMap {
    /// Insert a city entity.
    pub fn insert(&mut self, city_id: CityId, entity: Entity) {
        self.cities.insert(city_id, entity);
    }

    /// Get a city entity by ID.
    pub fn get(&self, city_id: CityId) -> Option<Entity> {
        self.cities.get(&city_id).copied()
    }

    /// Remove a city entity.
    pub fn remove(&mut self, city_id: CityId) -> Option<Entity> {
        self.cities.remove(&city_id)
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.cities.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================
    // GameStateResource Tests
    // ============================================

    #[test]
    fn test_game_state_resource_new() {
        let settings = GameSettings::new("Test Game".to_string());
        let resource = GameStateResource::new(settings, [42u8; 32]);
        assert_eq!(resource.turn(), 0);
        assert!(!resource.is_ended());
    }

    #[test]
    fn test_game_state_resource_default() {
        let resource = GameStateResource::default();
        assert_eq!(resource.turn(), 0);
        assert!(!resource.is_ended());
    }

    #[test]
    fn test_game_state_resource_state_access() {
        let settings = GameSettings::new("Test Game".to_string());
        let resource = GameStateResource::new(settings, [0u8; 32]);

        let state = resource.state();
        assert_eq!(state.turn, 0);
    }

    #[test]
    fn test_game_state_resource_state_mut_access() {
        let settings = GameSettings::new("Test Game".to_string());
        let mut resource = GameStateResource::new(settings, [0u8; 32]);

        let state = resource.state_mut();
        state.turn = 5;

        assert_eq!(resource.turn(), 5);
    }

    #[test]
    fn test_game_state_resource_current_player() {
        let settings = GameSettings::new("Test Game".to_string());
        let resource = GameStateResource::new(settings, [0u8; 32]);

        let player_id = resource.current_player();
        assert_eq!(player_id, 0);
    }

    #[test]
    fn test_game_state_resource_different_seeds() {
        let settings1 = GameSettings::new("Test 1".to_string());
        let settings2 = GameSettings::new("Test 2".to_string());

        let resource1 = GameStateResource::new(settings1, [1u8; 32]);
        let resource2 = GameStateResource::new(settings2, [2u8; 32]);

        // Both should start at turn 0 regardless of seed
        assert_eq!(resource1.turn(), 0);
        assert_eq!(resource2.turn(), 0);
    }

    // ============================================
    // SelectedEntity Tests
    // ============================================

    #[test]
    fn test_selected_entity_none() {
        let selection = SelectedEntity::none();
        assert!(!selection.has_selection());
        assert!(selection.entity.is_none());
        assert!(selection.coord.is_none());
        assert!(selection.unit_id.is_none());
        assert!(selection.city_id.is_none());
    }

    #[test]
    fn test_selected_entity_unit() {
        let entity = Entity::from_raw(1);
        let selection = SelectedEntity::unit(entity, 42, HexCoord::new(5, 5));

        assert!(selection.has_selection());
        assert!(selection.is_unit());
        assert!(!selection.is_city());
        assert!(!selection.is_tile());
        assert_eq!(selection.entity, Some(entity));
        assert_eq!(selection.unit_id, Some(42));
        assert_eq!(selection.city_id, None);
        assert_eq!(selection.coord, Some(HexCoord::new(5, 5)));
    }

    #[test]
    fn test_selected_entity_city() {
        let entity = Entity::from_raw(2);
        let selection = SelectedEntity::city(entity, 100, HexCoord::new(10, 10));

        assert!(selection.has_selection());
        assert!(!selection.is_unit());
        assert!(selection.is_city());
        assert!(!selection.is_tile());
        assert_eq!(selection.entity, Some(entity));
        assert_eq!(selection.city_id, Some(100));
        assert_eq!(selection.unit_id, None);
    }

    #[test]
    fn test_selected_entity_tile() {
        let entity = Entity::from_raw(3);
        let selection = SelectedEntity::tile(entity, HexCoord::new(7, 8));

        assert!(selection.has_selection());
        assert!(!selection.is_unit());
        assert!(!selection.is_city());
        assert!(selection.is_tile());
        assert_eq!(selection.entity, Some(entity));
        assert_eq!(selection.unit_id, None);
        assert_eq!(selection.city_id, None);
        assert_eq!(selection.coord, Some(HexCoord::new(7, 8)));
    }

    #[test]
    fn test_selected_entity_clear() {
        let entity = Entity::from_raw(1);
        let mut selection = SelectedEntity::unit(entity, 42, HexCoord::new(5, 5));

        assert!(selection.has_selection());

        selection.clear();

        assert!(!selection.has_selection());
        assert!(selection.entity.is_none());
    }

    #[test]
    fn test_selected_entity_default() {
        let selection = SelectedEntity::default();
        assert!(!selection.has_selection());
        assert_eq!(selection.selection_type, SelectionType::None);
    }

    #[test]
    fn test_selected_entity_clone() {
        let entity = Entity::from_raw(1);
        let selection = SelectedEntity::unit(entity, 42, HexCoord::new(5, 5));
        let cloned = selection.clone();

        assert_eq!(selection.entity, cloned.entity);
        assert_eq!(selection.unit_id, cloned.unit_id);
    }

    // ============================================
    // SelectionType Tests
    // ============================================

    #[test]
    fn test_selection_type_default() {
        let st = SelectionType::default();
        assert_eq!(st, SelectionType::None);
    }

    #[test]
    fn test_selection_type_variants() {
        let none = SelectionType::None;
        let unit = SelectionType::Unit;
        let city = SelectionType::City;
        let tile = SelectionType::Tile;

        assert_ne!(none, unit);
        assert_ne!(unit, city);
        assert_ne!(city, tile);
    }

    // ============================================
    // CurrentTurn Tests
    // ============================================

    #[test]
    fn test_current_turn_new() {
        let turn = CurrentTurn::new(1, 0);
        assert_eq!(turn.turn, 1);
        assert_eq!(turn.current_player, 0);
        assert!(turn.time_remaining.is_none());
        assert!(!turn.turn_ended);
        assert!(!turn.waiting_for_players);
    }

    #[test]
    fn test_current_turn_with_timer() {
        let turn = CurrentTurn::with_timer(1, 0, 60.0);
        assert_eq!(turn.turn, 1);
        assert_eq!(turn.time_remaining, Some(60.0));
    }

    #[test]
    fn test_current_turn_is_player_turn() {
        let turn = CurrentTurn::new(1, 0);
        assert!(turn.is_player_turn(0));
        assert!(!turn.is_player_turn(1));
        assert!(!turn.is_player_turn(2));
    }

    #[test]
    fn test_current_turn_is_player_turn_after_end() {
        let mut turn = CurrentTurn::new(1, 0);
        turn.end_turn();

        // Even the current player can't act after ending turn
        assert!(!turn.is_player_turn(0));
    }

    #[test]
    fn test_current_turn_end_turn() {
        let mut turn = CurrentTurn::new(1, 0);
        assert!(!turn.turn_ended);

        turn.end_turn();

        assert!(turn.turn_ended);
    }

    #[test]
    fn test_current_turn_next_turn() {
        let mut turn = CurrentTurn::new(1, 0);
        turn.end_turn();
        turn.waiting_for_players = true;

        turn.next_turn(2, 1);

        assert_eq!(turn.turn, 2);
        assert_eq!(turn.current_player, 1);
        assert!(!turn.turn_ended);
        assert!(!turn.waiting_for_players);
    }

    #[test]
    fn test_current_turn_update_timer_not_expired() {
        let mut turn = CurrentTurn::with_timer(1, 0, 60.0);

        let expired = turn.update_timer(30.0);

        assert!(!expired);
        assert_eq!(turn.time_remaining, Some(30.0));
    }

    #[test]
    fn test_current_turn_update_timer_expired() {
        let mut turn = CurrentTurn::with_timer(1, 0, 60.0);

        let expired = turn.update_timer(65.0);

        assert!(expired);
        assert_eq!(turn.time_remaining, Some(0.0));
    }

    #[test]
    fn test_current_turn_update_timer_exact() {
        let mut turn = CurrentTurn::with_timer(1, 0, 60.0);

        let expired = turn.update_timer(60.0);

        assert!(expired);
        assert_eq!(turn.time_remaining, Some(0.0));
    }

    #[test]
    fn test_current_turn_update_timer_no_timer() {
        let mut turn = CurrentTurn::new(1, 0);

        let expired = turn.update_timer(100.0);

        assert!(!expired);
        assert!(turn.time_remaining.is_none());
    }

    #[test]
    fn test_current_turn_default() {
        let turn = CurrentTurn::default();
        assert_eq!(turn.turn, 1);
        assert_eq!(turn.current_player, 0);
    }

    #[test]
    fn test_current_turn_clone() {
        let turn = CurrentTurn::with_timer(5, 2, 45.0);
        let cloned = turn.clone();

        assert_eq!(turn.turn, cloned.turn);
        assert_eq!(turn.current_player, cloned.current_player);
        assert_eq!(turn.time_remaining, cloned.time_remaining);
    }

    // ============================================
    // GameSettingsResource Tests
    // ============================================

    #[test]
    fn test_game_settings_resource_local() {
        let settings = GameSettings::new("Test".to_string());
        let resource = GameSettingsResource::local(settings, 0);

        assert!(!resource.is_networked);
        assert_eq!(resource.local_player_id, 0);
    }

    #[test]
    fn test_game_settings_resource_networked() {
        let settings = GameSettings::new("Networked Test".to_string());
        let resource = GameSettingsResource::networked(settings, 2);

        assert!(resource.is_networked);
        assert_eq!(resource.local_player_id, 2);
    }

    #[test]
    fn test_game_settings_resource_has_fog_of_war() {
        let mut settings = GameSettings::new("Test".to_string());
        settings.fog_of_war = true;
        let resource = GameSettingsResource::local(settings, 0);

        assert!(resource.has_fog_of_war());
    }

    #[test]
    fn test_game_settings_resource_no_fog_of_war() {
        let mut settings = GameSettings::new("Test".to_string());
        settings.fog_of_war = false;
        let resource = GameSettingsResource::local(settings, 0);

        assert!(!resource.has_fog_of_war());
    }

    #[test]
    fn test_game_settings_resource_production_multiplier() {
        let settings = GameSettings::new("Test".to_string());
        let resource = GameSettingsResource::local(settings, 0);

        let multiplier = resource.production_multiplier();
        assert!(multiplier > 0.0);
    }

    #[test]
    fn test_game_settings_resource_research_multiplier() {
        let settings = GameSettings::new("Test".to_string());
        let resource = GameSettingsResource::local(settings, 0);

        let multiplier = resource.research_multiplier();
        assert!(multiplier > 0.0);
    }

    #[test]
    fn test_game_settings_resource_default() {
        let resource = GameSettingsResource::default();

        assert!(!resource.is_networked);
        assert_eq!(resource.local_player_id, 0);
    }

    #[test]
    fn test_game_settings_resource_clone() {
        let settings = GameSettings::new("Test".to_string());
        let resource = GameSettingsResource::local(settings, 1);
        let cloned = resource.clone();

        assert_eq!(resource.local_player_id, cloned.local_player_id);
        assert_eq!(resource.is_networked, cloned.is_networked);
    }

    // ============================================
    // CameraState Tests
    // ============================================

    #[test]
    fn test_camera_state_new() {
        let camera = CameraState::new();

        assert_eq!(camera.position, Vec2::ZERO);
        assert_eq!(camera.zoom, 1.0);
        assert_eq!(camera.min_zoom, 0.25);
        assert_eq!(camera.max_zoom, 4.0);
        assert!(!camera.is_dragging);
        assert!(camera.target_position.is_none());
    }

    #[test]
    fn test_camera_state_set_zoom() {
        let mut camera = CameraState::new();

        camera.set_zoom(2.0);
        assert_eq!(camera.zoom, 2.0);
    }

    #[test]
    fn test_camera_state_set_zoom_clamp_max() {
        let mut camera = CameraState::new();

        camera.set_zoom(10.0);
        assert_eq!(camera.zoom, camera.max_zoom);
    }

    #[test]
    fn test_camera_state_set_zoom_clamp_min() {
        let mut camera = CameraState::new();

        camera.set_zoom(0.01);
        assert_eq!(camera.zoom, camera.min_zoom);
    }

    #[test]
    fn test_camera_state_zoom_in() {
        let mut camera = CameraState::new();
        camera.zoom = 1.0;

        camera.zoom_in(1.5);

        assert!((camera.zoom - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_camera_state_zoom_out() {
        let mut camera = CameraState::new();
        camera.zoom = 2.0;

        camera.zoom_out(2.0);

        assert!((camera.zoom - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_camera_state_center_on() {
        let mut camera = CameraState::new();

        camera.center_on(Vec2::new(100.0, 200.0));

        assert_eq!(camera.target_position, Some(Vec2::new(100.0, 200.0)));
    }

    #[test]
    fn test_camera_state_jump_to() {
        let mut camera = CameraState::new();
        camera.target_position = Some(Vec2::new(50.0, 50.0));

        camera.jump_to(Vec2::new(100.0, 200.0));

        assert_eq!(camera.position, Vec2::new(100.0, 200.0));
        assert!(camera.target_position.is_none());
    }

    #[test]
    fn test_camera_state_default() {
        let camera = CameraState::default();
        assert_eq!(camera.zoom, 1.0);
    }

    // ============================================
    // PendingAction Tests
    // ============================================

    #[test]
    fn test_pending_action_default() {
        let action = PendingAction::default();
        assert!(!action.has_pending());
        assert!(action.action.is_none());
        assert!(action.target.is_none());
    }

    #[test]
    fn test_pending_action_clear() {
        let mut action = PendingAction {
            action: Some(PendingActionType::FoundCity { settler_id: 1 }),
            target: Some(HexCoord::new(5, 5)),
        };

        assert!(action.has_pending());

        action.clear();

        assert!(!action.has_pending());
        assert!(action.action.is_none());
        assert!(action.target.is_none());
    }

    #[test]
    fn test_pending_action_has_pending_true() {
        let action = PendingAction {
            action: Some(PendingActionType::MoveUnit {
                unit_id: 1,
                path: vec![HexCoord::new(0, 0), HexCoord::new(1, 0)],
            }),
            target: None,
        };

        assert!(action.has_pending());
    }

    #[test]
    fn test_pending_action_type_move_unit() {
        let action = PendingActionType::MoveUnit {
            unit_id: 42,
            path: vec![HexCoord::new(0, 0), HexCoord::new(1, 1)],
        };

        match action {
            PendingActionType::MoveUnit { unit_id, path } => {
                assert_eq!(unit_id, 42);
                assert_eq!(path.len(), 2);
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_pending_action_type_attack_unit() {
        let action = PendingActionType::AttackUnit {
            attacker_id: 1,
            defender_id: 2,
        };

        match action {
            PendingActionType::AttackUnit {
                attacker_id,
                defender_id,
            } => {
                assert_eq!(attacker_id, 1);
                assert_eq!(defender_id, 2);
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_pending_action_type_found_city() {
        let action = PendingActionType::FoundCity { settler_id: 5 };

        match action {
            PendingActionType::FoundCity { settler_id } => {
                assert_eq!(settler_id, 5);
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_pending_action_type_attack_city() {
        let action = PendingActionType::AttackCity {
            attacker_id: 10,
            city_id: 20,
        };

        match action {
            PendingActionType::AttackCity {
                attacker_id,
                city_id,
            } => {
                assert_eq!(attacker_id, 10);
                assert_eq!(city_id, 20);
            }
            _ => panic!("Wrong action type"),
        }
    }

    // ============================================
    // UiState Tests
    // ============================================

    #[test]
    fn test_ui_state_default() {
        let ui = UiState::default();

        assert!(!ui.menu_open);
        assert!(!ui.production_panel_open);
        assert!(!ui.tech_tree_open);
        assert!(!ui.diplomacy_open);
        assert!(ui.hovered_hex.is_none());
        assert!(!ui.command_mode);
        assert!(ui.tooltip.is_none());
    }

    #[test]
    fn test_ui_state_close_all() {
        let mut ui = UiState {
            menu_open: true,
            production_panel_open: true,
            tech_tree_open: true,
            diplomacy_open: true,
            hovered_hex: None,
            command_mode: false,
            tooltip: None,
        };

        ui.close_all();

        assert!(!ui.menu_open);
        assert!(!ui.production_panel_open);
        assert!(!ui.tech_tree_open);
        assert!(!ui.diplomacy_open);
    }

    #[test]
    fn test_ui_state_toggle_menu_open() {
        let mut ui = UiState::default();

        ui.toggle_menu();

        assert!(ui.menu_open);
    }

    #[test]
    fn test_ui_state_toggle_menu_close() {
        let mut ui = UiState {
            menu_open: true,
            ..Default::default()
        };

        ui.toggle_menu();

        assert!(!ui.menu_open);
    }

    #[test]
    fn test_ui_state_toggle_menu_closes_other_panels() {
        let mut ui = UiState {
            production_panel_open: true,
            tech_tree_open: true,
            diplomacy_open: true,
            ..Default::default()
        };

        ui.toggle_menu();

        assert!(ui.menu_open);
        assert!(!ui.production_panel_open);
        assert!(!ui.tech_tree_open);
        assert!(!ui.diplomacy_open);
    }

    // ============================================
    // TileEntityMap Tests
    // ============================================

    #[test]
    fn test_tile_entity_map_default() {
        let map = TileEntityMap::default();
        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_tile_entity_map_insert() {
        let mut map = TileEntityMap::default();
        let coord = HexCoord::new(5, 5);
        let entity = Entity::from_raw(1);

        map.insert(coord, entity);

        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());
    }

    #[test]
    fn test_tile_entity_map_get() {
        let mut map = TileEntityMap::default();
        let coord = HexCoord::new(5, 5);
        let entity = Entity::from_raw(1);

        map.insert(coord, entity);

        assert_eq!(map.get(&coord), Some(entity));
    }

    #[test]
    fn test_tile_entity_map_get_nonexistent() {
        let map = TileEntityMap::default();
        let coord = HexCoord::new(5, 5);

        assert_eq!(map.get(&coord), None);
    }

    #[test]
    fn test_tile_entity_map_remove() {
        let mut map = TileEntityMap::default();
        let coord = HexCoord::new(5, 5);
        let entity = Entity::from_raw(1);

        map.insert(coord, entity);
        let removed = map.remove(&coord);

        assert_eq!(removed, Some(entity));
        assert!(map.get(&coord).is_none());
        assert!(map.is_empty());
    }

    #[test]
    fn test_tile_entity_map_remove_nonexistent() {
        let mut map = TileEntityMap::default();
        let coord = HexCoord::new(5, 5);

        let removed = map.remove(&coord);

        assert!(removed.is_none());
    }

    #[test]
    fn test_tile_entity_map_clear() {
        let mut map = TileEntityMap::default();

        map.insert(HexCoord::new(1, 1), Entity::from_raw(1));
        map.insert(HexCoord::new(2, 2), Entity::from_raw(2));
        map.insert(HexCoord::new(3, 3), Entity::from_raw(3));

        assert_eq!(map.len(), 3);

        map.clear();

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_tile_entity_map_overwrite() {
        let mut map = TileEntityMap::default();
        let coord = HexCoord::new(5, 5);
        let entity1 = Entity::from_raw(1);
        let entity2 = Entity::from_raw(2);

        map.insert(coord, entity1);
        map.insert(coord, entity2);

        assert_eq!(map.get(&coord), Some(entity2));
        assert_eq!(map.len(), 1);
    }

    // ============================================
    // UnitEntityMap Tests
    // ============================================

    #[test]
    fn test_unit_entity_map_default() {
        let map = UnitEntityMap::default();
        assert!(map.units.is_empty());
    }

    #[test]
    fn test_unit_entity_map_insert() {
        let mut map = UnitEntityMap::default();
        let unit_id: u64 = 42;
        let entity = Entity::from_raw(1);

        map.insert(unit_id, entity);

        assert_eq!(map.units.len(), 1);
    }

    #[test]
    fn test_unit_entity_map_get() {
        let mut map = UnitEntityMap::default();
        let unit_id: u64 = 42;
        let entity = Entity::from_raw(1);

        map.insert(unit_id, entity);

        assert_eq!(map.get(unit_id), Some(entity));
    }

    #[test]
    fn test_unit_entity_map_get_nonexistent() {
        let map = UnitEntityMap::default();

        assert_eq!(map.get(999), None);
    }

    #[test]
    fn test_unit_entity_map_remove() {
        let mut map = UnitEntityMap::default();
        let unit_id: u64 = 42;
        let entity = Entity::from_raw(1);

        map.insert(unit_id, entity);
        let removed = map.remove(unit_id);

        assert_eq!(removed, Some(entity));
        assert!(map.get(unit_id).is_none());
    }

    #[test]
    fn test_unit_entity_map_clear() {
        let mut map = UnitEntityMap::default();

        map.insert(1, Entity::from_raw(1));
        map.insert(2, Entity::from_raw(2));

        map.clear();

        assert!(map.units.is_empty());
    }

    // ============================================
    // CityEntityMap Tests
    // ============================================

    #[test]
    fn test_city_entity_map_default() {
        let map = CityEntityMap::default();
        assert!(map.cities.is_empty());
    }

    #[test]
    fn test_city_entity_map_insert() {
        let mut map = CityEntityMap::default();
        let city_id: u64 = 100;
        let entity = Entity::from_raw(1);

        map.insert(city_id, entity);

        assert_eq!(map.cities.len(), 1);
    }

    #[test]
    fn test_city_entity_map_get() {
        let mut map = CityEntityMap::default();
        let city_id: u64 = 100;
        let entity = Entity::from_raw(1);

        map.insert(city_id, entity);

        assert_eq!(map.get(city_id), Some(entity));
    }

    #[test]
    fn test_city_entity_map_get_nonexistent() {
        let map = CityEntityMap::default();

        assert_eq!(map.get(999), None);
    }

    #[test]
    fn test_city_entity_map_remove() {
        let mut map = CityEntityMap::default();
        let city_id: u64 = 100;
        let entity = Entity::from_raw(1);

        map.insert(city_id, entity);
        let removed = map.remove(city_id);

        assert_eq!(removed, Some(entity));
        assert!(map.get(city_id).is_none());
    }

    #[test]
    fn test_city_entity_map_clear() {
        let mut map = CityEntityMap::default();

        map.insert(1, Entity::from_raw(1));
        map.insert(2, Entity::from_raw(2));

        map.clear();

        assert!(map.cities.is_empty());
    }

    // ============================================
    // Integration tests - Multiple operations
    // ============================================

    #[test]
    fn test_entity_maps_multiple_operations() {
        let mut tile_map = TileEntityMap::default();
        let mut unit_map = UnitEntityMap::default();
        let mut city_map = CityEntityMap::default();

        // Add tiles
        for q in 0..5 {
            for r in 0..5 {
                let coord = HexCoord::new(q, r);
                let entity = Entity::from_raw((q * 5 + r) as u32);
                tile_map.insert(coord, entity);
            }
        }

        // Add units
        for i in 0..3 {
            unit_map.insert(i as u64, Entity::from_raw(100 + i));
        }

        // Add cities
        city_map.insert(1, Entity::from_raw(200));

        // Verify counts
        assert_eq!(tile_map.len(), 25);
        assert_eq!(unit_map.units.len(), 3);
        assert_eq!(city_map.cities.len(), 1);

        // Remove some
        tile_map.remove(&HexCoord::new(0, 0));
        unit_map.remove(0);

        assert_eq!(tile_map.len(), 24);
        assert_eq!(unit_map.units.len(), 2);
    }

    #[test]
    fn test_selection_state_transitions() {
        let mut selection;

        // Select a unit
        let unit_entity = Entity::from_raw(1);
        selection = SelectedEntity::unit(unit_entity, 42, HexCoord::new(5, 5));
        assert!(selection.is_unit());

        // Switch to city
        let city_entity = Entity::from_raw(2);
        selection = SelectedEntity::city(city_entity, 100, HexCoord::new(10, 10));
        assert!(selection.is_city());
        assert!(!selection.is_unit());

        // Switch to tile
        let tile_entity = Entity::from_raw(3);
        selection = SelectedEntity::tile(tile_entity, HexCoord::new(7, 7));
        assert!(selection.is_tile());
        assert!(!selection.is_city());

        // Clear
        selection.clear();
        assert!(!selection.has_selection());
    }

    #[test]
    fn test_turn_state_full_cycle() {
        let mut turn = CurrentTurn::new(1, 0);

        // Player 0's turn
        assert!(turn.is_player_turn(0));

        // End turn
        turn.end_turn();
        assert!(!turn.is_player_turn(0));

        // Advance to player 1
        turn.next_turn(1, 1);
        assert!(!turn.is_player_turn(0));
        assert!(turn.is_player_turn(1));

        // End turn
        turn.end_turn();

        // Back to player 0, turn 2
        turn.next_turn(2, 0);
        assert_eq!(turn.turn, 2);
        assert!(turn.is_player_turn(0));
    }
}

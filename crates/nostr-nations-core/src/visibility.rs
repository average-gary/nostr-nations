//! Visibility filtering system for game events and state.
//!
//! This module provides fog of war functionality, filtering game events and
//! state based on what each player can see. It ensures that players only
//! receive information they should have access to, preventing cheating
//! in multiplayer games.
//!
//! # Visibility Rules
//!
//! - Own units and cities are always visible
//! - Allied units and cities are visible (with open borders treaty)
//! - Enemy units are only visible if within vision range of own units/cities
//! - Explored tiles show last known state (fog of war)
//! - Unit health is hidden for enemies unless in combat

use crate::city::City;
use crate::events::{GameAction, GameEvent};
use crate::game_state::{DiplomaticStatus, GameState, TreatyType};
use crate::hex::HexCoord;
use crate::map::Tile;
use crate::player::Player;
use crate::types::{CityId, PlayerId, UnitId};
use crate::unit::Unit;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Default vision range for units.
const DEFAULT_UNIT_VISION: u32 = 2;

/// Vision range for cities.
const CITY_VISION_RANGE: u32 = 2;

/// Vision bonus from hills.
const HILLS_VISION_BONUS: u32 = 1;

/// Filters events and game state based on player visibility.
///
/// This struct tracks what tiles, units, and cities a player can currently see,
/// and provides methods to filter events and game state accordingly.
#[derive(Clone, Debug)]
pub struct VisibilityFilter {
    /// The player this filter applies to.
    player_id: PlayerId,
    /// Set of tiles currently visible to the player.
    visible_tiles: HashSet<HexCoord>,
    /// Set of unit IDs currently visible to the player.
    visible_units: HashSet<UnitId>,
    /// Set of city IDs currently visible to the player.
    visible_cities: HashSet<CityId>,
    /// Allied players whose units/cities are always visible.
    allied_players: HashSet<PlayerId>,
}

impl VisibilityFilter {
    /// Create a new visibility filter for a player.
    pub fn new(player_id: PlayerId) -> Self {
        Self {
            player_id,
            visible_tiles: HashSet::new(),
            visible_units: HashSet::new(),
            visible_cities: HashSet::new(),
            allied_players: HashSet::new(),
        }
    }

    /// Update visibility based on current game state.
    ///
    /// This recalculates all visible tiles, units, and cities based on
    /// the positions of the player's units and cities, as well as
    /// diplomatic relationships.
    pub fn update_from_game_state(&mut self, game: &GameState) {
        self.visible_tiles.clear();
        self.visible_units.clear();
        self.visible_cities.clear();
        self.allied_players.clear();

        // Find allied players (those with open borders or Allied status)
        for (i, _player) in game.players.iter().enumerate() {
            let other_id = i as PlayerId;
            if other_id == self.player_id {
                continue;
            }
            if let Some(rel) = game.diplomacy.get(self.player_id, other_id) {
                if rel.status == DiplomaticStatus::Allied || rel.has_treaty(TreatyType::OpenBorders)
                {
                    self.allied_players.insert(other_id);
                }
            }
        }

        // Calculate visible tiles from own units
        for unit in game.units.values() {
            if unit.owner == self.player_id {
                let vision_range = self.get_unit_vision_range(unit, game);
                self.add_visible_tiles_in_range(&unit.position, vision_range, game);
            }
        }

        // Calculate visible tiles from own cities
        for city in game.cities.values() {
            if city.owner == self.player_id {
                self.add_visible_tiles_in_range(&city.position, CITY_VISION_RANGE, game);
            }
        }

        // Determine visible units
        for (unit_id, unit) in &game.units {
            if self.can_see_unit_internal(unit) {
                self.visible_units.insert(*unit_id);
            }
        }

        // Determine visible cities
        for (city_id, city) in &game.cities {
            if self.can_see_city_internal(city) {
                self.visible_cities.insert(*city_id);
            }
        }
    }

    /// Add visible tiles within range of a position.
    fn add_visible_tiles_in_range(&mut self, center: &HexCoord, range: u32, game: &GameState) {
        for coord in center.hexes_in_radius(range) {
            if game.map.in_bounds(&coord) {
                self.visible_tiles.insert(coord);
            }
        }
    }

    /// Get the vision range for a unit, accounting for terrain and promotions.
    fn get_unit_vision_range(&self, unit: &Unit, game: &GameState) -> u32 {
        let mut range = DEFAULT_UNIT_VISION;

        // Check if unit is on hills for bonus vision
        if let Some(tile) = game.map.get(&unit.position) {
            if tile.feature == Some(crate::terrain::Feature::Hills) {
                range += HILLS_VISION_BONUS;
            }
        }

        // TODO: Check for Sentry promotion (adds +1 vision)
        if unit.promotions.contains(&crate::unit::Promotion::Sentry) {
            range += 1;
        }

        range
    }

    /// Internal check if a unit is visible.
    fn can_see_unit_internal(&self, unit: &Unit) -> bool {
        // Own units are always visible
        if unit.owner == self.player_id {
            return true;
        }

        // Allied units are visible
        if self.allied_players.contains(&unit.owner) {
            return true;
        }

        // Enemy units are visible if on a visible tile
        self.visible_tiles.contains(&unit.position)
    }

    /// Internal check if a city is visible.
    fn can_see_city_internal(&self, city: &City) -> bool {
        // Own cities are always visible
        if city.owner == self.player_id {
            return true;
        }

        // Allied cities are visible
        if self.allied_players.contains(&city.owner) {
            return true;
        }

        // Enemy cities are visible if on a visible tile
        self.visible_tiles.contains(&city.position)
    }

    /// Check if the player can see a specific tile.
    pub fn can_see_tile(&self, coord: &HexCoord) -> bool {
        self.visible_tiles.contains(coord)
    }

    /// Check if the player can see a specific unit.
    pub fn can_see_unit(&self, unit_id: UnitId) -> bool {
        self.visible_units.contains(&unit_id)
    }

    /// Check if the player can see a specific city.
    pub fn can_see_city(&self, city_id: CityId) -> bool {
        self.visible_cities.contains(&city_id)
    }

    /// Get the player ID this filter is for.
    pub fn player_id(&self) -> PlayerId {
        self.player_id
    }

    /// Get all currently visible tiles.
    pub fn visible_tiles(&self) -> &HashSet<HexCoord> {
        &self.visible_tiles
    }

    /// Get all currently visible unit IDs.
    pub fn visible_units(&self) -> &HashSet<UnitId> {
        &self.visible_units
    }

    /// Get all currently visible city IDs.
    pub fn visible_cities(&self) -> &HashSet<CityId> {
        &self.visible_cities
    }

    /// Filter a game event based on visibility.
    ///
    /// Returns a `FilteredEvent` indicating whether the event is fully visible,
    /// partially visible (with some information redacted), or completely hidden.
    pub fn filter_event(&self, event: &GameEvent) -> FilteredEvent {
        // Events from the player themselves are always fully visible
        if event.player_id == self.player_id {
            return FilteredEvent::FullyVisible(event.clone());
        }

        match &event.action {
            // Game lifecycle events are always visible
            GameAction::CreateGame { .. }
            | GameAction::JoinGame { .. }
            | GameAction::StartGame
            | GameAction::EndGame { .. } => FilteredEvent::FullyVisible(event.clone()),

            // End turn is visible (turn order is public)
            GameAction::EndTurn => FilteredEvent::FullyVisible(event.clone()),

            // Research changes are hidden (tech is secret)
            GameAction::SetResearch { .. } => FilteredEvent::Hidden,

            // Diplomacy events between this player and another are visible
            GameAction::DeclareWar { target_player }
            | GameAction::ProposePeace { target_player } => {
                if *target_player == self.player_id || event.player_id == self.player_id {
                    FilteredEvent::FullyVisible(event.clone())
                } else {
                    FilteredEvent::Hidden
                }
            }

            GameAction::AcceptPeace { from_player } | GameAction::RejectPeace { from_player } => {
                if *from_player == self.player_id || event.player_id == self.player_id {
                    FilteredEvent::FullyVisible(event.clone())
                } else {
                    FilteredEvent::Hidden
                }
            }

            // Unit movement - visible if we can see the destination
            GameAction::MoveUnit { unit_id, path } => {
                if self.visible_units.contains(unit_id) {
                    FilteredEvent::FullyVisible(event.clone())
                } else if let Some(dest) = path.last() {
                    if self.visible_tiles.contains(dest) {
                        // Can see destination but not full path - redact partial
                        FilteredEvent::PartiallyVisible(event.clone())
                    } else {
                        FilteredEvent::Hidden
                    }
                } else {
                    FilteredEvent::Hidden
                }
            }

            // Combat events - visible if we can see either combatant
            GameAction::AttackUnit {
                attacker_id,
                defender_id,
                ..
            } => {
                if self.visible_units.contains(attacker_id)
                    || self.visible_units.contains(defender_id)
                {
                    FilteredEvent::FullyVisible(event.clone())
                } else {
                    FilteredEvent::Hidden
                }
            }

            GameAction::AttackCity {
                attacker_id,
                city_id,
                ..
            } => {
                if self.visible_units.contains(attacker_id) || self.visible_cities.contains(city_id)
                {
                    FilteredEvent::FullyVisible(event.clone())
                } else {
                    FilteredEvent::Hidden
                }
            }

            // City founding - visible if we can see the tile
            GameAction::FoundCity { settler_id, .. } => {
                if self.visible_units.contains(settler_id) {
                    FilteredEvent::FullyVisible(event.clone())
                } else {
                    FilteredEvent::Hidden
                }
            }

            // Unit state changes - visible if we can see the unit
            GameAction::FortifyUnit { unit_id }
            | GameAction::SleepUnit { unit_id }
            | GameAction::WakeUnit { unit_id }
            | GameAction::DeleteUnit { unit_id }
            | GameAction::UpgradeUnit { unit_id, .. } => {
                if self.visible_units.contains(unit_id) {
                    FilteredEvent::FullyVisible(event.clone())
                } else {
                    FilteredEvent::Hidden
                }
            }

            // Worker actions - visible if we can see the tile
            GameAction::BuildImprovement { unit_id, .. }
            | GameAction::BuildRoad { unit_id }
            | GameAction::RemoveFeature { unit_id } => {
                if self.visible_units.contains(unit_id) {
                    FilteredEvent::FullyVisible(event.clone())
                } else {
                    FilteredEvent::Hidden
                }
            }

            // City production - visible only for own cities
            GameAction::SetProduction { city_id, .. }
            | GameAction::BuyItem { city_id, .. }
            | GameAction::AssignCitizen { city_id, .. }
            | GameAction::UnassignCitizen { city_id, .. }
            | GameAction::SellBuilding { city_id, .. } => {
                // Only visible for own cities, not just visible cities
                if let Some(player) = self.get_city_owner_from_id(*city_id) {
                    if player == self.player_id {
                        return FilteredEvent::FullyVisible(event.clone());
                    }
                }
                FilteredEvent::Hidden
            }

            // Randomness requests/responses are internal
            GameAction::RequestRandom { .. } | GameAction::ProvideRandom { .. } => {
                FilteredEvent::Hidden
            }
        }
    }

    /// Helper to track city ownership (would need game state in real implementation)
    fn get_city_owner_from_id(&self, _city_id: CityId) -> Option<PlayerId> {
        // In a real implementation, this would look up the city
        // For now, we check if it's in our visible cities
        // The actual owner check happens in filter_game_state
        None
    }

    /// Filter the complete game state, applying fog of war.
    ///
    /// Returns a `FilteredGameState` containing only information the player
    /// should be able to see.
    pub fn filter_game_state(&self, game: &GameState) -> FilteredGameState {
        let own_player = game
            .get_player(self.player_id)
            .cloned()
            .expect("Player should exist");

        // Get explored tiles from the player's record
        let explored_tiles = own_player.explored_tiles.clone();

        // Build visible tiles map
        let mut visible_tiles_map = HashMap::new();
        for coord in &self.visible_tiles {
            if let Some(tile) = game.map.get(coord) {
                visible_tiles_map.insert(*coord, tile.clone());
            }
        }

        // Build visible units map (with health redaction for enemies)
        let mut visible_units_map = HashMap::new();
        for unit_id in &self.visible_units {
            if let Some(unit) = game.units.get(unit_id) {
                let filtered_unit =
                    if unit.owner == self.player_id || self.allied_players.contains(&unit.owner) {
                        // Full info for own and allied units
                        unit.clone()
                    } else {
                        // Redact health for enemy units not in combat
                        redact_enemy_unit(unit)
                    };
                visible_units_map.insert(*unit_id, filtered_unit);
            }
        }

        // Build visible cities map
        let mut visible_cities_map = HashMap::new();
        for city_id in &self.visible_cities {
            if let Some(city) = game.cities.get(city_id) {
                let filtered_city =
                    if city.owner == self.player_id || self.allied_players.contains(&city.owner) {
                        // Full info for own and allied cities
                        city.clone()
                    } else {
                        // Limited info for enemy cities
                        redact_enemy_city(city)
                    };
                visible_cities_map.insert(*city_id, filtered_city);
            }
        }

        // Build player summaries for other players
        let other_players: Vec<PlayerSummary> = game
            .players
            .iter()
            .filter(|p| p.id != self.player_id)
            .map(|p| {
                let relationship = game
                    .diplomacy
                    .get(self.player_id, p.id)
                    .map(|r| r.status)
                    .unwrap_or(DiplomaticStatus::Neutral);

                PlayerSummary {
                    id: p.id,
                    name: p.name.clone(),
                    civilization: p.civilization.name.clone(),
                    is_alive: !p.eliminated,
                    relationship,
                }
            })
            .collect();

        FilteredGameState {
            visible_tiles: visible_tiles_map,
            explored_tiles,
            visible_units: visible_units_map,
            visible_cities: visible_cities_map,
            own_player,
            other_players,
            turn: game.turn,
            current_player: game.current_player,
        }
    }
}

/// Result of filtering an event.
#[derive(Clone, Debug)]
pub enum FilteredEvent {
    /// Player can see everything about this event.
    FullyVisible(GameEvent),
    /// Some information has been redacted from the event.
    PartiallyVisible(GameEvent),
    /// Event is completely hidden from the player.
    Hidden,
}

impl FilteredEvent {
    /// Check if the event is visible (fully or partially).
    pub fn is_visible(&self) -> bool {
        !matches!(self, FilteredEvent::Hidden)
    }

    /// Check if the event is fully visible.
    pub fn is_fully_visible(&self) -> bool {
        matches!(self, FilteredEvent::FullyVisible(_))
    }

    /// Get the event if visible, None if hidden.
    pub fn event(&self) -> Option<&GameEvent> {
        match self {
            FilteredEvent::FullyVisible(e) | FilteredEvent::PartiallyVisible(e) => Some(e),
            FilteredEvent::Hidden => None,
        }
    }
}

/// Filtered game state with fog of war applied.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FilteredGameState {
    /// Tiles currently visible to the player.
    pub visible_tiles: HashMap<HexCoord, Tile>,
    /// Tiles the player has explored (seen but not currently visible).
    pub explored_tiles: HashSet<HexCoord>,
    /// Units currently visible to the player.
    pub visible_units: HashMap<UnitId, Unit>,
    /// Cities currently visible to the player.
    pub visible_cities: HashMap<CityId, City>,
    /// Full information about the player's own state.
    pub own_player: Player,
    /// Limited information about other players.
    pub other_players: Vec<PlayerSummary>,
    /// Current turn number.
    pub turn: u32,
    /// Current player whose turn it is.
    pub current_player: PlayerId,
}

impl FilteredGameState {
    /// Check if a tile is currently visible.
    pub fn is_tile_visible(&self, coord: &HexCoord) -> bool {
        self.visible_tiles.contains_key(coord)
    }

    /// Check if a tile has been explored.
    pub fn is_tile_explored(&self, coord: &HexCoord) -> bool {
        self.explored_tiles.contains(coord)
    }

    /// Get the visibility status of a tile.
    pub fn tile_visibility(&self, coord: &HexCoord) -> TileVisibility {
        if self.visible_tiles.contains_key(coord) {
            TileVisibility::Visible
        } else if self.explored_tiles.contains(coord) {
            TileVisibility::Explored
        } else {
            TileVisibility::Unexplored
        }
    }

    /// Get all own units.
    pub fn own_units(&self) -> impl Iterator<Item = (&UnitId, &Unit)> {
        self.visible_units
            .iter()
            .filter(|(_, u)| u.owner == self.own_player.id)
    }

    /// Get all enemy units that are visible.
    pub fn visible_enemy_units(&self) -> impl Iterator<Item = (&UnitId, &Unit)> {
        let own_id = self.own_player.id;
        self.visible_units
            .iter()
            .filter(move |(_, u)| u.owner != own_id)
    }
}

/// Visibility status of a tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileVisibility {
    /// Tile is currently visible (within vision range).
    Visible,
    /// Tile has been explored but is not currently visible (fog of war).
    Explored,
    /// Tile has never been seen.
    Unexplored,
}

/// Limited information about other players visible in fog of war.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerSummary {
    /// Player's ID.
    pub id: PlayerId,
    /// Player's display name.
    pub name: String,
    /// Player's civilization name.
    pub civilization: String,
    /// Whether the player is still in the game.
    pub is_alive: bool,
    /// Diplomatic relationship with the viewing player.
    pub relationship: DiplomaticStatus,
}

/// Redact sensitive information from an enemy unit.
///
/// Enemy units show their type and position, but health is hidden
/// unless they are in combat (which would be handled separately).
fn redact_enemy_unit(unit: &Unit) -> Unit {
    let mut redacted = unit.clone();
    // Hide exact health - show as full health to prevent exploitation
    redacted.health = 100;
    // Hide experience/promotions (optional - could show promotions)
    redacted.experience = 0;
    // Hide queued path
    redacted.queued_path = None;
    redacted
}

/// Redact sensitive information from an enemy city.
///
/// Enemy cities show their position, name, and owner, but hide
/// detailed production and citizen information.
fn redact_enemy_city(city: &City) -> City {
    let mut redacted = city.clone();
    // Hide production details
    redacted.production = None;
    redacted.production_progress = 0;
    redacted.production_queue.clear();
    // Hide specialist assignments
    redacted.specialists = crate::city::Specialists::default();
    // Hide worked tiles
    redacted.worked_tiles.clear();
    redacted.worked_tiles.insert(city.position); // City center is obvious
                                                 // Keep visible: name, position, population (can be estimated), buildings (visible), owner
    redacted
}

/// Redact sensitive information from an event for a specific player.
///
/// This function modifies an event to remove or obscure information
/// that the viewing player shouldn't have access to.
pub fn redact_event_for_player(
    event: &GameEvent,
    viewer: PlayerId,
    filter: &VisibilityFilter,
) -> GameEvent {
    let mut redacted = event.clone();

    // If the event is from the viewer, no redaction needed
    if event.player_id == viewer {
        return redacted;
    }

    // Redact based on action type
    match &redacted.action {
        GameAction::MoveUnit { unit_id, path } => {
            // If we can see the unit but not all tiles in path, redact path
            if filter.can_see_unit(*unit_id) {
                let visible_path: Vec<HexCoord> = path
                    .iter()
                    .filter(|c| filter.can_see_tile(c))
                    .copied()
                    .collect();

                if visible_path.len() < path.len() {
                    // Create redacted path with only visible portions
                    redacted.action = GameAction::MoveUnit {
                        unit_id: *unit_id,
                        path: visible_path,
                    };
                }
            }
        }
        GameAction::AttackUnit {
            attacker_id,
            defender_id,
            random: _,
        } => {
            // Redact the random value (combat internals)
            redacted.action = GameAction::AttackUnit {
                attacker_id: *attacker_id,
                defender_id: *defender_id,
                random: 0.0, // Hide exact random roll
            };
        }
        GameAction::AttackCity {
            attacker_id,
            city_id,
            random: _,
        } => {
            // Redact the random value
            redacted.action = GameAction::AttackCity {
                attacker_id: *attacker_id,
                city_id: *city_id,
                random: 0.0,
            };
        }
        _ => {}
    }

    // Remove randomness proof from redacted events (internal detail)
    redacted.randomness_proof = None;

    redacted
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::City;
    use crate::events::GameAction;
    use crate::player::Civilization;
    use crate::settings::GameSettings;
    use crate::unit::{Unit, UnitType};

    fn create_test_game() -> GameState {
        let settings = GameSettings::new("Test".to_string());
        let mut game = GameState::new("test_game".to_string(), settings, [0u8; 32]);

        // Add two players
        let p1 = Player::new(
            0,
            "npub1".to_string(),
            "Player1".to_string(),
            Civilization::generic(),
        );
        let p2 = Player::new(
            1,
            "npub2".to_string(),
            "Player2".to_string(),
            Civilization::generic(),
        );
        game.add_player(p1).unwrap();
        game.add_player(p2).unwrap();
        game.start().unwrap();

        // Create a small map
        game.map = crate::map::Map::filled(20, 20, crate::terrain::Terrain::Grassland);

        game
    }

    fn add_unit(game: &mut GameState, owner: PlayerId, position: HexCoord) -> UnitId {
        let id = game.allocate_unit_id();
        let unit = Unit::new(id, owner, UnitType::Warrior, position);
        game.units.insert(id, unit);
        id
    }

    fn add_city(game: &mut GameState, owner: PlayerId, position: HexCoord, name: &str) -> CityId {
        let id = game.allocate_city_id();
        let city = City::new(id, owner, name.to_string(), position, false);
        game.cities.insert(id, city);
        id
    }

    // ========== VisibilityFilter Tests ==========

    #[test]
    fn test_visibility_filter_creation() {
        let filter = VisibilityFilter::new(0);
        assert_eq!(filter.player_id(), 0);
        assert!(filter.visible_tiles().is_empty());
        assert!(filter.visible_units().is_empty());
        assert!(filter.visible_cities().is_empty());
    }

    #[test]
    fn test_visibility_from_own_unit() {
        let mut game = create_test_game();
        let unit_pos = HexCoord::new(10, 10);
        add_unit(&mut game, 0, unit_pos);

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        // Should see tiles in radius of unit
        assert!(filter.can_see_tile(&unit_pos));
        assert!(filter.can_see_tile(&HexCoord::new(10, 11)));
        assert!(filter.can_see_tile(&HexCoord::new(11, 10)));

        // Should not see tiles far away
        assert!(!filter.can_see_tile(&HexCoord::new(0, 0)));
    }

    #[test]
    fn test_visibility_from_own_city() {
        let mut game = create_test_game();
        let city_pos = HexCoord::new(10, 10);
        add_city(&mut game, 0, city_pos, "TestCity");

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        // Should see tiles around city
        assert!(filter.can_see_tile(&city_pos));
        for neighbor in city_pos.neighbors() {
            assert!(filter.can_see_tile(&neighbor));
        }
    }

    #[test]
    fn test_own_units_always_visible() {
        let mut game = create_test_game();
        let unit_id = add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        assert!(filter.can_see_unit(unit_id));
    }

    #[test]
    fn test_enemy_unit_visible_in_range() {
        let mut game = create_test_game();
        // Player 0's unit at (10, 10)
        add_unit(&mut game, 0, HexCoord::new(10, 10));
        // Player 1's unit nearby at (11, 10)
        let enemy_id = add_unit(&mut game, 1, HexCoord::new(11, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        assert!(filter.can_see_unit(enemy_id));
    }

    #[test]
    fn test_enemy_unit_hidden_out_of_range() {
        let mut game = create_test_game();
        // Player 0's unit at (10, 10)
        add_unit(&mut game, 0, HexCoord::new(10, 10));
        // Player 1's unit far away at (0, 0)
        let enemy_id = add_unit(&mut game, 1, HexCoord::new(0, 0));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        assert!(!filter.can_see_unit(enemy_id));
    }

    #[test]
    fn test_city_visibility() {
        let mut game = create_test_game();
        // Player 0's unit near player 1's city
        add_unit(&mut game, 0, HexCoord::new(10, 10));
        // Player 1's city nearby
        let city_id = add_city(&mut game, 1, HexCoord::new(11, 10), "EnemyCity");

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        assert!(filter.can_see_city(city_id));
    }

    #[test]
    fn test_allied_units_visible() {
        let mut game = create_test_game();

        // Set up alliance (open borders)
        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1);

        // Add units
        add_unit(&mut game, 0, HexCoord::new(10, 10));
        let allied_unit = add_unit(&mut game, 1, HexCoord::new(0, 0)); // Far away

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        // Allied unit should be visible even if not in range
        assert!(filter.can_see_unit(allied_unit));
    }

    // ========== FilteredEvent Tests ==========

    #[test]
    fn test_own_events_fully_visible() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let event = GameEvent::new(
            "test".to_string(),
            0, // Player 0's event
            None,
            1,
            1,
            GameAction::EndTurn,
        );

        let filtered = filter.filter_event(&event);
        assert!(matches!(filtered, FilteredEvent::FullyVisible(_)));
    }

    #[test]
    fn test_game_lifecycle_events_visible() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let event = GameEvent::new(
            "test".to_string(),
            1, // Player 1's event
            None,
            1,
            1,
            GameAction::StartGame,
        );

        let filtered = filter.filter_event(&event);
        assert!(matches!(filtered, FilteredEvent::FullyVisible(_)));
    }

    #[test]
    fn test_research_events_hidden() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let event = GameEvent::new(
            "test".to_string(),
            1, // Enemy player
            None,
            1,
            1,
            GameAction::SetResearch {
                tech_id: "mining".to_string(),
            },
        );

        let filtered = filter.filter_event(&event);
        assert!(matches!(filtered, FilteredEvent::Hidden));
    }

    #[test]
    fn test_diplomacy_event_visibility() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        // War declaration targeting us is visible
        let war_event = GameEvent::new(
            "test".to_string(),
            1,
            None,
            1,
            1,
            GameAction::DeclareWar { target_player: 0 },
        );

        let filtered = filter.filter_event(&war_event);
        assert!(matches!(filtered, FilteredEvent::FullyVisible(_)));
    }

    #[test]
    fn test_combat_event_visibility() {
        let mut game = create_test_game();
        let our_unit = add_unit(&mut game, 0, HexCoord::new(10, 10));
        let enemy_unit = add_unit(&mut game, 1, HexCoord::new(11, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let attack_event = GameEvent::new(
            "test".to_string(),
            1,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: enemy_unit,
                defender_id: our_unit,
                random: 0.5,
            },
        );

        let filtered = filter.filter_event(&attack_event);
        assert!(filtered.is_visible());
    }

    // ========== FilteredGameState Tests ==========

    #[test]
    fn test_filtered_game_state_contains_own_player() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let filtered_state = filter.filter_game_state(&game);

        assert_eq!(filtered_state.own_player.id, 0);
        assert_eq!(filtered_state.own_player.name, "Player1");
    }

    #[test]
    fn test_filtered_game_state_visible_tiles() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let filtered_state = filter.filter_game_state(&game);

        assert!(filtered_state.is_tile_visible(&HexCoord::new(10, 10)));
        assert!(!filtered_state.is_tile_visible(&HexCoord::new(0, 0)));
    }

    #[test]
    fn test_filtered_game_state_enemy_unit_redaction() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));
        let enemy_id = add_unit(&mut game, 1, HexCoord::new(11, 10));

        // Damage the enemy unit
        game.units.get_mut(&enemy_id).unwrap().health = 50;

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let filtered_state = filter.filter_game_state(&game);

        // Enemy unit should be visible but health should be redacted
        let enemy_unit = filtered_state.visible_units.get(&enemy_id).unwrap();
        assert_eq!(enemy_unit.health, 100); // Redacted to full
    }

    #[test]
    fn test_filtered_game_state_own_unit_not_redacted() {
        let mut game = create_test_game();
        let our_id = add_unit(&mut game, 0, HexCoord::new(10, 10));

        // Damage our unit
        game.units.get_mut(&our_id).unwrap().health = 50;

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let filtered_state = filter.filter_game_state(&game);

        // Our unit should show actual health
        let our_unit = filtered_state.visible_units.get(&our_id).unwrap();
        assert_eq!(our_unit.health, 50);
    }

    #[test]
    fn test_filtered_game_state_enemy_city_redaction() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));
        let city_id = add_city(&mut game, 1, HexCoord::new(11, 10), "EnemyCity");

        // Set production on enemy city
        game.cities.get_mut(&city_id).unwrap().production =
            Some(crate::city::ProductionItem::Unit(UnitType::Warrior));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let filtered_state = filter.filter_game_state(&game);

        // Enemy city should be visible but production hidden
        let enemy_city = filtered_state.visible_cities.get(&city_id).unwrap();
        assert!(enemy_city.production.is_none());
    }

    #[test]
    fn test_player_summary_contains_basic_info() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let filtered_state = filter.filter_game_state(&game);

        assert_eq!(filtered_state.other_players.len(), 1);
        let summary = &filtered_state.other_players[0];
        assert_eq!(summary.id, 1);
        assert_eq!(summary.name, "Player2");
        assert!(summary.is_alive);
    }

    // ========== TileVisibility Tests ==========

    #[test]
    fn test_tile_visibility_states() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        // Mark some tiles as explored
        game.players[0].explored_tiles.insert(HexCoord::new(0, 0));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let filtered_state = filter.filter_game_state(&game);

        // Visible tile
        assert_eq!(
            filtered_state.tile_visibility(&HexCoord::new(10, 10)),
            TileVisibility::Visible
        );

        // Explored but not visible tile
        assert_eq!(
            filtered_state.tile_visibility(&HexCoord::new(0, 0)),
            TileVisibility::Explored
        );

        // Unexplored tile
        assert_eq!(
            filtered_state.tile_visibility(&HexCoord::new(19, 19)),
            TileVisibility::Unexplored
        );
    }

    // ========== Event Redaction Tests ==========

    #[test]
    fn test_redact_event_removes_random() {
        let mut game = create_test_game();
        let attacker = add_unit(&mut game, 1, HexCoord::new(11, 10));
        let defender = add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let event = GameEvent::new(
            "test".to_string(),
            1,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: attacker,
                defender_id: defender,
                random: 0.75,
            },
        );

        let redacted = redact_event_for_player(&event, 0, &filter);

        if let GameAction::AttackUnit { random, .. } = redacted.action {
            assert_eq!(random, 0.0);
        } else {
            panic!("Expected AttackUnit action");
        }
    }

    #[test]
    fn test_redact_event_removes_randomness_proof() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let mut event = GameEvent::new("test".to_string(), 1, None, 1, 1, GameAction::EndTurn);
        event.randomness_proof = Some(crate::cashu::RandomnessProof {
            mint_keyset_id: "test".to_string(),
            blinded_message: vec![1, 2, 3],
            blinded_signature: vec![4, 5, 6],
            signature: vec![7, 8, 9],
            random_bytes: [42u8; 32],
            context: "test".to_string(),
            timestamp: 12345,
        });

        let redacted = redact_event_for_player(&event, 0, &filter);
        assert!(redacted.randomness_proof.is_none());
    }

    #[test]
    fn test_own_events_not_redacted() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(10, 10));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        let event = GameEvent::new(
            "test".to_string(),
            0, // Own event
            None,
            1,
            1,
            GameAction::SetResearch {
                tech_id: "mining".to_string(),
            },
        );

        let redacted = redact_event_for_player(&event, 0, &filter);

        // Should be unchanged
        if let GameAction::SetResearch { tech_id } = &redacted.action {
            assert_eq!(tech_id, "mining");
        } else {
            panic!("Expected SetResearch action");
        }
    }

    // ========== Edge Case Tests ==========

    #[test]
    fn test_visibility_with_no_units() {
        let game = create_test_game();

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        // Should have no visibility
        assert!(filter.visible_tiles().is_empty());
        assert!(filter.visible_units().is_empty());
        assert!(filter.visible_cities().is_empty());
    }

    #[test]
    fn test_multiple_units_combine_visibility() {
        let mut game = create_test_game();
        add_unit(&mut game, 0, HexCoord::new(5, 5));
        add_unit(&mut game, 0, HexCoord::new(15, 15));

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        // Should see tiles around both units
        assert!(filter.can_see_tile(&HexCoord::new(5, 5)));
        assert!(filter.can_see_tile(&HexCoord::new(15, 15)));
    }

    #[test]
    fn test_hills_bonus_vision() {
        let mut game = create_test_game();

        // Place unit on hills
        let unit_pos = HexCoord::new(10, 10);
        if let Some(tile) = game.map.get_mut(&unit_pos) {
            tile.feature = Some(crate::terrain::Feature::Hills);
        }
        add_unit(&mut game, 0, unit_pos);

        let mut filter = VisibilityFilter::new(0);
        filter.update_from_game_state(&game);

        // Should have extended vision (default 2 + 1 for hills = 3)
        // Check a tile at distance 3
        let far_tile = HexCoord::new(10, 13);
        assert!(filter.can_see_tile(&far_tile));
    }

    #[test]
    fn test_filtered_event_is_visible() {
        let event = GameEvent::new("test".to_string(), 0, None, 1, 1, GameAction::EndTurn);

        let fully = FilteredEvent::FullyVisible(event.clone());
        let partial = FilteredEvent::PartiallyVisible(event.clone());
        let hidden = FilteredEvent::Hidden;

        assert!(fully.is_visible());
        assert!(fully.is_fully_visible());
        assert!(partial.is_visible());
        assert!(!partial.is_fully_visible());
        assert!(!hidden.is_visible());
        assert!(!hidden.is_fully_visible());
    }

    #[test]
    fn test_filtered_event_get_event() {
        let event = GameEvent::new("test".to_string(), 0, None, 1, 1, GameAction::EndTurn);

        let fully = FilteredEvent::FullyVisible(event.clone());
        let hidden = FilteredEvent::Hidden;

        assert!(fully.event().is_some());
        assert!(hidden.event().is_none());
    }
}

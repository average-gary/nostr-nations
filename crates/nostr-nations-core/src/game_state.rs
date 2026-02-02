//! Root game state containing all game data.

use crate::city::City;
use crate::map::Map;
use crate::player::Player;
use crate::settings::GameSettings;
use crate::types::{CityId, EventId, GameId, PlayerId, UnitId, VictoryType};
use crate::unit::Unit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The complete state of a game at any point in time.
///
/// This struct is designed to be:
/// - Fully serializable for save/load and Nostr events
/// - Reconstructable from a sequence of game events
/// - Comparable for determinism validation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    /// Unique identifier for this game (derived from initial event hash).
    pub id: GameId,
    /// Game configuration (immutable after start).
    pub settings: GameSettings,
    /// Current turn number (starts at 1).
    pub turn: u32,
    /// Which player's turn it currently is.
    pub current_player: PlayerId,
    /// All players in the game.
    pub players: Vec<Player>,
    /// The game map.
    pub map: Map,
    /// All units in the game, indexed by ID.
    pub units: HashMap<UnitId, Unit>,
    /// All cities in the game, indexed by ID.
    pub cities: HashMap<CityId, City>,
    /// Diplomatic relationships.
    pub diplomacy: DiplomacyState,
    /// Random seed for deterministic replay.
    pub seed: [u8; 32],
    /// Chain of Nostr event IDs for validation.
    pub event_chain: Vec<EventId>,
    /// Next available unit ID.
    pub next_unit_id: UnitId,
    /// Next available city ID.
    pub next_city_id: CityId,
    /// Game phase.
    pub phase: GamePhase,
    /// Victor (if game has ended).
    pub winner: Option<(PlayerId, VictoryType)>,
}

impl GameState {
    /// Create a new game with the given settings.
    pub fn new(id: GameId, settings: GameSettings, seed: [u8; 32]) -> Self {
        let (width, height) = settings.map_dimensions();
        Self {
            id,
            settings,
            turn: 0, // Will be 1 when game starts
            current_player: 0,
            players: Vec::new(),
            map: Map::new(width, height, false),
            units: HashMap::new(),
            cities: HashMap::new(),
            diplomacy: DiplomacyState::default(),
            seed,
            event_chain: Vec::new(),
            next_unit_id: 1,
            next_city_id: 1,
            phase: GamePhase::Setup,
            winner: None,
        }
    }

    /// Add a player to the game.
    pub fn add_player(&mut self, player: Player) -> Result<(), GameError> {
        if self.phase != GamePhase::Setup {
            return Err(GameError::GameAlreadyStarted);
        }
        if self.players.len() >= self.settings.player_count as usize {
            return Err(GameError::TooManyPlayers);
        }
        if self.players.iter().any(|p| p.pubkey == player.pubkey) {
            return Err(GameError::PlayerAlreadyJoined);
        }
        self.players.push(player);
        Ok(())
    }

    /// Start the game (transition from Setup to Playing).
    pub fn start(&mut self) -> Result<(), GameError> {
        if self.phase != GamePhase::Setup {
            return Err(GameError::InvalidPhase);
        }
        if self.players.len() < 2 {
            return Err(GameError::NotEnoughPlayers);
        }
        self.phase = GamePhase::Playing;
        self.turn = 1;
        self.current_player = 0;

        // Initialize diplomacy for all player pairs
        self.diplomacy.initialize(&self.players);

        Ok(())
    }

    /// Get a player by ID.
    pub fn get_player(&self, id: PlayerId) -> Option<&Player> {
        self.players.get(id as usize)
    }

    /// Get a mutable player by ID.
    pub fn get_player_mut(&mut self, id: PlayerId) -> Option<&mut Player> {
        self.players.get_mut(id as usize)
    }

    /// Get the current player.
    pub fn current_player(&self) -> Option<&Player> {
        self.get_player(self.current_player)
    }

    /// Get the current player mutably.
    pub fn current_player_mut(&mut self) -> Option<&mut Player> {
        self.get_player_mut(self.current_player)
    }

    /// Advance to the next player's turn.
    pub fn next_turn(&mut self) -> Result<(), GameError> {
        if self.phase != GamePhase::Playing {
            return Err(GameError::InvalidPhase);
        }

        // Find next non-eliminated player
        let mut next = (self.current_player + 1) % self.players.len() as u8;
        let mut attempts = 0;
        while self.players[next as usize].eliminated && attempts < self.players.len() {
            next = (next + 1) % self.players.len() as u8;
            attempts += 1;
        }

        // Check if only one player remains
        let active_players: Vec<_> = self.players.iter().filter(|p| !p.eliminated).collect();
        if active_players.len() <= 1 {
            if let Some(winner) = active_players.first() {
                self.winner = Some((winner.id, VictoryType::Domination));
                self.phase = GamePhase::Ended;
            }
            return Ok(());
        }

        // If we've cycled back to player 0, increment turn
        if next <= self.current_player {
            self.turn += 1;

            // Check max turns
            if self.settings.max_turns > 0 && self.turn > self.settings.max_turns {
                self.end_by_score();
                return Ok(());
            }
        }

        self.current_player = next;
        Ok(())
    }

    /// End the game by highest score.
    fn end_by_score(&mut self) {
        let winner = self
            .players
            .iter()
            .filter(|p| !p.eliminated)
            .max_by_key(|p| p.score.total)
            .map(|p| p.id);

        if let Some(winner_id) = winner {
            self.winner = Some((winner_id, VictoryType::Score));
        }
        self.phase = GamePhase::Ended;
    }

    /// Allocate a new unit ID.
    pub fn allocate_unit_id(&mut self) -> UnitId {
        let id = self.next_unit_id;
        self.next_unit_id += 1;
        id
    }

    /// Allocate a new city ID.
    pub fn allocate_city_id(&mut self) -> CityId {
        let id = self.next_city_id;
        self.next_city_id += 1;
        id
    }

    /// Add an event ID to the chain.
    pub fn add_event(&mut self, event_id: EventId) {
        self.event_chain.push(event_id);
    }

    /// Get the last event ID in the chain.
    pub fn last_event(&self) -> Option<&EventId> {
        self.event_chain.last()
    }

    /// Check if the game has ended.
    pub fn is_ended(&self) -> bool {
        self.phase == GamePhase::Ended
    }

    /// Check if it's a specific player's turn.
    pub fn is_player_turn(&self, player_id: PlayerId) -> bool {
        self.phase == GamePhase::Playing && self.current_player == player_id
    }
}

/// Phases of the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GamePhase {
    /// Game is being set up, players joining.
    #[default]
    Setup,
    /// Game is in progress.
    Playing,
    /// Game has ended.
    Ended,
}

/// Diplomatic state between all players.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DiplomacyState {
    /// Relationships between player pairs.
    /// Serialized as a sequence of key-value pairs since JSON requires string keys.
    #[serde(with = "tuple_key_map")]
    pub relationships: HashMap<(PlayerId, PlayerId), Relationship>,
}

/// Custom serialization module for HashMap with tuple keys.
/// JSON requires string keys, so we serialize as a sequence of [key, value] pairs.
mod tuple_key_map {
    use super::*;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(
        map: &HashMap<(PlayerId, PlayerId), Relationship>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeSeq;
        let mut seq = serializer.serialize_seq(Some(map.len()))?;
        for (key, value) in map {
            seq.serialize_element(&(key, value))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<(PlayerId, PlayerId), Relationship>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let pairs: Vec<((PlayerId, PlayerId), Relationship)> =
            Deserialize::deserialize(deserializer)?;
        Ok(pairs.into_iter().collect())
    }
}

/// Minimum relationship score for friendly treaties (OpenBorders, ResearchAgreement, TradeAgreement)
pub const FRIENDLY_TREATY_THRESHOLD: i32 = 50;

/// Score below which war becomes more likely
pub const WAR_LIKELIHOOD_THRESHOLD: i32 = -50;

/// Score change when declaring war
pub const WAR_DECLARATION_SCORE_PENALTY: i32 = -40;

/// Score change when making peace
pub const PEACE_SCORE_BONUS: i32 = 20;

/// Score change when breaking a treaty
pub const TREATY_BREAK_SCORE_PENALTY: i32 = -30;

/// Score change when signing a treaty
pub const TREATY_SIGN_SCORE_BONUS: i32 = 10;

impl DiplomacyState {
    /// Initialize relationships for all player pairs.
    pub fn initialize(&mut self, players: &[Player]) {
        for i in 0..players.len() {
            for j in (i + 1)..players.len() {
                let key = (players[i].id, players[j].id);
                self.relationships.insert(key, Relationship::default());
            }
        }
    }

    /// Get the relationship between two players.
    pub fn get(&self, a: PlayerId, b: PlayerId) -> Option<&Relationship> {
        let key = if a < b { (a, b) } else { (b, a) };
        self.relationships.get(&key)
    }

    /// Get mutable relationship between two players.
    pub fn get_mut(&mut self, a: PlayerId, b: PlayerId) -> Option<&mut Relationship> {
        let key = if a < b { (a, b) } else { (b, a) };
        self.relationships.get_mut(&key)
    }

    /// Declare war between two players. Breaks all treaties and sets status to War.
    pub fn declare_war(&mut self, a: PlayerId, b: PlayerId, turn: u32) {
        if let Some(rel) = self.get_mut(a, b) {
            // Can't declare war if already at war
            if rel.status == DiplomaticStatus::War {
                return;
            }

            rel.status = DiplomaticStatus::War;
            rel.clear_treaties();
            rel.turns_at_war = 0;
            rel.turns_at_peace = 0;
            rel.last_interaction_turn = turn;
            rel.relationship_score =
                (rel.relationship_score + WAR_DECLARATION_SCORE_PENALTY).clamp(-100, 100);
        }
    }

    /// Make peace between two players. Sets status to Neutral and adds Peace treaty.
    pub fn make_peace(&mut self, a: PlayerId, b: PlayerId, turn: u32) {
        if let Some(rel) = self.get_mut(a, b) {
            // Can only make peace if at war
            if rel.status != DiplomaticStatus::War {
                return;
            }

            rel.status = DiplomaticStatus::Neutral;
            rel.turns_at_peace = 0;
            rel.last_interaction_turn = turn;
            rel.relationship_score = (rel.relationship_score + PEACE_SCORE_BONUS).clamp(-100, 100);

            // Add peace treaty (10 turns minimum peace)
            rel.add_treaty(ActiveTreaty {
                treaty_type: TreatyType::Peace,
                turn_signed: turn,
                duration: Some(10),
            });
        }
    }

    /// Propose a treaty between two players. Returns true if treaty was accepted.
    ///
    /// Treaty requirements:
    /// - Cannot propose treaties while at war (except Peace via make_peace)
    /// - Friendly treaties (OpenBorders, ResearchAgreement, TradeAgreement) require score >= 50
    /// - DefensivePact requires Allied status
    pub fn propose_treaty(
        &mut self,
        a: PlayerId,
        b: PlayerId,
        treaty_type: TreatyType,
        turn: u32,
    ) -> bool {
        if let Some(rel) = self.get_mut(a, b) {
            // Can't propose treaties while at war
            if rel.status == DiplomaticStatus::War {
                return false;
            }

            // Already have this treaty
            if rel.has_treaty(treaty_type) {
                return false;
            }

            // Check requirements based on treaty type
            let can_sign = match treaty_type {
                TreatyType::Peace => {
                    // Peace treaties are handled by make_peace
                    false
                }
                TreatyType::DefensivePact => {
                    // Requires Allied status
                    rel.status == DiplomaticStatus::Allied
                }
                TreatyType::OpenBorders
                | TreatyType::ResearchAgreement
                | TreatyType::TradeAgreement => {
                    // Requires friendly relationship score
                    rel.relationship_score >= FRIENDLY_TREATY_THRESHOLD
                }
            };

            if can_sign {
                rel.add_treaty(ActiveTreaty {
                    treaty_type,
                    turn_signed: turn,
                    duration: None, // Permanent until broken
                });
                rel.last_interaction_turn = turn;
                rel.relationship_score =
                    (rel.relationship_score + TREATY_SIGN_SCORE_BONUS).clamp(-100, 100);
                return true;
            }
        }
        false
    }

    /// Break a treaty between two players.
    pub fn break_treaty(&mut self, a: PlayerId, b: PlayerId, treaty_type: TreatyType, turn: u32) {
        if let Some(rel) = self.get_mut(a, b) {
            if rel.remove_treaty(treaty_type) {
                rel.last_interaction_turn = turn;
                rel.relationship_score =
                    (rel.relationship_score + TREATY_BREAK_SCORE_PENALTY).clamp(-100, 100);
            }
        }
    }

    /// Check if two players have a specific treaty.
    pub fn has_treaty(&self, a: PlayerId, b: PlayerId, treaty_type: TreatyType) -> bool {
        self.get(a, b)
            .map(|rel| rel.has_treaty(treaty_type))
            .unwrap_or(false)
    }

    /// Check if two players are at war.
    pub fn are_at_war(&self, a: PlayerId, b: PlayerId) -> bool {
        self.get(a, b)
            .map(|rel| rel.status == DiplomaticStatus::War)
            .unwrap_or(false)
    }

    /// Check if units from player a can pass through player b's territory.
    /// Returns true if they are allied or have an open borders treaty.
    pub fn can_units_pass(&self, a: PlayerId, b: PlayerId) -> bool {
        if a == b {
            return true;
        }
        self.get(a, b)
            .map(|rel| {
                rel.status == DiplomaticStatus::Allied || rel.has_treaty(TreatyType::OpenBorders)
            })
            .unwrap_or(false)
    }

    /// Update all relationships for a new turn.
    /// - Increments war/peace counters
    /// - Expires treaties that have run out
    /// - Updates war weariness
    pub fn update_turn(&mut self, turn: u32) {
        for rel in self.relationships.values_mut() {
            // Update counters
            if rel.status == DiplomaticStatus::War {
                rel.turns_at_war += 1;
                // Increase war weariness over time
                rel.war_weariness = rel.war_weariness.saturating_add(1);
            } else {
                rel.turns_at_peace += 1;
                // War weariness slowly decreases during peace
                rel.war_weariness = rel.war_weariness.saturating_sub(1);
            }

            // Expire old treaties
            rel.expire_treaties(turn);
        }
    }

    /// Modify the relationship score between two players.
    /// Score is clamped to -100..=100.
    pub fn modify_relationship_score(&mut self, a: PlayerId, b: PlayerId, delta: i32) {
        if let Some(rel) = self.get_mut(a, b) {
            rel.relationship_score = (rel.relationship_score + delta).clamp(-100, 100);
        }
    }

    /// Get the relationship score between two players.
    /// Returns 0 if no relationship exists.
    pub fn get_relationship_score(&self, a: PlayerId, b: PlayerId) -> i32 {
        self.get(a, b)
            .map(|rel| rel.relationship_score)
            .unwrap_or(0)
    }

    /// Check if the relationship score suggests war is likely.
    /// Used for AI decision making.
    pub fn is_war_likely(&self, a: PlayerId, b: PlayerId) -> bool {
        self.get_relationship_score(a, b) < WAR_LIKELIHOOD_THRESHOLD
    }

    /// Check if the relationship score allows friendly treaties.
    /// Used for AI and UI decisions.
    pub fn can_propose_friendly_treaty(&self, a: PlayerId, b: PlayerId) -> bool {
        let score = self.get_relationship_score(a, b);
        score >= FRIENDLY_TREATY_THRESHOLD
    }
}

/// Relationship between two players.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub status: DiplomaticStatus,
    pub war_weariness: u32,
    pub turns_at_war: u32,
    pub turns_at_peace: u32,
    /// Active treaties between the players
    pub treaties: Vec<ActiveTreaty>,
    /// Relationship score from -100 to +100
    pub relationship_score: i32,
    /// Turn of last diplomatic interaction
    pub last_interaction_turn: u32,
}

impl Default for Relationship {
    fn default() -> Self {
        Self {
            status: DiplomaticStatus::Neutral,
            war_weariness: 0,
            turns_at_war: 0,
            turns_at_peace: 0,
            treaties: Vec::new(),
            relationship_score: 0,
            last_interaction_turn: 0,
        }
    }
}

impl Relationship {
    /// Check if this relationship has a specific treaty type active.
    pub fn has_treaty(&self, treaty_type: TreatyType) -> bool {
        self.treaties.iter().any(|t| t.treaty_type == treaty_type)
    }

    /// Remove a treaty by type. Returns true if a treaty was removed.
    pub fn remove_treaty(&mut self, treaty_type: TreatyType) -> bool {
        let initial_len = self.treaties.len();
        self.treaties.retain(|t| t.treaty_type != treaty_type);
        self.treaties.len() < initial_len
    }

    /// Add a treaty if not already present.
    pub fn add_treaty(&mut self, treaty: ActiveTreaty) {
        if !self.has_treaty(treaty.treaty_type) {
            self.treaties.push(treaty);
        }
    }

    /// Remove all treaties (e.g., when war is declared).
    pub fn clear_treaties(&mut self) {
        self.treaties.clear();
    }

    /// Update expired treaties based on current turn.
    pub fn expire_treaties(&mut self, current_turn: u32) {
        self.treaties.retain(|t| {
            if let Some(duration) = t.duration {
                t.turn_signed + duration > current_turn
            } else {
                true // Permanent treaties don't expire
            }
        });
    }
}

/// Status between two players.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DiplomaticStatus {
    War,
    Hostile,
    #[default]
    Neutral,
    Friendly,
    Allied,
}

/// Types of treaties that can be signed between players.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TreatyType {
    /// End war between nations
    Peace,
    /// Units can pass through each other's territory
    OpenBorders,
    /// Mutual defense - if one is attacked, the other joins the war
    DefensivePact,
    /// Shared research bonus
    ResearchAgreement,
    /// Trade bonus for both parties
    TradeAgreement,
}

/// An active treaty between two players.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ActiveTreaty {
    /// The type of treaty
    pub treaty_type: TreatyType,
    /// Turn when the treaty was signed
    pub turn_signed: u32,
    /// Duration in turns. None = permanent until broken
    pub duration: Option<u32>,
}

/// Errors that can occur during game operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GameError {
    GameAlreadyStarted,
    TooManyPlayers,
    PlayerAlreadyJoined,
    InvalidPhase,
    NotEnoughPlayers,
    NotPlayerTurn,
    InvalidAction,
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::GameAlreadyStarted => write!(f, "Game has already started"),
            GameError::TooManyPlayers => write!(f, "Maximum players reached"),
            GameError::PlayerAlreadyJoined => write!(f, "Player has already joined"),
            GameError::InvalidPhase => write!(f, "Invalid operation for current game phase"),
            GameError::NotEnoughPlayers => write!(f, "Not enough players to start"),
            GameError::NotPlayerTurn => write!(f, "It's not this player's turn"),
            GameError::InvalidAction => write!(f, "Invalid action"),
        }
    }
}

impl std::error::Error for GameError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::Civilization;

    fn create_test_game() -> GameState {
        let settings = GameSettings::new("Test".to_string());
        GameState::new("game1".to_string(), settings, [0u8; 32])
    }

    fn create_test_player(id: PlayerId, name: &str) -> Player {
        Player::new(
            id,
            format!("npub{}", id),
            name.to_string(),
            Civilization::generic(),
        )
    }

    #[test]
    fn test_game_creation() {
        let game = create_test_game();
        assert_eq!(game.phase, GamePhase::Setup);
        assert_eq!(game.turn, 0);
        assert!(game.players.is_empty());
    }

    #[test]
    fn test_add_players() {
        let mut game = create_test_game();
        let p1 = create_test_player(0, "Player1");
        let p2 = create_test_player(1, "Player2");

        assert!(game.add_player(p1).is_ok());
        assert!(game.add_player(p2).is_ok());
        assert_eq!(game.players.len(), 2);
    }

    #[test]
    fn test_duplicate_player() {
        let mut game = create_test_game();
        let p1 = create_test_player(0, "Player1");
        let p1_dup = create_test_player(0, "Player1Clone");

        assert!(game.add_player(p1).is_ok());
        assert_eq!(game.add_player(p1_dup), Err(GameError::PlayerAlreadyJoined));
    }

    #[test]
    fn test_start_game() {
        let mut game = create_test_game();
        game.add_player(create_test_player(0, "P1")).unwrap();
        game.add_player(create_test_player(1, "P2")).unwrap();

        assert!(game.start().is_ok());
        assert_eq!(game.phase, GamePhase::Playing);
        assert_eq!(game.turn, 1);
        assert_eq!(game.current_player, 0);
    }

    #[test]
    fn test_start_without_players() {
        let mut game = create_test_game();
        assert_eq!(game.start(), Err(GameError::NotEnoughPlayers));
    }

    #[test]
    fn test_next_turn() {
        let mut game = create_test_game();
        game.add_player(create_test_player(0, "P1")).unwrap();
        game.add_player(create_test_player(1, "P2")).unwrap();
        game.start().unwrap();

        assert_eq!(game.current_player, 0);
        game.next_turn().unwrap();
        assert_eq!(game.current_player, 1);
        game.next_turn().unwrap();
        assert_eq!(game.current_player, 0);
        assert_eq!(game.turn, 2);
    }

    #[test]
    fn test_diplomacy_initialization() {
        let mut game = create_test_game();
        game.add_player(create_test_player(0, "P1")).unwrap();
        game.add_player(create_test_player(1, "P2")).unwrap();
        game.start().unwrap();

        let rel = game.diplomacy.get(0, 1).unwrap();
        assert_eq!(rel.status, DiplomaticStatus::Neutral);
    }

    #[test]
    fn test_id_allocation() {
        let mut game = create_test_game();
        assert_eq!(game.allocate_unit_id(), 1);
        assert_eq!(game.allocate_unit_id(), 2);
        assert_eq!(game.allocate_city_id(), 1);
        assert_eq!(game.allocate_city_id(), 2);
    }

    #[test]
    fn test_game_serialization() {
        let mut game = create_test_game();
        game.add_player(create_test_player(0, "P1")).unwrap();
        game.add_player(create_test_player(1, "P2")).unwrap();
        game.start().unwrap();

        let json = serde_json::to_string(&game).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, game.id);
        assert_eq!(restored.turn, game.turn);
        assert_eq!(restored.players.len(), game.players.len());
        assert_eq!(restored.phase, game.phase);
    }

    // ========== Treaty System Tests ==========

    fn create_started_game() -> GameState {
        let mut game = create_test_game();
        game.add_player(create_test_player(0, "P1")).unwrap();
        game.add_player(create_test_player(1, "P2")).unwrap();
        game.start().unwrap();
        game
    }

    #[test]
    fn test_declare_war() {
        let mut game = create_started_game();

        assert!(!game.diplomacy.are_at_war(0, 1));
        game.diplomacy.declare_war(0, 1, 1);
        assert!(game.diplomacy.are_at_war(0, 1));

        let rel = game.diplomacy.get(0, 1).unwrap();
        assert_eq!(rel.status, DiplomaticStatus::War);
        assert_eq!(rel.last_interaction_turn, 1);
        assert_eq!(rel.relationship_score, WAR_DECLARATION_SCORE_PENALTY);
    }

    #[test]
    fn test_declare_war_breaks_treaties() {
        let mut game = create_started_game();

        // Set up a high relationship score to allow treaty
        game.diplomacy.modify_relationship_score(0, 1, 60);
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1));
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));

        // Declare war - should break all treaties
        game.diplomacy.declare_war(0, 1, 2);
        assert!(!game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));
    }

    #[test]
    fn test_declare_war_while_at_war() {
        let mut game = create_started_game();

        game.diplomacy.declare_war(0, 1, 1);
        let score_after_first = game.diplomacy.get_relationship_score(0, 1);

        // Declaring war again should do nothing
        game.diplomacy.declare_war(0, 1, 2);
        assert_eq!(
            game.diplomacy.get_relationship_score(0, 1),
            score_after_first
        );
    }

    #[test]
    fn test_make_peace() {
        let mut game = create_started_game();

        game.diplomacy.declare_war(0, 1, 1);
        assert!(game.diplomacy.are_at_war(0, 1));

        game.diplomacy.make_peace(0, 1, 5);
        assert!(!game.diplomacy.are_at_war(0, 1));

        let rel = game.diplomacy.get(0, 1).unwrap();
        assert_eq!(rel.status, DiplomaticStatus::Neutral);
        assert!(rel.has_treaty(TreatyType::Peace));
        assert_eq!(rel.last_interaction_turn, 5);
    }

    #[test]
    fn test_make_peace_adds_peace_treaty() {
        let mut game = create_started_game();

        game.diplomacy.declare_war(0, 1, 1);
        game.diplomacy.make_peace(0, 1, 5);

        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::Peace));

        // Peace treaty should have 10 turn duration
        let rel = game.diplomacy.get(0, 1).unwrap();
        let peace_treaty = rel
            .treaties
            .iter()
            .find(|t| t.treaty_type == TreatyType::Peace)
            .unwrap();
        assert_eq!(peace_treaty.duration, Some(10));
        assert_eq!(peace_treaty.turn_signed, 5);
    }

    #[test]
    fn test_make_peace_not_at_war() {
        let mut game = create_started_game();

        // Should do nothing if not at war
        game.diplomacy.make_peace(0, 1, 1);
        assert!(!game.diplomacy.has_treaty(0, 1, TreatyType::Peace));
    }

    #[test]
    fn test_propose_treaty_requires_score() {
        let mut game = create_started_game();

        // Default score is 0, should fail
        assert!(!game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1));

        // Increase score to threshold
        game.diplomacy
            .modify_relationship_score(0, 1, FRIENDLY_TREATY_THRESHOLD);
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1));
    }

    #[test]
    fn test_propose_treaty_while_at_war() {
        let mut game = create_started_game();

        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy.declare_war(0, 1, 1);

        // Can't propose treaties while at war
        assert!(!game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 2));
    }

    #[test]
    fn test_propose_duplicate_treaty() {
        let mut game = create_started_game();

        game.diplomacy.modify_relationship_score(0, 1, 60);
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1));

        // Can't propose same treaty twice
        assert!(!game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 2));
    }

    #[test]
    fn test_propose_defensive_pact_requires_allied() {
        let mut game = create_started_game();

        // High score but not allied - should fail
        game.diplomacy.modify_relationship_score(0, 1, 100);
        assert!(!game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::DefensivePact, 1));

        // Set status to Allied
        game.diplomacy.get_mut(0, 1).unwrap().status = DiplomaticStatus::Allied;
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::DefensivePact, 1));
    }

    #[test]
    fn test_propose_peace_treaty_fails() {
        let mut game = create_started_game();

        // Peace treaties must be made via make_peace
        game.diplomacy.modify_relationship_score(0, 1, 100);
        assert!(!game.diplomacy.propose_treaty(0, 1, TreatyType::Peace, 1));
    }

    #[test]
    fn test_break_treaty() {
        let mut game = create_started_game();

        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::TradeAgreement, 1);
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::TradeAgreement));

        let score_before = game.diplomacy.get_relationship_score(0, 1);
        game.diplomacy
            .break_treaty(0, 1, TreatyType::TradeAgreement, 5);

        assert!(!game.diplomacy.has_treaty(0, 1, TreatyType::TradeAgreement));
        assert_eq!(
            game.diplomacy.get_relationship_score(0, 1),
            score_before + TREATY_BREAK_SCORE_PENALTY
        );
    }

    #[test]
    fn test_break_nonexistent_treaty() {
        let mut game = create_started_game();

        let score_before = game.diplomacy.get_relationship_score(0, 1);
        game.diplomacy
            .break_treaty(0, 1, TreatyType::TradeAgreement, 1);

        // Score should not change if treaty didn't exist
        assert_eq!(game.diplomacy.get_relationship_score(0, 1), score_before);
    }

    #[test]
    fn test_has_treaty() {
        let mut game = create_started_game();

        assert!(!game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));

        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1);

        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));
        // Order shouldn't matter
        assert!(game.diplomacy.has_treaty(1, 0, TreatyType::OpenBorders));
    }

    #[test]
    fn test_can_units_pass() {
        let mut game = create_started_game();

        // Same player - always true
        assert!(game.diplomacy.can_units_pass(0, 0));

        // Different players, no treaty - false
        assert!(!game.diplomacy.can_units_pass(0, 1));

        // With open borders treaty
        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1);
        assert!(game.diplomacy.can_units_pass(0, 1));
        assert!(game.diplomacy.can_units_pass(1, 0));
    }

    #[test]
    fn test_can_units_pass_allied() {
        let mut game = create_started_game();

        game.diplomacy.get_mut(0, 1).unwrap().status = DiplomaticStatus::Allied;
        assert!(game.diplomacy.can_units_pass(0, 1));
    }

    #[test]
    fn test_update_turn_war_counters() {
        let mut game = create_started_game();

        game.diplomacy.declare_war(0, 1, 1);

        // Simulate several turns
        for turn in 2..=5 {
            game.diplomacy.update_turn(turn);
        }

        let rel = game.diplomacy.get(0, 1).unwrap();
        assert_eq!(rel.turns_at_war, 4);
        assert_eq!(rel.war_weariness, 4);
    }

    #[test]
    fn test_update_turn_peace_counters() {
        let mut game = create_started_game();

        // Set some initial war weariness
        game.diplomacy.get_mut(0, 1).unwrap().war_weariness = 5;

        // Simulate turns at peace
        for turn in 1..=3 {
            game.diplomacy.update_turn(turn);
        }

        let rel = game.diplomacy.get(0, 1).unwrap();
        assert_eq!(rel.turns_at_peace, 3);
        assert_eq!(rel.war_weariness, 2); // Decreased by 3
    }

    #[test]
    fn test_update_turn_expires_treaties() {
        let mut game = create_started_game();

        // Create a treaty that lasts 5 turns
        game.diplomacy
            .get_mut(0, 1)
            .unwrap()
            .add_treaty(ActiveTreaty {
                treaty_type: TreatyType::Peace,
                turn_signed: 1,
                duration: Some(5),
            });

        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::Peace));

        // Update to turn 6 - treaty should still exist (1 + 5 = 6, so > 5)
        game.diplomacy.update_turn(5);
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::Peace));

        // Update to turn 7 - treaty should expire (1 + 5 = 6, not > 6)
        game.diplomacy.update_turn(6);
        assert!(!game.diplomacy.has_treaty(0, 1, TreatyType::Peace));
    }

    #[test]
    fn test_permanent_treaties_dont_expire() {
        let mut game = create_started_game();

        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1);

        // Simulate many turns
        for turn in 2..=100 {
            game.diplomacy.update_turn(turn);
        }

        // Permanent treaty should still exist
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));
    }

    #[test]
    fn test_modify_relationship_score() {
        let mut game = create_started_game();

        assert_eq!(game.diplomacy.get_relationship_score(0, 1), 0);

        game.diplomacy.modify_relationship_score(0, 1, 30);
        assert_eq!(game.diplomacy.get_relationship_score(0, 1), 30);

        game.diplomacy.modify_relationship_score(0, 1, -50);
        assert_eq!(game.diplomacy.get_relationship_score(0, 1), -20);
    }

    #[test]
    fn test_relationship_score_clamping() {
        let mut game = create_started_game();

        // Test upper bound
        game.diplomacy.modify_relationship_score(0, 1, 200);
        assert_eq!(game.diplomacy.get_relationship_score(0, 1), 100);

        // Test lower bound
        game.diplomacy.modify_relationship_score(0, 1, -300);
        assert_eq!(game.diplomacy.get_relationship_score(0, 1), -100);
    }

    #[test]
    fn test_is_war_likely() {
        let mut game = create_started_game();

        assert!(!game.diplomacy.is_war_likely(0, 1)); // Score is 0

        game.diplomacy.modify_relationship_score(0, 1, -49);
        assert!(!game.diplomacy.is_war_likely(0, 1)); // Not below threshold

        game.diplomacy.modify_relationship_score(0, 1, -2);
        assert!(game.diplomacy.is_war_likely(0, 1)); // Now at -51
    }

    #[test]
    fn test_can_propose_friendly_treaty() {
        let mut game = create_started_game();

        assert!(!game.diplomacy.can_propose_friendly_treaty(0, 1)); // Score is 0

        game.diplomacy.modify_relationship_score(0, 1, 49);
        assert!(!game.diplomacy.can_propose_friendly_treaty(0, 1)); // Not at threshold

        game.diplomacy.modify_relationship_score(0, 1, 1);
        assert!(game.diplomacy.can_propose_friendly_treaty(0, 1)); // Now at 50
    }

    #[test]
    fn test_relationship_has_treaty() {
        let mut rel = Relationship::default();

        assert!(!rel.has_treaty(TreatyType::OpenBorders));

        rel.add_treaty(ActiveTreaty {
            treaty_type: TreatyType::OpenBorders,
            turn_signed: 1,
            duration: None,
        });

        assert!(rel.has_treaty(TreatyType::OpenBorders));
        assert!(!rel.has_treaty(TreatyType::TradeAgreement));
    }

    #[test]
    fn test_relationship_remove_treaty() {
        let mut rel = Relationship::default();

        rel.add_treaty(ActiveTreaty {
            treaty_type: TreatyType::OpenBorders,
            turn_signed: 1,
            duration: None,
        });
        rel.add_treaty(ActiveTreaty {
            treaty_type: TreatyType::TradeAgreement,
            turn_signed: 1,
            duration: None,
        });

        assert!(rel.remove_treaty(TreatyType::OpenBorders));
        assert!(!rel.has_treaty(TreatyType::OpenBorders));
        assert!(rel.has_treaty(TreatyType::TradeAgreement));

        // Removing non-existent treaty returns false
        assert!(!rel.remove_treaty(TreatyType::OpenBorders));
    }

    #[test]
    fn test_relationship_clear_treaties() {
        let mut rel = Relationship::default();

        rel.add_treaty(ActiveTreaty {
            treaty_type: TreatyType::OpenBorders,
            turn_signed: 1,
            duration: None,
        });
        rel.add_treaty(ActiveTreaty {
            treaty_type: TreatyType::TradeAgreement,
            turn_signed: 1,
            duration: None,
        });

        rel.clear_treaties();
        assert!(rel.treaties.is_empty());
    }

    #[test]
    fn test_relationship_add_duplicate_treaty() {
        let mut rel = Relationship::default();

        rel.add_treaty(ActiveTreaty {
            treaty_type: TreatyType::OpenBorders,
            turn_signed: 1,
            duration: None,
        });
        rel.add_treaty(ActiveTreaty {
            treaty_type: TreatyType::OpenBorders,
            turn_signed: 5,
            duration: None,
        });

        // Should only have one treaty
        assert_eq!(rel.treaties.len(), 1);
        // Should keep the original
        assert_eq!(rel.treaties[0].turn_signed, 1);
    }

    #[test]
    fn test_diplomacy_serialization_with_treaties() {
        let mut game = create_started_game();

        // Set up some diplomatic state
        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1);
        game.diplomacy
            .propose_treaty(0, 1, TreatyType::TradeAgreement, 2);

        let json = serde_json::to_string(&game).unwrap();
        let restored: GameState = serde_json::from_str(&json).unwrap();

        assert!(restored.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));
        assert!(restored
            .diplomacy
            .has_treaty(0, 1, TreatyType::TradeAgreement));
        assert_eq!(
            restored.diplomacy.get_relationship_score(0, 1),
            game.diplomacy.get_relationship_score(0, 1)
        );
    }

    #[test]
    fn test_all_treaty_types() {
        let mut game = create_started_game();

        // Set up for friendly treaties
        game.diplomacy.modify_relationship_score(0, 1, 60);
        game.diplomacy.get_mut(0, 1).unwrap().status = DiplomaticStatus::Allied;

        // Test all treaty types except Peace (which needs war first)
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::OpenBorders, 1));
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::ResearchAgreement, 2));
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::TradeAgreement, 3));
        assert!(game
            .diplomacy
            .propose_treaty(0, 1, TreatyType::DefensivePact, 4));

        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::OpenBorders));
        assert!(game
            .diplomacy
            .has_treaty(0, 1, TreatyType::ResearchAgreement));
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::TradeAgreement));
        assert!(game.diplomacy.has_treaty(0, 1, TreatyType::DefensivePact));
    }
}

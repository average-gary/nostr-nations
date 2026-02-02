//! Nostr event system for game actions.
//!
//! All game actions are recorded as Nostr events, enabling:
//! - Deterministic replay
//! - Chain validation (each event references the previous)
//! - Distributed game state
//!
//! Event kinds used:
//! - 30100: Game creation
//! - 30101: Player join
//! - 30102: Game start
//! - 30103: Game action (move, attack, build, etc.)
//! - 30104: Turn end
//! - 30105: Game end
//! - 30106: Randomness request
//! - 30107: Randomness response (from Cashu)

use crate::cashu::RandomnessProof;
use crate::city::ProductionItem;
use crate::hex::HexCoord;
use crate::terrain::Improvement;
use crate::types::{CityId, GameId, PlayerId, TechId, UnitId};
use serde::{Deserialize, Serialize};

/// Nostr event kind constants for game events.
pub mod kinds {
    pub const GAME_CREATE: u32 = 30100;
    pub const PLAYER_JOIN: u32 = 30101;
    pub const GAME_START: u32 = 30102;
    pub const GAME_ACTION: u32 = 30103;
    pub const TURN_END: u32 = 30104;
    pub const GAME_END: u32 = 30105;
    pub const RANDOM_REQUEST: u32 = 30106;
    pub const RANDOM_RESPONSE: u32 = 30107;
}

/// A game event that will be serialized into a Nostr event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameEvent {
    /// Event ID (Nostr event ID / hash).
    pub id: String,
    /// Game this event belongs to.
    pub game_id: GameId,
    /// Player who created this event.
    pub player_id: PlayerId,
    /// Previous event ID in the chain.
    pub prev_event_id: Option<String>,
    /// Turn number when this event occurred.
    pub turn: u32,
    /// Sequence number within the turn.
    pub sequence: u32,
    /// The actual action.
    pub action: GameAction,
    /// Unix timestamp.
    pub timestamp: u64,
    /// Cashu randomness proof if randomness was used.
    pub randomness_proof: Option<RandomnessProof>,
}

impl GameEvent {
    /// Create a new game event.
    pub fn new(
        game_id: GameId,
        player_id: PlayerId,
        prev_event_id: Option<String>,
        turn: u32,
        sequence: u32,
        action: GameAction,
    ) -> Self {
        Self {
            id: String::new(), // Will be set when signed
            game_id,
            player_id,
            prev_event_id,
            turn,
            sequence,
            action,
            timestamp: 0, // Will be set when signed
            randomness_proof: None,
        }
    }

    /// Create a new game event with a randomness proof.
    pub fn with_randomness(
        game_id: GameId,
        player_id: PlayerId,
        prev_event_id: Option<String>,
        turn: u32,
        sequence: u32,
        action: GameAction,
        proof: RandomnessProof,
    ) -> Self {
        Self {
            id: String::new(),
            game_id,
            player_id,
            prev_event_id,
            turn,
            sequence,
            action,
            timestamp: 0,
            randomness_proof: Some(proof),
        }
    }

    /// Get the Nostr event kind for this action.
    pub fn kind(&self) -> u32 {
        match &self.action {
            GameAction::CreateGame { .. } => kinds::GAME_CREATE,
            GameAction::JoinGame { .. } => kinds::PLAYER_JOIN,
            GameAction::StartGame => kinds::GAME_START,
            GameAction::EndTurn => kinds::TURN_END,
            GameAction::EndGame { .. } => kinds::GAME_END,
            GameAction::RequestRandom { .. } => kinds::RANDOM_REQUEST,
            GameAction::ProvideRandom { .. } => kinds::RANDOM_RESPONSE,
            _ => kinds::GAME_ACTION,
        }
    }

    /// Serialize the event content for signing.
    pub fn content(&self) -> String {
        serde_json::to_string(&self.action).unwrap_or_default()
    }

    /// Generate Nostr tags for this event.
    pub fn tags(&self) -> Vec<Vec<String>> {
        let mut tags = vec![
            vec!["g".to_string(), self.game_id.clone()], // Game ID tag
            vec!["p".to_string(), self.player_id.to_string()], // Player tag
            vec!["turn".to_string(), self.turn.to_string()],
            vec!["seq".to_string(), self.sequence.to_string()],
        ];

        // Add previous event reference
        if let Some(ref prev) = self.prev_event_id {
            tags.push(vec!["e".to_string(), prev.clone(), "reply".to_string()]);
        }

        // Add Cashu proof if present (serialized as JSON)
        if let Some(ref proof) = self.randomness_proof {
            if let Ok(proof_json) = serde_json::to_string(proof) {
                tags.push(vec!["cashu".to_string(), proof_json]);
            }
        }

        tags
    }

    /// Get the random value from the proof if present.
    pub fn random_value(&self) -> Option<f32> {
        self.randomness_proof.as_ref().map(|p| p.to_f32())
    }

    /// Check if this event has a valid randomness proof.
    pub fn has_randomness_proof(&self) -> bool {
        self.randomness_proof.is_some()
    }
}

/// All possible game actions.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum GameAction {
    // Game lifecycle
    CreateGame {
        settings_json: String,
        seed: [u8; 32],
    },
    JoinGame {
        player_name: String,
        civilization_id: String,
    },
    StartGame,
    EndTurn,
    EndGame {
        winner_id: PlayerId,
        victory_type: String,
    },

    // Unit actions
    MoveUnit {
        unit_id: UnitId,
        path: Vec<HexCoord>,
    },
    AttackUnit {
        attacker_id: UnitId,
        defender_id: UnitId,
        random: f32,
    },
    AttackCity {
        attacker_id: UnitId,
        city_id: CityId,
        random: f32,
    },
    FoundCity {
        settler_id: UnitId,
        name: String,
    },
    FortifyUnit {
        unit_id: UnitId,
    },
    SleepUnit {
        unit_id: UnitId,
    },
    WakeUnit {
        unit_id: UnitId,
    },
    DeleteUnit {
        unit_id: UnitId,
    },
    UpgradeUnit {
        unit_id: UnitId,
        gold_cost: i32,
    },

    // Worker actions
    BuildImprovement {
        unit_id: UnitId,
        improvement: Improvement,
    },
    BuildRoad {
        unit_id: UnitId,
    },
    RemoveFeature {
        unit_id: UnitId,
    },

    // City actions
    SetProduction {
        city_id: CityId,
        item: ProductionItem,
    },
    BuyItem {
        city_id: CityId,
        item: ProductionItem,
        gold_cost: i32,
    },
    AssignCitizen {
        city_id: CityId,
        tile: HexCoord,
    },
    UnassignCitizen {
        city_id: CityId,
        tile: HexCoord,
    },
    SellBuilding {
        city_id: CityId,
        building: String,
    },

    // Research
    SetResearch {
        tech_id: TechId,
    },

    // Diplomacy
    DeclareWar {
        target_player: PlayerId,
    },
    ProposePeace {
        target_player: PlayerId,
    },
    AcceptPeace {
        from_player: PlayerId,
    },
    RejectPeace {
        from_player: PlayerId,
    },

    // Randomness (Cashu integration)
    RequestRandom {
        purpose: String,
        blinded_message: String,
    },
    ProvideRandom {
        request_id: String,
        blind_signature: String,
    },
}

impl GameAction {
    /// Check if this action requires randomness.
    pub fn requires_random(&self) -> bool {
        matches!(
            self,
            GameAction::AttackUnit { .. }
                | GameAction::AttackCity { .. }
                | GameAction::CreateGame { .. }
        )
    }

    /// Get a human-readable description of the action.
    pub fn description(&self) -> String {
        match self {
            GameAction::CreateGame { .. } => "Created game".to_string(),
            GameAction::JoinGame { player_name, .. } => format!("{} joined", player_name),
            GameAction::StartGame => "Game started".to_string(),
            GameAction::EndTurn => "Ended turn".to_string(),
            GameAction::EndGame {
                winner_id,
                victory_type,
            } => {
                format!("Player {} won by {}", winner_id, victory_type)
            }
            GameAction::MoveUnit { unit_id, path } => {
                format!("Unit {} moved to {:?}", unit_id, path.last())
            }
            GameAction::AttackUnit {
                attacker_id,
                defender_id,
                ..
            } => {
                format!("Unit {} attacked unit {}", attacker_id, defender_id)
            }
            GameAction::AttackCity {
                attacker_id,
                city_id,
                ..
            } => {
                format!("Unit {} attacked city {}", attacker_id, city_id)
            }
            GameAction::FoundCity { name, .. } => format!("Founded city {}", name),
            GameAction::FortifyUnit { unit_id } => format!("Unit {} fortified", unit_id),
            GameAction::SetProduction { city_id, item } => {
                format!("City {} producing {:?}", city_id, item)
            }
            GameAction::SetResearch { tech_id } => format!("Researching {}", tech_id),
            GameAction::DeclareWar { target_player } => {
                format!("Declared war on player {}", target_player)
            }
            _ => format!("{:?}", self),
        }
    }
}

/// Event chain for validation and replay.
#[derive(Clone, Debug, Default)]
pub struct EventChain {
    /// All events in order.
    events: Vec<GameEvent>,
    /// Index by event ID.
    by_id: std::collections::HashMap<String, usize>,
    /// Last event ID.
    last_id: Option<String>,
}

impl EventChain {
    /// Create a new empty event chain.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an event to the chain.
    pub fn add(&mut self, event: GameEvent) -> Result<(), EventChainError> {
        // Validate chain linkage
        if let Some(ref prev_id) = event.prev_event_id {
            if self.last_id.as_ref() != Some(prev_id) {
                return Err(EventChainError::InvalidPreviousEvent);
            }
        } else if !self.events.is_empty() {
            return Err(EventChainError::MissingPreviousEvent);
        }

        // Validate sequence
        if !self.events.is_empty() {
            let last = self.events.last().unwrap();
            if event.turn < last.turn {
                return Err(EventChainError::InvalidTurnSequence);
            }
            if event.turn == last.turn && event.sequence <= last.sequence {
                return Err(EventChainError::InvalidSequence);
            }
        }

        // Validate randomness proof for actions that require it
        if event.action.requires_random() && event.randomness_proof.is_none() {
            return Err(EventChainError::MissingRandomnessProof);
        }

        let idx = self.events.len();
        self.last_id = Some(event.id.clone());
        self.by_id.insert(event.id.clone(), idx);
        self.events.push(event);

        Ok(())
    }

    /// Get an event by ID.
    pub fn get(&self, id: &str) -> Option<&GameEvent> {
        self.by_id.get(id).map(|&idx| &self.events[idx])
    }

    /// Get the last event.
    pub fn last(&self) -> Option<&GameEvent> {
        self.events.last()
    }

    /// Get all events.
    pub fn events(&self) -> &[GameEvent] {
        &self.events
    }

    /// Get events for a specific turn.
    pub fn events_for_turn(&self, turn: u32) -> Vec<&GameEvent> {
        self.events.iter().filter(|e| e.turn == turn).collect()
    }

    /// Get the total number of events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the chain is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Verify the entire chain is valid.
    pub fn verify(&self) -> Result<(), EventChainError> {
        let mut prev_id: Option<&String> = None;

        for event in &self.events {
            // Check linkage
            if event.prev_event_id.as_ref() != prev_id {
                return Err(EventChainError::BrokenChain);
            }
            prev_id = Some(&event.id);
        }

        Ok(())
    }

    /// Get all randomness proofs in the chain.
    pub fn randomness_proofs(&self) -> Vec<&RandomnessProof> {
        self.events
            .iter()
            .filter_map(|e| e.randomness_proof.as_ref())
            .collect()
    }
}

/// Errors from event chain operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventChainError {
    InvalidPreviousEvent,
    MissingPreviousEvent,
    InvalidTurnSequence,
    InvalidSequence,
    BrokenChain,
    EventNotFound,
    MissingRandomnessProof,
    InvalidRandomnessProof,
}

impl std::fmt::Display for EventChainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventChainError::InvalidPreviousEvent => write!(f, "Previous event ID doesn't match"),
            EventChainError::MissingPreviousEvent => write!(f, "Previous event ID required"),
            EventChainError::InvalidTurnSequence => write!(f, "Turn number decreased"),
            EventChainError::InvalidSequence => write!(f, "Sequence number invalid"),
            EventChainError::BrokenChain => write!(f, "Event chain is broken"),
            EventChainError::EventNotFound => write!(f, "Event not found"),
            EventChainError::MissingRandomnessProof => {
                write!(f, "Randomness proof required for this action")
            }
            EventChainError::InvalidRandomnessProof => write!(f, "Randomness proof is invalid"),
        }
    }
}

impl std::error::Error for EventChainError {}

/// Builder for creating game events with proper chaining.
pub struct EventBuilder {
    game_id: GameId,
    player_id: PlayerId,
    turn: u32,
    sequence: u32,
    last_event_id: Option<String>,
}

impl EventBuilder {
    /// Create a new event builder.
    pub fn new(game_id: GameId, player_id: PlayerId) -> Self {
        Self {
            game_id,
            player_id,
            turn: 0,
            sequence: 0,
            last_event_id: None,
        }
    }

    /// Set the current turn.
    pub fn set_turn(&mut self, turn: u32) {
        if turn > self.turn {
            self.turn = turn;
            self.sequence = 0;
        }
    }

    /// Set the last event ID for chaining.
    pub fn set_last_event(&mut self, event_id: String) {
        self.last_event_id = Some(event_id);
    }

    /// Build an event.
    pub fn build(&mut self, action: GameAction) -> GameEvent {
        self.sequence += 1;

        GameEvent::new(
            self.game_id.clone(),
            self.player_id,
            self.last_event_id.clone(),
            self.turn,
            self.sequence,
            action,
        )
    }

    /// Build an event with a randomness proof.
    pub fn build_with_randomness(
        &mut self,
        action: GameAction,
        proof: RandomnessProof,
    ) -> GameEvent {
        self.sequence += 1;

        GameEvent::with_randomness(
            self.game_id.clone(),
            self.player_id,
            self.last_event_id.clone(),
            self.turn,
            self.sequence,
            action,
            proof,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_event(id: &str, prev: Option<&str>, turn: u32, seq: u32) -> GameEvent {
        let mut event = GameEvent::new(
            "test_game".to_string(),
            0,
            prev.map(|s| s.to_string()),
            turn,
            seq,
            GameAction::EndTurn,
        );
        event.id = id.to_string();
        event
    }

    #[test]
    fn test_event_chain_add() {
        let mut chain = EventChain::new();

        let event1 = create_test_event("evt1", None, 1, 1);
        assert!(chain.add(event1).is_ok());

        let event2 = create_test_event("evt2", Some("evt1"), 1, 2);
        assert!(chain.add(event2).is_ok());

        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_event_chain_invalid_link() {
        let mut chain = EventChain::new();

        let event1 = create_test_event("evt1", None, 1, 1);
        chain.add(event1).unwrap();

        // Try to add event with wrong previous ID
        let event2 = create_test_event("evt2", Some("wrong"), 1, 2);
        assert_eq!(
            chain.add(event2),
            Err(EventChainError::InvalidPreviousEvent)
        );
    }

    #[test]
    fn test_event_chain_verify() {
        let mut chain = EventChain::new();

        chain.add(create_test_event("evt1", None, 1, 1)).unwrap();
        chain
            .add(create_test_event("evt2", Some("evt1"), 1, 2))
            .unwrap();
        chain
            .add(create_test_event("evt3", Some("evt2"), 2, 1))
            .unwrap();

        assert!(chain.verify().is_ok());
    }

    #[test]
    fn test_event_builder() {
        let mut builder = EventBuilder::new("game1".to_string(), 0);
        builder.set_turn(1);

        let event1 = builder.build(GameAction::EndTurn);
        assert_eq!(event1.turn, 1);
        assert_eq!(event1.sequence, 1);

        let event2 = builder.build(GameAction::EndTurn);
        assert_eq!(event2.turn, 1);
        assert_eq!(event2.sequence, 2);

        builder.set_turn(2);
        let event3 = builder.build(GameAction::EndTurn);
        assert_eq!(event3.turn, 2);
        assert_eq!(event3.sequence, 1);
    }

    #[test]
    fn test_action_requires_random() {
        assert!(GameAction::AttackUnit {
            attacker_id: 1,
            defender_id: 2,
            random: 0.5,
        }
        .requires_random());

        assert!(!GameAction::EndTurn.requires_random());
        assert!(!GameAction::FortifyUnit { unit_id: 1 }.requires_random());
    }

    #[test]
    fn test_event_tags() {
        let event = GameEvent::new(
            "game123".to_string(),
            0,
            Some("prev_evt".to_string()),
            5,
            3,
            GameAction::EndTurn,
        );

        let tags = event.tags();
        assert!(tags.iter().any(|t| t[0] == "g" && t[1] == "game123"));
        assert!(tags.iter().any(|t| t[0] == "turn" && t[1] == "5"));
        assert!(tags.iter().any(|t| t[0] == "e" && t[1] == "prev_evt"));
    }

    #[test]
    fn test_event_kind() {
        let create = GameEvent::new(
            "g".to_string(),
            0,
            None,
            0,
            1,
            GameAction::CreateGame {
                settings_json: "{}".to_string(),
                seed: [0; 32],
            },
        );
        assert_eq!(create.kind(), kinds::GAME_CREATE);

        let action = GameEvent::new(
            "g".to_string(),
            0,
            None,
            1,
            1,
            GameAction::MoveUnit {
                unit_id: 1,
                path: vec![],
            },
        );
        assert_eq!(action.kind(), kinds::GAME_ACTION);
    }

    #[test]
    fn test_event_with_randomness_proof() {
        let proof = RandomnessProof {
            mint_keyset_id: "test".to_string(),
            blinded_message: vec![1, 2, 3],
            blinded_signature: vec![4, 5, 6],
            signature: vec![7, 8, 9],
            random_bytes: [42u8; 32],
            context: "combat".to_string(),
            timestamp: 12345,
        };

        let event = GameEvent::with_randomness(
            "game1".to_string(),
            0,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
            proof.clone(),
        );

        assert!(event.has_randomness_proof());
        assert!(event.random_value().is_some());

        let tags = event.tags();
        assert!(tags.iter().any(|t| t[0] == "cashu"));
    }
}

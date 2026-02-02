//! Conflict detection and resolution for multiplayer synchronization.
//!
//! This module handles conflicts that arise in distributed multiplayer games:
//! - Concurrent modifications to the same entity
//! - Timestamp collisions
//! - Missing event predecessors
//! - Invalid state transitions
//!
//! # Resolution Strategies
//!
//! Multiple resolution strategies are supported:
//! - `FirstWins`: First event by timestamp wins (deterministic)
//! - `LastWins`: Last event by timestamp wins
//! - `HostPriority`: Host's events always take precedence
//! - `Merge`: Attempt to merge compatible changes
//! - `Reject`: Reject the conflicting event

use crate::delta::EntityType;
use nostr_nations_core::events::GameEvent;
use nostr_nations_core::types::PlayerId;
use std::collections::HashMap;

/// Types of conflicts that can occur during synchronization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConflictType {
    /// Same entity modified by multiple players concurrently.
    ConcurrentModification {
        entity_type: EntityType,
        entity_id: u64,
        player_a: PlayerId,
        player_b: PlayerId,
    },
    /// Events with the same timestamp but different content.
    TimestampCollision {
        timestamp: u64,
        event_a_id: String,
        event_b_id: String,
    },
    /// Event references a non-existent previous event.
    MissingPredecessor {
        event_id: String,
        missing_id: String,
    },
    /// Invalid state transition detected.
    InvalidTransition { event_id: String, reason: String },
}

impl std::fmt::Display for ConflictType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConflictType::ConcurrentModification {
                entity_type,
                entity_id,
                player_a,
                player_b,
            } => {
                write!(
                    f,
                    "Concurrent modification of {:?}:{} by players {} and {}",
                    entity_type, entity_id, player_a, player_b
                )
            }
            ConflictType::TimestampCollision {
                timestamp,
                event_a_id,
                event_b_id,
            } => {
                write!(
                    f,
                    "Timestamp collision at {}: events {} and {}",
                    timestamp, event_a_id, event_b_id
                )
            }
            ConflictType::MissingPredecessor {
                event_id,
                missing_id,
            } => {
                write!(
                    f,
                    "Event {} references missing predecessor {}",
                    event_id, missing_id
                )
            }
            ConflictType::InvalidTransition { event_id, reason } => {
                write!(f, "Invalid transition in event {}: {}", event_id, reason)
            }
        }
    }
}

impl std::error::Error for ConflictType {}

/// Detects conflicts in game event streams.
#[derive(Debug, Default)]
pub struct ConflictDetector {
    /// Previously seen events by ID.
    seen_events: HashMap<String, GameEvent>,
    /// Entity version tracking: (EntityType, entity_id) -> Vec<(player_id, timestamp)>.
    entity_versions: HashMap<(EntityType, u64), Vec<(u8, u64)>>,
    /// Events by timestamp for collision detection.
    events_by_timestamp: HashMap<u64, Vec<String>>,
}

impl ConflictDetector {
    /// Create a new conflict detector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check an event for conflicts without adding it.
    pub fn check_event(&mut self, event: &GameEvent) -> Vec<ConflictType> {
        let mut conflicts = Vec::new();

        // Check for missing predecessor
        if let Some(ref prev_id) = event.prev_event_id {
            if !self.seen_events.contains_key(prev_id) && !self.seen_events.is_empty() {
                conflicts.push(ConflictType::MissingPredecessor {
                    event_id: event.id.clone(),
                    missing_id: prev_id.clone(),
                });
            }
        }

        // Check for timestamp collisions
        if let Some(existing_ids) = self.events_by_timestamp.get(&event.timestamp) {
            for existing_id in existing_ids {
                if existing_id != &event.id {
                    if let Some(existing) = self.seen_events.get(existing_id) {
                        // Only flag as collision if content is different
                        if existing.action.description() != event.action.description() {
                            conflicts.push(ConflictType::TimestampCollision {
                                timestamp: event.timestamp,
                                event_a_id: existing_id.clone(),
                                event_b_id: event.id.clone(),
                            });
                        }
                    }
                }
            }
        }

        // Check for concurrent modifications to entities
        let affected_entities = extract_entity_ids(event);
        for (entity_type, entity_id) in affected_entities {
            if let Some(versions) = self.entity_versions.get(&(entity_type, entity_id)) {
                // Look for concurrent modifications (same turn, different players)
                for (other_player, other_timestamp) in versions {
                    if *other_player != event.player_id {
                        // Check if within a concurrent modification window (same turn or within 1 second)
                        let time_diff = event.timestamp.abs_diff(*other_timestamp);

                        if time_diff < 1000 {
                            // Within 1 second, consider concurrent
                            conflicts.push(ConflictType::ConcurrentModification {
                                entity_type,
                                entity_id,
                                player_a: *other_player,
                                player_b: event.player_id,
                            });
                        }
                    }
                }
            }
        }

        // Check for invalid state transitions
        if let Some(transition_error) = self.validate_transition(event) {
            conflicts.push(ConflictType::InvalidTransition {
                event_id: event.id.clone(),
                reason: transition_error,
            });
        }

        conflicts
    }

    /// Add an event to the detector's tracking state.
    pub fn add_event(&mut self, event: GameEvent) {
        // Track by ID
        self.seen_events.insert(event.id.clone(), event.clone());

        // Track by timestamp
        self.events_by_timestamp
            .entry(event.timestamp)
            .or_default()
            .push(event.id.clone());

        // Track entity versions
        let affected_entities = extract_entity_ids(&event);
        for (entity_type, entity_id) in affected_entities {
            self.entity_versions
                .entry((entity_type, entity_id))
                .or_default()
                .push((event.player_id, event.timestamp));
        }
    }

    /// Clear all tracked state.
    pub fn clear(&mut self) {
        self.seen_events.clear();
        self.entity_versions.clear();
        self.events_by_timestamp.clear();
    }

    /// Get the number of tracked events.
    pub fn event_count(&self) -> usize {
        self.seen_events.len()
    }

    /// Check if an event ID has been seen.
    pub fn has_event(&self, event_id: &str) -> bool {
        self.seen_events.contains_key(event_id)
    }

    /// Get a previously seen event by ID.
    pub fn get_event(&self, event_id: &str) -> Option<&GameEvent> {
        self.seen_events.get(event_id)
    }

    /// Validate state transitions for an event.
    fn validate_transition(&self, event: &GameEvent) -> Option<String> {
        use nostr_nations_core::events::GameAction;

        match &event.action {
            GameAction::EndTurn => {
                // EndTurn should only be valid on the player's turn
                // This would need game state context, so we do basic validation
                None
            }
            GameAction::StartGame => {
                // Check if game was already started
                for existing in self.seen_events.values() {
                    if matches!(existing.action, GameAction::StartGame)
                        && existing.game_id == event.game_id
                    {
                        return Some("Game already started".to_string());
                    }
                }
                None
            }
            GameAction::JoinGame { .. } => {
                // Check if game already started
                for existing in self.seen_events.values() {
                    if matches!(existing.action, GameAction::StartGame)
                        && existing.game_id == event.game_id
                    {
                        return Some("Cannot join after game started".to_string());
                    }
                }
                None
            }
            GameAction::MoveUnit { unit_id, .. } => {
                // Check if unit was deleted
                for existing in self.seen_events.values() {
                    if let GameAction::DeleteUnit {
                        unit_id: deleted_id,
                    } = &existing.action
                    {
                        if deleted_id == unit_id && existing.timestamp < event.timestamp {
                            return Some(format!("Unit {} was deleted", unit_id));
                        }
                    }
                }
                None
            }
            GameAction::DeleteUnit { unit_id } => {
                // Check if unit was already deleted
                for existing in self.seen_events.values() {
                    if let GameAction::DeleteUnit {
                        unit_id: deleted_id,
                    } = &existing.action
                    {
                        if deleted_id == unit_id && existing.id != event.id {
                            return Some(format!("Unit {} already deleted", unit_id));
                        }
                    }
                }
                None
            }
            _ => None,
        }
    }
}

/// Strategies for resolving conflicts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ResolutionStrategy {
    /// First event wins (by timestamp, then by event ID for determinism).
    #[default]
    FirstWins,
    /// Last event wins.
    LastWins,
    /// Host's events always win.
    HostPriority,
    /// Attempt to merge compatible changes.
    Merge,
    /// Reject the conflicting event.
    Reject,
}

/// Result of conflict resolution.
#[derive(Clone, Debug)]
pub enum Resolution {
    /// Accept the event with the given ID.
    Accept(String),
    /// Reject the event with the given ID.
    Reject(String),
    /// Create a merged event (for Merge strategy).
    Merge(GameEvent),
    /// Need a full resync (conflict cannot be resolved locally).
    RequestResync,
}

impl PartialEq for Resolution {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Resolution::Accept(a), Resolution::Accept(b)) => a == b,
            (Resolution::Reject(a), Resolution::Reject(b)) => a == b,
            (Resolution::Merge(a), Resolution::Merge(b)) => a.id == b.id,
            (Resolution::RequestResync, Resolution::RequestResync) => true,
            _ => false,
        }
    }
}

impl Resolution {
    /// Check if this resolution accepts an event.
    pub fn is_accept(&self) -> bool {
        matches!(self, Resolution::Accept(_))
    }

    /// Check if this resolution rejects an event.
    pub fn is_reject(&self) -> bool {
        matches!(self, Resolution::Reject(_))
    }

    /// Check if this resolution is a merge.
    pub fn is_merge(&self) -> bool {
        matches!(self, Resolution::Merge(_))
    }

    /// Check if this resolution requests a resync.
    pub fn needs_resync(&self) -> bool {
        matches!(self, Resolution::RequestResync)
    }

    /// Get the accepted event ID if this is an Accept resolution.
    pub fn accepted_id(&self) -> Option<&str> {
        match self {
            Resolution::Accept(id) => Some(id),
            _ => None,
        }
    }

    /// Get the rejected event ID if this is a Reject resolution.
    pub fn rejected_id(&self) -> Option<&str> {
        match self {
            Resolution::Reject(id) => Some(id),
            _ => None,
        }
    }
}

/// Resolves conflicts based on a configurable strategy.
#[derive(Clone, Debug)]
pub struct ConflictResolver {
    /// The resolution strategy to use.
    strategy: ResolutionStrategy,
    /// The host player ID (for HostPriority strategy).
    host_player: Option<u8>,
}

impl ConflictResolver {
    /// Create a new conflict resolver with the given strategy.
    pub fn new(strategy: ResolutionStrategy) -> Self {
        Self {
            strategy,
            host_player: None,
        }
    }

    /// Set the host player for HostPriority resolution.
    pub fn with_host(mut self, host: u8) -> Self {
        self.host_player = Some(host);
        self
    }

    /// Get the current strategy.
    pub fn strategy(&self) -> ResolutionStrategy {
        self.strategy
    }

    /// Get the host player if set.
    pub fn host_player(&self) -> Option<u8> {
        self.host_player
    }

    /// Resolve a conflict given the relevant events.
    pub fn resolve(&self, conflict: &ConflictType, events: &[GameEvent]) -> Resolution {
        match conflict {
            ConflictType::ConcurrentModification {
                player_a, player_b, ..
            } => self.resolve_concurrent_modification(*player_a, *player_b, events),

            ConflictType::TimestampCollision {
                event_a_id,
                event_b_id,
                ..
            } => self.resolve_timestamp_collision(event_a_id, event_b_id, events),

            ConflictType::MissingPredecessor { .. } => {
                // Missing predecessor always requires resync
                Resolution::RequestResync
            }

            ConflictType::InvalidTransition { event_id, .. } => {
                // Invalid transitions are always rejected
                Resolution::Reject(event_id.clone())
            }
        }
    }

    /// Resolve a concurrent modification conflict.
    fn resolve_concurrent_modification(
        &self,
        _player_a: PlayerId,
        _player_b: PlayerId,
        events: &[GameEvent],
    ) -> Resolution {
        match self.strategy {
            ResolutionStrategy::FirstWins => {
                // Find earliest event
                let event = events.iter().min_by_key(|e| (e.timestamp, &e.id));
                match event {
                    Some(e) => Resolution::Accept(e.id.clone()),
                    None => Resolution::RequestResync,
                }
            }
            ResolutionStrategy::LastWins => {
                // Find latest event
                let event = events.iter().max_by_key(|e| (e.timestamp, &e.id));
                match event {
                    Some(e) => Resolution::Accept(e.id.clone()),
                    None => Resolution::RequestResync,
                }
            }
            ResolutionStrategy::HostPriority => {
                if let Some(host) = self.host_player {
                    // Find host's event
                    let host_event = events.iter().find(|e| e.player_id == host);
                    match host_event {
                        Some(e) => Resolution::Accept(e.id.clone()),
                        None => {
                            // No host event, fall back to first wins
                            let event = events.iter().min_by_key(|e| (e.timestamp, &e.id));
                            match event {
                                Some(e) => Resolution::Accept(e.id.clone()),
                                None => Resolution::RequestResync,
                            }
                        }
                    }
                } else {
                    // No host set, need resync
                    Resolution::RequestResync
                }
            }
            ResolutionStrategy::Merge => {
                // For now, merge falls back to first wins for concurrent modifications
                // A proper merge would need game-specific logic
                let event = events.iter().min_by_key(|e| (e.timestamp, &e.id));
                match event {
                    Some(e) => Resolution::Accept(e.id.clone()),
                    None => Resolution::RequestResync,
                }
            }
            ResolutionStrategy::Reject => {
                // Reject the later event
                let later_event = events.iter().max_by_key(|e| (e.timestamp, &e.id));
                match later_event {
                    Some(e) => Resolution::Reject(e.id.clone()),
                    None => Resolution::RequestResync,
                }
            }
        }
    }

    /// Resolve a timestamp collision conflict.
    fn resolve_timestamp_collision(
        &self,
        event_a_id: &str,
        event_b_id: &str,
        events: &[GameEvent],
    ) -> Resolution {
        let event_a = events.iter().find(|e| e.id == event_a_id);
        let event_b = events.iter().find(|e| e.id == event_b_id);

        match (event_a, event_b) {
            (Some(a), Some(b)) => {
                match self.strategy {
                    ResolutionStrategy::FirstWins | ResolutionStrategy::Merge => {
                        // Use lexicographic ordering of IDs for determinism
                        if a.id < b.id {
                            Resolution::Accept(a.id.clone())
                        } else {
                            Resolution::Accept(b.id.clone())
                        }
                    }
                    ResolutionStrategy::LastWins => {
                        // Use reverse lexicographic ordering
                        if a.id > b.id {
                            Resolution::Accept(a.id.clone())
                        } else {
                            Resolution::Accept(b.id.clone())
                        }
                    }
                    ResolutionStrategy::HostPriority => {
                        if let Some(host) = self.host_player {
                            if a.player_id == host {
                                Resolution::Accept(a.id.clone())
                            } else if b.player_id == host {
                                Resolution::Accept(b.id.clone())
                            } else {
                                // Neither is host, use first wins
                                if a.id < b.id {
                                    Resolution::Accept(a.id.clone())
                                } else {
                                    Resolution::Accept(b.id.clone())
                                }
                            }
                        } else {
                            Resolution::RequestResync
                        }
                    }
                    ResolutionStrategy::Reject => {
                        // Reject the lexicographically later one
                        if a.id > b.id {
                            Resolution::Reject(a.id.clone())
                        } else {
                            Resolution::Reject(b.id.clone())
                        }
                    }
                }
            }
            _ => Resolution::RequestResync,
        }
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new(ResolutionStrategy::default())
    }
}

/// Automatically resolve a list of conflicts.
pub fn auto_resolve_conflicts(
    conflicts: &[ConflictType],
    events: &[GameEvent],
    resolver: &ConflictResolver,
) -> Vec<Resolution> {
    conflicts
        .iter()
        .map(|conflict| resolver.resolve(conflict, events))
        .collect()
}

/// Extract entity IDs affected by an event.
fn extract_entity_ids(event: &GameEvent) -> Vec<(EntityType, u64)> {
    use nostr_nations_core::events::GameAction;

    let mut entities = Vec::new();

    // Add player as always affected
    entities.push((EntityType::Player, event.player_id as u64));

    match &event.action {
        GameAction::FoundCity { settler_id, .. } => {
            entities.push((EntityType::Unit, *settler_id));
        }
        GameAction::MoveUnit { unit_id, .. }
        | GameAction::FortifyUnit { unit_id }
        | GameAction::SleepUnit { unit_id }
        | GameAction::WakeUnit { unit_id }
        | GameAction::DeleteUnit { unit_id }
        | GameAction::UpgradeUnit { unit_id, .. }
        | GameAction::BuildImprovement { unit_id, .. }
        | GameAction::BuildRoad { unit_id }
        | GameAction::RemoveFeature { unit_id } => {
            entities.push((EntityType::Unit, *unit_id));
        }
        GameAction::AttackUnit {
            attacker_id,
            defender_id,
            ..
        } => {
            entities.push((EntityType::Unit, *attacker_id));
            entities.push((EntityType::Unit, *defender_id));
        }
        GameAction::AttackCity {
            attacker_id,
            city_id,
            ..
        } => {
            entities.push((EntityType::Unit, *attacker_id));
            entities.push((EntityType::City, *city_id));
        }
        GameAction::SetProduction { city_id, .. }
        | GameAction::BuyItem { city_id, .. }
        | GameAction::AssignCitizen { city_id, .. }
        | GameAction::UnassignCitizen { city_id, .. }
        | GameAction::SellBuilding { city_id, .. } => {
            entities.push((EntityType::City, *city_id));
        }
        GameAction::DeclareWar { target_player } | GameAction::ProposePeace { target_player } => {
            entities.push((EntityType::Player, *target_player as u64));
            entities.push((EntityType::Diplomacy, event.player_id as u64));
        }
        GameAction::AcceptPeace { from_player } | GameAction::RejectPeace { from_player } => {
            entities.push((EntityType::Player, *from_player as u64));
            entities.push((EntityType::Diplomacy, event.player_id as u64));
        }
        _ => {}
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;

    fn create_test_event(id: &str, player_id: u8, timestamp: u64, prev: Option<&str>) -> GameEvent {
        let mut event = GameEvent::new(
            "test_game".to_string(),
            player_id,
            prev.map(|s| s.to_string()),
            1,
            1,
            GameAction::EndTurn,
        );
        event.id = id.to_string();
        event.timestamp = timestamp;
        event
    }

    fn create_unit_move_event(id: &str, player_id: u8, unit_id: u64, timestamp: u64) -> GameEvent {
        let mut event = GameEvent::new(
            "test_game".to_string(),
            player_id,
            None,
            1,
            1,
            GameAction::MoveUnit {
                unit_id,
                path: vec![],
            },
        );
        event.id = id.to_string();
        event.timestamp = timestamp;
        event
    }

    // ==================== ConflictType Tests ====================

    #[test]
    fn test_conflict_type_display() {
        let conflict = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 42,
            player_a: 1,
            player_b: 2,
        };
        let display = format!("{}", conflict);
        assert!(display.contains("Unit"));
        assert!(display.contains("42"));
        assert!(display.contains("1"));
        assert!(display.contains("2"));

        let conflict = ConflictType::TimestampCollision {
            timestamp: 12345,
            event_a_id: "evt_a".to_string(),
            event_b_id: "evt_b".to_string(),
        };
        let display = format!("{}", conflict);
        assert!(display.contains("12345"));
        assert!(display.contains("evt_a"));
        assert!(display.contains("evt_b"));

        let conflict = ConflictType::MissingPredecessor {
            event_id: "evt1".to_string(),
            missing_id: "evt0".to_string(),
        };
        let display = format!("{}", conflict);
        assert!(display.contains("evt1"));
        assert!(display.contains("evt0"));

        let conflict = ConflictType::InvalidTransition {
            event_id: "evt1".to_string(),
            reason: "Game already started".to_string(),
        };
        let display = format!("{}", conflict);
        assert!(display.contains("evt1"));
        assert!(display.contains("Game already started"));
    }

    #[test]
    fn test_conflict_type_equality() {
        let c1 = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 1,
            player_a: 1,
            player_b: 2,
        };
        let c2 = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 1,
            player_a: 1,
            player_b: 2,
        };
        let c3 = ConflictType::ConcurrentModification {
            entity_type: EntityType::City,
            entity_id: 1,
            player_a: 1,
            player_b: 2,
        };

        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
    }

    // ==================== ConflictDetector Tests ====================

    #[test]
    fn test_conflict_detector_new() {
        let detector = ConflictDetector::new();
        assert_eq!(detector.event_count(), 0);
    }

    #[test]
    fn test_conflict_detector_add_event() {
        let mut detector = ConflictDetector::new();
        let event = create_test_event("evt1", 0, 1000, None);

        detector.add_event(event.clone());

        assert_eq!(detector.event_count(), 1);
        assert!(detector.has_event("evt1"));
        assert!(!detector.has_event("evt2"));
    }

    #[test]
    fn test_conflict_detector_get_event() {
        let mut detector = ConflictDetector::new();
        let event = create_test_event("evt1", 0, 1000, None);

        detector.add_event(event.clone());

        let retrieved = detector.get_event("evt1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "evt1");

        assert!(detector.get_event("nonexistent").is_none());
    }

    #[test]
    fn test_conflict_detector_clear() {
        let mut detector = ConflictDetector::new();
        detector.add_event(create_test_event("evt1", 0, 1000, None));
        detector.add_event(create_test_event("evt2", 0, 2000, Some("evt1")));

        assert_eq!(detector.event_count(), 2);

        detector.clear();

        assert_eq!(detector.event_count(), 0);
        assert!(!detector.has_event("evt1"));
    }

    #[test]
    fn test_conflict_detector_missing_predecessor() {
        let mut detector = ConflictDetector::new();

        // Add an initial event
        let evt1 = create_test_event("evt1", 0, 1000, None);
        detector.add_event(evt1);

        // Check an event with missing predecessor
        let evt3 = create_test_event("evt3", 0, 3000, Some("evt2"));
        let conflicts = detector.check_event(&evt3);

        assert_eq!(conflicts.len(), 1);
        assert!(matches!(
            &conflicts[0],
            ConflictType::MissingPredecessor {
                event_id,
                missing_id
            } if event_id == "evt3" && missing_id == "evt2"
        ));
    }

    #[test]
    fn test_conflict_detector_timestamp_collision() {
        let mut detector = ConflictDetector::new();

        // Add first event - EndTurn action
        let evt1 = create_test_event("evt1", 0, 1000, None);
        detector.add_event(evt1);

        // Check event with same timestamp but different action (different description)
        let mut evt2 = GameEvent::new(
            "test_game".to_string(),
            1,
            None,
            1,
            1,
            GameAction::FortifyUnit { unit_id: 42 },
        );
        evt2.id = "evt2".to_string();
        evt2.timestamp = 1000;

        let conflicts = detector.check_event(&evt2);

        assert_eq!(conflicts.len(), 1);
        assert!(matches!(
            &conflicts[0],
            ConflictType::TimestampCollision { timestamp, .. } if *timestamp == 1000
        ));
    }

    #[test]
    fn test_conflict_detector_concurrent_modification() {
        let mut detector = ConflictDetector::new();

        // Player 1 moves unit 42
        let evt1 = create_unit_move_event("evt1", 1, 42, 1000);
        detector.add_event(evt1);

        // Player 2 moves the same unit 42 within 1 second
        let evt2 = create_unit_move_event("evt2", 2, 42, 1500);
        let conflicts = detector.check_event(&evt2);

        assert!(conflicts.iter().any(|c| matches!(
            c,
            ConflictType::ConcurrentModification {
                entity_type: EntityType::Unit,
                entity_id: 42,
                ..
            }
        )));
    }

    #[test]
    fn test_conflict_detector_no_concurrent_modification_same_player() {
        let mut detector = ConflictDetector::new();

        // Player 1 moves unit 42 twice
        let evt1 = create_unit_move_event("evt1", 1, 42, 1000);
        detector.add_event(evt1);

        let evt2 = create_unit_move_event("evt2", 1, 42, 1500);
        let conflicts = detector.check_event(&evt2);

        // Should not be a concurrent modification (same player)
        assert!(conflicts
            .iter()
            .all(|c| !matches!(c, ConflictType::ConcurrentModification { .. })));
    }

    #[test]
    fn test_conflict_detector_invalid_transition_double_start() {
        let mut detector = ConflictDetector::new();

        // Add game start event
        let mut evt1 = GameEvent::new(
            "test_game".to_string(),
            0,
            None,
            0,
            1,
            GameAction::StartGame,
        );
        evt1.id = "evt1".to_string();
        evt1.timestamp = 1000;
        detector.add_event(evt1);

        // Try to start again
        let mut evt2 = GameEvent::new(
            "test_game".to_string(),
            0,
            Some("evt1".to_string()),
            0,
            2,
            GameAction::StartGame,
        );
        evt2.id = "evt2".to_string();
        evt2.timestamp = 2000;

        let conflicts = detector.check_event(&evt2);

        assert!(conflicts.iter().any(|c| matches!(
            c,
            ConflictType::InvalidTransition { reason, .. } if reason.contains("already started")
        )));
    }

    #[test]
    fn test_conflict_detector_invalid_transition_join_after_start() {
        let mut detector = ConflictDetector::new();

        // Add game start event
        let mut evt1 = GameEvent::new(
            "test_game".to_string(),
            0,
            None,
            0,
            1,
            GameAction::StartGame,
        );
        evt1.id = "evt1".to_string();
        evt1.timestamp = 1000;
        detector.add_event(evt1);

        // Try to join after start
        let mut evt2 = GameEvent::new(
            "test_game".to_string(),
            1,
            Some("evt1".to_string()),
            0,
            2,
            GameAction::JoinGame {
                player_name: "Alice".to_string(),
                civilization_id: "rome".to_string(),
            },
        );
        evt2.id = "evt2".to_string();
        evt2.timestamp = 2000;

        let conflicts = detector.check_event(&evt2);

        assert!(conflicts.iter().any(|c| matches!(
            c,
            ConflictType::InvalidTransition { reason, .. } if reason.contains("Cannot join")
        )));
    }

    #[test]
    fn test_conflict_detector_invalid_transition_move_deleted_unit() {
        let mut detector = ConflictDetector::new();

        // Delete unit 42
        let mut evt1 = GameEvent::new(
            "test_game".to_string(),
            0,
            None,
            1,
            1,
            GameAction::DeleteUnit { unit_id: 42 },
        );
        evt1.id = "evt1".to_string();
        evt1.timestamp = 1000;
        detector.add_event(evt1);

        // Try to move deleted unit
        let mut evt2 = GameEvent::new(
            "test_game".to_string(),
            0,
            Some("evt1".to_string()),
            1,
            2,
            GameAction::MoveUnit {
                unit_id: 42,
                path: vec![],
            },
        );
        evt2.id = "evt2".to_string();
        evt2.timestamp = 2000;

        let conflicts = detector.check_event(&evt2);

        assert!(conflicts.iter().any(|c| matches!(
            c,
            ConflictType::InvalidTransition { reason, .. } if reason.contains("deleted")
        )));
    }

    // ==================== ResolutionStrategy Tests ====================

    #[test]
    fn test_resolution_strategy_default() {
        let strategy = ResolutionStrategy::default();
        assert_eq!(strategy, ResolutionStrategy::FirstWins);
    }

    // ==================== Resolution Tests ====================

    #[test]
    fn test_resolution_helpers() {
        let accept = Resolution::Accept("evt1".to_string());
        assert!(accept.is_accept());
        assert!(!accept.is_reject());
        assert!(!accept.is_merge());
        assert!(!accept.needs_resync());
        assert_eq!(accept.accepted_id(), Some("evt1"));
        assert!(accept.rejected_id().is_none());

        let reject = Resolution::Reject("evt2".to_string());
        assert!(!reject.is_accept());
        assert!(reject.is_reject());
        assert_eq!(reject.rejected_id(), Some("evt2"));

        let resync = Resolution::RequestResync;
        assert!(resync.needs_resync());
    }

    // ==================== ConflictResolver Tests ====================

    #[test]
    fn test_conflict_resolver_new() {
        let resolver = ConflictResolver::new(ResolutionStrategy::FirstWins);
        assert_eq!(resolver.strategy(), ResolutionStrategy::FirstWins);
        assert!(resolver.host_player().is_none());
    }

    #[test]
    fn test_conflict_resolver_with_host() {
        let resolver = ConflictResolver::new(ResolutionStrategy::HostPriority).with_host(42);
        assert_eq!(resolver.strategy(), ResolutionStrategy::HostPriority);
        assert_eq!(resolver.host_player(), Some(42));
    }

    #[test]
    fn test_conflict_resolver_default() {
        let resolver = ConflictResolver::default();
        assert_eq!(resolver.strategy(), ResolutionStrategy::FirstWins);
    }

    #[test]
    fn test_resolve_concurrent_modification_first_wins() {
        let resolver = ConflictResolver::new(ResolutionStrategy::FirstWins);

        let evt1 = create_test_event("evt_b", 1, 2000, None);
        let evt2 = create_test_event("evt_a", 2, 1000, None);
        let events = vec![evt1, evt2];

        let conflict = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 42,
            player_a: 1,
            player_b: 2,
        };

        let resolution = resolver.resolve(&conflict, &events);
        assert_eq!(resolution, Resolution::Accept("evt_a".to_string()));
    }

    #[test]
    fn test_resolve_concurrent_modification_last_wins() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWins);

        let evt1 = create_test_event("evt_a", 1, 1000, None);
        let evt2 = create_test_event("evt_b", 2, 2000, None);
        let events = vec![evt1, evt2];

        let conflict = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 42,
            player_a: 1,
            player_b: 2,
        };

        let resolution = resolver.resolve(&conflict, &events);
        assert_eq!(resolution, Resolution::Accept("evt_b".to_string()));
    }

    #[test]
    fn test_resolve_concurrent_modification_host_priority() {
        let resolver = ConflictResolver::new(ResolutionStrategy::HostPriority).with_host(2);

        let evt1 = create_test_event("evt_a", 1, 1000, None); // Not host
        let evt2 = create_test_event("evt_b", 2, 2000, None); // Host
        let events = vec![evt1, evt2];

        let conflict = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 42,
            player_a: 1,
            player_b: 2,
        };

        let resolution = resolver.resolve(&conflict, &events);
        assert_eq!(resolution, Resolution::Accept("evt_b".to_string()));
    }

    #[test]
    fn test_resolve_concurrent_modification_host_priority_no_host_event() {
        let resolver = ConflictResolver::new(ResolutionStrategy::HostPriority).with_host(99);

        let evt1 = create_test_event("evt_a", 1, 1000, None);
        let evt2 = create_test_event("evt_b", 2, 2000, None);
        let events = vec![evt1, evt2];

        let conflict = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 42,
            player_a: 1,
            player_b: 2,
        };

        // Falls back to first wins
        let resolution = resolver.resolve(&conflict, &events);
        assert_eq!(resolution, Resolution::Accept("evt_a".to_string()));
    }

    #[test]
    fn test_resolve_concurrent_modification_reject() {
        let resolver = ConflictResolver::new(ResolutionStrategy::Reject);

        let evt1 = create_test_event("evt_a", 1, 1000, None);
        let evt2 = create_test_event("evt_b", 2, 2000, None);
        let events = vec![evt1, evt2];

        let conflict = ConflictType::ConcurrentModification {
            entity_type: EntityType::Unit,
            entity_id: 42,
            player_a: 1,
            player_b: 2,
        };

        // Rejects the later event
        let resolution = resolver.resolve(&conflict, &events);
        assert_eq!(resolution, Resolution::Reject("evt_b".to_string()));
    }

    #[test]
    fn test_resolve_timestamp_collision_first_wins() {
        let resolver = ConflictResolver::new(ResolutionStrategy::FirstWins);

        let evt1 = create_test_event("evt_b", 1, 1000, None);
        let evt2 = create_test_event("evt_a", 2, 1000, None);
        let events = vec![evt1, evt2];

        let conflict = ConflictType::TimestampCollision {
            timestamp: 1000,
            event_a_id: "evt_b".to_string(),
            event_b_id: "evt_a".to_string(),
        };

        // Lexicographically first ID wins
        let resolution = resolver.resolve(&conflict, &events);
        assert_eq!(resolution, Resolution::Accept("evt_a".to_string()));
    }

    #[test]
    fn test_resolve_timestamp_collision_last_wins() {
        let resolver = ConflictResolver::new(ResolutionStrategy::LastWins);

        let evt1 = create_test_event("evt_a", 1, 1000, None);
        let evt2 = create_test_event("evt_b", 2, 1000, None);
        let events = vec![evt1, evt2];

        let conflict = ConflictType::TimestampCollision {
            timestamp: 1000,
            event_a_id: "evt_a".to_string(),
            event_b_id: "evt_b".to_string(),
        };

        // Lexicographically last ID wins
        let resolution = resolver.resolve(&conflict, &events);
        assert_eq!(resolution, Resolution::Accept("evt_b".to_string()));
    }

    #[test]
    fn test_resolve_missing_predecessor() {
        let resolver = ConflictResolver::new(ResolutionStrategy::FirstWins);

        let conflict = ConflictType::MissingPredecessor {
            event_id: "evt2".to_string(),
            missing_id: "evt1".to_string(),
        };

        let resolution = resolver.resolve(&conflict, &[]);
        assert_eq!(resolution, Resolution::RequestResync);
    }

    #[test]
    fn test_resolve_invalid_transition() {
        let resolver = ConflictResolver::new(ResolutionStrategy::FirstWins);

        let conflict = ConflictType::InvalidTransition {
            event_id: "evt1".to_string(),
            reason: "Game already started".to_string(),
        };

        let resolution = resolver.resolve(&conflict, &[]);
        assert_eq!(resolution, Resolution::Reject("evt1".to_string()));
    }

    // ==================== auto_resolve_conflicts Tests ====================

    #[test]
    fn test_auto_resolve_conflicts() {
        let resolver = ConflictResolver::new(ResolutionStrategy::FirstWins);

        let evt1 = create_test_event("evt_a", 1, 1000, None);
        let evt2 = create_test_event("evt_b", 2, 2000, None);
        let events = vec![evt1, evt2];

        let conflicts = vec![
            ConflictType::ConcurrentModification {
                entity_type: EntityType::Unit,
                entity_id: 42,
                player_a: 1,
                player_b: 2,
            },
            ConflictType::MissingPredecessor {
                event_id: "evt3".to_string(),
                missing_id: "evt0".to_string(),
            },
        ];

        let resolutions = auto_resolve_conflicts(&conflicts, &events, &resolver);

        assert_eq!(resolutions.len(), 2);
        assert!(resolutions[0].is_accept());
        assert!(resolutions[1].needs_resync());
    }

    #[test]
    fn test_auto_resolve_empty_conflicts() {
        let resolver = ConflictResolver::default();
        let resolutions = auto_resolve_conflicts(&[], &[], &resolver);
        assert!(resolutions.is_empty());
    }

    // ==================== extract_entity_ids Tests ====================

    #[test]
    fn test_extract_entity_ids_move_unit() {
        let event = create_unit_move_event("evt1", 1, 42, 1000);
        let entities = extract_entity_ids(&event);

        assert!(entities.contains(&(EntityType::Player, 1)));
        assert!(entities.contains(&(EntityType::Unit, 42)));
    }

    #[test]
    fn test_extract_entity_ids_attack_unit() {
        let mut event = GameEvent::new(
            "test_game".to_string(),
            1,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 10,
                defender_id: 20,
                random: 0.5,
            },
        );
        event.id = "evt1".to_string();

        let entities = extract_entity_ids(&event);

        assert!(entities.contains(&(EntityType::Player, 1)));
        assert!(entities.contains(&(EntityType::Unit, 10)));
        assert!(entities.contains(&(EntityType::Unit, 20)));
    }

    #[test]
    fn test_extract_entity_ids_city_action() {
        use nostr_nations_core::city::ProductionItem;
        use nostr_nations_core::unit::UnitType;

        let mut event = GameEvent::new(
            "test_game".to_string(),
            1,
            None,
            1,
            1,
            GameAction::SetProduction {
                city_id: 5,
                item: ProductionItem::Unit(UnitType::Warrior),
            },
        );
        event.id = "evt1".to_string();

        let entities = extract_entity_ids(&event);

        assert!(entities.contains(&(EntityType::Player, 1)));
        assert!(entities.contains(&(EntityType::City, 5)));
    }

    #[test]
    fn test_extract_entity_ids_diplomacy() {
        let mut event = GameEvent::new(
            "test_game".to_string(),
            1,
            None,
            1,
            1,
            GameAction::DeclareWar { target_player: 2 },
        );
        event.id = "evt1".to_string();

        let entities = extract_entity_ids(&event);

        assert!(entities.contains(&(EntityType::Player, 1)));
        assert!(entities.contains(&(EntityType::Player, 2)));
        assert!(entities.contains(&(EntityType::Diplomacy, 1)));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_conflict_detection_and_resolution_flow() {
        let mut detector = ConflictDetector::new();
        let resolver = ConflictResolver::new(ResolutionStrategy::FirstWins);

        // Player 1 moves unit
        let evt1 = create_unit_move_event("evt1", 1, 42, 1000);
        let conflicts1 = detector.check_event(&evt1);
        assert!(conflicts1.is_empty());
        detector.add_event(evt1.clone());

        // Player 2 moves the same unit concurrently
        let evt2 = create_unit_move_event("evt2", 2, 42, 1100);
        let conflicts2 = detector.check_event(&evt2);

        assert!(!conflicts2.is_empty());

        // Resolve conflicts
        let resolutions = auto_resolve_conflicts(&conflicts2, &[evt1, evt2], &resolver);

        // First event should win
        assert!(resolutions.iter().any(|r| r.is_accept()));
    }

    #[test]
    fn test_multiple_conflict_types() {
        let mut detector = ConflictDetector::new();

        // Add initial event - EndTurn action
        let evt1 = create_test_event("evt1", 0, 1000, None);
        detector.add_event(evt1);

        // Create an event with both missing predecessor and timestamp collision
        // Use a different action to trigger timestamp collision (different description)
        let mut evt2 = GameEvent::new(
            "test_game".to_string(),
            1,
            Some("missing".to_string()),
            1,
            1,
            GameAction::FortifyUnit { unit_id: 42 },
        );
        evt2.id = "evt2".to_string();
        evt2.timestamp = 1000;

        let conflicts = detector.check_event(&evt2);

        // Should detect multiple conflicts
        assert!(conflicts.len() >= 2);
        assert!(conflicts
            .iter()
            .any(|c| matches!(c, ConflictType::MissingPredecessor { .. })));
        assert!(conflicts
            .iter()
            .any(|c| matches!(c, ConflictType::TimestampCollision { .. })));
    }
}

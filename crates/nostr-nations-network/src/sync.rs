//! Game state synchronization protocol.
//!
//! This module handles syncing game state between peers:
//! - Initial sync when joining a game
//! - Incremental sync during gameplay
//! - Conflict resolution for simultaneous actions
//!
//! # Sync Protocol
//!
//! 1. Client sends SyncRequest with their latest turn/sequence
//! 2. Host sends all events after that point
//! 3. Client applies events and confirms sync
//! 4. During gameplay, events are broadcast to all peers

use nostr_nations_core::events::{EventChain, GameEvent};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// State of synchronization.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncState {
    /// Not syncing.
    Idle,
    /// Requesting sync from host.
    Requesting,
    /// Receiving events.
    Receiving,
    /// Applying events to local state.
    Applying,
    /// Sync complete.
    Synced,
    /// Sync failed.
    Failed(String),
}

/// Request for game state sync.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncRequest {
    /// Game ID.
    pub game_id: String,
    /// Player making the request.
    pub player_id: u32,
    /// Last turn we have events for.
    pub last_turn: u32,
    /// Last sequence number we have.
    pub last_sequence: u32,
    /// Last event ID we have (for chain validation).
    pub last_event_id: Option<String>,
}

/// Response with events for sync.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SyncResponse {
    /// Game ID.
    pub game_id: String,
    /// Whether more events are available.
    pub has_more: bool,
    /// Events in this response.
    pub events: Vec<GameEvent>,
    /// Current game turn.
    pub current_turn: u32,
    /// Hash of full event chain (for validation).
    pub chain_hash: Option<String>,
}

/// Result of a sync operation.
#[derive(Clone, Debug)]
pub struct SyncResult {
    /// Number of events received.
    pub events_received: usize,
    /// Number of events applied.
    pub events_applied: usize,
    /// Final sync state.
    pub state: SyncState,
    /// Any errors encountered.
    pub errors: Vec<String>,
}

/// Manages game state synchronization.
pub struct SyncManager {
    /// Current sync state.
    state: SyncState,
    /// Game ID.
    game_id: String,
    /// Our player ID.
    player_id: u32,
    /// Pending events to apply.
    pending_events: VecDeque<GameEvent>,
    /// Events we've confirmed as synced.
    confirmed_turn: u32,
    /// Last confirmed sequence.
    confirmed_sequence: u32,
    /// Last confirmed event ID.
    confirmed_event_id: Option<String>,
}

impl SyncManager {
    /// Create a new sync manager.
    pub fn new(game_id: String, player_id: u32) -> Self {
        Self {
            state: SyncState::Idle,
            game_id,
            player_id,
            pending_events: VecDeque::new(),
            confirmed_turn: 0,
            confirmed_sequence: 0,
            confirmed_event_id: None,
        }
    }

    /// Get current sync state.
    pub fn state(&self) -> &SyncState {
        &self.state
    }

    /// Check if we're fully synced.
    pub fn is_synced(&self) -> bool {
        self.state == SyncState::Synced && self.pending_events.is_empty()
    }

    /// Create a sync request for current state.
    pub fn create_request(&mut self) -> SyncRequest {
        self.state = SyncState::Requesting;

        SyncRequest {
            game_id: self.game_id.clone(),
            player_id: self.player_id,
            last_turn: self.confirmed_turn,
            last_sequence: self.confirmed_sequence,
            last_event_id: self.confirmed_event_id.clone(),
        }
    }

    /// Process a sync response from the host.
    pub fn handle_response(&mut self, response: SyncResponse) -> SyncResult {
        if response.game_id != self.game_id {
            self.state = SyncState::Failed("Wrong game ID".to_string());
            return SyncResult {
                events_received: 0,
                events_applied: 0,
                state: self.state.clone(),
                errors: vec!["Wrong game ID".to_string()],
            };
        }

        self.state = SyncState::Receiving;
        let events_received = response.events.len();

        // Queue events for application
        for event in response.events {
            self.pending_events.push_back(event);
        }

        if response.has_more {
            // Need to request more events
            self.state = SyncState::Requesting;
        } else {
            self.state = SyncState::Applying;
        }

        SyncResult {
            events_received,
            events_applied: 0,
            state: self.state.clone(),
            errors: Vec::new(),
        }
    }

    /// Get the next pending event to apply.
    pub fn next_event(&mut self) -> Option<GameEvent> {
        self.pending_events.pop_front()
    }

    /// Confirm an event was successfully applied.
    pub fn confirm_event(&mut self, event: &GameEvent) {
        self.confirmed_turn = event.turn;
        self.confirmed_sequence = event.sequence;
        self.confirmed_event_id = Some(event.id.clone());

        if self.pending_events.is_empty() {
            self.state = SyncState::Synced;
        }
    }

    /// Report a failed event application.
    pub fn report_failure(&mut self, error: String) {
        self.state = SyncState::Failed(error);
        self.pending_events.clear();
    }

    /// Get the number of pending events.
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    /// Reset sync state (for reconnection).
    pub fn reset(&mut self) {
        self.state = SyncState::Idle;
        self.pending_events.clear();
    }
}

/// Creates sync responses for host.
pub struct SyncResponder {
    /// Game ID.
    game_id: String,
    /// Maximum events per response.
    max_events_per_response: usize,
}

impl SyncResponder {
    /// Create a new sync responder.
    pub fn new(game_id: String) -> Self {
        Self {
            game_id,
            max_events_per_response: 100,
        }
    }

    /// Set maximum events per response.
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events_per_response = max;
        self
    }

    /// Create a sync response for a request.
    pub fn respond(&self, request: &SyncRequest, chain: &EventChain) -> SyncResponse {
        let mut events = Vec::new();
        let mut found_start = request.last_event_id.is_none();

        for event in chain.events() {
            // Find starting point
            if !found_start {
                if Some(&event.id) == request.last_event_id.as_ref() {
                    found_start = true;
                }
                continue;
            }

            // Check if event is after requested point
            if event.turn > request.last_turn
                || (event.turn == request.last_turn && event.sequence > request.last_sequence)
            {
                events.push(event.clone());

                if events.len() >= self.max_events_per_response {
                    break;
                }
            }
        }

        let has_more = events.len() >= self.max_events_per_response;
        let current_turn = chain.last().map(|e| e.turn).unwrap_or(0);

        SyncResponse {
            game_id: self.game_id.clone(),
            has_more,
            events,
            current_turn,
            chain_hash: None, // Could compute hash for validation
        }
    }
}

/// Tracks which events peers have confirmed.
#[derive(Default)]
pub struct PeerSyncTracker {
    /// Peer ID -> (turn, sequence) of last confirmed event.
    peer_progress: std::collections::HashMap<String, (u32, u32)>,
}

impl PeerSyncTracker {
    /// Create a new peer sync tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Update a peer's progress.
    pub fn update_progress(&mut self, peer_id: &str, turn: u32, sequence: u32) {
        self.peer_progress
            .insert(peer_id.to_string(), (turn, sequence));
    }

    /// Get a peer's progress.
    pub fn get_progress(&self, peer_id: &str) -> Option<(u32, u32)> {
        self.peer_progress.get(peer_id).copied()
    }

    /// Get the minimum progress across all peers (for pruning).
    pub fn min_progress(&self) -> Option<(u32, u32)> {
        self.peer_progress.values().min().copied()
    }

    /// Remove a peer from tracking.
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peer_progress.remove(peer_id);
    }

    /// Check if all peers are at least at the given point.
    pub fn all_at_least(&self, turn: u32, sequence: u32) -> bool {
        self.peer_progress
            .values()
            .all(|(t, s)| *t > turn || (*t == turn && *s >= sequence))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;

    fn create_test_event(id: &str, turn: u32, seq: u32) -> GameEvent {
        let mut event = GameEvent::new(
            "test_game".to_string(),
            0,
            None,
            turn,
            seq,
            GameAction::EndTurn,
        );
        event.id = id.to_string();
        event
    }

    fn create_test_event_with_prev(id: &str, prev: Option<&str>, turn: u32, seq: u32) -> GameEvent {
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

    // ==================== SyncState Tests ====================

    #[test]
    fn test_sync_state_equality() {
        assert_eq!(SyncState::Idle, SyncState::Idle);
        assert_eq!(SyncState::Requesting, SyncState::Requesting);
        assert_eq!(SyncState::Receiving, SyncState::Receiving);
        assert_eq!(SyncState::Applying, SyncState::Applying);
        assert_eq!(SyncState::Synced, SyncState::Synced);
        assert_eq!(
            SyncState::Failed("error".to_string()),
            SyncState::Failed("error".to_string())
        );

        assert_ne!(SyncState::Idle, SyncState::Requesting);
        assert_ne!(
            SyncState::Failed("error1".to_string()),
            SyncState::Failed("error2".to_string())
        );
    }

    #[test]
    fn test_sync_state_debug() {
        let state = SyncState::Requesting;
        assert!(format!("{:?}", state).contains("Requesting"));

        let failed = SyncState::Failed("network error".to_string());
        assert!(format!("{:?}", failed).contains("network error"));
    }

    // ==================== SyncRequest Serialization Tests ====================

    #[test]
    fn test_sync_request_serialization() {
        let request = SyncRequest {
            game_id: "game123".to_string(),
            player_id: 42,
            last_turn: 5,
            last_sequence: 10,
            last_event_id: Some("evt_abc".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: SyncRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.game_id, request.game_id);
        assert_eq!(deserialized.player_id, request.player_id);
        assert_eq!(deserialized.last_turn, request.last_turn);
        assert_eq!(deserialized.last_sequence, request.last_sequence);
        assert_eq!(deserialized.last_event_id, request.last_event_id);
    }

    #[test]
    fn test_sync_request_serialization_no_last_event() {
        let request = SyncRequest {
            game_id: "game123".to_string(),
            player_id: 0,
            last_turn: 0,
            last_sequence: 0,
            last_event_id: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: SyncRequest = serde_json::from_str(&json).unwrap();

        assert!(deserialized.last_event_id.is_none());
    }

    // ==================== SyncResponse Serialization Tests ====================

    #[test]
    fn test_sync_response_serialization() {
        let response = SyncResponse {
            game_id: "game123".to_string(),
            has_more: true,
            events: vec![create_test_event("evt1", 1, 1)],
            current_turn: 5,
            chain_hash: Some("abc123".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SyncResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.game_id, response.game_id);
        assert!(deserialized.has_more);
        assert_eq!(deserialized.events.len(), 1);
        assert_eq!(deserialized.current_turn, 5);
        assert_eq!(deserialized.chain_hash, Some("abc123".to_string()));
    }

    #[test]
    fn test_sync_response_serialization_empty_events() {
        let response = SyncResponse {
            game_id: "game123".to_string(),
            has_more: false,
            events: vec![],
            current_turn: 0,
            chain_hash: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SyncResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.events.is_empty());
        assert!(!deserialized.has_more);
    }

    #[test]
    fn test_sync_response_serialization_multiple_events() {
        let response = SyncResponse {
            game_id: "game123".to_string(),
            has_more: false,
            events: vec![
                create_test_event("evt1", 1, 1),
                create_test_event("evt2", 1, 2),
                create_test_event("evt3", 2, 1),
            ],
            current_turn: 2,
            chain_hash: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: SyncResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.events.len(), 3);
        assert_eq!(deserialized.events[0].id, "evt1");
        assert_eq!(deserialized.events[1].id, "evt2");
        assert_eq!(deserialized.events[2].id, "evt3");
    }

    // ==================== SyncManager Tests ====================

    #[test]
    fn test_sync_manager_new() {
        let manager = SyncManager::new("game1".to_string(), 42);

        assert_eq!(*manager.state(), SyncState::Idle);
        assert!(!manager.is_synced());
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_sync_manager_request() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        let request = manager.create_request();

        assert_eq!(request.game_id, "game1");
        assert_eq!(request.player_id, 0);
        assert_eq!(request.last_turn, 0);
        assert_eq!(*manager.state(), SyncState::Requesting);
    }

    #[test]
    fn test_sync_manager_request_with_progress() {
        let mut manager = SyncManager::new("game1".to_string(), 1);

        // Simulate having confirmed some events
        let response = SyncResponse {
            game_id: "game1".to_string(),
            has_more: false,
            events: vec![create_test_event("evt1", 3, 5)],
            current_turn: 3,
            chain_hash: None,
        };

        manager.create_request();
        manager.handle_response(response);

        let evt = manager.next_event().unwrap();
        manager.confirm_event(&evt);

        // Now create a new request
        let request = manager.create_request();
        assert_eq!(request.last_turn, 3);
        assert_eq!(request.last_sequence, 5);
        assert_eq!(request.last_event_id, Some("evt1".to_string()));
    }

    #[test]
    fn test_sync_manager_response() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let response = SyncResponse {
            game_id: "game1".to_string(),
            has_more: false,
            events: vec![create_test_event("evt1", 1, 1)],
            current_turn: 1,
            chain_hash: None,
        };

        let result = manager.handle_response(response);
        assert_eq!(result.events_received, 1);
        assert_eq!(*manager.state(), SyncState::Applying);
    }

    #[test]
    fn test_sync_manager_response_wrong_game() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let response = SyncResponse {
            game_id: "wrong_game".to_string(),
            has_more: false,
            events: vec![],
            current_turn: 0,
            chain_hash: None,
        };

        let result = manager.handle_response(response);
        assert_eq!(result.events_received, 0);
        assert!(matches!(manager.state(), SyncState::Failed(_)));
        assert!(result.errors.contains(&"Wrong game ID".to_string()));
    }

    #[test]
    fn test_sync_manager_response_has_more() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let response = SyncResponse {
            game_id: "game1".to_string(),
            has_more: true,
            events: vec![create_test_event("evt1", 1, 1)],
            current_turn: 1,
            chain_hash: None,
        };

        let result = manager.handle_response(response);
        assert_eq!(result.events_received, 1);
        assert_eq!(*manager.state(), SyncState::Requesting);
    }

    #[test]
    fn test_sync_manager_apply_events() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let events = vec![
            create_test_event("evt1", 1, 1),
            create_test_event("evt2", 1, 2),
        ];

        let response = SyncResponse {
            game_id: "game1".to_string(),
            has_more: false,
            events,
            current_turn: 1,
            chain_hash: None,
        };

        manager.handle_response(response);

        // Apply first event
        let evt1 = manager.next_event().unwrap();
        assert_eq!(evt1.id, "evt1");
        manager.confirm_event(&evt1);

        assert!(!manager.is_synced());

        // Apply second event
        let evt2 = manager.next_event().unwrap();
        assert_eq!(evt2.id, "evt2");
        manager.confirm_event(&evt2);

        assert!(manager.is_synced());
    }

    #[test]
    fn test_sync_manager_next_event_empty() {
        let mut manager = SyncManager::new("game1".to_string(), 0);

        assert!(manager.next_event().is_none());
    }

    #[test]
    fn test_sync_manager_report_failure() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let response = SyncResponse {
            game_id: "game1".to_string(),
            has_more: false,
            events: vec![create_test_event("evt1", 1, 1)],
            current_turn: 1,
            chain_hash: None,
        };

        manager.handle_response(response);
        assert_eq!(manager.pending_count(), 1);

        manager.report_failure("Invalid event signature".to_string());

        assert!(matches!(
            manager.state(),
            SyncState::Failed(msg) if msg == "Invalid event signature"
        ));
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_sync_manager_reset() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let response = SyncResponse {
            game_id: "game1".to_string(),
            has_more: false,
            events: vec![create_test_event("evt1", 1, 1)],
            current_turn: 1,
            chain_hash: None,
        };

        manager.handle_response(response);
        assert_eq!(manager.pending_count(), 1);

        manager.reset();

        assert_eq!(*manager.state(), SyncState::Idle);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_sync_manager_is_synced_with_pending() {
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let response = SyncResponse {
            game_id: "game1".to_string(),
            has_more: false,
            events: vec![create_test_event("evt1", 1, 1)],
            current_turn: 1,
            chain_hash: None,
        };

        manager.handle_response(response);

        // Even though we're in Applying state, we have pending events
        assert!(!manager.is_synced());
    }

    // ==================== SyncResponder Tests ====================

    #[test]
    fn test_sync_responder_new() {
        let responder = SyncResponder::new("game1".to_string());

        // Verify it's created (we can't easily test internals)
        let chain = EventChain::new();
        let request = SyncRequest {
            game_id: "game1".to_string(),
            player_id: 0,
            last_turn: 0,
            last_sequence: 0,
            last_event_id: None,
        };

        let response = responder.respond(&request, &chain);
        assert_eq!(response.game_id, "game1");
    }

    #[test]
    fn test_sync_responder_with_max_events() {
        let responder = SyncResponder::new("game1".to_string()).with_max_events(2);

        let mut chain = EventChain::new();
        chain
            .add(create_test_event_with_prev("evt1", None, 1, 1))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt2", Some("evt1"), 1, 2))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt3", Some("evt2"), 1, 3))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt4", Some("evt3"), 2, 1))
            .unwrap();

        let request = SyncRequest {
            game_id: "game1".to_string(),
            player_id: 0,
            last_turn: 0,
            last_sequence: 0,
            last_event_id: None,
        };

        let response = responder.respond(&request, &chain);
        assert_eq!(response.events.len(), 2);
        assert!(response.has_more);
    }

    #[test]
    fn test_sync_responder_respond_empty_chain() {
        let responder = SyncResponder::new("game1".to_string());

        let chain = EventChain::new();
        let request = SyncRequest {
            game_id: "game1".to_string(),
            player_id: 0,
            last_turn: 0,
            last_sequence: 0,
            last_event_id: None,
        };

        let response = responder.respond(&request, &chain);
        assert!(response.events.is_empty());
        assert!(!response.has_more);
        assert_eq!(response.current_turn, 0);
    }

    #[test]
    fn test_sync_responder_respond_from_beginning() {
        let responder = SyncResponder::new("game1".to_string());

        let mut chain = EventChain::new();
        chain
            .add(create_test_event_with_prev("evt1", None, 1, 1))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt2", Some("evt1"), 1, 2))
            .unwrap();

        let request = SyncRequest {
            game_id: "game1".to_string(),
            player_id: 0,
            last_turn: 0,
            last_sequence: 0,
            last_event_id: None,
        };

        let response = responder.respond(&request, &chain);
        assert_eq!(response.events.len(), 2);
        assert_eq!(response.events[0].id, "evt1");
        assert_eq!(response.events[1].id, "evt2");
        assert_eq!(response.current_turn, 1);
    }

    #[test]
    fn test_sync_responder_respond_from_midpoint() {
        let responder = SyncResponder::new("game1".to_string());

        let mut chain = EventChain::new();
        chain
            .add(create_test_event_with_prev("evt1", None, 1, 1))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt2", Some("evt1"), 1, 2))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt3", Some("evt2"), 2, 1))
            .unwrap();

        let request = SyncRequest {
            game_id: "game1".to_string(),
            player_id: 0,
            last_turn: 1,
            last_sequence: 1,
            last_event_id: Some("evt1".to_string()),
        };

        let response = responder.respond(&request, &chain);
        assert_eq!(response.events.len(), 2);
        assert_eq!(response.events[0].id, "evt2");
        assert_eq!(response.events[1].id, "evt3");
    }

    #[test]
    fn test_sync_responder_respond_fully_synced() {
        let responder = SyncResponder::new("game1".to_string());

        let mut chain = EventChain::new();
        chain
            .add(create_test_event_with_prev("evt1", None, 1, 1))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt2", Some("evt1"), 1, 2))
            .unwrap();

        let request = SyncRequest {
            game_id: "game1".to_string(),
            player_id: 0,
            last_turn: 1,
            last_sequence: 2,
            last_event_id: Some("evt2".to_string()),
        };

        let response = responder.respond(&request, &chain);
        assert!(response.events.is_empty());
        assert!(!response.has_more);
    }

    // ==================== PeerSyncTracker Tests ====================

    #[test]
    fn test_peer_sync_tracker_new() {
        let tracker = PeerSyncTracker::new();

        assert!(tracker.get_progress("any_peer").is_none());
        assert!(tracker.min_progress().is_none());
    }

    #[test]
    fn test_peer_sync_tracker() {
        let mut tracker = PeerSyncTracker::new();

        tracker.update_progress("peer1", 1, 1);
        tracker.update_progress("peer2", 1, 2);
        tracker.update_progress("peer3", 2, 1);

        assert_eq!(tracker.get_progress("peer1"), Some((1, 1)));
        assert_eq!(tracker.min_progress(), Some((1, 1)));

        assert!(tracker.all_at_least(1, 1));
        assert!(!tracker.all_at_least(1, 2));
        assert!(!tracker.all_at_least(2, 1));
    }

    #[test]
    fn test_peer_sync_tracker_update_progress() {
        let mut tracker = PeerSyncTracker::new();

        tracker.update_progress("peer1", 1, 1);
        assert_eq!(tracker.get_progress("peer1"), Some((1, 1)));

        tracker.update_progress("peer1", 2, 5);
        assert_eq!(tracker.get_progress("peer1"), Some((2, 5)));
    }

    #[test]
    fn test_peer_sync_tracker_remove_peer() {
        let mut tracker = PeerSyncTracker::new();

        tracker.update_progress("peer1", 1, 1);
        tracker.update_progress("peer2", 2, 2);

        tracker.remove_peer("peer1");

        assert!(tracker.get_progress("peer1").is_none());
        assert_eq!(tracker.get_progress("peer2"), Some((2, 2)));
    }

    #[test]
    fn test_peer_sync_tracker_remove_nonexistent_peer() {
        let mut tracker = PeerSyncTracker::new();

        // Should not panic
        tracker.remove_peer("nonexistent");
    }

    #[test]
    fn test_peer_sync_tracker_min_progress_single_peer() {
        let mut tracker = PeerSyncTracker::new();

        tracker.update_progress("peer1", 5, 10);

        assert_eq!(tracker.min_progress(), Some((5, 10)));
    }

    #[test]
    fn test_peer_sync_tracker_min_progress_multiple_peers() {
        let mut tracker = PeerSyncTracker::new();

        tracker.update_progress("peer1", 3, 5);
        tracker.update_progress("peer2", 2, 10);
        tracker.update_progress("peer3", 4, 1);

        // Minimum is (2, 10) because turn 2 < turn 3 < turn 4
        assert_eq!(tracker.min_progress(), Some((2, 10)));
    }

    #[test]
    fn test_peer_sync_tracker_all_at_least_empty() {
        let tracker = PeerSyncTracker::new();

        // With no peers, all_at_least should return true (vacuously true)
        assert!(tracker.all_at_least(100, 100));
    }

    #[test]
    fn test_peer_sync_tracker_all_at_least_various() {
        let mut tracker = PeerSyncTracker::new();

        tracker.update_progress("peer1", 5, 5);
        tracker.update_progress("peer2", 5, 10);
        tracker.update_progress("peer3", 6, 1);

        // All peers are at least at (5, 5)
        assert!(tracker.all_at_least(5, 5));

        // peer1 is not at (5, 6)
        assert!(!tracker.all_at_least(5, 6));

        // All peers are at least at (4, 100) because they're all in turn >= 5
        assert!(tracker.all_at_least(4, 100));

        // peer1 and peer2 are not at turn 6
        assert!(!tracker.all_at_least(6, 1));
    }

    // ==================== SyncResult Tests ====================

    #[test]
    fn test_sync_result_clone() {
        let result = SyncResult {
            events_received: 10,
            events_applied: 5,
            state: SyncState::Synced,
            errors: vec!["error1".to_string()],
        };

        let cloned = result.clone();
        assert_eq!(cloned.events_received, result.events_received);
        assert_eq!(cloned.events_applied, result.events_applied);
        assert_eq!(cloned.state, result.state);
        assert_eq!(cloned.errors, result.errors);
    }

    #[test]
    fn test_sync_result_debug() {
        let result = SyncResult {
            events_received: 10,
            events_applied: 5,
            state: SyncState::Synced,
            errors: vec![],
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("events_received"));
        assert!(debug_str.contains("10"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_sync_flow() {
        // Simulate a complete sync flow between manager and responder
        let mut manager = SyncManager::new("game1".to_string(), 0);
        let responder = SyncResponder::new("game1".to_string());

        // Build an event chain on the host
        let mut chain = EventChain::new();
        chain
            .add(create_test_event_with_prev("evt1", None, 1, 1))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt2", Some("evt1"), 1, 2))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt3", Some("evt2"), 2, 1))
            .unwrap();

        // Client creates request
        let request = manager.create_request();
        assert_eq!(*manager.state(), SyncState::Requesting);

        // Host responds
        let response = responder.respond(&request, &chain);
        assert_eq!(response.events.len(), 3);

        // Client handles response
        let result = manager.handle_response(response);
        assert_eq!(result.events_received, 3);
        assert_eq!(*manager.state(), SyncState::Applying);

        // Client applies events
        while let Some(event) = manager.next_event() {
            manager.confirm_event(&event);
        }

        assert!(manager.is_synced());
    }

    #[test]
    fn test_incremental_sync_flow() {
        // Simulate incremental sync (client already has some events)
        let mut manager = SyncManager::new("game1".to_string(), 0);
        let responder = SyncResponder::new("game1".to_string());

        // Initial sync
        let mut chain = EventChain::new();
        chain
            .add(create_test_event_with_prev("evt1", None, 1, 1))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt2", Some("evt1"), 1, 2))
            .unwrap();

        let request = manager.create_request();
        let response = responder.respond(&request, &chain);
        manager.handle_response(response);

        while let Some(event) = manager.next_event() {
            manager.confirm_event(&event);
        }
        assert!(manager.is_synced());

        // Host adds more events
        chain
            .add(create_test_event_with_prev("evt3", Some("evt2"), 2, 1))
            .unwrap();
        chain
            .add(create_test_event_with_prev("evt4", Some("evt3"), 2, 2))
            .unwrap();

        // Client requests sync again
        let request2 = manager.create_request();
        assert_eq!(request2.last_event_id, Some("evt2".to_string()));

        let response2 = responder.respond(&request2, &chain);
        assert_eq!(response2.events.len(), 2);
        assert_eq!(response2.events[0].id, "evt3");
        assert_eq!(response2.events[1].id, "evt4");

        manager.handle_response(response2);
        while let Some(event) = manager.next_event() {
            manager.confirm_event(&event);
        }
        assert!(manager.is_synced());
    }

    #[test]
    fn test_conflict_detection_via_wrong_game() {
        // A simple conflict scenario: response for wrong game
        let mut manager = SyncManager::new("game1".to_string(), 0);
        manager.create_request();

        let response = SyncResponse {
            game_id: "game2".to_string(), // Wrong game!
            has_more: false,
            events: vec![create_test_event("evt1", 1, 1)],
            current_turn: 1,
            chain_hash: None,
        };

        let result = manager.handle_response(response);

        assert!(matches!(manager.state(), SyncState::Failed(_)));
        assert!(!result.errors.is_empty());
    }
}

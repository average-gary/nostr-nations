//! Tauri event system for backend-to-frontend communication.
//!
//! This module defines event payload types and helper functions to emit
//! events from the Rust backend to the JavaScript/TypeScript frontend.
//!
//! # Event Types
//!
//! - `game_state_updated` - Full or partial game state updates
//! - `turn_event` - Turn lifecycle events (started, ended, player_turn)
//! - `combat_resolved` - Combat results with attacker, defender, and outcomes
//! - `network_event` - P2P networking events (peer connect/disconnect, sync)
//! - `notification` - User-facing notifications

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

// =============================================================================
// Event Names (constants for consistency)
// =============================================================================

/// Event name for game state updates.
pub const EVENT_GAME_STATE_UPDATED: &str = "game_state_updated";

/// Event name for turn events.
pub const EVENT_TURN: &str = "turn_event";

/// Event name for combat resolution.
pub const EVENT_COMBAT_RESOLVED: &str = "combat_resolved";

/// Event name for network events.
pub const EVENT_NETWORK: &str = "network_event";

/// Event name for notifications.
pub const EVENT_NOTIFICATION: &str = "notification";

// =============================================================================
// Game State Event
// =============================================================================

/// Payload for game state update events.
///
/// Can contain either a full game state snapshot or a partial update
/// with only the changed fields.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameStateUpdatedPayload {
    /// The game ID this update applies to.
    pub game_id: String,
    /// Current game phase (Setup, Playing, Ended).
    pub phase: String,
    /// Current turn number.
    pub turn: u32,
    /// ID of the current player.
    pub current_player: u8,
    /// Total number of players.
    pub player_count: usize,
    /// Map dimensions.
    pub map_dimensions: (u32, u32),
    /// Whether this is a full state update or partial.
    pub is_full_update: bool,
    /// Optional: Changed units (for partial updates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_units: Option<Vec<UnitUpdate>>,
    /// Optional: Changed cities (for partial updates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_cities: Option<Vec<CityUpdate>>,
    /// Optional: Changed tiles (for partial updates).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changed_tiles: Option<Vec<TileUpdate>>,
}

/// Minimal unit update for partial state updates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UnitUpdate {
    pub id: u64,
    pub owner: u8,
    pub unit_type: String,
    pub position: (i32, i32),
    pub health: u32,
    pub movement_remaining: u32,
    pub is_destroyed: bool,
}

/// Minimal city update for partial state updates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityUpdate {
    pub id: u64,
    pub owner: u8,
    pub name: String,
    pub position: (i32, i32),
    pub population: u32,
    pub health: u32,
}

/// Minimal tile update for partial state updates.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TileUpdate {
    pub position: (i32, i32),
    pub improvement: Option<String>,
    pub road: Option<String>,
    pub owner: Option<u8>,
}

// =============================================================================
// Turn Event
// =============================================================================

/// Types of turn events.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TurnEventType {
    /// A new turn has started (all players).
    TurnStarted,
    /// The current player's turn has ended.
    TurnEnded,
    /// It is now a specific player's turn.
    PlayerTurn,
}

/// Payload for turn-related events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TurnEventPayload {
    /// Type of turn event.
    pub event_type: TurnEventType,
    /// Current turn number.
    pub turn: u32,
    /// Player whose turn it is (or was).
    pub player_id: u8,
    /// Player name for display.
    pub player_name: String,
    /// Previous turn number (for turn_started).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_turn: Option<u32>,
    /// Whether this is the local player's turn.
    pub is_local_player: bool,
}

// =============================================================================
// Combat Event
// =============================================================================

/// Payload for combat resolution events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatResolvedPayload {
    /// Attacker information.
    pub attacker: CombatantInfo,
    /// Defender information.
    pub defender: CombatantInfo,
    /// Combat results.
    pub results: CombatResults,
    /// Position where combat occurred.
    pub position: (i32, i32),
    /// Timestamp of the combat.
    pub timestamp: u64,
}

/// Information about a unit involved in combat.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatantInfo {
    /// Unit ID.
    pub unit_id: u64,
    /// Owner player ID.
    pub owner_id: u8,
    /// Owner player name.
    pub owner_name: String,
    /// Unit type name.
    pub unit_type: String,
    /// Health before combat.
    pub health_before: u32,
    /// Health after combat.
    pub health_after: u32,
    /// Combat strength used.
    pub strength: u32,
}

/// Results of a combat engagement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatResults {
    /// Damage dealt to defender.
    pub defender_damage: u32,
    /// Damage dealt to attacker.
    pub attacker_damage: u32,
    /// Whether the defender was destroyed.
    pub defender_destroyed: bool,
    /// Whether the attacker was destroyed.
    pub attacker_destroyed: bool,
    /// Experience gained by attacker.
    pub attacker_xp: u32,
    /// Experience gained by defender.
    pub defender_xp: u32,
    /// Was this a ranged attack?
    pub was_ranged: bool,
}

// =============================================================================
// Network Event
// =============================================================================

/// Types of network events.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NetworkEventType {
    /// A new peer has connected.
    PeerConnected,
    /// A peer has disconnected.
    PeerDisconnected,
    /// Synchronization with peers is complete.
    SyncComplete,
    /// Synchronization started.
    SyncStarted,
    /// Connection error occurred.
    ConnectionError,
}

/// Payload for network-related events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkEventPayload {
    /// Type of network event.
    pub event_type: NetworkEventType,
    /// Peer ID (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_id: Option<String>,
    /// Peer display name (if known).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub peer_name: Option<String>,
    /// Current total peer count.
    pub peer_count: usize,
    /// Error message (if event_type is ConnectionError).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    /// Sync progress (0-100, if syncing).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_progress: Option<u8>,
}

// =============================================================================
// Notification Event
// =============================================================================

/// Types of notifications.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    /// Informational message.
    Info,
    /// Success message.
    Success,
    /// Warning message.
    Warning,
    /// Error message.
    Error,
    /// Achievement unlocked.
    Achievement,
    /// Diplomatic event (war declared, peace treaty, etc.).
    Diplomacy,
    /// Research completed.
    Research,
    /// Production completed.
    Production,
    /// Combat notification.
    Combat,
}

/// Payload for notification events.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationPayload {
    /// Type of notification.
    pub notification_type: NotificationType,
    /// Notification title.
    pub title: String,
    /// Notification message body.
    pub message: String,
    /// Optional icon name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Duration in milliseconds (None = user dismisses).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    /// Optional action to perform on click.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<NotificationAction>,
}

/// Action that can be triggered from a notification.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Action type.
    pub action_type: String,
    /// Action label for button.
    pub label: String,
    /// Optional data for the action.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

// =============================================================================
// Event Emission Helper Functions
// =============================================================================

/// Emit a game state updated event.
///
/// # Arguments
///
/// * `app_handle` - The Tauri app handle.
/// * `payload` - The game state update payload.
///
/// # Returns
///
/// Result indicating success or failure.
pub fn emit_game_state_updated(
    app_handle: &AppHandle,
    payload: GameStateUpdatedPayload,
) -> Result<(), tauri::Error> {
    app_handle.emit(EVENT_GAME_STATE_UPDATED, payload)
}

/// Emit a turn event.
///
/// # Arguments
///
/// * `app_handle` - The Tauri app handle.
/// * `event_type` - The type of turn event.
/// * `payload` - The turn event payload.
///
/// # Returns
///
/// Result indicating success or failure.
pub fn emit_turn_event(
    app_handle: &AppHandle,
    payload: TurnEventPayload,
) -> Result<(), tauri::Error> {
    app_handle.emit(EVENT_TURN, payload)
}

/// Emit a combat resolved event.
///
/// # Arguments
///
/// * `app_handle` - The Tauri app handle.
/// * `payload` - The combat resolution payload.
///
/// # Returns
///
/// Result indicating success or failure.
pub fn emit_combat_resolved(
    app_handle: &AppHandle,
    payload: CombatResolvedPayload,
) -> Result<(), tauri::Error> {
    app_handle.emit(EVENT_COMBAT_RESOLVED, payload)
}

/// Emit a network event.
///
/// # Arguments
///
/// * `app_handle` - The Tauri app handle.
/// * `payload` - The network event payload.
///
/// # Returns
///
/// Result indicating success or failure.
pub fn emit_network_event(
    app_handle: &AppHandle,
    payload: NetworkEventPayload,
) -> Result<(), tauri::Error> {
    app_handle.emit(EVENT_NETWORK, payload)
}

/// Emit a notification event.
///
/// # Arguments
///
/// * `app_handle` - The Tauri app handle.
/// * `payload` - The notification payload.
///
/// # Returns
///
/// Result indicating success or failure.
pub fn emit_notification(
    app_handle: &AppHandle,
    payload: NotificationPayload,
) -> Result<(), tauri::Error> {
    app_handle.emit(EVENT_NOTIFICATION, payload)
}

// =============================================================================
// Convenience Builders
// =============================================================================

impl NotificationPayload {
    /// Create a simple info notification.
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            notification_type: NotificationType::Info,
            title: title.into(),
            message: message.into(),
            icon: None,
            duration_ms: Some(5000),
            action: None,
        }
    }

    /// Create a success notification.
    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            notification_type: NotificationType::Success,
            title: title.into(),
            message: message.into(),
            icon: None,
            duration_ms: Some(5000),
            action: None,
        }
    }

    /// Create a warning notification.
    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            notification_type: NotificationType::Warning,
            title: title.into(),
            message: message.into(),
            icon: None,
            duration_ms: Some(7000),
            action: None,
        }
    }

    /// Create an error notification.
    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            notification_type: NotificationType::Error,
            title: title.into(),
            message: message.into(),
            icon: None,
            duration_ms: None, // User must dismiss errors
            action: None,
        }
    }

    /// Set the notification duration.
    pub fn with_duration(mut self, duration_ms: u64) -> Self {
        self.duration_ms = Some(duration_ms);
        self
    }

    /// Set the notification icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

impl NetworkEventPayload {
    /// Create a peer connected event.
    pub fn peer_connected(peer_id: String, peer_name: Option<String>, peer_count: usize) -> Self {
        Self {
            event_type: NetworkEventType::PeerConnected,
            peer_id: Some(peer_id),
            peer_name,
            peer_count,
            error_message: None,
            sync_progress: None,
        }
    }

    /// Create a peer disconnected event.
    pub fn peer_disconnected(
        peer_id: String,
        peer_name: Option<String>,
        peer_count: usize,
    ) -> Self {
        Self {
            event_type: NetworkEventType::PeerDisconnected,
            peer_id: Some(peer_id),
            peer_name,
            peer_count,
            error_message: None,
            sync_progress: None,
        }
    }

    /// Create a sync complete event.
    pub fn sync_complete(peer_count: usize) -> Self {
        Self {
            event_type: NetworkEventType::SyncComplete,
            peer_id: None,
            peer_name: None,
            peer_count,
            error_message: None,
            sync_progress: Some(100),
        }
    }

    /// Create a connection error event.
    pub fn connection_error(error: impl Into<String>, peer_count: usize) -> Self {
        Self {
            event_type: NetworkEventType::ConnectionError,
            peer_id: None,
            peer_name: None,
            peer_count,
            error_message: Some(error.into()),
            sync_progress: None,
        }
    }
}

impl TurnEventPayload {
    /// Create a turn started event.
    pub fn turn_started(
        turn: u32,
        player_id: u8,
        player_name: String,
        is_local_player: bool,
    ) -> Self {
        Self {
            event_type: TurnEventType::TurnStarted,
            turn,
            player_id,
            player_name,
            previous_turn: Some(turn.saturating_sub(1)),
            is_local_player,
        }
    }

    /// Create a turn ended event.
    pub fn turn_ended(
        turn: u32,
        player_id: u8,
        player_name: String,
        is_local_player: bool,
    ) -> Self {
        Self {
            event_type: TurnEventType::TurnEnded,
            turn,
            player_id,
            player_name,
            previous_turn: None,
            is_local_player,
        }
    }

    /// Create a player turn event.
    pub fn player_turn(
        turn: u32,
        player_id: u8,
        player_name: String,
        is_local_player: bool,
    ) -> Self {
        Self {
            event_type: TurnEventType::PlayerTurn,
            turn,
            player_id,
            player_name,
            previous_turn: None,
            is_local_player,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notification_builders() {
        let notif = NotificationPayload::info("Test", "Test message");
        assert_eq!(notif.notification_type, NotificationType::Info);
        assert_eq!(notif.title, "Test");
        assert_eq!(notif.message, "Test message");

        let error = NotificationPayload::error("Error", "Something went wrong");
        assert_eq!(error.notification_type, NotificationType::Error);
        assert!(error.duration_ms.is_none()); // Errors don't auto-dismiss
    }

    #[test]
    fn test_network_event_builders() {
        let connected = NetworkEventPayload::peer_connected(
            "peer123".to_string(),
            Some("Alice".to_string()),
            3,
        );
        assert_eq!(connected.event_type, NetworkEventType::PeerConnected);
        assert_eq!(connected.peer_count, 3);

        let error = NetworkEventPayload::connection_error("Connection refused", 0);
        assert_eq!(error.event_type, NetworkEventType::ConnectionError);
        assert!(error.error_message.is_some());
    }

    #[test]
    fn test_turn_event_builders() {
        let started = TurnEventPayload::turn_started(5, 0, "Player1".to_string(), true);
        assert_eq!(started.event_type, TurnEventType::TurnStarted);
        assert_eq!(started.turn, 5);
        assert_eq!(started.previous_turn, Some(4));

        let ended = TurnEventPayload::turn_ended(5, 0, "Player1".to_string(), true);
        assert_eq!(ended.event_type, TurnEventType::TurnEnded);
    }

    #[test]
    fn test_payload_serialization() {
        let payload = GameStateUpdatedPayload {
            game_id: "game123".to_string(),
            phase: "Playing".to_string(),
            turn: 10,
            current_player: 1,
            player_count: 4,
            map_dimensions: (80, 50),
            is_full_update: false,
            changed_units: Some(vec![UnitUpdate {
                id: 1,
                owner: 0,
                unit_type: "Warrior".to_string(),
                position: (5, 3),
                health: 80,
                movement_remaining: 1,
                is_destroyed: false,
            }]),
            changed_cities: None,
            changed_tiles: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("game123"));
        assert!(json.contains("Warrior"));
        // changed_cities should not appear since it's None
        assert!(!json.contains("changed_cities"));
    }

    #[test]
    fn test_combat_payload_serialization() {
        let payload = CombatResolvedPayload {
            attacker: CombatantInfo {
                unit_id: 1,
                owner_id: 0,
                owner_name: "Player1".to_string(),
                unit_type: "Swordsman".to_string(),
                health_before: 100,
                health_after: 75,
                strength: 14,
            },
            defender: CombatantInfo {
                unit_id: 2,
                owner_id: 1,
                owner_name: "Player2".to_string(),
                unit_type: "Warrior".to_string(),
                health_before: 100,
                health_after: 0,
                strength: 8,
            },
            results: CombatResults {
                defender_damage: 100,
                attacker_damage: 25,
                defender_destroyed: true,
                attacker_destroyed: false,
                attacker_xp: 8,
                defender_xp: 0,
                was_ranged: false,
            },
            position: (10, 5),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Swordsman"));
        assert!(json.contains("defender_destroyed"));
    }

    // =========================================================================
    // Notification Builder Tests
    // =========================================================================

    #[test]
    fn test_notification_success_builder() {
        let notif = NotificationPayload::success("Victory!", "You won the battle");
        assert_eq!(notif.notification_type, NotificationType::Success);
        assert_eq!(notif.title, "Victory!");
        assert_eq!(notif.message, "You won the battle");
        assert_eq!(notif.duration_ms, Some(5000));
        assert!(notif.icon.is_none());
        assert!(notif.action.is_none());
    }

    #[test]
    fn test_notification_warning_builder() {
        let notif = NotificationPayload::warning("Caution", "Enemy approaching");
        assert_eq!(notif.notification_type, NotificationType::Warning);
        assert_eq!(notif.title, "Caution");
        assert_eq!(notif.message, "Enemy approaching");
        assert_eq!(notif.duration_ms, Some(7000)); // Warning has longer duration
        assert!(notif.icon.is_none());
    }

    #[test]
    fn test_notification_error_builder() {
        let notif = NotificationPayload::error("Connection Lost", "Failed to sync");
        assert_eq!(notif.notification_type, NotificationType::Error);
        assert_eq!(notif.title, "Connection Lost");
        assert_eq!(notif.message, "Failed to sync");
        assert!(notif.duration_ms.is_none()); // Errors require manual dismiss
    }

    #[test]
    fn test_notification_info_builder() {
        let notif = NotificationPayload::info("Hint", "Press E to end turn");
        assert_eq!(notif.notification_type, NotificationType::Info);
        assert_eq!(notif.title, "Hint");
        assert_eq!(notif.message, "Press E to end turn");
        assert_eq!(notif.duration_ms, Some(5000));
    }

    #[test]
    fn test_notification_with_duration() {
        let notif = NotificationPayload::info("Quick", "Flash message").with_duration(1000);
        assert_eq!(notif.duration_ms, Some(1000));

        // Can override error's None duration
        let error = NotificationPayload::error("Error", "Message").with_duration(10000);
        assert_eq!(error.duration_ms, Some(10000));
    }

    #[test]
    fn test_notification_with_icon() {
        let notif = NotificationPayload::success("Done", "Task complete").with_icon("checkmark");
        assert_eq!(notif.icon, Some("checkmark".to_string()));
    }

    #[test]
    fn test_notification_builder_chaining() {
        let notif = NotificationPayload::warning("Alert", "Low health")
            .with_duration(3000)
            .with_icon("heart");

        assert_eq!(notif.notification_type, NotificationType::Warning);
        assert_eq!(notif.duration_ms, Some(3000));
        assert_eq!(notif.icon, Some("heart".to_string()));
    }

    // =========================================================================
    // Turn Event Tests
    // =========================================================================

    #[test]
    fn test_turn_started_event() {
        let event = TurnEventPayload::turn_started(1, 0, "Alice".to_string(), true);
        assert_eq!(event.event_type, TurnEventType::TurnStarted);
        assert_eq!(event.turn, 1);
        assert_eq!(event.player_id, 0);
        assert_eq!(event.player_name, "Alice");
        assert!(event.is_local_player);
        assert_eq!(event.previous_turn, Some(0)); // saturating_sub(1) from turn 1
    }

    #[test]
    fn test_turn_started_at_turn_zero() {
        let event = TurnEventPayload::turn_started(0, 0, "Alice".to_string(), true);
        assert_eq!(event.previous_turn, Some(0)); // saturating_sub prevents underflow
    }

    #[test]
    fn test_turn_ended_event() {
        let event = TurnEventPayload::turn_ended(10, 2, "Bob".to_string(), false);
        assert_eq!(event.event_type, TurnEventType::TurnEnded);
        assert_eq!(event.turn, 10);
        assert_eq!(event.player_id, 2);
        assert_eq!(event.player_name, "Bob");
        assert!(!event.is_local_player);
        assert!(event.previous_turn.is_none());
    }

    #[test]
    fn test_player_turn_event() {
        let event = TurnEventPayload::player_turn(5, 1, "Charlie".to_string(), true);
        assert_eq!(event.event_type, TurnEventType::PlayerTurn);
        assert_eq!(event.turn, 5);
        assert_eq!(event.player_id, 1);
        assert_eq!(event.player_name, "Charlie");
        assert!(event.is_local_player);
        assert!(event.previous_turn.is_none());
    }

    #[test]
    fn test_turn_event_type_serialization() {
        let started = TurnEventType::TurnStarted;
        let ended = TurnEventType::TurnEnded;
        let player = TurnEventType::PlayerTurn;

        assert_eq!(serde_json::to_string(&started).unwrap(), "\"turn_started\"");
        assert_eq!(serde_json::to_string(&ended).unwrap(), "\"turn_ended\"");
        assert_eq!(serde_json::to_string(&player).unwrap(), "\"player_turn\"");
    }

    // =========================================================================
    // Network Event Tests
    // =========================================================================

    #[test]
    fn test_network_peer_connected() {
        let event = NetworkEventPayload::peer_connected(
            "peer_abc".to_string(),
            Some("Dave".to_string()),
            5,
        );
        assert_eq!(event.event_type, NetworkEventType::PeerConnected);
        assert_eq!(event.peer_id, Some("peer_abc".to_string()));
        assert_eq!(event.peer_name, Some("Dave".to_string()));
        assert_eq!(event.peer_count, 5);
        assert!(event.error_message.is_none());
        assert!(event.sync_progress.is_none());
    }

    #[test]
    fn test_network_peer_connected_without_name() {
        let event = NetworkEventPayload::peer_connected("peer_xyz".to_string(), None, 1);
        assert_eq!(event.peer_id, Some("peer_xyz".to_string()));
        assert!(event.peer_name.is_none());
    }

    #[test]
    fn test_network_peer_disconnected() {
        let event = NetworkEventPayload::peer_disconnected(
            "peer_123".to_string(),
            Some("Eve".to_string()),
            2,
        );
        assert_eq!(event.event_type, NetworkEventType::PeerDisconnected);
        assert_eq!(event.peer_id, Some("peer_123".to_string()));
        assert_eq!(event.peer_name, Some("Eve".to_string()));
        assert_eq!(event.peer_count, 2);
    }

    #[test]
    fn test_network_sync_complete() {
        let event = NetworkEventPayload::sync_complete(4);
        assert_eq!(event.event_type, NetworkEventType::SyncComplete);
        assert!(event.peer_id.is_none());
        assert!(event.peer_name.is_none());
        assert_eq!(event.peer_count, 4);
        assert_eq!(event.sync_progress, Some(100));
        assert!(event.error_message.is_none());
    }

    #[test]
    fn test_network_sync_started() {
        // Manual construction since there's no builder for sync_started
        let event = NetworkEventPayload {
            event_type: NetworkEventType::SyncStarted,
            peer_id: None,
            peer_name: None,
            peer_count: 3,
            error_message: None,
            sync_progress: Some(0),
        };
        assert_eq!(event.event_type, NetworkEventType::SyncStarted);
        assert_eq!(event.sync_progress, Some(0));
    }

    #[test]
    fn test_network_sync_progress_variations() {
        // Test various progress values
        for progress in [0u8, 25, 50, 75, 99, 100] {
            let event = NetworkEventPayload {
                event_type: NetworkEventType::SyncStarted,
                peer_id: None,
                peer_name: None,
                peer_count: 2,
                error_message: None,
                sync_progress: Some(progress),
            };
            assert_eq!(event.sync_progress, Some(progress));
        }
    }

    #[test]
    fn test_network_connection_error() {
        let event = NetworkEventPayload::connection_error("Timeout after 30s", 0);
        assert_eq!(event.event_type, NetworkEventType::ConnectionError);
        assert_eq!(event.error_message, Some("Timeout after 30s".to_string()));
        assert_eq!(event.peer_count, 0);
        assert!(event.peer_id.is_none());
        assert!(event.sync_progress.is_none());
    }

    #[test]
    fn test_network_event_type_serialization() {
        assert_eq!(
            serde_json::to_string(&NetworkEventType::PeerConnected).unwrap(),
            "\"peer_connected\""
        );
        assert_eq!(
            serde_json::to_string(&NetworkEventType::PeerDisconnected).unwrap(),
            "\"peer_disconnected\""
        );
        assert_eq!(
            serde_json::to_string(&NetworkEventType::SyncComplete).unwrap(),
            "\"sync_complete\""
        );
        assert_eq!(
            serde_json::to_string(&NetworkEventType::SyncStarted).unwrap(),
            "\"sync_started\""
        );
        assert_eq!(
            serde_json::to_string(&NetworkEventType::ConnectionError).unwrap(),
            "\"connection_error\""
        );
    }

    // =========================================================================
    // GameStateUpdatedPayload Partial Update Tests
    // =========================================================================

    #[test]
    fn test_game_state_partial_update_with_units() {
        let payload = GameStateUpdatedPayload {
            game_id: "game_001".to_string(),
            phase: "Playing".to_string(),
            turn: 5,
            current_player: 0,
            player_count: 2,
            map_dimensions: (100, 100),
            is_full_update: false,
            changed_units: Some(vec![
                UnitUpdate {
                    id: 1,
                    owner: 0,
                    unit_type: "Warrior".to_string(),
                    position: (10, 20),
                    health: 100,
                    movement_remaining: 2,
                    is_destroyed: false,
                },
                UnitUpdate {
                    id: 2,
                    owner: 1,
                    unit_type: "Archer".to_string(),
                    position: (15, 25),
                    health: 50,
                    movement_remaining: 0,
                    is_destroyed: false,
                },
            ]),
            changed_cities: None,
            changed_tiles: None,
        };

        assert!(!payload.is_full_update);
        assert!(payload.changed_units.is_some());
        assert_eq!(payload.changed_units.as_ref().unwrap().len(), 2);
        assert!(payload.changed_cities.is_none());
        assert!(payload.changed_tiles.is_none());
    }

    #[test]
    fn test_game_state_partial_update_with_cities() {
        let payload = GameStateUpdatedPayload {
            game_id: "game_002".to_string(),
            phase: "Playing".to_string(),
            turn: 10,
            current_player: 1,
            player_count: 4,
            map_dimensions: (80, 60),
            is_full_update: false,
            changed_units: None,
            changed_cities: Some(vec![CityUpdate {
                id: 100,
                owner: 0,
                name: "New York".to_string(),
                position: (50, 30),
                population: 5,
                health: 200,
            }]),
            changed_tiles: None,
        };

        assert!(payload.changed_cities.is_some());
        let city = &payload.changed_cities.as_ref().unwrap()[0];
        assert_eq!(city.name, "New York");
        assert_eq!(city.population, 5);
    }

    #[test]
    fn test_game_state_partial_update_with_tiles() {
        let payload = GameStateUpdatedPayload {
            game_id: "game_003".to_string(),
            phase: "Playing".to_string(),
            turn: 15,
            current_player: 2,
            player_count: 3,
            map_dimensions: (120, 80),
            is_full_update: false,
            changed_units: None,
            changed_cities: None,
            changed_tiles: Some(vec![
                TileUpdate {
                    position: (5, 5),
                    improvement: Some("Farm".to_string()),
                    road: None,
                    owner: Some(0),
                },
                TileUpdate {
                    position: (6, 5),
                    improvement: None,
                    road: Some("Road".to_string()),
                    owner: Some(0),
                },
            ]),
        };

        assert!(payload.changed_tiles.is_some());
        let tiles = payload.changed_tiles.as_ref().unwrap();
        assert_eq!(tiles.len(), 2);
        assert_eq!(tiles[0].improvement, Some("Farm".to_string()));
        assert_eq!(tiles[1].road, Some("Road".to_string()));
    }

    #[test]
    fn test_game_state_full_update() {
        let payload = GameStateUpdatedPayload {
            game_id: "game_full".to_string(),
            phase: "Setup".to_string(),
            turn: 0,
            current_player: 0,
            player_count: 2,
            map_dimensions: (50, 50),
            is_full_update: true,
            changed_units: None,
            changed_cities: None,
            changed_tiles: None,
        };

        assert!(payload.is_full_update);
    }

    // =========================================================================
    // Serialization Round-Trip Tests
    // =========================================================================

    #[test]
    fn test_notification_payload_roundtrip() {
        let original = NotificationPayload::warning("Test", "Message")
            .with_duration(2500)
            .with_icon("alert");

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: NotificationPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.notification_type, original.notification_type);
        assert_eq!(deserialized.title, original.title);
        assert_eq!(deserialized.message, original.message);
        assert_eq!(deserialized.duration_ms, original.duration_ms);
        assert_eq!(deserialized.icon, original.icon);
    }

    #[test]
    fn test_turn_event_payload_roundtrip() {
        let original = TurnEventPayload::turn_started(42, 3, "TestPlayer".to_string(), false);

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: TurnEventPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.event_type, original.event_type);
        assert_eq!(deserialized.turn, original.turn);
        assert_eq!(deserialized.player_id, original.player_id);
        assert_eq!(deserialized.player_name, original.player_name);
        assert_eq!(deserialized.previous_turn, original.previous_turn);
        assert_eq!(deserialized.is_local_player, original.is_local_player);
    }

    #[test]
    fn test_network_event_payload_roundtrip() {
        let original = NetworkEventPayload::peer_connected(
            "peer_roundtrip".to_string(),
            Some("RoundtripUser".to_string()),
            7,
        );

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: NetworkEventPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.event_type, original.event_type);
        assert_eq!(deserialized.peer_id, original.peer_id);
        assert_eq!(deserialized.peer_name, original.peer_name);
        assert_eq!(deserialized.peer_count, original.peer_count);
    }

    #[test]
    fn test_game_state_payload_roundtrip() {
        let original = GameStateUpdatedPayload {
            game_id: "roundtrip_game".to_string(),
            phase: "Playing".to_string(),
            turn: 25,
            current_player: 1,
            player_count: 4,
            map_dimensions: (100, 80),
            is_full_update: false,
            changed_units: Some(vec![UnitUpdate {
                id: 999,
                owner: 2,
                unit_type: "Knight".to_string(),
                position: (33, 44),
                health: 75,
                movement_remaining: 3,
                is_destroyed: false,
            }]),
            changed_cities: None,
            changed_tiles: Some(vec![TileUpdate {
                position: (10, 10),
                improvement: Some("Mine".to_string()),
                road: Some("Railroad".to_string()),
                owner: Some(2),
            }]),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: GameStateUpdatedPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.game_id, original.game_id);
        assert_eq!(deserialized.turn, original.turn);
        assert_eq!(deserialized.changed_units.as_ref().unwrap()[0].id, 999);
        assert_eq!(
            deserialized.changed_tiles.as_ref().unwrap()[0].improvement,
            Some("Mine".to_string())
        );
    }

    #[test]
    fn test_combat_resolved_payload_roundtrip() {
        let original = CombatResolvedPayload {
            attacker: CombatantInfo {
                unit_id: 10,
                owner_id: 0,
                owner_name: "Attacker".to_string(),
                unit_type: "Tank".to_string(),
                health_before: 100,
                health_after: 80,
                strength: 50,
            },
            defender: CombatantInfo {
                unit_id: 20,
                owner_id: 1,
                owner_name: "Defender".to_string(),
                unit_type: "Infantry".to_string(),
                health_before: 100,
                health_after: 0,
                strength: 25,
            },
            results: CombatResults {
                defender_damage: 100,
                attacker_damage: 20,
                defender_destroyed: true,
                attacker_destroyed: false,
                attacker_xp: 15,
                defender_xp: 5,
                was_ranged: false,
            },
            position: (50, 60),
            timestamp: 9999999999,
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CombatResolvedPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.attacker.unit_id, original.attacker.unit_id);
        assert_eq!(deserialized.defender.health_after, 0);
        assert!(deserialized.results.defender_destroyed);
        assert_eq!(deserialized.position, (50, 60));
    }

    // =========================================================================
    // Edge Case Tests
    // =========================================================================

    #[test]
    fn test_notification_empty_strings() {
        let notif = NotificationPayload::info("", "");
        assert_eq!(notif.title, "");
        assert_eq!(notif.message, "");

        let json = serde_json::to_string(&notif).unwrap();
        let deserialized: NotificationPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, "");
        assert_eq!(deserialized.message, "");
    }

    #[test]
    fn test_notification_with_zero_duration() {
        let notif = NotificationPayload::info("Test", "Test").with_duration(0);
        assert_eq!(notif.duration_ms, Some(0));
    }

    #[test]
    fn test_notification_with_empty_icon() {
        let notif = NotificationPayload::info("Test", "Test").with_icon("");
        assert_eq!(notif.icon, Some("".to_string()));
    }

    #[test]
    fn test_turn_event_max_values() {
        let event =
            TurnEventPayload::turn_started(u32::MAX, u8::MAX, "MaxPlayer".to_string(), true);
        assert_eq!(event.turn, u32::MAX);
        assert_eq!(event.player_id, u8::MAX);
        assert_eq!(event.previous_turn, Some(u32::MAX - 1));
    }

    #[test]
    fn test_network_event_zero_peer_count() {
        let event = NetworkEventPayload::sync_complete(0);
        assert_eq!(event.peer_count, 0);
    }

    #[test]
    fn test_network_event_empty_error_message() {
        let event = NetworkEventPayload::connection_error("", 0);
        assert_eq!(event.error_message, Some("".to_string()));
    }

    #[test]
    fn test_unit_update_destroyed_state() {
        let unit = UnitUpdate {
            id: 1,
            owner: 0,
            unit_type: "Warrior".to_string(),
            position: (0, 0),
            health: 0,
            movement_remaining: 0,
            is_destroyed: true,
        };
        assert!(unit.is_destroyed);
        assert_eq!(unit.health, 0);
    }

    #[test]
    fn test_tile_update_all_none_optionals() {
        let tile = TileUpdate {
            position: (0, 0),
            improvement: None,
            road: None,
            owner: None,
        };

        let json = serde_json::to_string(&tile).unwrap();
        // None values should still serialize (not skipped in TileUpdate)
        let deserialized: TileUpdate = serde_json::from_str(&json).unwrap();
        assert!(deserialized.improvement.is_none());
        assert!(deserialized.road.is_none());
        assert!(deserialized.owner.is_none());
    }

    #[test]
    fn test_game_state_zero_dimensions() {
        let payload = GameStateUpdatedPayload {
            game_id: "zero_dim".to_string(),
            phase: "Setup".to_string(),
            turn: 0,
            current_player: 0,
            player_count: 0,
            map_dimensions: (0, 0),
            is_full_update: true,
            changed_units: None,
            changed_cities: None,
            changed_tiles: None,
        };

        assert_eq!(payload.map_dimensions, (0, 0));
        assert_eq!(payload.player_count, 0);
    }

    #[test]
    fn test_city_update_zero_population() {
        let city = CityUpdate {
            id: 1,
            owner: 0,
            name: "Ghost Town".to_string(),
            position: (25, 25),
            population: 0,
            health: 0,
        };
        assert_eq!(city.population, 0);
        assert_eq!(city.health, 0);
    }

    #[test]
    fn test_notification_type_serialization() {
        assert_eq!(
            serde_json::to_string(&NotificationType::Info).unwrap(),
            "\"info\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Success).unwrap(),
            "\"success\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Warning).unwrap(),
            "\"warning\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Error).unwrap(),
            "\"error\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Achievement).unwrap(),
            "\"achievement\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Diplomacy).unwrap(),
            "\"diplomacy\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Research).unwrap(),
            "\"research\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Production).unwrap(),
            "\"production\""
        );
        assert_eq!(
            serde_json::to_string(&NotificationType::Combat).unwrap(),
            "\"combat\""
        );
    }

    #[test]
    fn test_notification_action_serialization() {
        let action = NotificationAction {
            action_type: "navigate".to_string(),
            label: "Go to unit".to_string(),
            data: Some(serde_json::json!({"unit_id": 42})),
        };

        let json = serde_json::to_string(&action).unwrap();
        let deserialized: NotificationAction = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.action_type, "navigate");
        assert_eq!(deserialized.label, "Go to unit");
        assert!(deserialized.data.is_some());
    }

    #[test]
    fn test_notification_with_action() {
        let payload = NotificationPayload {
            notification_type: NotificationType::Combat,
            title: "Battle Won".to_string(),
            message: "Your warrior defeated an enemy".to_string(),
            icon: Some("sword".to_string()),
            duration_ms: Some(5000),
            action: Some(NotificationAction {
                action_type: "focus".to_string(),
                label: "View unit".to_string(),
                data: None,
            }),
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("action"));
        assert!(json.contains("focus"));
    }

    #[test]
    fn test_skip_serializing_none_fields() {
        let payload = GameStateUpdatedPayload {
            game_id: "test".to_string(),
            phase: "Playing".to_string(),
            turn: 1,
            current_player: 0,
            player_count: 2,
            map_dimensions: (10, 10),
            is_full_update: true,
            changed_units: None,
            changed_cities: None,
            changed_tiles: None,
        };

        let json = serde_json::to_string(&payload).unwrap();
        // None fields with skip_serializing_if should not appear
        assert!(!json.contains("changed_units"));
        assert!(!json.contains("changed_cities"));
        assert!(!json.contains("changed_tiles"));
    }

    #[test]
    fn test_network_event_skip_serializing_none() {
        let event = NetworkEventPayload::sync_complete(5);
        let json = serde_json::to_string(&event).unwrap();

        // These None fields should not appear in JSON
        assert!(!json.contains("peer_id"));
        assert!(!json.contains("peer_name"));
        assert!(!json.contains("error_message"));
    }

    #[test]
    fn test_combatant_info_zero_strength() {
        let combatant = CombatantInfo {
            unit_id: 1,
            owner_id: 0,
            owner_name: "Test".to_string(),
            unit_type: "Scout".to_string(),
            health_before: 100,
            health_after: 100,
            strength: 0, // Non-combat unit
        };
        assert_eq!(combatant.strength, 0);
    }

    #[test]
    fn test_combat_results_ranged_attack() {
        let results = CombatResults {
            defender_damage: 50,
            attacker_damage: 0, // Ranged doesn't take damage
            defender_destroyed: false,
            attacker_destroyed: false,
            attacker_xp: 5,
            defender_xp: 2,
            was_ranged: true,
        };
        assert!(results.was_ranged);
        assert_eq!(results.attacker_damage, 0);
    }

    #[test]
    fn test_negative_positions() {
        let unit = UnitUpdate {
            id: 1,
            owner: 0,
            unit_type: "Warrior".to_string(),
            position: (-10, -20),
            health: 100,
            movement_remaining: 2,
            is_destroyed: false,
        };
        assert_eq!(unit.position, (-10, -20));

        let tile = TileUpdate {
            position: (-5, -5),
            improvement: None,
            road: None,
            owner: None,
        };
        assert_eq!(tile.position, (-5, -5));
    }

    // =========================================================================
    // Event Integration Tests - Creating events from game state changes
    // =========================================================================

    #[test]
    fn test_create_event_from_unit_movement() {
        // Simulate a unit moving from one position to another
        let unit_before_move = UnitUpdate {
            id: 42,
            owner: 0,
            unit_type: "Warrior".to_string(),
            position: (10, 10),
            health: 100,
            movement_remaining: 2,
            is_destroyed: false,
        };

        // After movement
        let unit_after_move = UnitUpdate {
            id: 42,
            owner: 0,
            unit_type: "Warrior".to_string(),
            position: (11, 10),
            health: 100,
            movement_remaining: 1,
            is_destroyed: false,
        };

        let payload = GameStateUpdatedPayload {
            game_id: "movement_test".to_string(),
            phase: "Playing".to_string(),
            turn: 5,
            current_player: 0,
            player_count: 2,
            map_dimensions: (50, 50),
            is_full_update: false,
            changed_units: Some(vec![unit_after_move.clone()]),
            changed_cities: None,
            changed_tiles: None,
        };

        assert!(payload.changed_units.is_some());
        let units = payload.changed_units.unwrap();
        assert_eq!(units.len(), 1);
        assert_eq!(units[0].position, (11, 10));
        assert_eq!(
            units[0].movement_remaining,
            unit_before_move.movement_remaining - 1
        );
    }

    #[test]
    fn test_create_event_from_city_growth() {
        // Simulate city population growth
        let city_before = CityUpdate {
            id: 1,
            owner: 0,
            name: "Rome".to_string(),
            position: (25, 25),
            population: 3,
            health: 200,
        };

        let city_after = CityUpdate {
            id: 1,
            owner: 0,
            name: "Rome".to_string(),
            position: (25, 25),
            population: 4, // Grew by 1
            health: 200,
        };

        let payload = GameStateUpdatedPayload {
            game_id: "growth_test".to_string(),
            phase: "Playing".to_string(),
            turn: 10,
            current_player: 0,
            player_count: 2,
            map_dimensions: (50, 50),
            is_full_update: false,
            changed_units: None,
            changed_cities: Some(vec![city_after.clone()]),
            changed_tiles: None,
        };

        assert!(payload.changed_cities.is_some());
        let cities = payload.changed_cities.unwrap();
        assert_eq!(cities[0].population, city_before.population + 1);
    }

    #[test]
    fn test_create_event_from_tile_improvement() {
        // Simulate building a farm on a tile
        let tile_before = TileUpdate {
            position: (15, 20),
            improvement: None,
            road: None,
            owner: Some(0),
        };

        let tile_after = TileUpdate {
            position: (15, 20),
            improvement: Some("Farm".to_string()),
            road: None,
            owner: Some(0),
        };

        let payload = GameStateUpdatedPayload {
            game_id: "improvement_test".to_string(),
            phase: "Playing".to_string(),
            turn: 8,
            current_player: 0,
            player_count: 2,
            map_dimensions: (50, 50),
            is_full_update: false,
            changed_units: None,
            changed_cities: None,
            changed_tiles: Some(vec![tile_after.clone()]),
        };

        assert!(payload.changed_tiles.is_some());
        let tiles = payload.changed_tiles.unwrap();
        assert_eq!(tiles[0].improvement, Some("Farm".to_string()));
        assert!(tile_before.improvement.is_none()); // Verify original had no improvement
    }

    #[test]
    fn test_create_combat_event_from_battle() {
        // Simulate creating a combat event from a battle outcome
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let payload = CombatResolvedPayload {
            attacker: CombatantInfo {
                unit_id: 100,
                owner_id: 0,
                owner_name: "Player 1".to_string(),
                unit_type: "Swordsman".to_string(),
                health_before: 100,
                health_after: 65,
                strength: 14,
            },
            defender: CombatantInfo {
                unit_id: 200,
                owner_id: 1,
                owner_name: "Player 2".to_string(),
                unit_type: "Warrior".to_string(),
                health_before: 100,
                health_after: 0,
                strength: 8,
            },
            results: CombatResults {
                defender_damage: 100,
                attacker_damage: 35,
                defender_destroyed: true,
                attacker_destroyed: false,
                attacker_xp: 10,
                defender_xp: 0,
                was_ranged: false,
            },
            position: (30, 25),
            timestamp,
        };

        // Verify all required combat information is present
        assert!(payload.timestamp > 0);
        assert_eq!(
            payload.attacker.health_before - payload.attacker.health_after,
            payload.results.attacker_damage
        );
        assert_eq!(
            payload.defender.health_before - payload.defender.health_after,
            payload.results.defender_damage
        );
        assert!(payload.results.defender_destroyed);
        assert!(!payload.results.attacker_destroyed);
    }

    #[test]
    fn test_create_turn_sequence_events() {
        // Simulate a complete turn sequence: turn_started -> player_turn -> turn_ended
        let turn = 5;
        let player_id = 0;
        let player_name = "Alice".to_string();

        let turn_started =
            TurnEventPayload::turn_started(turn, player_id, player_name.clone(), true);
        let player_turn = TurnEventPayload::player_turn(turn, player_id, player_name.clone(), true);
        let turn_ended = TurnEventPayload::turn_ended(turn, player_id, player_name.clone(), true);

        // Verify event sequence
        assert_eq!(turn_started.event_type, TurnEventType::TurnStarted);
        assert_eq!(player_turn.event_type, TurnEventType::PlayerTurn);
        assert_eq!(turn_ended.event_type, TurnEventType::TurnEnded);

        // All events should have same turn number
        assert_eq!(turn_started.turn, turn);
        assert_eq!(player_turn.turn, turn);
        assert_eq!(turn_ended.turn, turn);

        // All events should reference same player
        assert_eq!(turn_started.player_id, player_id);
        assert_eq!(player_turn.player_id, player_id);
        assert_eq!(turn_ended.player_id, player_id);
    }

    #[test]
    fn test_create_network_connection_sequence() {
        // Simulate a connection sequence: peer connects -> sync starts -> sync completes
        let peer_id = "peer_123".to_string();
        let peer_name = Some("Bob".to_string());

        let connected = NetworkEventPayload::peer_connected(peer_id.clone(), peer_name.clone(), 1);

        let sync_started = NetworkEventPayload {
            event_type: NetworkEventType::SyncStarted,
            peer_id: Some(peer_id.clone()),
            peer_name: peer_name.clone(),
            peer_count: 1,
            error_message: None,
            sync_progress: Some(0),
        };

        let sync_complete = NetworkEventPayload::sync_complete(1);

        // Verify connection sequence
        assert_eq!(connected.event_type, NetworkEventType::PeerConnected);
        assert_eq!(sync_started.event_type, NetworkEventType::SyncStarted);
        assert_eq!(sync_started.sync_progress, Some(0));
        assert_eq!(sync_complete.event_type, NetworkEventType::SyncComplete);
        assert_eq!(sync_complete.sync_progress, Some(100));
    }

    // =========================================================================
    // Notification Type Coverage Tests
    // =========================================================================

    #[test]
    fn test_notification_achievement_type() {
        let payload = NotificationPayload {
            notification_type: NotificationType::Achievement,
            title: "First Victory".to_string(),
            message: "Won your first battle!".to_string(),
            icon: Some("trophy".to_string()),
            duration_ms: Some(10000),
            action: None,
        };

        assert_eq!(payload.notification_type, NotificationType::Achievement);
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("\"achievement\""));
    }

    #[test]
    fn test_notification_diplomacy_type() {
        let payload = NotificationPayload {
            notification_type: NotificationType::Diplomacy,
            title: "War Declared".to_string(),
            message: "Rome has declared war on you!".to_string(),
            icon: Some("war".to_string()),
            duration_ms: None, // Important diplomatic events should require dismissal
            action: Some(NotificationAction {
                action_type: "open_diplomacy".to_string(),
                label: "View Details".to_string(),
                data: Some(serde_json::json!({"civ_id": 1})),
            }),
        };

        assert_eq!(payload.notification_type, NotificationType::Diplomacy);
        assert!(payload.action.is_some());
    }

    #[test]
    fn test_notification_research_type() {
        let payload = NotificationPayload {
            notification_type: NotificationType::Research,
            title: "Research Complete".to_string(),
            message: "You have discovered Bronze Working!".to_string(),
            icon: Some("science".to_string()),
            duration_ms: Some(8000),
            action: Some(NotificationAction {
                action_type: "open_tech_tree".to_string(),
                label: "Choose Next".to_string(),
                data: None,
            }),
        };

        assert_eq!(payload.notification_type, NotificationType::Research);
    }

    #[test]
    fn test_notification_production_type() {
        let payload = NotificationPayload {
            notification_type: NotificationType::Production,
            title: "Production Complete".to_string(),
            message: "Rome has finished building a Warrior".to_string(),
            icon: Some("hammer".to_string()),
            duration_ms: Some(5000),
            action: Some(NotificationAction {
                action_type: "focus_city".to_string(),
                label: "View City".to_string(),
                data: Some(serde_json::json!({"city_id": 1})),
            }),
        };

        assert_eq!(payload.notification_type, NotificationType::Production);
    }

    #[test]
    fn test_notification_combat_type() {
        let payload = NotificationPayload {
            notification_type: NotificationType::Combat,
            title: "Unit Attacked".to_string(),
            message: "Your Warrior was attacked by an enemy Archer".to_string(),
            icon: Some("sword".to_string()),
            duration_ms: Some(6000),
            action: Some(NotificationAction {
                action_type: "focus_unit".to_string(),
                label: "View Unit".to_string(),
                data: Some(serde_json::json!({"unit_id": 42, "position": [10, 15]})),
            }),
        };

        assert_eq!(payload.notification_type, NotificationType::Combat);
    }

    // =========================================================================
    // Timestamp Tests
    // =========================================================================

    #[test]
    fn test_combat_event_timestamp_validity() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let payload = CombatResolvedPayload {
            attacker: CombatantInfo {
                unit_id: 1,
                owner_id: 0,
                owner_name: "Test".to_string(),
                unit_type: "Warrior".to_string(),
                health_before: 100,
                health_after: 80,
                strength: 8,
            },
            defender: CombatantInfo {
                unit_id: 2,
                owner_id: 1,
                owner_name: "Enemy".to_string(),
                unit_type: "Warrior".to_string(),
                health_before: 100,
                health_after: 60,
                strength: 8,
            },
            results: CombatResults {
                defender_damage: 40,
                attacker_damage: 20,
                defender_destroyed: false,
                attacker_destroyed: false,
                attacker_xp: 5,
                defender_xp: 3,
                was_ranged: false,
            },
            position: (0, 0),
            timestamp: now,
        };

        // Timestamp should be recent (within last minute for test purposes)
        assert!(payload.timestamp >= now - 60);
        assert!(payload.timestamp <= now + 60);
    }

    #[test]
    fn test_timestamp_serialization() {
        let payload = CombatResolvedPayload {
            attacker: CombatantInfo {
                unit_id: 1,
                owner_id: 0,
                owner_name: "A".to_string(),
                unit_type: "W".to_string(),
                health_before: 100,
                health_after: 100,
                strength: 1,
            },
            defender: CombatantInfo {
                unit_id: 2,
                owner_id: 1,
                owner_name: "B".to_string(),
                unit_type: "W".to_string(),
                health_before: 100,
                health_after: 100,
                strength: 1,
            },
            results: CombatResults {
                defender_damage: 0,
                attacker_damage: 0,
                defender_destroyed: false,
                attacker_destroyed: false,
                attacker_xp: 0,
                defender_xp: 0,
                was_ranged: false,
            },
            position: (0, 0),
            timestamp: 1700000000, // Specific timestamp for testing
        };

        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("1700000000"));

        let deserialized: CombatResolvedPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.timestamp, 1700000000);
    }

    // =========================================================================
    // Complex Scenario Tests
    // =========================================================================

    #[test]
    fn test_multi_unit_battle_events() {
        // Test creating multiple unit updates from a battle involving several units
        let units = vec![
            UnitUpdate {
                id: 1,
                owner: 0,
                unit_type: "Swordsman".to_string(),
                position: (10, 10),
                health: 60,
                movement_remaining: 0,
                is_destroyed: false,
            },
            UnitUpdate {
                id: 2,
                owner: 1,
                unit_type: "Warrior".to_string(),
                position: (10, 11),
                health: 0,
                movement_remaining: 0,
                is_destroyed: true,
            },
            UnitUpdate {
                id: 3,
                owner: 0,
                unit_type: "Archer".to_string(),
                position: (9, 10),
                health: 100,
                movement_remaining: 1,
                is_destroyed: false,
            },
        ];

        let payload = GameStateUpdatedPayload {
            game_id: "battle_test".to_string(),
            phase: "Playing".to_string(),
            turn: 15,
            current_player: 0,
            player_count: 2,
            map_dimensions: (50, 50),
            is_full_update: false,
            changed_units: Some(units),
            changed_cities: None,
            changed_tiles: None,
        };

        let changed = payload.changed_units.unwrap();
        assert_eq!(changed.len(), 3);

        // Verify destroyed unit
        let destroyed = changed.iter().find(|u| u.is_destroyed).unwrap();
        assert_eq!(destroyed.health, 0);
        assert_eq!(destroyed.owner, 1);
    }

    #[test]
    fn test_city_capture_events() {
        // Test events generated when a city is captured
        let original_owner = 1;
        let new_owner = 0;

        let city_update = CityUpdate {
            id: 5,
            owner: new_owner,
            name: "Conquered City".to_string(),
            position: (30, 30),
            population: 2, // Population often reduced on capture
            health: 50,    // Damaged from assault
        };

        let payload = GameStateUpdatedPayload {
            game_id: "capture_test".to_string(),
            phase: "Playing".to_string(),
            turn: 20,
            current_player: new_owner,
            player_count: 2,
            map_dimensions: (50, 50),
            is_full_update: false,
            changed_units: None,
            changed_cities: Some(vec![city_update]),
            changed_tiles: None,
        };

        let cities = payload.changed_cities.unwrap();
        assert_eq!(cities[0].owner, new_owner);
        assert_ne!(cities[0].owner, original_owner);
    }

    #[test]
    fn test_combined_updates_payload() {
        // Test a complex payload with all types of changes
        let payload = GameStateUpdatedPayload {
            game_id: "complex_test".to_string(),
            phase: "Playing".to_string(),
            turn: 25,
            current_player: 0,
            player_count: 4,
            map_dimensions: (80, 60),
            is_full_update: false,
            changed_units: Some(vec![UnitUpdate {
                id: 1,
                owner: 0,
                unit_type: "Settler".to_string(),
                position: (40, 30),
                health: 100,
                movement_remaining: 0,
                is_destroyed: true, // Settler consumed to build city
            }]),
            changed_cities: Some(vec![CityUpdate {
                id: 10,
                owner: 0,
                name: "New City".to_string(),
                position: (40, 30),
                population: 1,
                health: 200,
            }]),
            changed_tiles: Some(vec![
                TileUpdate {
                    position: (40, 30),
                    improvement: None,
                    road: None,
                    owner: Some(0),
                },
                TileUpdate {
                    position: (39, 30),
                    improvement: None,
                    road: None,
                    owner: Some(0), // City territory expanded
                },
                TileUpdate {
                    position: (41, 30),
                    improvement: None,
                    road: None,
                    owner: Some(0),
                },
            ]),
        };

        // Verify all components present
        assert!(payload.changed_units.is_some());
        assert!(payload.changed_cities.is_some());
        assert!(payload.changed_tiles.is_some());

        // Settler consumed, city founded at same location
        let units = payload.changed_units.as_ref().unwrap();
        let cities = payload.changed_cities.as_ref().unwrap();
        let tiles = payload.changed_tiles.as_ref().unwrap();

        assert!(units[0].is_destroyed);
        assert_eq!(units[0].position, cities[0].position);
        assert_eq!(tiles.len(), 3); // City + 2 adjacent tiles claimed
    }

    #[test]
    fn test_notification_action_with_complex_data() {
        let action = NotificationAction {
            action_type: "multi_select".to_string(),
            label: "Select Units".to_string(),
            data: Some(serde_json::json!({
                "unit_ids": [1, 2, 3, 4, 5],
                "positions": [[10, 10], [11, 10], [12, 10], [10, 11], [11, 11]],
                "metadata": {
                    "group_name": "Army Alpha",
                    "total_strength": 150
                }
            })),
        };

        let json = serde_json::to_string(&action).unwrap();
        let deserialized: NotificationAction = serde_json::from_str(&json).unwrap();

        assert!(deserialized.data.is_some());
        let data = deserialized.data.unwrap();
        assert!(data["unit_ids"].is_array());
        assert_eq!(data["unit_ids"].as_array().unwrap().len(), 5);
    }

    // =========================================================================
    // JSON Schema Validation Tests
    // =========================================================================

    #[test]
    fn test_all_event_types_produce_valid_json() {
        // GameStateUpdatedPayload
        let game_state = GameStateUpdatedPayload {
            game_id: "test".to_string(),
            phase: "Playing".to_string(),
            turn: 1,
            current_player: 0,
            player_count: 2,
            map_dimensions: (10, 10),
            is_full_update: true,
            changed_units: None,
            changed_cities: None,
            changed_tiles: None,
        };
        assert!(serde_json::to_string(&game_state).is_ok());

        // TurnEventPayload
        let turn = TurnEventPayload::turn_started(1, 0, "Test".to_string(), true);
        assert!(serde_json::to_string(&turn).is_ok());

        // CombatResolvedPayload
        let combat = CombatResolvedPayload {
            attacker: CombatantInfo {
                unit_id: 1,
                owner_id: 0,
                owner_name: "A".to_string(),
                unit_type: "W".to_string(),
                health_before: 100,
                health_after: 80,
                strength: 8,
            },
            defender: CombatantInfo {
                unit_id: 2,
                owner_id: 1,
                owner_name: "B".to_string(),
                unit_type: "W".to_string(),
                health_before: 100,
                health_after: 60,
                strength: 8,
            },
            results: CombatResults {
                defender_damage: 40,
                attacker_damage: 20,
                defender_destroyed: false,
                attacker_destroyed: false,
                attacker_xp: 5,
                defender_xp: 3,
                was_ranged: false,
            },
            position: (5, 5),
            timestamp: 12345,
        };
        assert!(serde_json::to_string(&combat).is_ok());

        // NetworkEventPayload
        let network = NetworkEventPayload::peer_connected("peer".to_string(), None, 1);
        assert!(serde_json::to_string(&network).is_ok());

        // NotificationPayload
        let notification = NotificationPayload::info("Title", "Message");
        assert!(serde_json::to_string(&notification).is_ok());
    }

    #[test]
    fn test_deserialization_from_json_strings() {
        // Test that payloads can be deserialized from raw JSON strings
        // (simulating frontend -> backend communication)

        let game_state_json = r#"{
            "game_id": "test_game",
            "phase": "Playing",
            "turn": 5,
            "current_player": 1,
            "player_count": 2,
            "map_dimensions": [50, 50],
            "is_full_update": false
        }"#;
        let game_state: GameStateUpdatedPayload = serde_json::from_str(game_state_json).unwrap();
        assert_eq!(game_state.game_id, "test_game");
        assert_eq!(game_state.map_dimensions, (50, 50));

        let turn_json = r#"{
            "event_type": "turn_started",
            "turn": 10,
            "player_id": 0,
            "player_name": "Alice",
            "previous_turn": 9,
            "is_local_player": true
        }"#;
        let turn: TurnEventPayload = serde_json::from_str(turn_json).unwrap();
        assert_eq!(turn.event_type, TurnEventType::TurnStarted);
        assert_eq!(turn.previous_turn, Some(9));

        let network_json = r#"{
            "event_type": "peer_connected",
            "peer_id": "abc123",
            "peer_name": "Bob",
            "peer_count": 3
        }"#;
        let network: NetworkEventPayload = serde_json::from_str(network_json).unwrap();
        assert_eq!(network.event_type, NetworkEventType::PeerConnected);
        assert_eq!(network.peer_id, Some("abc123".to_string()));
    }

    #[test]
    fn test_unicode_in_payloads() {
        // Test that unicode characters are handled correctly
        let notification =
            NotificationPayload::info("Victoire!", "Vous avez captur la ville de Nmes");
        let json = serde_json::to_string(&notification).unwrap();
        let deserialized: NotificationPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.title, "Victoire!");
        assert!(deserialized.message.contains("Nmes"));

        let player_name = "".to_string();
        let turn = TurnEventPayload::turn_started(1, 0, player_name.clone(), true);
        let json = serde_json::to_string(&turn).unwrap();
        let deserialized: TurnEventPayload = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.player_name, player_name);
    }

    #[test]
    fn test_special_characters_in_strings() {
        let notification =
            NotificationPayload::info("Test \"quoted\" title", "Message with\nnewline and\ttab");
        let json = serde_json::to_string(&notification).unwrap();
        let deserialized: NotificationPayload = serde_json::from_str(&json).unwrap();
        assert!(deserialized.title.contains("\"quoted\""));
        assert!(deserialized.message.contains("\n"));
        assert!(deserialized.message.contains("\t"));
    }
}

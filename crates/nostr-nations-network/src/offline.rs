//! Offline scenario handling for Nostr Nations.
//!
//! This module provides functionality for handling offline gameplay scenarios:
//! - **OfflineManager**: Manages offline state and event queuing
//! - **OfflineStorage**: Persists events and game state locally
//! - **OfflineSyncStrategy**: Defines how to handle reconnection
//! - **ConnectionMonitor**: Monitors connection health and triggers offline mode
//!
//! # Usage
//!
//! ```rust,ignore
//! use nostr_nations_network::offline::{OfflineManager, OfflineStorage, ConnectionMonitor};
//!
//! let mut manager = OfflineManager::new();
//! let storage = OfflineStorage::new("/path/to/storage");
//! let mut monitor = ConnectionMonitor::new(5000, 10000);
//!
//! // When connection fails
//! monitor.record_failure();
//! if monitor.should_go_offline() {
//!     manager.go_offline();
//! }
//!
//! // Queue events while offline
//! manager.queue_event(event);
//!
//! // When connection restored
//! let pending = manager.go_online();
//! // Send pending events to network
//! ```

use nostr_nations_core::events::GameEvent;
use nostr_nations_core::game_state::GameState;
use std::fs;
use std::io;
use std::path::PathBuf;

/// Error types for offline storage operations.
#[derive(Debug)]
pub enum StorageError {
    /// IO error during file operations.
    Io(io::Error),
    /// Serialization/deserialization error.
    Serialization(String),
    /// Storage path does not exist.
    PathNotFound(PathBuf),
    /// Failed to create storage directory.
    DirectoryCreationFailed(PathBuf),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Io(e) => write!(f, "IO error: {}", e),
            StorageError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::PathNotFound(path) => write!(f, "Path not found: {:?}", path),
            StorageError::DirectoryCreationFailed(path) => {
                write!(f, "Failed to create directory: {:?}", path)
            }
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for StorageError {
    fn from(err: io::Error) -> Self {
        StorageError::Io(err)
    }
}

// ==================== OfflineSyncStrategy ====================

/// Strategy for handling reconnection after being offline.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum OfflineSyncStrategy {
    /// Queue events locally and send them when online.
    /// This is the default strategy - events are stored and sent in order
    /// when connectivity is restored.
    #[default]
    QueueAndSync,
    /// Pause the game until reconnected.
    /// No local actions are allowed while offline - the game waits
    /// for connectivity to be restored.
    PauseUntilOnline,
    /// Allow local turns with merge on reconnect.
    /// Players can continue playing locally, and changes are merged
    /// with the canonical state when reconnected. May require conflict resolution.
    LocalTurnsWithMerge,
}

impl OfflineSyncStrategy {
    /// Check if this strategy allows local actions while offline.
    pub fn allows_local_actions(&self) -> bool {
        match self {
            OfflineSyncStrategy::QueueAndSync => true,
            OfflineSyncStrategy::PauseUntilOnline => false,
            OfflineSyncStrategy::LocalTurnsWithMerge => true,
        }
    }

    /// Check if this strategy requires merge on reconnect.
    pub fn requires_merge(&self) -> bool {
        matches!(self, OfflineSyncStrategy::LocalTurnsWithMerge)
    }
}

// ==================== OfflineManager ====================

/// Manages offline state and synchronization.
///
/// The `OfflineManager` tracks whether the client is online or offline,
/// queues events that occur while offline, and handles the transition
/// back to online state.
#[derive(Clone, Debug)]
pub struct OfflineManager {
    /// Whether the client is currently online.
    is_online: bool,
    /// Events queued while offline, waiting to be sent.
    pending_events: Vec<GameEvent>,
    /// The last turn number that was successfully synced.
    last_sync_turn: u32,
    /// Timestamp of the last successful sync.
    last_sync_timestamp: u64,
    /// Number of connection attempts since going offline.
    connection_attempts: u32,
    /// Maximum number of turns allowed offline before requiring resync.
    max_offline_turns: u32,
    /// The sync strategy to use.
    sync_strategy: OfflineSyncStrategy,
}

impl Default for OfflineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl OfflineManager {
    /// Create a new OfflineManager starting in online state.
    pub fn new() -> Self {
        Self {
            is_online: true,
            pending_events: Vec::new(),
            last_sync_turn: 0,
            last_sync_timestamp: 0,
            connection_attempts: 0,
            max_offline_turns: 10,
            sync_strategy: OfflineSyncStrategy::default(),
        }
    }

    /// Create a new OfflineManager with custom settings.
    pub fn with_config(max_offline_turns: u32, sync_strategy: OfflineSyncStrategy) -> Self {
        Self {
            is_online: true,
            pending_events: Vec::new(),
            last_sync_turn: 0,
            last_sync_timestamp: 0,
            connection_attempts: 0,
            max_offline_turns,
            sync_strategy,
        }
    }

    // ==================== Connection State ====================

    /// Transition to offline state.
    ///
    /// Called when connectivity is lost. The manager will begin
    /// queuing events until `go_online()` is called.
    pub fn go_offline(&mut self) {
        self.is_online = false;
    }

    /// Transition to online state and return pending events.
    ///
    /// Returns all events that were queued while offline.
    /// The caller is responsible for sending these events to the network.
    /// The pending queue is cleared after this call.
    pub fn go_online(&mut self) -> Vec<GameEvent> {
        self.is_online = true;
        self.reset_connection_attempts();
        std::mem::take(&mut self.pending_events)
    }

    /// Check if the manager is currently in online state.
    pub fn is_online(&self) -> bool {
        self.is_online
    }

    /// Get the current sync strategy.
    pub fn sync_strategy(&self) -> OfflineSyncStrategy {
        self.sync_strategy
    }

    /// Set the sync strategy.
    pub fn set_sync_strategy(&mut self, strategy: OfflineSyncStrategy) {
        self.sync_strategy = strategy;
    }

    // ==================== Event Queuing ====================

    /// Queue an event for later synchronization.
    ///
    /// Events are stored in order and will be returned when `go_online()` is called.
    /// If the manager is online, the event is still queued (caller should check
    /// `is_online()` first if they want to send immediately).
    pub fn queue_event(&mut self, event: GameEvent) {
        self.pending_events.push(event);
    }

    /// Get the number of pending events.
    pub fn pending_count(&self) -> usize {
        self.pending_events.len()
    }

    /// Check if there are any pending events.
    pub fn has_pending_events(&self) -> bool {
        !self.pending_events.is_empty()
    }

    /// Get a reference to pending events without consuming them.
    pub fn pending_events(&self) -> &[GameEvent] {
        &self.pending_events
    }

    /// Clear all pending events.
    ///
    /// Use this if events need to be discarded (e.g., due to conflict resolution
    /// choosing to discard local changes).
    pub fn clear_pending(&mut self) {
        self.pending_events.clear();
    }

    // ==================== Sync Status ====================

    /// Check if a full resync is needed.
    ///
    /// Returns true if too many turns have passed since the last sync,
    /// which may indicate that the local state has diverged too much
    /// from the canonical state.
    pub fn needs_resync(&self, current_turn: u32) -> bool {
        if self.last_sync_turn == 0 {
            return false; // Never synced, no baseline to compare against
        }
        current_turn.saturating_sub(self.last_sync_turn) > self.max_offline_turns
    }

    /// Record a successful sync at the given turn.
    pub fn record_sync(&mut self, turn: u32) {
        self.last_sync_turn = turn;
        self.last_sync_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Get the last sync turn number.
    pub fn last_sync_turn(&self) -> u32 {
        self.last_sync_turn
    }

    /// Get the last sync timestamp.
    pub fn last_sync_timestamp(&self) -> u64 {
        self.last_sync_timestamp
    }

    /// Get the maximum allowed offline turns.
    pub fn max_offline_turns(&self) -> u32 {
        self.max_offline_turns
    }

    /// Set the maximum allowed offline turns.
    pub fn set_max_offline_turns(&mut self, max_turns: u32) {
        self.max_offline_turns = max_turns;
    }

    // ==================== Auto-reconnection ====================

    /// Check if a reconnection attempt should be made.
    ///
    /// Uses exponential backoff - attempts are allowed less frequently
    /// as the number of failed attempts increases.
    pub fn should_attempt_reconnect(&self) -> bool {
        if self.is_online {
            return false;
        }
        // Use exponential backoff: allow attempt every 2^attempts times
        // Cap at 2^5 = 32 to prevent excessive delays
        let backoff_factor = 1u32 << self.connection_attempts.min(5);
        // Simple heuristic: always allow first attempt, then use backoff
        self.connection_attempts == 0 || self.connection_attempts.is_multiple_of(backoff_factor)
    }

    /// Record a connection attempt.
    pub fn record_connection_attempt(&mut self) {
        self.connection_attempts = self.connection_attempts.saturating_add(1);
    }

    /// Reset the connection attempt counter.
    ///
    /// Called when a connection is successfully established.
    pub fn reset_connection_attempts(&mut self) {
        self.connection_attempts = 0;
    }

    /// Get the number of connection attempts.
    pub fn connection_attempts(&self) -> u32 {
        self.connection_attempts
    }
}

// ==================== OfflineStorage ====================

/// Persistent storage for offline events and game state.
///
/// Stores pending events and game state to disk so they survive
/// application restarts while offline.
#[derive(Clone, Debug)]
pub struct OfflineStorage {
    /// Root path for storage files.
    storage_path: PathBuf,
}

impl OfflineStorage {
    /// Create a new OfflineStorage at the given path.
    ///
    /// The path should be a directory where storage files will be created.
    /// The directory will be created if it doesn't exist.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            storage_path: path.into(),
        }
    }

    /// Get the storage path.
    pub fn storage_path(&self) -> &PathBuf {
        &self.storage_path
    }

    /// Ensure the storage directory exists.
    fn ensure_directory(&self) -> Result<(), StorageError> {
        if !self.storage_path.exists() {
            fs::create_dir_all(&self.storage_path)
                .map_err(|_| StorageError::DirectoryCreationFailed(self.storage_path.clone()))?;
        }
        Ok(())
    }

    /// Get the path for pending events file.
    fn pending_events_path(&self) -> PathBuf {
        self.storage_path.join("pending_events.json")
    }

    /// Get the path for game state file.
    fn game_state_path(&self) -> PathBuf {
        self.storage_path.join("game_state.json")
    }

    /// Save pending events to disk.
    pub fn save_pending_events(&self, events: &[GameEvent]) -> Result<(), StorageError> {
        self.ensure_directory()?;
        let json = serde_json::to_string_pretty(events)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(self.pending_events_path(), json)?;
        Ok(())
    }

    /// Load pending events from disk.
    ///
    /// Returns an empty vector if the file doesn't exist.
    pub fn load_pending_events(&self) -> Result<Vec<GameEvent>, StorageError> {
        let path = self.pending_events_path();
        if !path.exists() {
            return Ok(Vec::new());
        }
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    /// Save game state to disk.
    pub fn save_game_state(&self, game: &GameState) -> Result<(), StorageError> {
        self.ensure_directory()?;
        let json = serde_json::to_string_pretty(game)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;
        fs::write(self.game_state_path(), json)?;
        Ok(())
    }

    /// Load game state from disk.
    pub fn load_game_state(&self) -> Result<GameState, StorageError> {
        let path = self.game_state_path();
        if !path.exists() {
            return Err(StorageError::PathNotFound(path));
        }
        let json = fs::read_to_string(path)?;
        serde_json::from_str(&json).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    /// Check if a saved game state exists.
    pub fn has_game_state(&self) -> bool {
        self.game_state_path().exists()
    }

    /// Check if there are saved pending events.
    pub fn has_pending_events(&self) -> bool {
        self.pending_events_path().exists()
    }

    /// Clear all stored data.
    pub fn clear(&self) -> Result<(), StorageError> {
        let events_path = self.pending_events_path();
        let state_path = self.game_state_path();

        if events_path.exists() {
            fs::remove_file(events_path)?;
        }
        if state_path.exists() {
            fs::remove_file(state_path)?;
        }
        Ok(())
    }

    /// Clear only pending events.
    pub fn clear_pending_events(&self) -> Result<(), StorageError> {
        let path = self.pending_events_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    /// Clear only game state.
    pub fn clear_game_state(&self) -> Result<(), StorageError> {
        let path = self.game_state_path();
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }
}

// ==================== ConnectionMonitor ====================

/// Monitors connection health and determines when to go offline.
///
/// The monitor tracks successful and failed connection attempts,
/// using configurable thresholds to determine when connectivity
/// has been lost.
#[derive(Clone, Debug)]
pub struct ConnectionMonitor {
    /// Interval between health checks in milliseconds.
    check_interval_ms: u64,
    /// Timeout for connection attempts in milliseconds.
    timeout_ms: u64,
    /// Number of consecutive failures.
    consecutive_failures: u32,
    /// Number of failures before considering connection unhealthy.
    failure_threshold: u32,
    /// Total successful connection count (for statistics).
    total_successes: u64,
    /// Total failure count (for statistics).
    total_failures: u64,
    /// Timestamp of last successful connection.
    last_success_timestamp: u64,
}

impl Default for ConnectionMonitor {
    fn default() -> Self {
        Self::new(5000, 10000)
    }
}

impl ConnectionMonitor {
    /// Create a new ConnectionMonitor with the specified intervals.
    ///
    /// # Arguments
    ///
    /// * `check_interval_ms` - How often to check connection health (milliseconds)
    /// * `timeout_ms` - How long to wait for a response before considering it failed
    pub fn new(check_interval_ms: u64, timeout_ms: u64) -> Self {
        Self {
            check_interval_ms,
            timeout_ms,
            consecutive_failures: 0,
            failure_threshold: 3,
            total_successes: 0,
            total_failures: 0,
            last_success_timestamp: 0,
        }
    }

    /// Create a new ConnectionMonitor with a custom failure threshold.
    pub fn with_threshold(check_interval_ms: u64, timeout_ms: u64, failure_threshold: u32) -> Self {
        Self {
            check_interval_ms,
            timeout_ms,
            consecutive_failures: 0,
            failure_threshold,
            total_successes: 0,
            total_failures: 0,
            last_success_timestamp: 0,
        }
    }

    /// Record a successful connection/ping.
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.total_successes += 1;
        self.last_success_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Record a failed connection/ping.
    pub fn record_failure(&mut self) {
        self.consecutive_failures = self.consecutive_failures.saturating_add(1);
        self.total_failures += 1;
    }

    /// Check if the connection is currently considered healthy.
    ///
    /// The connection is healthy if consecutive failures are below the threshold.
    pub fn is_connection_healthy(&self) -> bool {
        self.consecutive_failures < self.failure_threshold
    }

    /// Check if the client should transition to offline mode.
    ///
    /// Returns true if consecutive failures have reached or exceeded the threshold.
    pub fn should_go_offline(&self) -> bool {
        self.consecutive_failures >= self.failure_threshold
    }

    /// Get the number of consecutive failures.
    pub fn consecutive_failures(&self) -> u32 {
        self.consecutive_failures
    }

    /// Get the failure threshold.
    pub fn failure_threshold(&self) -> u32 {
        self.failure_threshold
    }

    /// Set the failure threshold.
    pub fn set_failure_threshold(&mut self, threshold: u32) {
        self.failure_threshold = threshold;
    }

    /// Get the check interval in milliseconds.
    pub fn check_interval_ms(&self) -> u64 {
        self.check_interval_ms
    }

    /// Set the check interval in milliseconds.
    pub fn set_check_interval_ms(&mut self, interval: u64) {
        self.check_interval_ms = interval;
    }

    /// Get the timeout in milliseconds.
    pub fn timeout_ms(&self) -> u64 {
        self.timeout_ms
    }

    /// Set the timeout in milliseconds.
    pub fn set_timeout_ms(&mut self, timeout: u64) {
        self.timeout_ms = timeout;
    }

    /// Get the total number of successful connections.
    pub fn total_successes(&self) -> u64 {
        self.total_successes
    }

    /// Get the total number of failed connections.
    pub fn total_failures(&self) -> u64 {
        self.total_failures
    }

    /// Get the timestamp of the last successful connection.
    pub fn last_success_timestamp(&self) -> u64 {
        self.last_success_timestamp
    }

    /// Reset all counters and statistics.
    pub fn reset(&mut self) {
        self.consecutive_failures = 0;
        self.total_successes = 0;
        self.total_failures = 0;
        self.last_success_timestamp = 0;
    }

    /// Calculate the success rate as a percentage.
    ///
    /// Returns None if no connections have been attempted.
    pub fn success_rate(&self) -> Option<f64> {
        let total = self.total_successes + self.total_failures;
        if total == 0 {
            None
        } else {
            Some(self.total_successes as f64 / total as f64 * 100.0)
        }
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;
    use nostr_nations_core::settings::GameSettings;
    use tempfile::TempDir;

    // Helper to create a test event
    fn create_test_event(id: &str, turn: u32) -> GameEvent {
        let mut event = GameEvent::new("game1".to_string(), 0, None, turn, 1, GameAction::EndTurn);
        event.id = id.to_string();
        event.timestamp = 1000 + turn as u64;
        event
    }

    // Helper to create a test game state
    fn create_test_game_state() -> GameState {
        let settings = GameSettings::default();
        GameState::new("game1".to_string(), settings, [0u8; 32])
    }

    // ==================== OfflineSyncStrategy Tests ====================

    #[test]
    fn test_sync_strategy_default() {
        let strategy = OfflineSyncStrategy::default();
        assert_eq!(strategy, OfflineSyncStrategy::QueueAndSync);
    }

    #[test]
    fn test_sync_strategy_allows_local_actions() {
        assert!(OfflineSyncStrategy::QueueAndSync.allows_local_actions());
        assert!(!OfflineSyncStrategy::PauseUntilOnline.allows_local_actions());
        assert!(OfflineSyncStrategy::LocalTurnsWithMerge.allows_local_actions());
    }

    #[test]
    fn test_sync_strategy_requires_merge() {
        assert!(!OfflineSyncStrategy::QueueAndSync.requires_merge());
        assert!(!OfflineSyncStrategy::PauseUntilOnline.requires_merge());
        assert!(OfflineSyncStrategy::LocalTurnsWithMerge.requires_merge());
    }

    #[test]
    fn test_sync_strategy_clone_and_eq() {
        let strategy = OfflineSyncStrategy::LocalTurnsWithMerge;
        let cloned = strategy;
        assert_eq!(strategy, cloned);
    }

    // ==================== OfflineManager Tests ====================

    #[test]
    fn test_offline_manager_new() {
        let manager = OfflineManager::new();
        assert!(manager.is_online());
        assert_eq!(manager.pending_count(), 0);
        assert_eq!(manager.last_sync_turn(), 0);
        assert_eq!(manager.connection_attempts(), 0);
    }

    #[test]
    fn test_offline_manager_default() {
        let manager = OfflineManager::default();
        assert!(manager.is_online());
        assert_eq!(manager.max_offline_turns(), 10);
    }

    #[test]
    fn test_offline_manager_with_config() {
        let manager = OfflineManager::with_config(5, OfflineSyncStrategy::PauseUntilOnline);
        assert_eq!(manager.max_offline_turns(), 5);
        assert_eq!(
            manager.sync_strategy(),
            OfflineSyncStrategy::PauseUntilOnline
        );
    }

    #[test]
    fn test_offline_manager_go_offline() {
        let mut manager = OfflineManager::new();
        assert!(manager.is_online());

        manager.go_offline();
        assert!(!manager.is_online());
    }

    #[test]
    fn test_offline_manager_go_online_returns_pending() {
        let mut manager = OfflineManager::new();
        manager.go_offline();

        let event1 = create_test_event("e1", 1);
        let event2 = create_test_event("e2", 2);
        manager.queue_event(event1);
        manager.queue_event(event2);

        assert_eq!(manager.pending_count(), 2);

        let pending = manager.go_online();
        assert!(manager.is_online());
        assert_eq!(pending.len(), 2);
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_offline_manager_queue_event() {
        let mut manager = OfflineManager::new();
        assert!(!manager.has_pending_events());

        let event = create_test_event("e1", 1);
        manager.queue_event(event);

        assert!(manager.has_pending_events());
        assert_eq!(manager.pending_count(), 1);
    }

    #[test]
    fn test_offline_manager_pending_events_ref() {
        let mut manager = OfflineManager::new();
        let event = create_test_event("e1", 1);
        manager.queue_event(event);

        let pending = manager.pending_events();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "e1");
    }

    #[test]
    fn test_offline_manager_clear_pending() {
        let mut manager = OfflineManager::new();
        manager.queue_event(create_test_event("e1", 1));
        manager.queue_event(create_test_event("e2", 2));

        assert_eq!(manager.pending_count(), 2);

        manager.clear_pending();
        assert_eq!(manager.pending_count(), 0);
    }

    #[test]
    fn test_offline_manager_needs_resync_never_synced() {
        let manager = OfflineManager::new();
        // Never synced, so no baseline - doesn't need resync
        assert!(!manager.needs_resync(100));
    }

    #[test]
    fn test_offline_manager_needs_resync_within_limit() {
        let mut manager = OfflineManager::new();
        manager.record_sync(10);

        // 5 turns later, within max_offline_turns (10)
        assert!(!manager.needs_resync(15));
    }

    #[test]
    fn test_offline_manager_needs_resync_exceeded() {
        let mut manager = OfflineManager::new();
        manager.record_sync(10);

        // 15 turns later, exceeds max_offline_turns (10)
        assert!(manager.needs_resync(25));
    }

    #[test]
    fn test_offline_manager_record_sync() {
        let mut manager = OfflineManager::new();
        manager.record_sync(5);

        assert_eq!(manager.last_sync_turn(), 5);
        assert!(manager.last_sync_timestamp() > 0);
    }

    #[test]
    fn test_offline_manager_connection_attempts() {
        let mut manager = OfflineManager::new();
        manager.go_offline();

        assert!(manager.should_attempt_reconnect());

        manager.record_connection_attempt();
        assert_eq!(manager.connection_attempts(), 1);

        // After first attempt, backoff kicks in
        manager.record_connection_attempt();
        assert_eq!(manager.connection_attempts(), 2);
    }

    #[test]
    fn test_offline_manager_reset_connection_attempts() {
        let mut manager = OfflineManager::new();
        manager.record_connection_attempt();
        manager.record_connection_attempt();
        assert_eq!(manager.connection_attempts(), 2);

        manager.reset_connection_attempts();
        assert_eq!(manager.connection_attempts(), 0);
    }

    #[test]
    fn test_offline_manager_should_attempt_reconnect_when_online() {
        let manager = OfflineManager::new();
        assert!(!manager.should_attempt_reconnect());
    }

    #[test]
    fn test_offline_manager_should_attempt_reconnect_backoff() {
        let mut manager = OfflineManager::new();
        manager.go_offline();

        // First attempt always allowed (connection_attempts = 0)
        assert!(manager.should_attempt_reconnect());
        manager.record_connection_attempt(); // Now connection_attempts = 1

        // After 1 attempt, backoff = 2^1 = 2, 1 % 2 == 1 != 0, so not allowed
        assert!(!manager.should_attempt_reconnect());
        manager.record_connection_attempt(); // Now connection_attempts = 2

        // After 2 attempts, backoff = 2^2 = 4, 2 % 4 == 2 != 0, so not allowed
        assert!(!manager.should_attempt_reconnect());
        manager.record_connection_attempt(); // Now connection_attempts = 3

        // After 3 attempts, backoff = 2^3 = 8, 3 % 8 == 3 != 0, so not allowed
        assert!(!manager.should_attempt_reconnect());
        manager.record_connection_attempt(); // Now connection_attempts = 4

        // After 4 attempts, backoff = 2^4 = 16, 4 % 16 == 4 != 0, so not allowed
        // Eventually at attempt 8, backoff = 2^5 (capped) = 32, 8 % 32 == 8 != 0
        // But at attempt 32, 32 % 32 == 0, so allowed
        // Let's verify the cap works
        for _ in 4..32 {
            manager.record_connection_attempt();
        }
        // Now connection_attempts = 32, backoff = 2^5 = 32 (capped), 32 % 32 == 0
        assert!(manager.should_attempt_reconnect());
    }

    #[test]
    fn test_offline_manager_set_sync_strategy() {
        let mut manager = OfflineManager::new();
        assert_eq!(manager.sync_strategy(), OfflineSyncStrategy::QueueAndSync);

        manager.set_sync_strategy(OfflineSyncStrategy::LocalTurnsWithMerge);
        assert_eq!(
            manager.sync_strategy(),
            OfflineSyncStrategy::LocalTurnsWithMerge
        );
    }

    #[test]
    fn test_offline_manager_set_max_offline_turns() {
        let mut manager = OfflineManager::new();
        assert_eq!(manager.max_offline_turns(), 10);

        manager.set_max_offline_turns(20);
        assert_eq!(manager.max_offline_turns(), 20);
    }

    // ==================== OfflineStorage Tests ====================

    #[test]
    fn test_offline_storage_new() {
        let storage = OfflineStorage::new("/tmp/test");
        assert_eq!(storage.storage_path(), &PathBuf::from("/tmp/test"));
    }

    #[test]
    fn test_offline_storage_save_and_load_pending_events() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());

        let events = vec![create_test_event("e1", 1), create_test_event("e2", 2)];

        storage.save_pending_events(&events).unwrap();
        assert!(storage.has_pending_events());

        let loaded = storage.load_pending_events().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].id, "e1");
        assert_eq!(loaded[1].id, "e2");
    }

    #[test]
    fn test_offline_storage_load_pending_events_empty() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());

        // No file exists
        assert!(!storage.has_pending_events());

        let loaded = storage.load_pending_events().unwrap();
        assert!(loaded.is_empty());
    }

    #[test]
    fn test_offline_storage_save_and_load_game_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());

        let game = create_test_game_state();

        storage.save_game_state(&game).unwrap();
        assert!(storage.has_game_state());

        let loaded = storage.load_game_state().unwrap();
        assert_eq!(loaded.id, "game1");
    }

    #[test]
    fn test_offline_storage_load_game_state_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());

        let result = storage.load_game_state();
        assert!(matches!(result, Err(StorageError::PathNotFound(_))));
    }

    #[test]
    fn test_offline_storage_clear() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());

        let events = vec![create_test_event("e1", 1)];
        let game = create_test_game_state();

        storage.save_pending_events(&events).unwrap();
        storage.save_game_state(&game).unwrap();

        assert!(storage.has_pending_events());
        assert!(storage.has_game_state());

        storage.clear().unwrap();

        assert!(!storage.has_pending_events());
        assert!(!storage.has_game_state());
    }

    #[test]
    fn test_offline_storage_clear_pending_events() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());

        let events = vec![create_test_event("e1", 1)];
        let game = create_test_game_state();

        storage.save_pending_events(&events).unwrap();
        storage.save_game_state(&game).unwrap();

        storage.clear_pending_events().unwrap();

        assert!(!storage.has_pending_events());
        assert!(storage.has_game_state());
    }

    #[test]
    fn test_offline_storage_clear_game_state() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());

        let events = vec![create_test_event("e1", 1)];
        let game = create_test_game_state();

        storage.save_pending_events(&events).unwrap();
        storage.save_game_state(&game).unwrap();

        storage.clear_game_state().unwrap();

        assert!(storage.has_pending_events());
        assert!(!storage.has_game_state());
    }

    #[test]
    fn test_offline_storage_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path().join("nested").join("storage");
        let storage = OfflineStorage::new(&storage_path);

        assert!(!storage_path.exists());

        let events = vec![create_test_event("e1", 1)];
        storage.save_pending_events(&events).unwrap();

        assert!(storage_path.exists());
    }

    // ==================== ConnectionMonitor Tests ====================

    #[test]
    fn test_connection_monitor_new() {
        let monitor = ConnectionMonitor::new(5000, 10000);
        assert_eq!(monitor.check_interval_ms(), 5000);
        assert_eq!(monitor.timeout_ms(), 10000);
        assert_eq!(monitor.consecutive_failures(), 0);
        assert_eq!(monitor.failure_threshold(), 3);
    }

    #[test]
    fn test_connection_monitor_default() {
        let monitor = ConnectionMonitor::default();
        assert_eq!(monitor.check_interval_ms(), 5000);
        assert_eq!(monitor.timeout_ms(), 10000);
    }

    #[test]
    fn test_connection_monitor_with_threshold() {
        let monitor = ConnectionMonitor::with_threshold(1000, 2000, 5);
        assert_eq!(monitor.failure_threshold(), 5);
    }

    #[test]
    fn test_connection_monitor_record_success() {
        let mut monitor = ConnectionMonitor::new(5000, 10000);
        monitor.record_failure();
        monitor.record_failure();
        assert_eq!(monitor.consecutive_failures(), 2);

        monitor.record_success();
        assert_eq!(monitor.consecutive_failures(), 0);
        assert_eq!(monitor.total_successes(), 1);
        assert!(monitor.last_success_timestamp() > 0);
    }

    #[test]
    fn test_connection_monitor_record_failure() {
        let mut monitor = ConnectionMonitor::new(5000, 10000);
        monitor.record_failure();
        assert_eq!(monitor.consecutive_failures(), 1);
        assert_eq!(monitor.total_failures(), 1);

        monitor.record_failure();
        assert_eq!(monitor.consecutive_failures(), 2);
        assert_eq!(monitor.total_failures(), 2);
    }

    #[test]
    fn test_connection_monitor_is_connection_healthy() {
        let mut monitor = ConnectionMonitor::new(5000, 10000);
        assert!(monitor.is_connection_healthy());

        monitor.record_failure();
        monitor.record_failure();
        assert!(monitor.is_connection_healthy()); // Still under threshold (3)

        monitor.record_failure();
        assert!(!monitor.is_connection_healthy()); // At threshold
    }

    #[test]
    fn test_connection_monitor_should_go_offline() {
        let mut monitor = ConnectionMonitor::new(5000, 10000);
        assert!(!monitor.should_go_offline());

        monitor.record_failure();
        monitor.record_failure();
        assert!(!monitor.should_go_offline());

        monitor.record_failure();
        assert!(monitor.should_go_offline()); // At threshold (3)
    }

    #[test]
    fn test_connection_monitor_reset() {
        let mut monitor = ConnectionMonitor::new(5000, 10000);
        monitor.record_success();
        monitor.record_failure();
        monitor.record_failure();

        monitor.reset();

        assert_eq!(monitor.consecutive_failures(), 0);
        assert_eq!(monitor.total_successes(), 0);
        assert_eq!(monitor.total_failures(), 0);
        assert_eq!(monitor.last_success_timestamp(), 0);
    }

    #[test]
    fn test_connection_monitor_success_rate() {
        let mut monitor = ConnectionMonitor::new(5000, 10000);
        assert!(monitor.success_rate().is_none());

        monitor.record_success();
        monitor.record_success();
        monitor.record_failure();

        let rate = monitor.success_rate().unwrap();
        assert!((rate - 66.666).abs() < 1.0); // ~66.67%
    }

    #[test]
    fn test_connection_monitor_setters() {
        let mut monitor = ConnectionMonitor::new(5000, 10000);

        monitor.set_check_interval_ms(2000);
        assert_eq!(monitor.check_interval_ms(), 2000);

        monitor.set_timeout_ms(5000);
        assert_eq!(monitor.timeout_ms(), 5000);

        monitor.set_failure_threshold(5);
        assert_eq!(monitor.failure_threshold(), 5);
    }

    // ==================== StorageError Tests ====================

    #[test]
    fn test_storage_error_display() {
        let io_err = StorageError::Io(io::Error::new(io::ErrorKind::NotFound, "file not found"));
        assert!(format!("{}", io_err).contains("IO error"));

        let ser_err = StorageError::Serialization("invalid json".to_string());
        assert!(format!("{}", ser_err).contains("Serialization error"));

        let path_err = StorageError::PathNotFound(PathBuf::from("/test"));
        assert!(format!("{}", path_err).contains("Path not found"));

        let dir_err = StorageError::DirectoryCreationFailed(PathBuf::from("/test"));
        assert!(format!("{}", dir_err).contains("Failed to create directory"));
    }

    #[test]
    fn test_storage_error_from_io() {
        let io_err = io::Error::new(io::ErrorKind::PermissionDenied, "access denied");
        let storage_err: StorageError = io_err.into();
        assert!(matches!(storage_err, StorageError::Io(_)));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_offline_workflow_complete() {
        let temp_dir = TempDir::new().unwrap();
        let storage = OfflineStorage::new(temp_dir.path());
        let mut manager = OfflineManager::new();
        let mut monitor = ConnectionMonitor::new(5000, 10000);

        // Simulate connection failures
        monitor.record_failure();
        monitor.record_failure();
        monitor.record_failure();
        assert!(monitor.should_go_offline());

        // Go offline
        manager.go_offline();
        assert!(!manager.is_online());

        // Queue events while offline
        manager.queue_event(create_test_event("e1", 1));
        manager.queue_event(create_test_event("e2", 2));

        // Persist events
        storage
            .save_pending_events(manager.pending_events())
            .unwrap();

        // Simulate app restart - load from storage
        let loaded_events = storage.load_pending_events().unwrap();
        let mut manager2 = OfflineManager::new();
        manager2.go_offline();
        for event in loaded_events {
            manager2.queue_event(event);
        }

        assert_eq!(manager2.pending_count(), 2);

        // Connection restored
        monitor.record_success();
        let pending = manager2.go_online();

        assert!(manager2.is_online());
        assert_eq!(pending.len(), 2);

        // Clean up
        storage.clear().unwrap();
    }

    #[test]
    fn test_resync_detection() {
        let mut manager = OfflineManager::with_config(5, OfflineSyncStrategy::QueueAndSync);

        // First sync at turn 10
        manager.record_sync(10);
        assert!(!manager.needs_resync(12)); // 2 turns later

        // Go offline and miss several turns
        manager.go_offline();
        assert!(!manager.needs_resync(14)); // 4 turns later - still OK

        // Too many turns passed
        assert!(manager.needs_resync(20)); // 10 turns later - needs resync
    }
}

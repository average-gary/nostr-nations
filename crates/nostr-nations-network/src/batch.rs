//! Event batching for network optimization.
//!
//! This module provides functionality to batch multiple small events into
//! single messages to reduce network round trips and improve throughput.

use nostr_nations_core::events::GameEvent;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Configuration for event batching.
#[derive(Clone, Debug)]
pub struct BatchConfig {
    /// Maximum number of events in a batch.
    pub max_batch_size: usize,
    /// Maximum time to wait before sending a batch.
    pub max_batch_timeout: Duration,
    /// Minimum size threshold for compression (bytes).
    pub compression_threshold: usize,
    /// Whether compression is enabled.
    pub compression_enabled: bool,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_batch_size: 50,
            max_batch_timeout: Duration::from_millis(100),
            compression_threshold: 1024,
            compression_enabled: true,
        }
    }
}

/// A batch of events ready for transmission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EventBatch {
    /// Batch identifier for deduplication.
    pub batch_id: u64,
    /// Events in this batch.
    pub events: Vec<GameEvent>,
    /// Whether the payload is compressed.
    pub compressed: bool,
    /// Original size before compression (for stats).
    pub original_size: usize,
    /// Timestamp when the batch was created.
    pub created_at: u64,
}

impl EventBatch {
    /// Create a new batch with the given events.
    pub fn new(batch_id: u64, events: Vec<GameEvent>) -> Self {
        Self {
            batch_id,
            events,
            compressed: false,
            original_size: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Get the number of events in this batch.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Serialize the batch to bytes.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize a batch from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Manages batching of outgoing events.
pub struct EventBatcher {
    /// Configuration.
    config: BatchConfig,
    /// Pending events waiting to be batched.
    pending: VecDeque<GameEvent>,
    /// Time when the first event was added to the current batch.
    batch_start: Option<Instant>,
    /// Next batch ID.
    next_batch_id: u64,
    /// Statistics.
    stats: BatchStats,
}

/// Statistics for batching operations.
#[derive(Clone, Debug, Default)]
pub struct BatchStats {
    /// Total events batched.
    pub events_batched: u64,
    /// Total batches created.
    pub batches_created: u64,
    /// Total bytes before compression.
    pub bytes_before_compression: u64,
    /// Total bytes after compression.
    pub bytes_after_compression: u64,
    /// Average batch size.
    pub avg_batch_size: f64,
}

impl EventBatcher {
    /// Create a new event batcher with the given configuration.
    pub fn new(config: BatchConfig) -> Self {
        Self {
            config,
            pending: VecDeque::new(),
            batch_start: None,
            next_batch_id: 1,
            stats: BatchStats::default(),
        }
    }

    /// Create a new event batcher with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(BatchConfig::default())
    }

    /// Add an event to the pending batch.
    pub fn add_event(&mut self, event: GameEvent) {
        if self.batch_start.is_none() {
            self.batch_start = Some(Instant::now());
        }
        self.pending.push_back(event);
    }

    /// Check if a batch is ready to be sent.
    pub fn is_batch_ready(&self) -> bool {
        // Batch is ready if we have max events
        if self.pending.len() >= self.config.max_batch_size {
            return true;
        }

        // Or if timeout has elapsed
        if let Some(start) = self.batch_start {
            if start.elapsed() >= self.config.max_batch_timeout {
                return !self.pending.is_empty();
            }
        }

        false
    }

    /// Flush the current batch (regardless of size/timeout).
    pub fn flush(&mut self) -> Option<EventBatch> {
        if self.pending.is_empty() {
            return None;
        }

        let events: Vec<GameEvent> = self.pending.drain(..).collect();
        self.batch_start = None;

        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;

        // Update stats
        self.stats.events_batched += events.len() as u64;
        self.stats.batches_created += 1;
        self.stats.avg_batch_size =
            self.stats.events_batched as f64 / self.stats.batches_created as f64;

        Some(EventBatch::new(batch_id, events))
    }

    /// Take the next ready batch, if any.
    pub fn take_batch(&mut self) -> Option<EventBatch> {
        if self.is_batch_ready() {
            self.flush()
        } else {
            None
        }
    }

    /// Get the number of pending events.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    /// Get batching statistics.
    pub fn stats(&self) -> &BatchStats {
        &self.stats
    }

    /// Get the configuration.
    pub fn config(&self) -> &BatchConfig {
        &self.config
    }

    /// Update the configuration.
    pub fn set_config(&mut self, config: BatchConfig) {
        self.config = config;
    }
}

/// Manages unbatching of incoming events.
pub struct EventUnbatcher {
    /// Set of seen batch IDs for deduplication.
    seen_batches: std::collections::HashSet<u64>,
    /// Maximum number of batch IDs to remember.
    max_seen_batches: usize,
    /// Statistics.
    stats: UnbatchStats,
}

/// Statistics for unbatching operations.
#[derive(Clone, Debug, Default)]
pub struct UnbatchStats {
    /// Total batches processed.
    pub batches_processed: u64,
    /// Total events extracted.
    pub events_extracted: u64,
    /// Duplicate batches ignored.
    pub duplicates_ignored: u64,
}

impl EventUnbatcher {
    /// Create a new unbatcher.
    pub fn new() -> Self {
        Self {
            seen_batches: std::collections::HashSet::new(),
            max_seen_batches: 10000,
            stats: UnbatchStats::default(),
        }
    }

    /// Process a batch and extract events.
    /// Returns None if the batch was already seen (duplicate).
    pub fn process_batch(&mut self, batch: EventBatch) -> Option<Vec<GameEvent>> {
        // Check for duplicate
        if self.seen_batches.contains(&batch.batch_id) {
            self.stats.duplicates_ignored += 1;
            return None;
        }

        // Remember this batch ID
        if self.seen_batches.len() >= self.max_seen_batches {
            // Remove oldest entries (simple strategy: clear half)
            let to_keep: Vec<u64> = self
                .seen_batches
                .iter()
                .skip(self.max_seen_batches / 2)
                .copied()
                .collect();
            self.seen_batches.clear();
            for id in to_keep {
                self.seen_batches.insert(id);
            }
        }
        self.seen_batches.insert(batch.batch_id);

        // Update stats
        self.stats.batches_processed += 1;
        self.stats.events_extracted += batch.events.len() as u64;

        Some(batch.events)
    }

    /// Get unbatching statistics.
    pub fn stats(&self) -> &UnbatchStats {
        &self.stats
    }

    /// Clear the seen batches cache.
    pub fn clear_cache(&mut self) {
        self.seen_batches.clear();
    }
}

impl Default for EventUnbatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;

    fn create_test_event(id: &str) -> GameEvent {
        let mut event = GameEvent::new("test_game".to_string(), 0, None, 1, 1, GameAction::EndTurn);
        event.id = id.to_string();
        event
    }

    // ==================== BatchConfig Tests ====================

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.max_batch_size, 50);
        assert_eq!(config.max_batch_timeout, Duration::from_millis(100));
        assert_eq!(config.compression_threshold, 1024);
        assert!(config.compression_enabled);
    }

    #[test]
    fn test_batch_config_custom() {
        let config = BatchConfig {
            max_batch_size: 100,
            max_batch_timeout: Duration::from_millis(200),
            compression_threshold: 2048,
            compression_enabled: false,
        };
        assert_eq!(config.max_batch_size, 100);
        assert!(!config.compression_enabled);
    }

    // ==================== EventBatch Tests ====================

    #[test]
    fn test_event_batch_new() {
        let events = vec![create_test_event("e1"), create_test_event("e2")];
        let batch = EventBatch::new(1, events);

        assert_eq!(batch.batch_id, 1);
        assert_eq!(batch.len(), 2);
        assert!(!batch.is_empty());
        assert!(!batch.compressed);
    }

    #[test]
    fn test_event_batch_empty() {
        let batch = EventBatch::new(1, vec![]);
        assert!(batch.is_empty());
        assert_eq!(batch.len(), 0);
    }

    #[test]
    fn test_event_batch_serialization() {
        let events = vec![create_test_event("e1")];
        let batch = EventBatch::new(42, events);

        let bytes = batch.to_bytes().unwrap();
        let restored = EventBatch::from_bytes(&bytes).unwrap();

        assert_eq!(restored.batch_id, 42);
        assert_eq!(restored.len(), 1);
        assert_eq!(restored.events[0].id, "e1");
    }

    // ==================== EventBatcher Tests ====================

    #[test]
    fn test_event_batcher_new() {
        let batcher = EventBatcher::with_defaults();
        assert_eq!(batcher.pending_count(), 0);
        assert!(!batcher.is_batch_ready());
    }

    #[test]
    fn test_event_batcher_add_event() {
        let mut batcher = EventBatcher::with_defaults();

        batcher.add_event(create_test_event("e1"));
        assert_eq!(batcher.pending_count(), 1);

        batcher.add_event(create_test_event("e2"));
        assert_eq!(batcher.pending_count(), 2);
    }

    #[test]
    fn test_event_batcher_batch_ready_by_size() {
        let config = BatchConfig {
            max_batch_size: 3,
            ..Default::default()
        };
        let mut batcher = EventBatcher::new(config);

        batcher.add_event(create_test_event("e1"));
        batcher.add_event(create_test_event("e2"));
        assert!(!batcher.is_batch_ready());

        batcher.add_event(create_test_event("e3"));
        assert!(batcher.is_batch_ready());
    }

    #[test]
    fn test_event_batcher_flush() {
        let mut batcher = EventBatcher::with_defaults();

        batcher.add_event(create_test_event("e1"));
        batcher.add_event(create_test_event("e2"));

        let batch = batcher.flush().unwrap();
        assert_eq!(batch.len(), 2);
        assert_eq!(batcher.pending_count(), 0);
    }

    #[test]
    fn test_event_batcher_flush_empty() {
        let mut batcher = EventBatcher::with_defaults();
        assert!(batcher.flush().is_none());
    }

    #[test]
    fn test_event_batcher_take_batch() {
        let config = BatchConfig {
            max_batch_size: 2,
            ..Default::default()
        };
        let mut batcher = EventBatcher::new(config);

        batcher.add_event(create_test_event("e1"));
        assert!(batcher.take_batch().is_none());

        batcher.add_event(create_test_event("e2"));
        let batch = batcher.take_batch().unwrap();
        assert_eq!(batch.len(), 2);
    }

    #[test]
    fn test_event_batcher_stats() {
        let config = BatchConfig {
            max_batch_size: 2,
            ..Default::default()
        };
        let mut batcher = EventBatcher::new(config);

        batcher.add_event(create_test_event("e1"));
        batcher.add_event(create_test_event("e2"));
        batcher.flush();

        batcher.add_event(create_test_event("e3"));
        batcher.flush();

        let stats = batcher.stats();
        assert_eq!(stats.batches_created, 2);
        assert_eq!(stats.events_batched, 3);
        assert!((stats.avg_batch_size - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_event_batcher_batch_ids_increment() {
        let config = BatchConfig {
            max_batch_size: 1,
            ..Default::default()
        };
        let mut batcher = EventBatcher::new(config);

        batcher.add_event(create_test_event("e1"));
        let batch1 = batcher.flush().unwrap();

        batcher.add_event(create_test_event("e2"));
        let batch2 = batcher.flush().unwrap();

        assert_eq!(batch1.batch_id, 1);
        assert_eq!(batch2.batch_id, 2);
    }

    // ==================== EventUnbatcher Tests ====================

    #[test]
    fn test_event_unbatcher_new() {
        let unbatcher = EventUnbatcher::new();
        assert_eq!(unbatcher.stats().batches_processed, 0);
    }

    #[test]
    fn test_event_unbatcher_process_batch() {
        let mut unbatcher = EventUnbatcher::new();

        let batch = EventBatch::new(1, vec![create_test_event("e1"), create_test_event("e2")]);
        let events = unbatcher.process_batch(batch).unwrap();

        assert_eq!(events.len(), 2);
        assert_eq!(unbatcher.stats().batches_processed, 1);
        assert_eq!(unbatcher.stats().events_extracted, 2);
    }

    #[test]
    fn test_event_unbatcher_duplicate_detection() {
        let mut unbatcher = EventUnbatcher::new();

        let batch1 = EventBatch::new(1, vec![create_test_event("e1")]);
        let batch2 = EventBatch::new(1, vec![create_test_event("e1")]); // Same batch ID

        let result1 = unbatcher.process_batch(batch1);
        let result2 = unbatcher.process_batch(batch2);

        assert!(result1.is_some());
        assert!(result2.is_none());
        assert_eq!(unbatcher.stats().duplicates_ignored, 1);
    }

    #[test]
    fn test_event_unbatcher_different_batches() {
        let mut unbatcher = EventUnbatcher::new();

        let batch1 = EventBatch::new(1, vec![create_test_event("e1")]);
        let batch2 = EventBatch::new(2, vec![create_test_event("e2")]);

        let result1 = unbatcher.process_batch(batch1);
        let result2 = unbatcher.process_batch(batch2);

        assert!(result1.is_some());
        assert!(result2.is_some());
        assert_eq!(unbatcher.stats().batches_processed, 2);
    }

    #[test]
    fn test_event_unbatcher_clear_cache() {
        let mut unbatcher = EventUnbatcher::new();

        let batch = EventBatch::new(1, vec![create_test_event("e1")]);
        unbatcher.process_batch(batch);

        unbatcher.clear_cache();

        // Same batch ID should now be processed again
        let batch2 = EventBatch::new(1, vec![create_test_event("e1")]);
        let result = unbatcher.process_batch(batch2);
        assert!(result.is_some());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_batcher_unbatcher_roundtrip() {
        let config = BatchConfig {
            max_batch_size: 2,
            ..Default::default()
        };
        let mut batcher = EventBatcher::new(config);
        let mut unbatcher = EventUnbatcher::new();

        // Batch events
        batcher.add_event(create_test_event("e1"));
        batcher.add_event(create_test_event("e2"));
        let batch = batcher.flush().unwrap();

        // Serialize and deserialize (simulating network)
        let bytes = batch.to_bytes().unwrap();
        let received = EventBatch::from_bytes(&bytes).unwrap();

        // Unbatch
        let events = unbatcher.process_batch(received).unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].id, "e1");
        assert_eq!(events[1].id, "e2");
    }
}

//! Event caching and deduplication.
//!
//! This module provides LRU caching for recently synced events
//! and deduplication of incoming events.

use nostr_nations_core::events::GameEvent;
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

/// Configuration for the event cache.
#[derive(Clone, Debug)]
pub struct CacheConfig {
    /// Maximum number of events to cache.
    pub max_events: usize,
    /// Maximum age of cached events.
    pub max_age: Duration,
    /// Enable deduplication tracking.
    pub enable_dedup: bool,
    /// Maximum number of event IDs to track for deduplication.
    pub max_dedup_ids: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_events: 1000,
            max_age: Duration::from_secs(300), // 5 minutes
            enable_dedup: true,
            max_dedup_ids: 10000,
        }
    }
}

/// A cached event with metadata.
#[derive(Clone, Debug)]
pub struct CachedEvent {
    /// The event.
    pub event: GameEvent,
    /// When the event was cached.
    pub cached_at: Instant,
    /// Number of times this event was accessed.
    pub access_count: u64,
    /// Last access time.
    pub last_accessed: Instant,
}

impl CachedEvent {
    /// Create a new cached event.
    pub fn new(event: GameEvent) -> Self {
        let now = Instant::now();
        Self {
            event,
            cached_at: now,
            access_count: 0,
            last_accessed: now,
        }
    }

    /// Check if the cached event has expired.
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.cached_at.elapsed() > max_age
    }

    /// Record an access to this cached event.
    pub fn record_access(&mut self) {
        self.access_count += 1;
        self.last_accessed = Instant::now();
    }

    /// Get the age of this cached event.
    pub fn age(&self) -> Duration {
        self.cached_at.elapsed()
    }
}

/// LRU cache for game events.
pub struct EventCache {
    /// Configuration.
    config: CacheConfig,
    /// Event storage by ID.
    events: HashMap<String, CachedEvent>,
    /// LRU ordering (most recent at front).
    lru_order: VecDeque<String>,
    /// Set of seen event IDs for deduplication.
    seen_ids: HashSet<String>,
    /// Order of seen IDs for LRU eviction.
    seen_order: VecDeque<String>,
    /// Statistics.
    stats: CacheStats,
}

/// Cache statistics.
#[derive(Clone, Debug, Default)]
pub struct CacheStats {
    /// Cache hits.
    pub hits: u64,
    /// Cache misses.
    pub misses: u64,
    /// Events added.
    pub inserts: u64,
    /// Events evicted.
    pub evictions: u64,
    /// Events expired.
    pub expirations: u64,
    /// Duplicate events detected.
    pub duplicates: u64,
}

impl CacheStats {
    /// Get the hit rate as a percentage.
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64 / total as f64) * 100.0
        }
    }
}

impl EventCache {
    /// Create a new event cache.
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            events: HashMap::new(),
            lru_order: VecDeque::new(),
            seen_ids: HashSet::new(),
            seen_order: VecDeque::new(),
            stats: CacheStats::default(),
        }
    }

    /// Create with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(CacheConfig::default())
    }

    /// Insert an event into the cache.
    /// Returns true if the event was new, false if it was a duplicate.
    pub fn insert(&mut self, event: GameEvent) -> bool {
        let id = event.id.clone();

        // Check for duplicate
        if self.config.enable_dedup && self.seen_ids.contains(&id) {
            self.stats.duplicates += 1;
            return false;
        }

        // Evict if at capacity
        while self.events.len() >= self.config.max_events {
            self.evict_lru();
        }

        // Insert the event
        self.events.insert(id.clone(), CachedEvent::new(event));
        self.lru_order.push_front(id.clone());

        // Track for deduplication
        if self.config.enable_dedup {
            self.track_seen(id);
        }

        self.stats.inserts += 1;
        true
    }

    /// Get an event from the cache.
    pub fn get(&mut self, id: &str) -> Option<&GameEvent> {
        // First check if it exists and update access
        if let Some(cached) = self.events.get_mut(id) {
            cached.record_access();
            self.update_lru(id);
            self.stats.hits += 1;
            // Re-borrow as immutable
            return self.events.get(id).map(|c| &c.event);
        }

        self.stats.misses += 1;
        None
    }

    /// Get an event without updating LRU (peek).
    pub fn peek(&self, id: &str) -> Option<&GameEvent> {
        self.events.get(id).map(|c| &c.event)
    }

    /// Check if an event is in the cache.
    pub fn contains(&self, id: &str) -> bool {
        self.events.contains_key(id)
    }

    /// Check if an event has been seen (for deduplication).
    pub fn is_duplicate(&self, id: &str) -> bool {
        self.config.enable_dedup && self.seen_ids.contains(id)
    }

    /// Remove an event from the cache.
    pub fn remove(&mut self, id: &str) -> Option<GameEvent> {
        if let Some(cached) = self.events.remove(id) {
            self.lru_order.retain(|i| i != id);
            Some(cached.event)
        } else {
            None
        }
    }

    /// Clear all cached events.
    pub fn clear(&mut self) {
        self.events.clear();
        self.lru_order.clear();
    }

    /// Clear the deduplication cache.
    pub fn clear_dedup(&mut self) {
        self.seen_ids.clear();
        self.seen_order.clear();
    }

    /// Remove expired events.
    pub fn expire_old(&mut self) -> usize {
        let max_age = self.config.max_age;
        let expired: Vec<String> = self
            .events
            .iter()
            .filter(|(_, cached)| cached.is_expired(max_age))
            .map(|(id, _)| id.clone())
            .collect();

        for id in &expired {
            self.events.remove(id);
            self.lru_order.retain(|i| i != id);
        }

        self.stats.expirations += expired.len() as u64;
        expired.len()
    }

    /// Get the number of cached events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Get cache statistics.
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Get the configuration.
    pub fn config(&self) -> &CacheConfig {
        &self.config
    }

    /// Get all cached event IDs.
    pub fn event_ids(&self) -> impl Iterator<Item = &String> {
        self.events.keys()
    }

    /// Get events for a specific game.
    pub fn events_for_game(&self, game_id: &str) -> Vec<&GameEvent> {
        self.events
            .values()
            .filter(|cached| cached.event.game_id == game_id)
            .map(|cached| &cached.event)
            .collect()
    }

    /// Evict the least recently used event.
    fn evict_lru(&mut self) {
        if let Some(id) = self.lru_order.pop_back() {
            self.events.remove(&id);
            self.stats.evictions += 1;
        }
    }

    /// Update LRU order for an accessed item.
    fn update_lru(&mut self, id: &str) {
        self.lru_order.retain(|i| i != id);
        self.lru_order.push_front(id.to_string());
    }

    /// Track a seen event ID for deduplication.
    fn track_seen(&mut self, id: String) {
        if self.seen_ids.len() >= self.config.max_dedup_ids {
            // Evict oldest seen ID
            if let Some(old_id) = self.seen_order.pop_back() {
                self.seen_ids.remove(&old_id);
            }
        }

        self.seen_ids.insert(id.clone());
        self.seen_order.push_front(id);
    }
}

/// Deduplication filter for incoming events.
pub struct EventDeduplicator {
    /// Set of seen event IDs.
    seen: HashSet<String>,
    /// Order for LRU eviction.
    order: VecDeque<String>,
    /// Maximum IDs to track.
    max_ids: usize,
    /// Statistics.
    stats: DedupStats,
}

/// Deduplication statistics.
#[derive(Clone, Debug, Default)]
pub struct DedupStats {
    /// Total events checked.
    pub events_checked: u64,
    /// Duplicates detected.
    pub duplicates: u64,
    /// Unique events passed.
    pub unique: u64,
}

impl DedupStats {
    /// Get the duplicate rate as a percentage.
    pub fn duplicate_rate(&self) -> f64 {
        if self.events_checked == 0 {
            0.0
        } else {
            (self.duplicates as f64 / self.events_checked as f64) * 100.0
        }
    }
}

impl EventDeduplicator {
    /// Create a new deduplicator.
    pub fn new(max_ids: usize) -> Self {
        Self {
            seen: HashSet::new(),
            order: VecDeque::new(),
            max_ids,
            stats: DedupStats::default(),
        }
    }

    /// Check if an event is a duplicate.
    /// Returns true if duplicate, false if new.
    pub fn is_duplicate(&mut self, id: &str) -> bool {
        self.stats.events_checked += 1;

        if self.seen.contains(id) {
            self.stats.duplicates += 1;
            return true;
        }

        // Track the new ID
        if self.seen.len() >= self.max_ids {
            if let Some(old_id) = self.order.pop_back() {
                self.seen.remove(&old_id);
            }
        }

        self.seen.insert(id.to_string());
        self.order.push_front(id.to_string());
        self.stats.unique += 1;

        false
    }

    /// Check and return the event if not duplicate.
    pub fn filter(&mut self, event: GameEvent) -> Option<GameEvent> {
        if self.is_duplicate(&event.id) {
            None
        } else {
            Some(event)
        }
    }

    /// Filter a batch of events, returning only non-duplicates.
    pub fn filter_batch(&mut self, events: Vec<GameEvent>) -> Vec<GameEvent> {
        events.into_iter().filter_map(|e| self.filter(e)).collect()
    }

    /// Clear all tracked IDs.
    pub fn clear(&mut self) {
        self.seen.clear();
        self.order.clear();
    }

    /// Get statistics.
    pub fn stats(&self) -> &DedupStats {
        &self.stats
    }

    /// Get the number of tracked IDs.
    pub fn len(&self) -> usize {
        self.seen.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.seen.is_empty()
    }
}

/// Index for fast event lookup by various criteria.
pub struct EventIndex {
    /// Events by game ID.
    by_game: HashMap<String, HashSet<String>>,
    /// Events by player ID.
    by_player: HashMap<u8, HashSet<String>>,
    /// Events by turn.
    by_turn: HashMap<u32, HashSet<String>>,
}

impl EventIndex {
    /// Create a new event index.
    pub fn new() -> Self {
        Self {
            by_game: HashMap::new(),
            by_player: HashMap::new(),
            by_turn: HashMap::new(),
        }
    }

    /// Add an event to the index.
    pub fn add(&mut self, event: &GameEvent) {
        self.by_game
            .entry(event.game_id.clone())
            .or_default()
            .insert(event.id.clone());

        self.by_player
            .entry(event.player_id)
            .or_default()
            .insert(event.id.clone());

        self.by_turn
            .entry(event.turn)
            .or_default()
            .insert(event.id.clone());
    }

    /// Remove an event from the index.
    pub fn remove(&mut self, event: &GameEvent) {
        if let Some(set) = self.by_game.get_mut(&event.game_id) {
            set.remove(&event.id);
        }
        if let Some(set) = self.by_player.get_mut(&event.player_id) {
            set.remove(&event.id);
        }
        if let Some(set) = self.by_turn.get_mut(&event.turn) {
            set.remove(&event.id);
        }
    }

    /// Get event IDs for a game.
    pub fn by_game(&self, game_id: &str) -> impl Iterator<Item = &String> {
        self.by_game.get(game_id).into_iter().flatten()
    }

    /// Get event IDs for a player.
    pub fn by_player(&self, player_id: u8) -> impl Iterator<Item = &String> {
        self.by_player.get(&player_id).into_iter().flatten()
    }

    /// Get event IDs for a turn.
    pub fn by_turn(&self, turn: u32) -> impl Iterator<Item = &String> {
        self.by_turn.get(&turn).into_iter().flatten()
    }

    /// Clear the index.
    pub fn clear(&mut self) {
        self.by_game.clear();
        self.by_player.clear();
        self.by_turn.clear();
    }
}

impl Default for EventIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;

    fn create_event(id: &str, game_id: &str, player_id: u8, turn: u32) -> GameEvent {
        let mut event = GameEvent::new(
            game_id.to_string(),
            player_id,
            None,
            turn,
            1,
            GameAction::EndTurn,
        );
        event.id = id.to_string();
        event
    }

    // ==================== CacheConfig Tests ====================

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert_eq!(config.max_events, 1000);
        assert!(config.enable_dedup);
    }

    // ==================== CachedEvent Tests ====================

    #[test]
    fn test_cached_event_new() {
        let event = create_event("e1", "g1", 0, 1);
        let cached = CachedEvent::new(event);

        assert_eq!(cached.access_count, 0);
        assert!(!cached.is_expired(Duration::from_secs(60)));
    }

    #[test]
    fn test_cached_event_record_access() {
        let event = create_event("e1", "g1", 0, 1);
        let mut cached = CachedEvent::new(event);

        cached.record_access();
        assert_eq!(cached.access_count, 1);

        cached.record_access();
        assert_eq!(cached.access_count, 2);
    }

    // ==================== EventCache Tests ====================

    #[test]
    fn test_event_cache_new() {
        let cache = EventCache::with_defaults();
        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_event_cache_insert() {
        let mut cache = EventCache::with_defaults();
        let event = create_event("e1", "g1", 0, 1);

        let is_new = cache.insert(event);
        assert!(is_new);
        assert_eq!(cache.len(), 1);
    }

    #[test]
    fn test_event_cache_insert_duplicate() {
        let mut cache = EventCache::with_defaults();
        let event1 = create_event("e1", "g1", 0, 1);
        let event2 = create_event("e1", "g1", 0, 1); // Same ID

        assert!(cache.insert(event1));
        assert!(!cache.insert(event2));
        assert_eq!(cache.stats().duplicates, 1);
    }

    #[test]
    fn test_event_cache_get() {
        let mut cache = EventCache::with_defaults();
        let event = create_event("e1", "g1", 0, 1);

        cache.insert(event);

        let retrieved = cache.get("e1");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().id, "e1");
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_event_cache_get_miss() {
        let mut cache = EventCache::with_defaults();

        let retrieved = cache.get("nonexistent");
        assert!(retrieved.is_none());
        assert_eq!(cache.stats().misses, 1);
    }

    #[test]
    fn test_event_cache_contains() {
        let mut cache = EventCache::with_defaults();
        let event = create_event("e1", "g1", 0, 1);

        cache.insert(event);

        assert!(cache.contains("e1"));
        assert!(!cache.contains("e2"));
    }

    #[test]
    fn test_event_cache_remove() {
        let mut cache = EventCache::with_defaults();
        let event = create_event("e1", "g1", 0, 1);

        cache.insert(event);
        let removed = cache.remove("e1");

        assert!(removed.is_some());
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_event_cache_lru_eviction() {
        let config = CacheConfig {
            max_events: 3,
            ..Default::default()
        };
        let mut cache = EventCache::new(config);

        cache.insert(create_event("e1", "g1", 0, 1));
        cache.insert(create_event("e2", "g1", 0, 1));
        cache.insert(create_event("e3", "g1", 0, 1));

        // Access e1 to make it recently used
        cache.get("e1");

        // Insert e4, should evict e2 (oldest unused)
        cache.insert(create_event("e4", "g1", 0, 1));

        assert!(cache.contains("e1"));
        assert!(!cache.contains("e2"));
        assert!(cache.contains("e3"));
        assert!(cache.contains("e4"));
    }

    #[test]
    fn test_event_cache_clear() {
        let mut cache = EventCache::with_defaults();
        cache.insert(create_event("e1", "g1", 0, 1));
        cache.insert(create_event("e2", "g1", 0, 1));

        cache.clear();
        assert!(cache.is_empty());
    }

    #[test]
    fn test_event_cache_events_for_game() {
        let mut cache = EventCache::with_defaults();
        cache.insert(create_event("e1", "game1", 0, 1));
        cache.insert(create_event("e2", "game1", 0, 1));
        cache.insert(create_event("e3", "game2", 0, 1));

        let game1_events = cache.events_for_game("game1");
        assert_eq!(game1_events.len(), 2);

        let game2_events = cache.events_for_game("game2");
        assert_eq!(game2_events.len(), 1);
    }

    #[test]
    fn test_event_cache_stats() {
        let mut cache = EventCache::with_defaults();

        cache.insert(create_event("e1", "g1", 0, 1));
        cache.get("e1");
        cache.get("e2"); // Miss

        let stats = cache.stats();
        assert_eq!(stats.inserts, 1);
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate() - 50.0).abs() < 0.001);
    }

    // ==================== EventDeduplicator Tests ====================

    #[test]
    fn test_deduplicator_new() {
        let dedup = EventDeduplicator::new(100);
        assert!(dedup.is_empty());
    }

    #[test]
    fn test_deduplicator_is_duplicate() {
        let mut dedup = EventDeduplicator::new(100);

        assert!(!dedup.is_duplicate("e1"));
        assert!(dedup.is_duplicate("e1"));
        assert!(!dedup.is_duplicate("e2"));
    }

    #[test]
    fn test_deduplicator_filter() {
        let mut dedup = EventDeduplicator::new(100);
        let event1 = create_event("e1", "g1", 0, 1);
        let event2 = create_event("e1", "g1", 0, 1); // Same ID

        assert!(dedup.filter(event1).is_some());
        assert!(dedup.filter(event2).is_none());
    }

    #[test]
    fn test_deduplicator_filter_batch() {
        let mut dedup = EventDeduplicator::new(100);
        let events = vec![
            create_event("e1", "g1", 0, 1),
            create_event("e2", "g1", 0, 1),
            create_event("e1", "g1", 0, 1), // Duplicate
            create_event("e3", "g1", 0, 1),
        ];

        let filtered = dedup.filter_batch(events);
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_deduplicator_lru_eviction() {
        let mut dedup = EventDeduplicator::new(3);

        dedup.is_duplicate("e1");
        dedup.is_duplicate("e2");
        dedup.is_duplicate("e3");
        dedup.is_duplicate("e4"); // Should evict e1

        // e1 should no longer be tracked
        assert!(!dedup.is_duplicate("e1")); // Now new again
    }

    #[test]
    fn test_deduplicator_stats() {
        let mut dedup = EventDeduplicator::new(100);

        dedup.is_duplicate("e1");
        dedup.is_duplicate("e1");
        dedup.is_duplicate("e2");

        let stats = dedup.stats();
        assert_eq!(stats.events_checked, 3);
        assert_eq!(stats.duplicates, 1);
        assert_eq!(stats.unique, 2);
    }

    // ==================== EventIndex Tests ====================

    #[test]
    fn test_event_index_new() {
        let index = EventIndex::new();
        assert_eq!(index.by_game("g1").count(), 0);
    }

    #[test]
    fn test_event_index_add() {
        let mut index = EventIndex::new();
        let event = create_event("e1", "game1", 0, 5);

        index.add(&event);

        assert_eq!(index.by_game("game1").count(), 1);
        assert_eq!(index.by_player(0).count(), 1);
        assert_eq!(index.by_turn(5).count(), 1);
    }

    #[test]
    fn test_event_index_remove() {
        let mut index = EventIndex::new();
        let event = create_event("e1", "game1", 0, 5);

        index.add(&event);
        index.remove(&event);

        assert_eq!(index.by_game("game1").count(), 0);
    }

    #[test]
    fn test_event_index_multiple_events() {
        let mut index = EventIndex::new();

        index.add(&create_event("e1", "game1", 0, 1));
        index.add(&create_event("e2", "game1", 0, 1));
        index.add(&create_event("e3", "game1", 1, 2));
        index.add(&create_event("e4", "game2", 0, 1));

        assert_eq!(index.by_game("game1").count(), 3);
        assert_eq!(index.by_game("game2").count(), 1);
        assert_eq!(index.by_player(0).count(), 3);
        assert_eq!(index.by_player(1).count(), 1);
        assert_eq!(index.by_turn(1).count(), 3);
        assert_eq!(index.by_turn(2).count(), 1);
    }

    #[test]
    fn test_event_index_clear() {
        let mut index = EventIndex::new();
        index.add(&create_event("e1", "game1", 0, 1));

        index.clear();
        assert_eq!(index.by_game("game1").count(), 0);
    }
}

//! Subscription management for the local Nostr relay.
//!
//! Handles subscribing to events and notifying subscribers when new events arrive.

use crate::relay::filter::Filter;
use nostr_nations_core::events::GameEvent;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// Type alias for subscription callbacks.
pub type SubscriptionCallback = Box<dyn Fn(&GameEvent) + Send + Sync>;

/// A subscription to events matching a filter.
pub struct Subscription {
    /// Unique subscription ID.
    pub id: String,
    /// Filter to match events against.
    pub filter: Filter,
    /// Callback to invoke when a matching event arrives.
    callback: SubscriptionCallback,
    /// Whether the subscription is active.
    pub active: bool,
}

impl Subscription {
    /// Create a new subscription.
    pub fn new<F>(id: String, filter: Filter, callback: F) -> Self
    where
        F: Fn(&GameEvent) + Send + Sync + 'static,
    {
        Self {
            id,
            filter,
            callback: Box::new(callback),
            active: true,
        }
    }

    /// Check if an event matches this subscription's filter.
    pub fn matches(&self, event: &GameEvent) -> bool {
        self.active && self.filter.matches(event)
    }

    /// Invoke the callback with the given event.
    pub fn notify(&self, event: &GameEvent) {
        if self.active {
            (self.callback)(event);
        }
    }

    /// Deactivate the subscription.
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

impl std::fmt::Debug for Subscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subscription")
            .field("id", &self.id)
            .field("filter", &self.filter)
            .field("active", &self.active)
            .finish()
    }
}

/// Manager for active subscriptions.
///
/// Thread-safe subscription management with support for
/// adding, removing, and notifying subscriptions.
#[derive(Clone)]
pub struct SubscriptionManager {
    subscriptions: Arc<RwLock<HashMap<String, Arc<Mutex<Subscription>>>>>,
    next_id: Arc<Mutex<u64>>,
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SubscriptionManager {
    /// Create a new subscription manager.
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    /// Generate a unique subscription ID.
    fn generate_id(&self) -> String {
        let mut id = self.next_id.lock().unwrap();
        *id += 1;
        format!("sub_{}", *id)
    }

    /// Subscribe to events matching the given filter.
    ///
    /// Returns the subscription ID.
    pub fn subscribe<F>(&self, filter: Filter, callback: F) -> String
    where
        F: Fn(&GameEvent) + Send + Sync + 'static,
    {
        let sub_id = self.generate_id();
        let subscription = Subscription::new(sub_id.clone(), filter, callback);

        let mut subs = self.subscriptions.write().unwrap();
        subs.insert(sub_id.clone(), Arc::new(Mutex::new(subscription)));

        sub_id
    }

    /// Subscribe with a custom subscription ID.
    ///
    /// Returns true if the subscription was added, false if the ID already exists.
    pub fn subscribe_with_id<F>(&self, sub_id: String, filter: Filter, callback: F) -> bool
    where
        F: Fn(&GameEvent) + Send + Sync + 'static,
    {
        let mut subs = self.subscriptions.write().unwrap();

        if subs.contains_key(&sub_id) {
            return false;
        }

        let subscription = Subscription::new(sub_id.clone(), filter, callback);
        subs.insert(sub_id, Arc::new(Mutex::new(subscription)));

        true
    }

    /// Unsubscribe by subscription ID.
    ///
    /// Returns true if the subscription was removed.
    pub fn unsubscribe(&self, sub_id: &str) -> bool {
        let mut subs = self.subscriptions.write().unwrap();
        subs.remove(sub_id).is_some()
    }

    /// Notify all matching subscribers of a new event.
    ///
    /// Returns the number of subscribers that were notified.
    pub fn notify_subscribers(&self, event: &GameEvent) -> usize {
        let subs = self.subscriptions.read().unwrap();
        let mut notified = 0;

        for sub_arc in subs.values() {
            let sub = sub_arc.lock().unwrap();
            if sub.matches(event) {
                sub.notify(event);
                notified += 1;
            }
        }

        notified
    }

    /// Get the number of active subscriptions.
    pub fn subscription_count(&self) -> usize {
        let subs = self.subscriptions.read().unwrap();
        subs.values().filter(|s| s.lock().unwrap().active).count()
    }

    /// Check if a subscription exists.
    pub fn has_subscription(&self, sub_id: &str) -> bool {
        let subs = self.subscriptions.read().unwrap();
        subs.contains_key(sub_id)
    }

    /// Get the filter for a subscription.
    pub fn get_filter(&self, sub_id: &str) -> Option<Filter> {
        let subs = self.subscriptions.read().unwrap();
        subs.get(sub_id).map(|s| s.lock().unwrap().filter.clone())
    }

    /// Deactivate a subscription without removing it.
    pub fn deactivate(&self, sub_id: &str) -> bool {
        let subs = self.subscriptions.read().unwrap();
        if let Some(sub_arc) = subs.get(sub_id) {
            let mut sub = sub_arc.lock().unwrap();
            sub.deactivate();
            true
        } else {
            false
        }
    }

    /// Clear all subscriptions.
    pub fn clear(&self) {
        let mut subs = self.subscriptions.write().unwrap();
        subs.clear();
    }

    /// Get all subscription IDs.
    pub fn subscription_ids(&self) -> Vec<String> {
        let subs = self.subscriptions.read().unwrap();
        subs.keys().cloned().collect()
    }

    /// Get subscriptions that match the given event.
    pub fn matching_subscriptions(&self, event: &GameEvent) -> Vec<String> {
        let subs = self.subscriptions.read().unwrap();
        subs.iter()
            .filter(|(_, sub_arc)| {
                let sub = sub_arc.lock().unwrap();
                sub.matches(event)
            })
            .map(|(id, _)| id.clone())
            .collect()
    }
}

impl std::fmt::Debug for SubscriptionManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let subs = self.subscriptions.read().unwrap();
        f.debug_struct("SubscriptionManager")
            .field("subscription_count", &subs.len())
            .field("subscription_ids", &subs.keys().collect::<Vec<_>>())
            .finish()
    }
}

/// Builder for creating subscriptions with a fluent API.
pub struct SubscriptionBuilder {
    filter: Filter,
    id: Option<String>,
}

impl SubscriptionBuilder {
    /// Create a new subscription builder.
    pub fn new() -> Self {
        Self {
            filter: Filter::new(),
            id: None,
        }
    }

    /// Set a custom subscription ID.
    pub fn with_id(mut self, id: String) -> Self {
        self.id = Some(id);
        self
    }

    /// Filter by event IDs.
    pub fn ids(mut self, ids: Vec<String>) -> Self {
        self.filter = self.filter.with_ids(ids);
        self
    }

    /// Filter by authors.
    pub fn authors(mut self, authors: Vec<String>) -> Self {
        self.filter = self.filter.with_authors(authors);
        self
    }

    /// Filter by event kinds.
    pub fn kinds(mut self, kinds: Vec<u32>) -> Self {
        self.filter = self.filter.with_kinds(kinds);
        self
    }

    /// Filter by timestamp (since).
    pub fn since(mut self, timestamp: u64) -> Self {
        self.filter = self.filter.since(timestamp);
        self
    }

    /// Filter by timestamp (until).
    pub fn until(mut self, timestamp: u64) -> Self {
        self.filter = self.filter.until(timestamp);
        self
    }

    /// Filter by game ID.
    pub fn game(mut self, game_id: String) -> Self {
        self.filter = self.filter.with_game_id(game_id);
        self
    }

    /// Set the maximum number of events.
    pub fn limit(mut self, limit: usize) -> Self {
        self.filter = self.filter.limit(limit);
        self
    }

    /// Subscribe with the configured filter.
    pub fn subscribe<F>(self, manager: &SubscriptionManager, callback: F) -> String
    where
        F: Fn(&GameEvent) + Send + Sync + 'static,
    {
        if let Some(id) = self.id {
            manager.subscribe_with_id(id.clone(), self.filter, callback);
            id
        } else {
            manager.subscribe(self.filter, callback)
        }
    }
}

impl Default for SubscriptionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn create_test_event(id: &str, player_id: u8, game_id: &str, timestamp: u64) -> GameEvent {
        let mut event = GameEvent::new(
            game_id.to_string(),
            player_id,
            None,
            1,
            1,
            GameAction::EndTurn,
        );
        event.id = id.to_string();
        event.timestamp = timestamp;
        event
    }

    #[test]
    fn test_subscription_new() {
        let called = Arc::new(AtomicUsize::new(0));
        let called_clone = called.clone();

        let sub = Subscription::new("sub1".to_string(), Filter::new(), move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(sub.id, "sub1");
        assert!(sub.active);
    }

    #[test]
    fn test_subscription_matches() {
        let sub = Subscription::new(
            "sub1".to_string(),
            Filter::game("game1".to_string()),
            |_| {},
        );

        let event1 = create_test_event("e1", 0, "game1", 1000);
        let event2 = create_test_event("e2", 0, "game2", 1000);

        assert!(sub.matches(&event1));
        assert!(!sub.matches(&event2));
    }

    #[test]
    fn test_subscription_notify() {
        let called = Arc::new(AtomicUsize::new(0));
        let called_clone = called.clone();

        let sub = Subscription::new("sub1".to_string(), Filter::new(), move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = create_test_event("e1", 0, "game1", 1000);
        sub.notify(&event);

        assert_eq!(called.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_subscription_deactivate() {
        let called = Arc::new(AtomicUsize::new(0));
        let called_clone = called.clone();

        let mut sub = Subscription::new("sub1".to_string(), Filter::new(), move |_| {
            called_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = create_test_event("e1", 0, "game1", 1000);

        // Should notify when active
        sub.notify(&event);
        assert_eq!(called.load(Ordering::SeqCst), 1);

        // Deactivate
        sub.deactivate();
        assert!(!sub.active);

        // Should not notify when inactive
        sub.notify(&event);
        assert_eq!(called.load(Ordering::SeqCst), 1);

        // Should not match when inactive
        assert!(!sub.matches(&event));
    }

    #[test]
    fn test_manager_subscribe() {
        let manager = SubscriptionManager::new();

        let sub_id = manager.subscribe(Filter::new(), |_| {});

        assert!(manager.has_subscription(&sub_id));
        assert_eq!(manager.subscription_count(), 1);
    }

    #[test]
    fn test_manager_subscribe_with_id() {
        let manager = SubscriptionManager::new();

        let success = manager.subscribe_with_id("custom_id".to_string(), Filter::new(), |_| {});
        assert!(success);
        assert!(manager.has_subscription("custom_id"));

        // Duplicate ID should fail
        let duplicate = manager.subscribe_with_id("custom_id".to_string(), Filter::new(), |_| {});
        assert!(!duplicate);
    }

    #[test]
    fn test_manager_unsubscribe() {
        let manager = SubscriptionManager::new();

        let sub_id = manager.subscribe(Filter::new(), |_| {});
        assert!(manager.has_subscription(&sub_id));

        let removed = manager.unsubscribe(&sub_id);
        assert!(removed);
        assert!(!manager.has_subscription(&sub_id));

        // Removing again should return false
        let removed_again = manager.unsubscribe(&sub_id);
        assert!(!removed_again);
    }

    #[test]
    fn test_manager_notify_subscribers() {
        let manager = SubscriptionManager::new();

        let count1 = Arc::new(AtomicUsize::new(0));
        let count1_clone = count1.clone();

        let count2 = Arc::new(AtomicUsize::new(0));
        let count2_clone = count2.clone();

        // Subscribe to game1 events
        manager.subscribe(Filter::game("game1".to_string()), move |_| {
            count1_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Subscribe to game2 events
        manager.subscribe(Filter::game("game2".to_string()), move |_| {
            count2_clone.fetch_add(1, Ordering::SeqCst);
        });

        // Notify with game1 event
        let event = create_test_event("e1", 0, "game1", 1000);
        let notified = manager.notify_subscribers(&event);

        assert_eq!(notified, 1);
        assert_eq!(count1.load(Ordering::SeqCst), 1);
        assert_eq!(count2.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_manager_notify_multiple_matching() {
        let manager = SubscriptionManager::new();

        let total_count = Arc::new(AtomicUsize::new(0));

        for _ in 0..3 {
            let count_clone = total_count.clone();
            manager.subscribe(Filter::new(), move |_| {
                count_clone.fetch_add(1, Ordering::SeqCst);
            });
        }

        let event = create_test_event("e1", 0, "game1", 1000);
        let notified = manager.notify_subscribers(&event);

        assert_eq!(notified, 3);
        assert_eq!(total_count.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_manager_get_filter() {
        let manager = SubscriptionManager::new();

        let filter = Filter::game("game1".to_string()).with_kinds(vec![30100]);
        manager.subscribe_with_id("sub1".to_string(), filter, |_| {});

        let retrieved = manager.get_filter("sub1").unwrap();
        assert_eq!(retrieved.game_id, Some("game1".to_string()));
        assert_eq!(retrieved.kinds, Some(vec![30100]));

        // Non-existent subscription
        assert!(manager.get_filter("nonexistent").is_none());
    }

    #[test]
    fn test_manager_deactivate() {
        let manager = SubscriptionManager::new();

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        manager.subscribe_with_id("sub1".to_string(), Filter::new(), move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = create_test_event("e1", 0, "game1", 1000);

        // Should notify when active
        manager.notify_subscribers(&event);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Deactivate
        let deactivated = manager.deactivate("sub1");
        assert!(deactivated);

        // Should not notify after deactivation
        manager.notify_subscribers(&event);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Subscription still exists but is not counted as active
        assert!(manager.has_subscription("sub1"));
        assert_eq!(manager.subscription_count(), 0);
    }

    #[test]
    fn test_manager_clear() {
        let manager = SubscriptionManager::new();

        manager.subscribe(Filter::new(), |_| {});
        manager.subscribe(Filter::new(), |_| {});
        manager.subscribe(Filter::new(), |_| {});

        assert_eq!(manager.subscription_count(), 3);

        manager.clear();

        assert_eq!(manager.subscription_count(), 0);
    }

    #[test]
    fn test_manager_subscription_ids() {
        let manager = SubscriptionManager::new();

        manager.subscribe_with_id("sub1".to_string(), Filter::new(), |_| {});
        manager.subscribe_with_id("sub2".to_string(), Filter::new(), |_| {});

        let ids = manager.subscription_ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&"sub1".to_string()));
        assert!(ids.contains(&"sub2".to_string()));
    }

    #[test]
    fn test_manager_matching_subscriptions() {
        let manager = SubscriptionManager::new();

        manager.subscribe_with_id(
            "sub1".to_string(),
            Filter::game("game1".to_string()),
            |_| {},
        );
        manager.subscribe_with_id(
            "sub2".to_string(),
            Filter::game("game2".to_string()),
            |_| {},
        );
        manager.subscribe_with_id("sub3".to_string(), Filter::new(), |_| {});

        let event = create_test_event("e1", 0, "game1", 1000);
        let matching = manager.matching_subscriptions(&event);

        assert_eq!(matching.len(), 2);
        assert!(matching.contains(&"sub1".to_string()));
        assert!(matching.contains(&"sub3".to_string()));
        assert!(!matching.contains(&"sub2".to_string()));
    }

    #[test]
    fn test_manager_clone() {
        let manager1 = SubscriptionManager::new();
        manager1.subscribe_with_id("sub1".to_string(), Filter::new(), |_| {});

        let manager2 = manager1.clone();

        // Both should see the subscription
        assert!(manager1.has_subscription("sub1"));
        assert!(manager2.has_subscription("sub1"));

        // Adding to one should affect both
        manager2.subscribe_with_id("sub2".to_string(), Filter::new(), |_| {});
        assert!(manager1.has_subscription("sub2"));
    }

    #[test]
    fn test_subscription_builder() {
        let manager = SubscriptionManager::new();

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        let sub_id = SubscriptionBuilder::new()
            .with_id("custom_sub".to_string())
            .game("game1".to_string())
            .kinds(vec![30100, 30101])
            .since(1000)
            .subscribe(&manager, move |_| {
                count_clone.fetch_add(1, Ordering::SeqCst);
            });

        assert_eq!(sub_id, "custom_sub");
        assert!(manager.has_subscription("custom_sub"));

        let filter = manager.get_filter("custom_sub").unwrap();
        assert_eq!(filter.game_id, Some("game1".to_string()));
        assert_eq!(filter.kinds, Some(vec![30100, 30101]));
        assert_eq!(filter.since, Some(1000));
    }

    #[test]
    fn test_subscription_builder_auto_id() {
        let manager = SubscriptionManager::new();

        let sub_id = SubscriptionBuilder::new()
            .game("game1".to_string())
            .subscribe(&manager, |_| {});

        assert!(sub_id.starts_with("sub_"));
        assert!(manager.has_subscription(&sub_id));
    }

    #[test]
    fn test_manager_thread_safety() {
        use std::thread;

        let manager = SubscriptionManager::new();
        let count = Arc::new(AtomicUsize::new(0));

        // Spawn multiple threads to add subscriptions
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let manager = manager.clone();
                let count = count.clone();
                thread::spawn(move || {
                    let count_clone = count.clone();
                    manager.subscribe_with_id(format!("sub_{}", i), Filter::new(), move |_| {
                        count_clone.fetch_add(1, Ordering::SeqCst);
                    });
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(manager.subscription_count(), 10);

        // Notify from multiple threads
        let handles: Vec<_> = (0..5)
            .map(|_| {
                let manager = manager.clone();
                thread::spawn(move || {
                    let event = create_test_event("e1", 0, "game1", 1000);
                    manager.notify_subscribers(&event);
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }

        // Each of 5 threads notified 10 subscribers
        assert_eq!(count.load(Ordering::SeqCst), 50);
    }
}

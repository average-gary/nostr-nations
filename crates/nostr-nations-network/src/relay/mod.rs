//! Local Nostr relay implementation for offline play and event persistence.
//!
//! This module provides a local relay that stores game events in SQLite,
//! enabling offline gameplay and reliable event persistence.
//!
//! # Architecture
//!
//! The relay consists of three main components:
//!
//! - **Storage** ([`RelayStorage`]): SQLite-backed persistent storage for events
//! - **Subscriptions** ([`SubscriptionManager`]): Real-time event notifications
//! - **Filters** ([`Filter`]): NIP-01 compliant event filtering
//!
//! # Example
//!
//! ```rust,ignore
//! use nostr_nations_network::relay::{RelayStorage, SubscriptionManager, Filter};
//!
//! // Create storage
//! let storage = RelayStorage::new_in_memory()?;
//!
//! // Create subscription manager
//! let subscriptions = SubscriptionManager::new();
//!
//! // Subscribe to game events
//! let sub_id = subscriptions.subscribe(
//!     Filter::game("game123".to_string()),
//!     |event| {
//!         println!("New event: {:?}", event);
//!     }
//! );
//!
//! // Store an event (will notify subscribers)
//! storage.store_event(&event)?;
//! subscriptions.notify_subscribers(&event);
//!
//! // Query events
//! let events = storage.query_events(&Filter::new().since(1000).limit(10))?;
//! ```
//!
//! # NIP Compliance
//!
//! The filter implementation follows NIP-01 specifications:
//! - Event IDs (`ids`)
//! - Author public keys (`authors`)
//! - Event kinds (`kinds`)
//! - Timestamp ranges (`since`, `until`)
//! - Tag filters (`#e`, `#p`, etc.)
//! - Result limiting (`limit`)

pub mod filter;
pub mod storage;
pub mod subscription;

pub use filter::Filter;
pub use storage::{RelayStorage, StorageError};
pub use subscription::{Subscription, SubscriptionBuilder, SubscriptionCallback, SubscriptionManager};

/// Local relay combining storage and subscription management.
///
/// This struct provides a convenient wrapper around the storage and
/// subscription components, automatically notifying subscribers when
/// events are stored.
#[derive(Clone)]
pub struct LocalRelay {
    /// Event storage backend.
    pub storage: RelayStorage,
    /// Subscription manager for real-time notifications.
    pub subscriptions: SubscriptionManager,
}

impl LocalRelay {
    /// Create a new local relay with in-memory storage.
    pub fn new_in_memory() -> Result<Self, StorageError> {
        Ok(Self {
            storage: RelayStorage::new_in_memory()?,
            subscriptions: SubscriptionManager::new(),
        })
    }

    /// Create a new local relay with file-based storage.
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, StorageError> {
        Ok(Self {
            storage: RelayStorage::new(path)?,
            subscriptions: SubscriptionManager::new(),
        })
    }

    /// Store an event and notify matching subscribers.
    pub fn publish(&self, event: &nostr_nations_core::events::GameEvent) -> Result<usize, StorageError> {
        self.storage.store_event(event)?;
        Ok(self.subscriptions.notify_subscribers(event))
    }

    /// Subscribe to events matching the given filter.
    pub fn subscribe<F>(&self, filter: Filter, callback: F) -> String
    where
        F: Fn(&nostr_nations_core::events::GameEvent) + Send + Sync + 'static,
    {
        self.subscriptions.subscribe(filter, callback)
    }

    /// Unsubscribe by subscription ID.
    pub fn unsubscribe(&self, sub_id: &str) -> bool {
        self.subscriptions.unsubscribe(sub_id)
    }

    /// Query events from storage.
    pub fn query(&self, filter: &Filter) -> Result<Vec<nostr_nations_core::events::GameEvent>, StorageError> {
        self.storage.query_events(filter)
    }

    /// Get an event by ID.
    pub fn get_event(&self, id: &str) -> Result<nostr_nations_core::events::GameEvent, StorageError> {
        self.storage.get_event(id)
    }

    /// Delete an event by ID.
    pub fn delete_event(&self, id: &str) -> Result<bool, StorageError> {
        self.storage.delete_event(id)
    }

    /// Get the number of stored events.
    pub fn event_count(&self) -> Result<usize, StorageError> {
        self.storage.event_count()
    }

    /// Get the number of active subscriptions.
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.subscription_count()
    }
}

impl std::fmt::Debug for LocalRelay {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalRelay")
            .field("event_count", &self.storage.event_count().unwrap_or(0))
            .field("subscription_count", &self.subscriptions.subscription_count())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::{GameAction, GameEvent};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn create_test_event(id: &str, game_id: &str, timestamp: u64) -> GameEvent {
        let mut event = GameEvent::new(
            game_id.to_string(),
            0,
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
    fn test_local_relay_new_in_memory() {
        let relay = LocalRelay::new_in_memory();
        assert!(relay.is_ok());
    }

    #[test]
    fn test_local_relay_publish_and_notify() {
        let relay = LocalRelay::new_in_memory().unwrap();

        let count = Arc::new(AtomicUsize::new(0));
        let count_clone = count.clone();

        relay.subscribe(Filter::new(), move |_| {
            count_clone.fetch_add(1, Ordering::SeqCst);
        });

        let event = create_test_event("event1", "game1", 1000);
        let notified = relay.publish(&event).unwrap();

        assert_eq!(notified, 1);
        assert_eq!(count.load(Ordering::SeqCst), 1);

        // Event should be stored
        assert_eq!(relay.event_count().unwrap(), 1);
    }

    #[test]
    fn test_local_relay_subscribe_unsubscribe() {
        let relay = LocalRelay::new_in_memory().unwrap();

        let sub_id = relay.subscribe(Filter::new(), |_| {});
        assert_eq!(relay.subscription_count(), 1);

        let removed = relay.unsubscribe(&sub_id);
        assert!(removed);
        assert_eq!(relay.subscription_count(), 0);
    }

    #[test]
    fn test_local_relay_query() {
        let relay = LocalRelay::new_in_memory().unwrap();

        relay.publish(&create_test_event("event1", "game1", 1000)).unwrap();
        relay.publish(&create_test_event("event2", "game1", 2000)).unwrap();
        relay.publish(&create_test_event("event3", "game2", 3000)).unwrap();

        let events = relay.query(&Filter::game("game1".to_string())).unwrap();
        assert_eq!(events.len(), 2);
    }

    #[test]
    fn test_local_relay_get_event() {
        let relay = LocalRelay::new_in_memory().unwrap();

        relay.publish(&create_test_event("event1", "game1", 1000)).unwrap();

        let event = relay.get_event("event1").unwrap();
        assert_eq!(event.id, "event1");
    }

    #[test]
    fn test_local_relay_delete_event() {
        let relay = LocalRelay::new_in_memory().unwrap();

        relay.publish(&create_test_event("event1", "game1", 1000)).unwrap();
        assert_eq!(relay.event_count().unwrap(), 1);

        let deleted = relay.delete_event("event1").unwrap();
        assert!(deleted);
        assert_eq!(relay.event_count().unwrap(), 0);
    }

    #[test]
    fn test_local_relay_filtered_subscription() {
        let relay = LocalRelay::new_in_memory().unwrap();

        let game1_count = Arc::new(AtomicUsize::new(0));
        let game1_clone = game1_count.clone();

        let game2_count = Arc::new(AtomicUsize::new(0));
        let game2_clone = game2_count.clone();

        relay.subscribe(Filter::game("game1".to_string()), move |_| {
            game1_clone.fetch_add(1, Ordering::SeqCst);
        });

        relay.subscribe(Filter::game("game2".to_string()), move |_| {
            game2_clone.fetch_add(1, Ordering::SeqCst);
        });

        relay.publish(&create_test_event("e1", "game1", 1000)).unwrap();
        relay.publish(&create_test_event("e2", "game1", 2000)).unwrap();
        relay.publish(&create_test_event("e3", "game2", 3000)).unwrap();

        assert_eq!(game1_count.load(Ordering::SeqCst), 2);
        assert_eq!(game2_count.load(Ordering::SeqCst), 1);
    }
}

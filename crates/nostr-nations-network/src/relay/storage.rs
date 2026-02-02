//! SQLite storage backend for the local Nostr relay.
//!
//! Provides persistent storage for game events with NIP-01 compliant querying.

use crate::relay::filter::Filter;
use nostr_nations_core::events::GameEvent;
use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// SQLite-based storage for Nostr events.
///
/// Thread-safe wrapper around SQLite connection with methods
/// for storing and querying game events.
#[derive(Clone)]
pub struct RelayStorage {
    conn: Arc<Mutex<Connection>>,
}

/// Storage error types.
#[derive(Debug)]
pub enum StorageError {
    /// SQLite error.
    Sqlite(rusqlite::Error),
    /// Event not found.
    NotFound(String),
    /// Serialization error.
    Serialization(String),
    /// Lock error.
    LockError(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::Sqlite(e) => write!(f, "SQLite error: {}", e),
            StorageError::NotFound(id) => write!(f, "Event not found: {}", id),
            StorageError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::LockError(msg) => write!(f, "Lock error: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::Sqlite(e) => Some(e),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StorageError {
    fn from(err: rusqlite::Error) -> Self {
        StorageError::Sqlite(err)
    }
}

impl RelayStorage {
    /// Create a new storage instance with an in-memory database.
    pub fn new_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_db()?;
        Ok(storage)
    }

    /// Create a new storage instance with a file-based database.
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        let storage = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        storage.init_db()?;
        Ok(storage)
    }

    /// Initialize the database schema.
    pub fn init_db(&self) -> Result<(), StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        // Events table - stores the full serialized event
        conn.execute(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                pubkey TEXT NOT NULL,
                kind INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                content TEXT NOT NULL,
                sig TEXT,
                game_id TEXT,
                raw_event TEXT NOT NULL
            )",
            [],
        )?;

        // Tags table - for efficient tag-based queries
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                event_id TEXT NOT NULL,
                tag_name TEXT NOT NULL,
                tag_value TEXT NOT NULL,
                FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE CASCADE
            )",
            [],
        )?;

        // Subscriptions table - stores active subscription filters
        conn.execute(
            "CREATE TABLE IF NOT EXISTS subscriptions (
                id TEXT PRIMARY KEY,
                filter_json TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Create indexes for common queries
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_kind ON events(kind)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_pubkey ON events(pubkey)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_created_at ON events(created_at)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_events_game_id ON events(game_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_event_id ON tags(event_id)",
            [],
        )?;
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_tags_name_value ON tags(tag_name, tag_value)",
            [],
        )?;

        Ok(())
    }

    /// Store an event in the database.
    pub fn store_event(&self, event: &GameEvent) -> Result<(), StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let raw_event =
            serde_json::to_string(event).map_err(|e| StorageError::Serialization(e.to_string()))?;

        let content = event.content();
        let kind = event.kind();
        let pubkey = event.player_id.to_string();

        // Insert the event
        conn.execute(
            "INSERT OR REPLACE INTO events (id, pubkey, kind, created_at, content, game_id, raw_event)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                event.id,
                pubkey,
                kind,
                event.timestamp,
                content,
                event.game_id,
                raw_event
            ],
        )?;

        // Delete old tags for this event (in case of update)
        conn.execute("DELETE FROM tags WHERE event_id = ?1", params![event.id])?;

        // Insert tags
        let tags = event.tags();
        for tag in &tags {
            if tag.len() >= 2 {
                conn.execute(
                    "INSERT INTO tags (event_id, tag_name, tag_value) VALUES (?1, ?2, ?3)",
                    params![event.id, tag[0], tag[1]],
                )?;
            }
        }

        Ok(())
    }

    /// Retrieve an event by ID.
    pub fn get_event(&self, id: &str) -> Result<GameEvent, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let raw_event: String = conn
            .query_row(
                "SELECT raw_event FROM events WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StorageError::NotFound(id.to_string()),
                _ => StorageError::Sqlite(e),
            })?;

        serde_json::from_str(&raw_event).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    /// Query events using a NIP-01 filter.
    pub fn query_events(&self, filter: &Filter) -> Result<Vec<GameEvent>, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let mut sql = String::from("SELECT DISTINCT e.raw_event FROM events e");
        let mut conditions: Vec<String> = Vec::new();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        // Handle tag filters by joining with tags table
        let mut tag_join_idx = 0;
        if let Some(ref tags) = filter.tags {
            for (tag_name, tag_values) in tags {
                let tag_key = tag_name.trim_start_matches('#');
                let alias = format!("t{}", tag_join_idx);
                sql.push_str(&format!(
                    " INNER JOIN tags {} ON e.id = {}.event_id",
                    alias, alias
                ));
                conditions.push(format!("{}.tag_name = ?", alias));
                params_vec.push(Box::new(tag_key.to_string()));

                if !tag_values.is_empty() {
                    let placeholders: Vec<String> =
                        tag_values.iter().map(|_| "?".to_string()).collect();
                    conditions.push(format!(
                        "{}.tag_value IN ({})",
                        alias,
                        placeholders.join(", ")
                    ));
                    for v in tag_values {
                        params_vec.push(Box::new(v.clone()));
                    }
                }
                tag_join_idx += 1;
            }
        }

        // Build WHERE clause
        if let Some(ref ids) = filter.ids {
            if !ids.is_empty() {
                let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("e.id IN ({})", placeholders.join(", ")));
                for id in ids {
                    params_vec.push(Box::new(id.clone()));
                }
            }
        }

        if let Some(ref authors) = filter.authors {
            if !authors.is_empty() {
                let placeholders: Vec<String> = authors.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("e.pubkey IN ({})", placeholders.join(", ")));
                for author in authors {
                    params_vec.push(Box::new(author.clone()));
                }
            }
        }

        if let Some(ref kinds) = filter.kinds {
            if !kinds.is_empty() {
                let placeholders: Vec<String> = kinds.iter().map(|_| "?".to_string()).collect();
                conditions.push(format!("e.kind IN ({})", placeholders.join(", ")));
                for kind in kinds {
                    params_vec.push(Box::new(*kind as i64));
                }
            }
        }

        if let Some(since) = filter.since {
            conditions.push("e.created_at >= ?".to_string());
            params_vec.push(Box::new(since as i64));
        }

        if let Some(until) = filter.until {
            conditions.push("e.created_at <= ?".to_string());
            params_vec.push(Box::new(until as i64));
        }

        if let Some(ref game_id) = filter.game_id {
            conditions.push("e.game_id = ?".to_string());
            params_vec.push(Box::new(game_id.clone()));
        }

        if !conditions.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&conditions.join(" AND "));
        }

        // Order by created_at descending (newest first)
        sql.push_str(" ORDER BY e.created_at DESC");

        // Apply limit
        if let Some(limit) = filter.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        // Execute query
        let params_refs: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params_refs.as_slice(), |row| {
            let raw_event: String = row.get(0)?;
            Ok(raw_event)
        })?;

        let mut events = Vec::new();
        for row in rows {
            let raw_event = row?;
            if let Ok(event) = serde_json::from_str::<GameEvent>(&raw_event) {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Delete an event by ID.
    pub fn delete_event(&self, id: &str) -> Result<bool, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        // Tags will be deleted automatically due to ON DELETE CASCADE
        let rows_affected = conn.execute("DELETE FROM events WHERE id = ?1", params![id])?;

        Ok(rows_affected > 0)
    }

    /// Get the number of stored events.
    pub fn event_count(&self) -> Result<usize, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let count: i64 = conn.query_row("SELECT COUNT(*) FROM events", [], |row| row.get(0))?;

        Ok(count as usize)
    }

    /// Get all events for a specific game.
    pub fn get_game_events(&self, game_id: &str) -> Result<Vec<GameEvent>, StorageError> {
        self.query_events(&Filter::game(game_id.to_string()))
    }

    /// Delete all events for a specific game.
    pub fn delete_game_events(&self, game_id: &str) -> Result<usize, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let rows_affected =
            conn.execute("DELETE FROM events WHERE game_id = ?1", params![game_id])?;

        Ok(rows_affected)
    }

    /// Clear all events from the database.
    pub fn clear(&self) -> Result<(), StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        conn.execute("DELETE FROM events", [])?;
        conn.execute("DELETE FROM tags", [])?;

        Ok(())
    }

    /// Store a subscription filter.
    pub fn store_subscription(&self, sub_id: &str, filter: &Filter) -> Result<(), StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let filter_json = serde_json::to_string(filter)
            .map_err(|e| StorageError::Serialization(e.to_string()))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        conn.execute(
            "INSERT OR REPLACE INTO subscriptions (id, filter_json, created_at) VALUES (?1, ?2, ?3)",
            params![sub_id, filter_json, now as i64],
        )?;

        Ok(())
    }

    /// Get a stored subscription filter.
    pub fn get_subscription(&self, sub_id: &str) -> Result<Filter, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let filter_json: String = conn
            .query_row(
                "SELECT filter_json FROM subscriptions WHERE id = ?1",
                params![sub_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StorageError::NotFound(sub_id.to_string()),
                _ => StorageError::Sqlite(e),
            })?;

        serde_json::from_str(&filter_json).map_err(|e| StorageError::Serialization(e.to_string()))
    }

    /// Delete a subscription.
    pub fn delete_subscription(&self, sub_id: &str) -> Result<bool, StorageError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| StorageError::LockError(e.to_string()))?;

        let rows_affected =
            conn.execute("DELETE FROM subscriptions WHERE id = ?1", params![sub_id])?;

        Ok(rows_affected > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;

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
    fn test_storage_new_in_memory() {
        let storage = RelayStorage::new_in_memory();
        assert!(storage.is_ok());
    }

    #[test]
    fn test_store_and_get_event() {
        let storage = RelayStorage::new_in_memory().unwrap();
        let event = create_test_event("event1", 0, "game1", 1000);

        storage.store_event(&event).unwrap();

        let retrieved = storage.get_event("event1").unwrap();
        assert_eq!(retrieved.id, "event1");
        assert_eq!(retrieved.game_id, "game1");
    }

    #[test]
    fn test_get_nonexistent_event() {
        let storage = RelayStorage::new_in_memory().unwrap();

        let result = storage.get_event("nonexistent");
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[test]
    fn test_delete_event() {
        let storage = RelayStorage::new_in_memory().unwrap();
        let event = create_test_event("event1", 0, "game1", 1000);

        storage.store_event(&event).unwrap();
        assert_eq!(storage.event_count().unwrap(), 1);

        let deleted = storage.delete_event("event1").unwrap();
        assert!(deleted);
        assert_eq!(storage.event_count().unwrap(), 0);
    }

    #[test]
    fn test_delete_nonexistent_event() {
        let storage = RelayStorage::new_in_memory().unwrap();

        let deleted = storage.delete_event("nonexistent").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_query_events_by_ids() {
        let storage = RelayStorage::new_in_memory().unwrap();

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        storage
            .store_event(&create_test_event("event2", 0, "game1", 1001))
            .unwrap();
        storage
            .store_event(&create_test_event("event3", 0, "game1", 1002))
            .unwrap();

        let filter = Filter::ids(vec!["event1".to_string(), "event2".to_string()]);
        let events = storage.query_events(&filter).unwrap();

        assert_eq!(events.len(), 2);
        let ids: Vec<&str> = events.iter().map(|e| e.id.as_str()).collect();
        assert!(ids.contains(&"event1"));
        assert!(ids.contains(&"event2"));
    }

    #[test]
    fn test_query_events_by_kinds() {
        let storage = RelayStorage::new_in_memory().unwrap();

        let mut event1 = create_test_event("event1", 0, "game1", 1000);
        event1.action = GameAction::StartGame;

        let event2 = create_test_event("event2", 0, "game1", 1001);

        storage.store_event(&event1).unwrap();
        storage.store_event(&event2).unwrap();

        // Query for StartGame (kind 30102)
        let filter = Filter::kinds(vec![30102]);
        let events = storage.query_events(&filter).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "event1");
    }

    #[test]
    fn test_query_events_by_timestamp_range() {
        let storage = RelayStorage::new_in_memory().unwrap();

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        storage
            .store_event(&create_test_event("event2", 0, "game1", 2000))
            .unwrap();
        storage
            .store_event(&create_test_event("event3", 0, "game1", 3000))
            .unwrap();

        let filter = Filter::new().since(1500).until(2500);
        let events = storage.query_events(&filter).unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "event2");
    }

    #[test]
    fn test_query_events_by_game_id() {
        let storage = RelayStorage::new_in_memory().unwrap();

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        storage
            .store_event(&create_test_event("event2", 0, "game2", 1001))
            .unwrap();
        storage
            .store_event(&create_test_event("event3", 0, "game1", 1002))
            .unwrap();

        let filter = Filter::game("game1".to_string());
        let events = storage.query_events(&filter).unwrap();

        assert_eq!(events.len(), 2);
        assert!(events.iter().all(|e| e.game_id == "game1"));
    }

    #[test]
    fn test_query_events_with_limit() {
        let storage = RelayStorage::new_in_memory().unwrap();

        for i in 0..10 {
            storage
                .store_event(&create_test_event(
                    &format!("event{}", i),
                    0,
                    "game1",
                    1000 + i,
                ))
                .unwrap();
        }

        let filter = Filter::new().limit(5);
        let events = storage.query_events(&filter).unwrap();

        assert_eq!(events.len(), 5);
    }

    #[test]
    fn test_query_events_order_by_timestamp_desc() {
        let storage = RelayStorage::new_in_memory().unwrap();

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        storage
            .store_event(&create_test_event("event2", 0, "game1", 3000))
            .unwrap();
        storage
            .store_event(&create_test_event("event3", 0, "game1", 2000))
            .unwrap();

        let filter = Filter::new();
        let events = storage.query_events(&filter).unwrap();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].id, "event2"); // Newest first
        assert_eq!(events[1].id, "event3");
        assert_eq!(events[2].id, "event1");
    }

    #[test]
    fn test_get_game_events() {
        let storage = RelayStorage::new_in_memory().unwrap();

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        storage
            .store_event(&create_test_event("event2", 0, "game2", 1001))
            .unwrap();

        let events = storage.get_game_events("game1").unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].game_id, "game1");
    }

    #[test]
    fn test_delete_game_events() {
        let storage = RelayStorage::new_in_memory().unwrap();

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        storage
            .store_event(&create_test_event("event2", 0, "game1", 1001))
            .unwrap();
        storage
            .store_event(&create_test_event("event3", 0, "game2", 1002))
            .unwrap();

        let deleted = storage.delete_game_events("game1").unwrap();
        assert_eq!(deleted, 2);

        assert_eq!(storage.event_count().unwrap(), 1);
    }

    #[test]
    fn test_clear() {
        let storage = RelayStorage::new_in_memory().unwrap();

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        storage
            .store_event(&create_test_event("event2", 0, "game1", 1001))
            .unwrap();

        assert_eq!(storage.event_count().unwrap(), 2);

        storage.clear().unwrap();

        assert_eq!(storage.event_count().unwrap(), 0);
    }

    #[test]
    fn test_subscription_storage() {
        let storage = RelayStorage::new_in_memory().unwrap();

        let filter = Filter::new()
            .with_kinds(vec![30100, 30101])
            .with_game_id("game1".to_string());

        storage.store_subscription("sub1", &filter).unwrap();

        let retrieved = storage.get_subscription("sub1").unwrap();
        assert_eq!(retrieved.kinds, Some(vec![30100, 30101]));
        assert_eq!(retrieved.game_id, Some("game1".to_string()));
    }

    #[test]
    fn test_delete_subscription() {
        let storage = RelayStorage::new_in_memory().unwrap();

        let filter = Filter::new();
        storage.store_subscription("sub1", &filter).unwrap();

        let deleted = storage.delete_subscription("sub1").unwrap();
        assert!(deleted);

        let result = storage.get_subscription("sub1");
        assert!(matches!(result, Err(StorageError::NotFound(_))));
    }

    #[test]
    fn test_event_count() {
        let storage = RelayStorage::new_in_memory().unwrap();

        assert_eq!(storage.event_count().unwrap(), 0);

        storage
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();
        assert_eq!(storage.event_count().unwrap(), 1);

        storage
            .store_event(&create_test_event("event2", 0, "game1", 1001))
            .unwrap();
        assert_eq!(storage.event_count().unwrap(), 2);
    }

    #[test]
    fn test_update_event() {
        let storage = RelayStorage::new_in_memory().unwrap();

        let mut event = create_test_event("event1", 0, "game1", 1000);
        storage.store_event(&event).unwrap();

        // Update the event
        event.timestamp = 2000;
        storage.store_event(&event).unwrap();

        // Should still have only one event
        assert_eq!(storage.event_count().unwrap(), 1);

        let retrieved = storage.get_event("event1").unwrap();
        assert_eq!(retrieved.timestamp, 2000);
    }

    #[test]
    fn test_storage_clone() {
        let storage1 = RelayStorage::new_in_memory().unwrap();
        let storage2 = storage1.clone();

        storage1
            .store_event(&create_test_event("event1", 0, "game1", 1000))
            .unwrap();

        // Both should see the event (same underlying connection)
        assert_eq!(storage1.event_count().unwrap(), 1);
        assert_eq!(storage2.event_count().unwrap(), 1);
    }
}

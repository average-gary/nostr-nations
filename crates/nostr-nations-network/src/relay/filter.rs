//! NIP-01 filter implementation for querying Nostr events.
//!
//! Filters allow clients to request events matching specific criteria.
//! See: https://github.com/nostr-protocol/nips/blob/master/01.md

use nostr_nations_core::events::GameEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// NIP-01 compliant filter for querying events.
///
/// All conditions are AND'd together. Within arrays, conditions are OR'd.
/// For example, `kinds: [1, 2]` matches events with kind 1 OR kind 2.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Filter {
    /// Event IDs to match (OR'd).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,

    /// Author public keys to match (OR'd).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,

    /// Event kinds to match (OR'd).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kinds: Option<Vec<u32>>,

    /// Events created after this timestamp (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,

    /// Events created before this timestamp (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until: Option<u64>,

    /// Maximum number of events to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,

    /// Tag filters. Key is the tag name (without #), value is list of values to match.
    /// For example, `#e: ["eventid1", "eventid2"]` matches events with e tag containing either value.
    #[serde(flatten)]
    pub tags: Option<std::collections::HashMap<String, Vec<String>>>,

    /// Game ID filter (custom extension for Nostr Nations).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub game_id: Option<String>,
}

impl Filter {
    /// Create a new empty filter that matches all events.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a filter for specific event IDs.
    pub fn ids(ids: Vec<String>) -> Self {
        Self {
            ids: Some(ids),
            ..Default::default()
        }
    }

    /// Create a filter for events by specific authors.
    pub fn authors(authors: Vec<String>) -> Self {
        Self {
            authors: Some(authors),
            ..Default::default()
        }
    }

    /// Create a filter for specific event kinds.
    pub fn kinds(kinds: Vec<u32>) -> Self {
        Self {
            kinds: Some(kinds),
            ..Default::default()
        }
    }

    /// Create a filter for a specific game.
    pub fn game(game_id: String) -> Self {
        Self {
            game_id: Some(game_id),
            ..Default::default()
        }
    }

    /// Add ID filter.
    pub fn with_ids(mut self, ids: Vec<String>) -> Self {
        self.ids = Some(ids);
        self
    }

    /// Add author filter.
    pub fn with_authors(mut self, authors: Vec<String>) -> Self {
        self.authors = Some(authors);
        self
    }

    /// Add kind filter.
    pub fn with_kinds(mut self, kinds: Vec<u32>) -> Self {
        self.kinds = Some(kinds);
        self
    }

    /// Add since timestamp filter.
    pub fn since(mut self, timestamp: u64) -> Self {
        self.since = Some(timestamp);
        self
    }

    /// Add until timestamp filter.
    pub fn until(mut self, timestamp: u64) -> Self {
        self.until = Some(timestamp);
        self
    }

    /// Add limit.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Add a tag filter.
    pub fn with_tag(mut self, tag_name: &str, values: Vec<String>) -> Self {
        let tags = self.tags.get_or_insert_with(Default::default);
        tags.insert(format!("#{}", tag_name), values);
        self
    }

    /// Add game ID filter.
    pub fn with_game_id(mut self, game_id: String) -> Self {
        self.game_id = Some(game_id);
        self
    }

    /// Check if an event matches this filter.
    pub fn matches(&self, event: &GameEvent) -> bool {
        // Check ID filter
        if let Some(ref ids) = self.ids {
            if !ids.contains(&event.id) {
                return false;
            }
        }

        // Check author filter (using player_id as string)
        if let Some(ref authors) = self.authors {
            let player_str = event.player_id.to_string();
            if !authors.contains(&player_str) {
                return false;
            }
        }

        // Check kind filter
        if let Some(ref kinds) = self.kinds {
            if !kinds.contains(&event.kind()) {
                return false;
            }
        }

        // Check since filter
        if let Some(since) = self.since {
            if event.timestamp < since {
                return false;
            }
        }

        // Check until filter
        if let Some(until) = self.until {
            if event.timestamp > until {
                return false;
            }
        }

        // Check game ID filter
        if let Some(ref game_id) = self.game_id {
            if &event.game_id != game_id {
                return false;
            }
        }

        // Check tag filters
        if let Some(ref tag_filters) = self.tags {
            let event_tags = event.tags();
            for (tag_name, filter_values) in tag_filters {
                // Remove # prefix if present
                let tag_key = tag_name.trim_start_matches('#');

                // Find matching tags in the event
                let matching_values: HashSet<&str> = event_tags
                    .iter()
                    .filter(|t| !t.is_empty() && t[0] == tag_key)
                    .filter_map(|t| t.get(1).map(|s| s.as_str()))
                    .collect();

                // Check if any filter value matches
                let has_match = filter_values
                    .iter()
                    .any(|v| matching_values.contains(v.as_str()));
                if !has_match {
                    return false;
                }
            }
        }

        true
    }

    /// Check if this filter matches a raw Nostr-like event.
    /// This is useful for matching events that haven't been deserialized into GameEvent.
    pub fn matches_raw(
        &self,
        id: &str,
        pubkey: &str,
        kind: u32,
        created_at: u64,
        tags: &[Vec<String>],
    ) -> bool {
        // Check ID filter
        if let Some(ref ids) = self.ids {
            if !ids.iter().any(|filter_id| id.starts_with(filter_id)) {
                return false;
            }
        }

        // Check author filter
        if let Some(ref authors) = self.authors {
            if !authors.iter().any(|author| pubkey.starts_with(author)) {
                return false;
            }
        }

        // Check kind filter
        if let Some(ref kinds) = self.kinds {
            if !kinds.contains(&kind) {
                return false;
            }
        }

        // Check since filter
        if let Some(since) = self.since {
            if created_at < since {
                return false;
            }
        }

        // Check until filter
        if let Some(until) = self.until {
            if created_at > until {
                return false;
            }
        }

        // Check tag filters
        if let Some(ref tag_filters) = self.tags {
            for (tag_name, filter_values) in tag_filters {
                let tag_key = tag_name.trim_start_matches('#');

                let matching_values: HashSet<&str> = tags
                    .iter()
                    .filter(|t| !t.is_empty() && t[0] == tag_key)
                    .filter_map(|t| t.get(1).map(|s| s.as_str()))
                    .collect();

                let has_match = filter_values
                    .iter()
                    .any(|v| matching_values.contains(v.as_str()));
                if !has_match {
                    return false;
                }
            }
        }

        true
    }

    /// Check if the filter is empty (matches everything).
    pub fn is_empty(&self) -> bool {
        self.ids.is_none()
            && self.authors.is_none()
            && self.kinds.is_none()
            && self.since.is_none()
            && self.until.is_none()
            && self.tags.is_none()
            && self.game_id.is_none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::{GameAction, GameEvent};

    fn create_test_event(
        id: &str,
        player_id: u8,
        kind: u32,
        timestamp: u64,
        game_id: &str,
    ) -> GameEvent {
        let action = match kind {
            30100 => GameAction::CreateGame {
                settings_json: "{}".to_string(),
                seed: [0; 32],
            },
            30101 => GameAction::JoinGame {
                player_name: "Test".to_string(),
                civilization_id: "civ1".to_string(),
            },
            30102 => GameAction::StartGame,
            30104 => GameAction::EndTurn,
            _ => GameAction::EndTurn,
        };

        let mut event = GameEvent::new(game_id.to_string(), player_id, None, 1, 1, action);
        event.id = id.to_string();
        event.timestamp = timestamp;
        event
    }

    #[test]
    fn test_filter_new() {
        let filter = Filter::new();
        assert!(filter.is_empty());
    }

    #[test]
    fn test_filter_ids() {
        let filter = Filter::ids(vec!["event1".to_string(), "event2".to_string()]);

        let event1 = create_test_event("event1", 0, 30104, 1000, "game1");
        let event2 = create_test_event("event2", 0, 30104, 1000, "game1");
        let event3 = create_test_event("event3", 0, 30104, 1000, "game1");

        assert!(filter.matches(&event1));
        assert!(filter.matches(&event2));
        assert!(!filter.matches(&event3));
    }

    #[test]
    fn test_filter_authors() {
        let filter = Filter::authors(vec!["0".to_string(), "1".to_string()]);

        let event1 = create_test_event("event1", 0, 30104, 1000, "game1");
        let event2 = create_test_event("event2", 1, 30104, 1000, "game1");
        let event3 = create_test_event("event3", 2, 30104, 1000, "game1");

        assert!(filter.matches(&event1));
        assert!(filter.matches(&event2));
        assert!(!filter.matches(&event3));
    }

    #[test]
    fn test_filter_kinds() {
        let filter = Filter::kinds(vec![30100, 30102]);

        let event1 = create_test_event("event1", 0, 30100, 1000, "game1");
        let event2 = create_test_event("event2", 0, 30102, 1000, "game1");
        let event3 = create_test_event("event3", 0, 30104, 1000, "game1");

        assert!(filter.matches(&event1));
        assert!(filter.matches(&event2));
        assert!(!filter.matches(&event3));
    }

    #[test]
    fn test_filter_since() {
        let filter = Filter::new().since(1500);

        let event1 = create_test_event("event1", 0, 30104, 1000, "game1");
        let event2 = create_test_event("event2", 0, 30104, 1500, "game1");
        let event3 = create_test_event("event3", 0, 30104, 2000, "game1");

        assert!(!filter.matches(&event1));
        assert!(filter.matches(&event2));
        assert!(filter.matches(&event3));
    }

    #[test]
    fn test_filter_until() {
        let filter = Filter::new().until(1500);

        let event1 = create_test_event("event1", 0, 30104, 1000, "game1");
        let event2 = create_test_event("event2", 0, 30104, 1500, "game1");
        let event3 = create_test_event("event3", 0, 30104, 2000, "game1");

        assert!(filter.matches(&event1));
        assert!(filter.matches(&event2));
        assert!(!filter.matches(&event3));
    }

    #[test]
    fn test_filter_since_until_range() {
        let filter = Filter::new().since(1000).until(2000);

        let event1 = create_test_event("event1", 0, 30104, 500, "game1");
        let event2 = create_test_event("event2", 0, 30104, 1500, "game1");
        let event3 = create_test_event("event3", 0, 30104, 2500, "game1");

        assert!(!filter.matches(&event1));
        assert!(filter.matches(&event2));
        assert!(!filter.matches(&event3));
    }

    #[test]
    fn test_filter_game_id() {
        let filter = Filter::game("game1".to_string());

        let event1 = create_test_event("event1", 0, 30104, 1000, "game1");
        let event2 = create_test_event("event2", 0, 30104, 1000, "game2");

        assert!(filter.matches(&event1));
        assert!(!filter.matches(&event2));
    }

    #[test]
    fn test_filter_combined() {
        let filter = Filter::new()
            .with_kinds(vec![30104])
            .with_game_id("game1".to_string())
            .since(1000)
            .until(2000);

        // Matches all criteria
        let event1 = create_test_event("event1", 0, 30104, 1500, "game1");
        assert!(filter.matches(&event1));

        // Wrong kind
        let event2 = create_test_event("event2", 0, 30100, 1500, "game1");
        assert!(!filter.matches(&event2));

        // Wrong game
        let event3 = create_test_event("event3", 0, 30104, 1500, "game2");
        assert!(!filter.matches(&event3));

        // Wrong timestamp
        let event4 = create_test_event("event4", 0, 30104, 500, "game1");
        assert!(!filter.matches(&event4));
    }

    #[test]
    fn test_filter_builder_pattern() {
        let filter = Filter::new()
            .with_ids(vec!["id1".to_string()])
            .with_authors(vec!["author1".to_string()])
            .with_kinds(vec![30100])
            .since(1000)
            .until(2000)
            .limit(10);

        assert!(filter.ids.is_some());
        assert!(filter.authors.is_some());
        assert!(filter.kinds.is_some());
        assert_eq!(filter.since, Some(1000));
        assert_eq!(filter.until, Some(2000));
        assert_eq!(filter.limit, Some(10));
    }

    #[test]
    fn test_filter_matches_raw() {
        let filter = Filter::kinds(vec![30100, 30101]);

        assert!(filter.matches_raw("id1", "pubkey1", 30100, 1000, &[]));
        assert!(filter.matches_raw("id2", "pubkey2", 30101, 1000, &[]));
        assert!(!filter.matches_raw("id3", "pubkey3", 30102, 1000, &[]));
    }

    #[test]
    fn test_filter_matches_raw_with_tags() {
        let filter = Filter::new().with_tag("g", vec!["game123".to_string()]);

        let tags_match = vec![
            vec!["g".to_string(), "game123".to_string()],
            vec!["p".to_string(), "player1".to_string()],
        ];
        let tags_no_match = vec![vec!["g".to_string(), "game456".to_string()]];

        assert!(filter.matches_raw("id1", "pk1", 30100, 1000, &tags_match));
        assert!(!filter.matches_raw("id2", "pk2", 30100, 1000, &tags_no_match));
    }

    #[test]
    fn test_filter_is_empty() {
        let empty_filter = Filter::new();
        assert!(empty_filter.is_empty());

        let filter_with_ids = Filter::ids(vec!["id1".to_string()]);
        assert!(!filter_with_ids.is_empty());

        let filter_with_since = Filter::new().since(1000);
        assert!(!filter_with_since.is_empty());
    }

    #[test]
    fn test_filter_serialization() {
        let filter = Filter::new()
            .with_kinds(vec![30100, 30101])
            .since(1000)
            .limit(10);

        let json = serde_json::to_string(&filter).unwrap();
        let deserialized: Filter = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.kinds, Some(vec![30100, 30101]));
        assert_eq!(deserialized.since, Some(1000));
        assert_eq!(deserialized.limit, Some(10));
    }
}

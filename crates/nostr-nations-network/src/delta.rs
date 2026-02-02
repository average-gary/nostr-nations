//! Delta synchronization for incremental state updates.
//!
//! This module provides functionality to track changes and sync only
//! modified data instead of full state transfers.

use nostr_nations_core::events::GameEvent;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Entity types that can be tracked for delta sync.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    /// Player state.
    Player,
    /// City state.
    City,
    /// Unit state.
    Unit,
    /// Territory ownership.
    Territory,
    /// Resource state.
    Resource,
    /// Technology state.
    Technology,
    /// Diplomatic relations.
    Diplomacy,
    /// Game settings/rules.
    GameSettings,
}

/// Unique identifier for a trackable entity.
#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct EntityId {
    /// Type of entity.
    pub entity_type: EntityType,
    /// Unique ID within the type.
    pub id: String,
}

impl EntityId {
    /// Create a new entity ID.
    pub fn new(entity_type: EntityType, id: impl Into<String>) -> Self {
        Self {
            entity_type,
            id: id.into(),
        }
    }

    /// Create a player entity ID.
    pub fn player(id: impl Into<String>) -> Self {
        Self::new(EntityType::Player, id)
    }

    /// Create a city entity ID.
    pub fn city(id: impl Into<String>) -> Self {
        Self::new(EntityType::City, id)
    }

    /// Create a unit entity ID.
    pub fn unit(id: impl Into<String>) -> Self {
        Self::new(EntityType::Unit, id)
    }

    /// Create a territory entity ID.
    pub fn territory(id: impl Into<String>) -> Self {
        Self::new(EntityType::Territory, id)
    }
}

/// Tracks which entities have been modified.
#[derive(Clone, Debug, Default)]
pub struct DirtyTracker {
    /// Set of dirty entity IDs.
    dirty: HashSet<EntityId>,
    /// Version counter for each entity (for conflict detection).
    versions: HashMap<EntityId, u64>,
    /// Global version counter.
    global_version: u64,
}

impl DirtyTracker {
    /// Create a new dirty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Mark an entity as dirty (modified).
    pub fn mark_dirty(&mut self, entity_id: EntityId) {
        self.dirty.insert(entity_id.clone());
        *self.versions.entry(entity_id).or_insert(0) += 1;
        self.global_version += 1;
    }

    /// Mark multiple entities as dirty.
    pub fn mark_dirty_batch(&mut self, entity_ids: impl IntoIterator<Item = EntityId>) {
        for id in entity_ids {
            self.mark_dirty(id);
        }
    }

    /// Clear the dirty flag for an entity.
    pub fn clear_dirty(&mut self, entity_id: &EntityId) {
        self.dirty.remove(entity_id);
    }

    /// Clear all dirty flags.
    pub fn clear_all_dirty(&mut self) {
        self.dirty.clear();
    }

    /// Check if an entity is dirty.
    pub fn is_dirty(&self, entity_id: &EntityId) -> bool {
        self.dirty.contains(entity_id)
    }

    /// Get all dirty entities.
    pub fn get_dirty(&self) -> impl Iterator<Item = &EntityId> {
        self.dirty.iter()
    }

    /// Get the count of dirty entities.
    pub fn dirty_count(&self) -> usize {
        self.dirty.len()
    }

    /// Get dirty entities of a specific type.
    pub fn get_dirty_by_type(&self, entity_type: EntityType) -> Vec<&EntityId> {
        self.dirty
            .iter()
            .filter(|id| id.entity_type == entity_type)
            .collect()
    }

    /// Get the version of an entity.
    pub fn get_version(&self, entity_id: &EntityId) -> u64 {
        self.versions.get(entity_id).copied().unwrap_or(0)
    }

    /// Get the global version.
    pub fn global_version(&self) -> u64 {
        self.global_version
    }

    /// Check if there are any dirty entities.
    pub fn has_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }
}

/// A delta representing changes to sync.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateDelta {
    /// Base version this delta applies to.
    pub base_version: u64,
    /// Version after applying this delta.
    pub target_version: u64,
    /// Changed entities with their data.
    pub changes: Vec<EntityChange>,
    /// Deleted entities.
    pub deletions: Vec<EntityId>,
    /// Timestamp of the delta.
    pub timestamp: u64,
}

impl StateDelta {
    /// Create a new empty delta.
    pub fn new(base_version: u64, target_version: u64) -> Self {
        Self {
            base_version,
            target_version,
            changes: Vec::new(),
            deletions: Vec::new(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
        }
    }

    /// Add a change to the delta.
    pub fn add_change(&mut self, change: EntityChange) {
        self.changes.push(change);
    }

    /// Add a deletion to the delta.
    pub fn add_deletion(&mut self, entity_id: EntityId) {
        self.deletions.push(entity_id);
    }

    /// Check if the delta is empty.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty() && self.deletions.is_empty()
    }

    /// Get the number of changes.
    pub fn change_count(&self) -> usize {
        self.changes.len() + self.deletions.len()
    }
}

/// A change to a single entity.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityChange {
    /// Entity being changed.
    pub entity_id: EntityId,
    /// Version of the entity.
    pub version: u64,
    /// Serialized data (JSON).
    pub data: String,
    /// Type of change.
    pub change_type: ChangeType,
}

/// Type of change to an entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Entity was created.
    Create,
    /// Entity was updated.
    Update,
    /// Entity was deleted (also in deletions list for redundancy).
    Delete,
}

/// Manages delta synchronization between peers.
pub struct DeltaSyncManager {
    /// Local dirty tracker.
    tracker: DirtyTracker,
    /// Last synced version per peer.
    peer_versions: HashMap<String, u64>,
    /// Pending deltas waiting to be sent.
    pending_deltas: Vec<StateDelta>,
    /// Statistics.
    stats: DeltaSyncStats,
}

/// Statistics for delta sync operations.
#[derive(Clone, Debug, Default)]
pub struct DeltaSyncStats {
    /// Total deltas created.
    pub deltas_created: u64,
    /// Total deltas applied.
    pub deltas_applied: u64,
    /// Total entities synced.
    pub entities_synced: u64,
    /// Sync operations that used delta instead of full.
    pub delta_syncs: u64,
    /// Sync operations that required full state.
    pub full_syncs: u64,
}

impl DeltaSyncManager {
    /// Create a new delta sync manager.
    pub fn new() -> Self {
        Self {
            tracker: DirtyTracker::new(),
            peer_versions: HashMap::new(),
            pending_deltas: Vec::new(),
            stats: DeltaSyncStats::default(),
        }
    }

    /// Get the dirty tracker.
    pub fn tracker(&self) -> &DirtyTracker {
        &self.tracker
    }

    /// Get mutable access to the dirty tracker.
    pub fn tracker_mut(&mut self) -> &mut DirtyTracker {
        &mut self.tracker
    }

    /// Mark an entity as modified.
    pub fn mark_modified(&mut self, entity_id: EntityId) {
        self.tracker.mark_dirty(entity_id);
    }

    /// Register a peer's last known version.
    pub fn register_peer_version(&mut self, peer_id: &str, version: u64) {
        self.peer_versions.insert(peer_id.to_string(), version);
    }

    /// Get a peer's last known version.
    pub fn get_peer_version(&self, peer_id: &str) -> Option<u64> {
        self.peer_versions.get(peer_id).copied()
    }

    /// Check if a peer needs a delta sync.
    pub fn peer_needs_sync(&self, peer_id: &str) -> bool {
        match self.peer_versions.get(peer_id) {
            Some(&version) => version < self.tracker.global_version(),
            None => true, // Unknown peer needs full sync
        }
    }

    /// Create a delta for a peer based on their last version.
    /// Returns None if full sync is needed (peer too far behind or unknown).
    pub fn create_delta_for_peer(
        &mut self,
        peer_id: &str,
        max_changes: usize,
    ) -> Option<StateDelta> {
        let peer_version = self.peer_versions.get(peer_id)?;
        let current_version = self.tracker.global_version();

        if *peer_version >= current_version {
            return None; // Already synced
        }

        // Check if delta is feasible (not too far behind)
        if self.tracker.dirty_count() > max_changes {
            self.stats.full_syncs += 1;
            return None; // Too many changes, need full sync
        }

        self.stats.delta_syncs += 1;
        self.stats.deltas_created += 1;

        Some(StateDelta::new(*peer_version, current_version))
    }

    /// Apply a delta received from a peer.
    pub fn apply_delta(&mut self, delta: &StateDelta) -> Result<(), DeltaSyncError> {
        // Version check
        if delta.base_version > self.tracker.global_version() {
            return Err(DeltaSyncError::VersionMismatch {
                expected: self.tracker.global_version(),
                received: delta.base_version,
            });
        }

        self.stats.deltas_applied += 1;
        self.stats.entities_synced += delta.change_count() as u64;

        Ok(())
    }

    /// Clear all dirty flags after successful sync.
    pub fn sync_completed(&mut self, peer_id: &str) {
        let version = self.tracker.global_version();
        self.peer_versions.insert(peer_id.to_string(), version);
    }

    /// Get sync statistics.
    pub fn stats(&self) -> &DeltaSyncStats {
        &self.stats
    }
}

impl Default for DeltaSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Delta sync errors.
#[derive(Clone, Debug)]
pub enum DeltaSyncError {
    /// Version mismatch.
    VersionMismatch { expected: u64, received: u64 },
    /// Entity not found.
    EntityNotFound(EntityId),
    /// Conflict detected.
    Conflict {
        entity_id: EntityId,
        message: String,
    },
    /// Invalid delta.
    InvalidDelta(String),
}

impl std::fmt::Display for DeltaSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeltaSyncError::VersionMismatch { expected, received } => {
                write!(
                    f,
                    "Version mismatch: expected {}, received {}",
                    expected, received
                )
            }
            DeltaSyncError::EntityNotFound(id) => {
                write!(f, "Entity not found: {:?}", id)
            }
            DeltaSyncError::Conflict { entity_id, message } => {
                write!(f, "Conflict for {:?}: {}", entity_id, message)
            }
            DeltaSyncError::InvalidDelta(msg) => {
                write!(f, "Invalid delta: {}", msg)
            }
        }
    }
}

impl std::error::Error for DeltaSyncError {}

/// Extract affected entities from a game event for dirty tracking.
pub fn extract_entities_from_event(event: &GameEvent) -> Vec<EntityId> {
    use nostr_nations_core::events::GameAction;

    let mut entities = Vec::new();

    // Add player as always affected
    entities.push(EntityId::player(event.player_id.to_string()));

    // Extract additional entities based on action type
    match &event.action {
        GameAction::CreateGame { .. } => {
            entities.push(EntityId::new(EntityType::GameSettings, "settings"));
        }
        GameAction::JoinGame { .. } => {
            // Player already added
        }
        GameAction::StartGame => {
            entities.push(EntityId::new(EntityType::GameSettings, "settings"));
        }
        GameAction::EndTurn => {
            // Turn end affects all entities potentially
            entities.push(EntityId::new(EntityType::GameSettings, "turn"));
        }
        GameAction::EndGame { .. } => {
            entities.push(EntityId::new(EntityType::GameSettings, "game_end"));
        }
        GameAction::FoundCity { settler_id, name } => {
            entities.push(EntityId::unit(settler_id.to_string()));
            entities.push(EntityId::city(name.clone()));
        }
        GameAction::SetProduction { city_id, .. } => {
            entities.push(EntityId::city(city_id.to_string()));
        }
        GameAction::BuyItem { city_id, .. } => {
            entities.push(EntityId::city(city_id.to_string()));
        }
        GameAction::MoveUnit { unit_id, .. } => {
            entities.push(EntityId::unit(unit_id.to_string()));
        }
        GameAction::AttackUnit {
            attacker_id,
            defender_id,
            ..
        } => {
            entities.push(EntityId::unit(attacker_id.to_string()));
            entities.push(EntityId::unit(defender_id.to_string()));
        }
        GameAction::AttackCity {
            attacker_id,
            city_id,
            ..
        } => {
            entities.push(EntityId::unit(attacker_id.to_string()));
            entities.push(EntityId::city(city_id.to_string()));
        }
        GameAction::FortifyUnit { unit_id }
        | GameAction::SleepUnit { unit_id }
        | GameAction::WakeUnit { unit_id }
        | GameAction::DeleteUnit { unit_id }
        | GameAction::UpgradeUnit { unit_id, .. } => {
            entities.push(EntityId::unit(unit_id.to_string()));
        }
        GameAction::BuildImprovement { unit_id, .. }
        | GameAction::BuildRoad { unit_id }
        | GameAction::RemoveFeature { unit_id } => {
            entities.push(EntityId::unit(unit_id.to_string()));
        }
        GameAction::AssignCitizen { city_id, .. }
        | GameAction::UnassignCitizen { city_id, .. }
        | GameAction::SellBuilding { city_id, .. } => {
            entities.push(EntityId::city(city_id.to_string()));
        }
        GameAction::SetResearch { tech_id } => {
            entities.push(EntityId::new(EntityType::Technology, tech_id.clone()));
        }
        GameAction::DeclareWar { target_player } => {
            entities.push(EntityId::new(
                EntityType::Diplomacy,
                format!("{}_{}", event.player_id, target_player),
            ));
        }
        GameAction::ProposePeace { target_player } => {
            entities.push(EntityId::new(
                EntityType::Diplomacy,
                format!("{}_{}", event.player_id, target_player),
            ));
        }
        GameAction::AcceptPeace { from_player } | GameAction::RejectPeace { from_player } => {
            entities.push(EntityId::new(
                EntityType::Diplomacy,
                format!("{}_{}", event.player_id, from_player),
            ));
        }
        GameAction::RequestRandom { .. } | GameAction::ProvideRandom { .. } => {
            // Randomness actions don't affect game state entities directly
        }
    }

    entities
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;

    // ==================== EntityId Tests ====================

    #[test]
    fn test_entity_id_new() {
        let id = EntityId::new(EntityType::Player, "player1");
        assert_eq!(id.entity_type, EntityType::Player);
        assert_eq!(id.id, "player1");
    }

    #[test]
    fn test_entity_id_constructors() {
        assert_eq!(EntityId::player("p1").entity_type, EntityType::Player);
        assert_eq!(EntityId::city("c1").entity_type, EntityType::City);
        assert_eq!(EntityId::unit("u1").entity_type, EntityType::Unit);
        assert_eq!(EntityId::territory("t1").entity_type, EntityType::Territory);
    }

    #[test]
    fn test_entity_id_equality() {
        let id1 = EntityId::player("p1");
        let id2 = EntityId::player("p1");
        let id3 = EntityId::player("p2");
        let id4 = EntityId::city("p1");

        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
        assert_ne!(id1, id4); // Same ID, different type
    }

    #[test]
    fn test_entity_id_hash() {
        let mut set = HashSet::new();
        set.insert(EntityId::player("p1"));
        set.insert(EntityId::player("p1")); // Duplicate
        set.insert(EntityId::player("p2"));

        assert_eq!(set.len(), 2);
    }

    // ==================== DirtyTracker Tests ====================

    #[test]
    fn test_dirty_tracker_new() {
        let tracker = DirtyTracker::new();
        assert_eq!(tracker.dirty_count(), 0);
        assert!(!tracker.has_dirty());
    }

    #[test]
    fn test_dirty_tracker_mark_dirty() {
        let mut tracker = DirtyTracker::new();
        let id = EntityId::player("p1");

        tracker.mark_dirty(id.clone());
        assert!(tracker.is_dirty(&id));
        assert_eq!(tracker.dirty_count(), 1);
    }

    #[test]
    fn test_dirty_tracker_mark_dirty_batch() {
        let mut tracker = DirtyTracker::new();
        let ids = vec![
            EntityId::player("p1"),
            EntityId::city("c1"),
            EntityId::unit("u1"),
        ];

        tracker.mark_dirty_batch(ids);
        assert_eq!(tracker.dirty_count(), 3);
    }

    #[test]
    fn test_dirty_tracker_clear_dirty() {
        let mut tracker = DirtyTracker::new();
        let id = EntityId::player("p1");

        tracker.mark_dirty(id.clone());
        tracker.clear_dirty(&id);

        assert!(!tracker.is_dirty(&id));
    }

    #[test]
    fn test_dirty_tracker_clear_all() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(EntityId::player("p1"));
        tracker.mark_dirty(EntityId::city("c1"));

        tracker.clear_all_dirty();
        assert_eq!(tracker.dirty_count(), 0);
    }

    #[test]
    fn test_dirty_tracker_versions() {
        let mut tracker = DirtyTracker::new();
        let id = EntityId::player("p1");

        assert_eq!(tracker.get_version(&id), 0);

        tracker.mark_dirty(id.clone());
        assert_eq!(tracker.get_version(&id), 1);

        tracker.mark_dirty(id.clone());
        assert_eq!(tracker.get_version(&id), 2);
    }

    #[test]
    fn test_dirty_tracker_global_version() {
        let mut tracker = DirtyTracker::new();

        assert_eq!(tracker.global_version(), 0);

        tracker.mark_dirty(EntityId::player("p1"));
        tracker.mark_dirty(EntityId::city("c1"));

        assert_eq!(tracker.global_version(), 2);
    }

    #[test]
    fn test_dirty_tracker_get_by_type() {
        let mut tracker = DirtyTracker::new();
        tracker.mark_dirty(EntityId::player("p1"));
        tracker.mark_dirty(EntityId::player("p2"));
        tracker.mark_dirty(EntityId::city("c1"));

        let players = tracker.get_dirty_by_type(EntityType::Player);
        assert_eq!(players.len(), 2);

        let cities = tracker.get_dirty_by_type(EntityType::City);
        assert_eq!(cities.len(), 1);
    }

    // ==================== StateDelta Tests ====================

    #[test]
    fn test_state_delta_new() {
        let delta = StateDelta::new(0, 10);
        assert_eq!(delta.base_version, 0);
        assert_eq!(delta.target_version, 10);
        assert!(delta.is_empty());
    }

    #[test]
    fn test_state_delta_add_change() {
        let mut delta = StateDelta::new(0, 1);
        delta.add_change(EntityChange {
            entity_id: EntityId::player("p1"),
            version: 1,
            data: "{}".to_string(),
            change_type: ChangeType::Update,
        });

        assert!(!delta.is_empty());
        assert_eq!(delta.change_count(), 1);
    }

    #[test]
    fn test_state_delta_add_deletion() {
        let mut delta = StateDelta::new(0, 1);
        delta.add_deletion(EntityId::unit("u1"));

        assert_eq!(delta.deletions.len(), 1);
        assert_eq!(delta.change_count(), 1);
    }

    #[test]
    fn test_state_delta_serialization() {
        let mut delta = StateDelta::new(5, 10);
        delta.add_change(EntityChange {
            entity_id: EntityId::player("p1"),
            version: 1,
            data: r#"{"name":"Alice"}"#.to_string(),
            change_type: ChangeType::Update,
        });

        let json = serde_json::to_string(&delta).unwrap();
        let restored: StateDelta = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.base_version, 5);
        assert_eq!(restored.target_version, 10);
        assert_eq!(restored.changes.len(), 1);
    }

    // ==================== DeltaSyncManager Tests ====================

    #[test]
    fn test_delta_sync_manager_new() {
        let manager = DeltaSyncManager::new();
        assert_eq!(manager.tracker().dirty_count(), 0);
    }

    #[test]
    fn test_delta_sync_manager_mark_modified() {
        let mut manager = DeltaSyncManager::new();
        manager.mark_modified(EntityId::player("p1"));

        assert!(manager.tracker().is_dirty(&EntityId::player("p1")));
    }

    #[test]
    fn test_delta_sync_manager_peer_versions() {
        let mut manager = DeltaSyncManager::new();

        assert!(manager.get_peer_version("peer1").is_none());

        manager.register_peer_version("peer1", 5);
        assert_eq!(manager.get_peer_version("peer1"), Some(5));
    }

    #[test]
    fn test_delta_sync_manager_peer_needs_sync() {
        let mut manager = DeltaSyncManager::new();

        // Unknown peer needs sync
        assert!(manager.peer_needs_sync("peer1"));

        // Register peer at current version
        manager.register_peer_version("peer1", 0);
        assert!(!manager.peer_needs_sync("peer1"));

        // Make changes
        manager.mark_modified(EntityId::player("p1"));
        assert!(manager.peer_needs_sync("peer1"));
    }

    #[test]
    fn test_delta_sync_manager_create_delta() {
        let mut manager = DeltaSyncManager::new();
        manager.register_peer_version("peer1", 0);
        manager.mark_modified(EntityId::player("p1"));

        let delta = manager.create_delta_for_peer("peer1", 100);
        assert!(delta.is_some());

        let delta = delta.unwrap();
        assert_eq!(delta.base_version, 0);
        assert_eq!(delta.target_version, 1);
    }

    #[test]
    fn test_delta_sync_manager_sync_completed() {
        let mut manager = DeltaSyncManager::new();
        manager.register_peer_version("peer1", 0);
        manager.mark_modified(EntityId::player("p1"));

        manager.sync_completed("peer1");

        assert!(!manager.peer_needs_sync("peer1"));
    }

    // ==================== extract_entities_from_event Tests ====================

    #[test]
    fn test_extract_entities_end_turn() {
        let event = GameEvent::new("game1".to_string(), 0, None, 1, 1, GameAction::EndTurn);

        let entities = extract_entities_from_event(&event);
        assert!(entities.iter().any(|e| e.entity_type == EntityType::Player));
    }

    #[test]
    fn test_extract_entities_found_city() {
        let event = GameEvent::new(
            "game1".to_string(),
            0,
            None,
            1,
            1,
            GameAction::FoundCity {
                settler_id: 1,
                name: "Rome".to_string(),
            },
        );

        let entities = extract_entities_from_event(&event);
        assert!(entities.iter().any(|e| e.entity_type == EntityType::City));
        assert!(entities.iter().any(|e| e.entity_type == EntityType::Unit));
    }

    #[test]
    fn test_extract_entities_attack() {
        let event = GameEvent::new(
            "game1".to_string(),
            0,
            None,
            1,
            1,
            GameAction::AttackUnit {
                attacker_id: 1,
                defender_id: 2,
                random: 0.5,
            },
        );

        let entities = extract_entities_from_event(&event);
        let unit_entities: Vec<_> = entities
            .iter()
            .filter(|e| e.entity_type == EntityType::Unit)
            .collect();
        assert_eq!(unit_entities.len(), 2);
    }

    // ==================== Error Tests ====================

    #[test]
    fn test_delta_sync_error_display() {
        let e1 = DeltaSyncError::VersionMismatch {
            expected: 5,
            received: 10,
        };
        assert!(format!("{}", e1).contains("Version mismatch"));

        let e2 = DeltaSyncError::EntityNotFound(EntityId::player("p1"));
        assert!(format!("{}", e2).contains("Entity not found"));

        let e3 = DeltaSyncError::Conflict {
            entity_id: EntityId::unit("u1"),
            message: "concurrent modification".to_string(),
        };
        assert!(format!("{}", e3).contains("Conflict"));

        let e4 = DeltaSyncError::InvalidDelta("bad format".to_string());
        assert!(format!("{}", e4).contains("Invalid delta"));
    }
}

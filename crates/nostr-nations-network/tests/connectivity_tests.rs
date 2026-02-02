//! Connectivity and synchronization tests for the networking layer.
//!
//! These tests verify peer connectivity, connection recovery, multi-peer networking,
//! network partitions, message delivery, sync protocols, latency handling, and error conditions.
//!
//! Run with: `cargo test -p nostr-nations-network --test connectivity_tests`
//! Run ignored tests with: `cargo test -p nostr-nations-network --test connectivity_tests -- --ignored`

#![allow(dead_code)]

use nostr_nations_network::{
    BackoffConfig, BackoffState, BatchConfig, CacheConfig, ConnectionPool,
    ConnectionState, EventBatch, EventBatcher, EventCache, EventDeduplicator, EventUnbatcher,
    PeerManager, PeerMessage, PoolConfig, PooledConnectionState,
    PeerSyncTracker, SyncManager, SyncResponse, SyncState,
};
use nostr_nations_network::nostr_nations_core::events::{EventChain, GameAction, GameEvent};
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test game event with the given parameters.
fn create_test_event(id: &str, game_id: &str, turn: u32, seq: u32) -> GameEvent {
    let mut event = GameEvent::new(
        game_id.to_string(),
        0,
        None,
        turn,
        seq,
        GameAction::EndTurn,
    );
    event.id = id.to_string();
    event.timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    event
}

/// Create a test event with previous event linkage (using owned strings).
fn create_linked_event(id: &str, prev: Option<String>, game_id: &str, turn: u32, seq: u32) -> GameEvent {
    let mut event = GameEvent::new(
        game_id.to_string(),
        0,
        prev,
        turn,
        seq,
        GameAction::EndTurn,
    );
    event.id = id.to_string();
    event.timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    event
}

/// Simulated peer for testing.
struct SimulatedPeer {
    id: String,
    manager: PeerManager,
    cache: EventCache,
    sync_manager: SyncManager,
    received_events: Vec<GameEvent>,
    sent_events: Vec<GameEvent>,
}

impl SimulatedPeer {
    fn new(id: &str, game_id: &str, is_host: bool) -> Self {
        Self {
            id: id.to_string(),
            manager: PeerManager::new(id.to_string(), game_id.to_string(), is_host),
            cache: EventCache::new(CacheConfig {
                max_events: 1000,
                ..Default::default()
            }),
            sync_manager: SyncManager::new(game_id.to_string(), 0),
            received_events: Vec::new(),
            sent_events: Vec::new(),
        }
    }

    fn receive_event(&mut self, event: GameEvent) -> bool {
        if !self.cache.is_duplicate(&event.id) {
            self.cache.insert(event.clone());
            self.received_events.push(event);
            true
        } else {
            false
        }
    }

    fn send_event(&mut self, event: GameEvent) {
        self.sent_events.push(event);
    }
}

/// Simulated network for multi-peer testing.
struct SimulatedNetwork {
    peers: HashMap<String, SimulatedPeer>,
    message_queue: Vec<(String, String, GameEvent)>, // (from, to, event)
    partitioned_peers: HashSet<String>,
    latency_ms: u64,
    drop_rate: f64, // 0.0 to 1.0
}

impl SimulatedNetwork {
    fn new() -> Self {
        Self {
            peers: HashMap::new(),
            message_queue: Vec::new(),
            partitioned_peers: HashSet::new(),
            latency_ms: 0,
            drop_rate: 0.0,
        }
    }

    fn add_peer(&mut self, id: &str, game_id: &str, is_host: bool) {
        self.peers.insert(id.to_string(), SimulatedPeer::new(id, game_id, is_host));
    }

    fn remove_peer(&mut self, id: &str) -> Option<SimulatedPeer> {
        self.peers.remove(id)
    }

    fn partition_peer(&mut self, id: &str) {
        self.partitioned_peers.insert(id.to_string());
    }

    fn heal_partition(&mut self, id: &str) {
        self.partitioned_peers.remove(id);
    }

    fn broadcast(&mut self, from: &str, event: GameEvent) {
        let peer_ids: Vec<String> = self.peers.keys()
            .filter(|id| *id != from)
            .cloned()
            .collect();
        
        for to in peer_ids {
            if !self.partitioned_peers.contains(from) && !self.partitioned_peers.contains(&to) {
                // Simulate packet loss
                if simple_rand_float() >= self.drop_rate {
                    self.message_queue.push((from.to_string(), to, event.clone()));
                }
            }
        }
        
        // Mark as sent by sender
        if let Some(peer) = self.peers.get_mut(from) {
            peer.send_event(event);
        }
    }

    fn deliver_messages(&mut self) -> usize {
        let messages: Vec<_> = self.message_queue.drain(..).collect();
        let mut delivered = 0;
        
        for (_from, to, event) in messages {
            if let Some(peer) = self.peers.get_mut(&to) {
                if peer.receive_event(event) {
                    delivered += 1;
                }
            }
        }
        
        delivered
    }

    fn get_peer(&self, id: &str) -> Option<&SimulatedPeer> {
        self.peers.get(id)
    }

    fn get_peer_mut(&mut self, id: &str) -> Option<&mut SimulatedPeer> {
        self.peers.get_mut(id)
    }
}

/// Simple pseudo-random float generator for testing.
fn simple_rand_float() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

// ============================================================================
// 1. Basic Connectivity Tests
// ============================================================================

/// Test that a peer can connect to another peer.
#[tokio::test]
async fn test_peer_can_connect_to_another_peer() {
    // Create two peer managers
    let host = PeerManager::new("host_node".to_string(), "game1".to_string(), true);
    let _client = PeerManager::new("client_node".to_string(), "game1".to_string(), false);

    // Host creates a connection ticket
    let ticket = host.create_ticket(vec!["192.168.1.1:4433".to_string()], 3600);
    
    // Verify ticket contains correct information
    assert_eq!(ticket.node_id, "host_node");
    assert_eq!(ticket.game_id, "game1");
    assert!(!ticket.is_expired());
    
    // Simulate client connecting to host
    host.add_peer("client_node".to_string()).await;
    
    // Verify connection was established
    assert_eq!(host.peer_count().await, 1);
    
    let peer_info = host.get_peer("client_node").await.unwrap();
    assert_eq!(peer_info.state, ConnectionState::Connecting);
}

/// Test that connection handshake completes.
#[tokio::test]
async fn test_connection_handshake_completes() {
    let host = PeerManager::new("host".to_string(), "game1".to_string(), true);
    
    // Simulate connection
    host.add_peer("client".to_string()).await;
    
    // Simulate handshake message
    let hello = PeerMessage::Hello {
        peer_id: "client".to_string(),
        game_id: "game1".to_string(),
        player_name: "Player1".to_string(),
    };
    
    host.handle_message("client", hello).await;
    
    // Simulate join request
    let join_request = PeerMessage::JoinRequest {
        player_name: "Player1".to_string(),
        civilization_id: "rome".to_string(),
    };
    
    host.handle_message("client", join_request).await;
    
    // Host approves join
    host.peer_joined("client", "Player1".to_string(), 1).await;
    
    // Verify handshake completed
    let peer_info = host.get_peer("client").await.unwrap();
    assert_eq!(peer_info.state, ConnectionState::Joined);
    assert_eq!(peer_info.player_name, Some("Player1".to_string()));
    assert_eq!(peer_info.player_id, Some(1));
}

/// Test that peers exchange identification.
#[tokio::test]
async fn test_peers_exchange_identification() {
    let host = PeerManager::new("host".to_string(), "game1".to_string(), true);
    
    // Add multiple peers
    host.add_peer("peer1".to_string()).await;
    host.add_peer("peer2".to_string()).await;
    host.add_peer("peer3".to_string()).await;
    
    // Each peer sends identification
    for (peer_id, name, civ) in [
        ("peer1", "Alice", "egypt"),
        ("peer2", "Bob", "greece"),
        ("peer3", "Carol", "persia"),
    ] {
        let hello = PeerMessage::Hello {
            peer_id: peer_id.to_string(),
            game_id: "game1".to_string(),
            player_name: name.to_string(),
        };
        host.handle_message(peer_id, hello).await;
        
        let join = PeerMessage::JoinRequest {
            player_name: name.to_string(),
            civilization_id: civ.to_string(),
        };
        host.handle_message(peer_id, join).await;
    }
    
    // Approve all joins
    host.peer_joined("peer1", "Alice".to_string(), 1).await;
    host.peer_joined("peer2", "Bob".to_string(), 2).await;
    host.peer_joined("peer3", "Carol".to_string(), 3).await;
    
    // Verify all peers are identified
    let peers = host.get_peers().await;
    assert_eq!(peers.len(), 3);
    
    for peer in peers {
        assert_eq!(peer.state, ConnectionState::Joined);
        assert!(peer.player_name.is_some());
        assert!(peer.player_id.is_some());
    }
}

// ============================================================================
// 2. Connection Recovery Tests
// ============================================================================

/// Test reconnection after disconnect.
#[tokio::test]
async fn test_reconnection_after_disconnect() {
    let pool = ConnectionPool::new(PoolConfig {
        max_connections: 10,
        auto_reconnect: true,
        max_consecutive_failures: 3,
        ..Default::default()
    });
    
    // Add connection
    pool.add_connection("peer1".to_string(), "192.168.1.1:4433".to_string()).await.unwrap();
    pool.mark_connected("peer1").await;
    
    // Verify connected
    let conn = pool.get_connection("peer1").await.unwrap();
    assert_eq!(conn.health.state, PooledConnectionState::Connected);
    
    // Simulate disconnect
    pool.mark_disconnected("peer1").await;
    
    let conn = pool.get_connection("peer1").await.unwrap();
    assert_eq!(conn.health.state, PooledConnectionState::Disconnected);
    
    // Check reconnection candidates
    let candidates = pool.get_reconnection_candidates().await;
    assert!(candidates.contains(&"peer1".to_string()));
    
    // Simulate reconnection
    pool.mark_connected("peer1").await;
    
    let conn = pool.get_connection("peer1").await.unwrap();
    assert_eq!(conn.health.state, PooledConnectionState::Connected);
}

/// Test state recovery after reconnection.
#[tokio::test]
async fn test_state_recovery_after_reconnection() {
    let game_id = "recovery_test";
    
    // Simulate initial state with events
    let mut initial_events: Vec<GameEvent> = Vec::new();
    for i in 0..10 {
        let prev = if i > 0 { Some(format!("evt_{}", i - 1)) } else { None };
        initial_events.push(create_linked_event(
            &format!("evt_{}", i),
            prev,
            game_id,
            (i / 5) as u32 + 1,
            (i % 5) as u32 + 1,
        ));
    }
    
    // Client has partial state (first 5 events)
    let mut client_sync = SyncManager::new(game_id.to_string(), 0);
    
    // Simulate reconnection - client requests sync from turn 1, sequence 5
    let _request = client_sync.create_request();
    assert_eq!(*client_sync.state(), SyncState::Requesting);
    
    // Host sends remaining events
    let response = SyncResponse {
        game_id: game_id.to_string(),
        has_more: false,
        events: initial_events[5..].to_vec(),
        current_turn: 2,
        chain_hash: None,
    };
    
    let result = client_sync.handle_response(response);
    assert_eq!(result.events_received, 5);
    
    // Client applies events
    while let Some(event) = client_sync.next_event() {
        client_sync.confirm_event(&event);
    }
    
    assert!(client_sync.is_synced());
}

/// Test no duplicate events after recovery.
#[tokio::test]
async fn test_no_duplicate_events_after_recovery() {
    let mut dedup = EventDeduplicator::new(1000);
    let mut cache = EventCache::new(CacheConfig {
        max_events: 1000,
        enable_dedup: true,
        ..Default::default()
    });
    
    // Initial events received
    let initial_events: Vec<GameEvent> = (0..10)
        .map(|i| create_test_event(&format!("evt_{}", i), "game1", 1, i))
        .collect();
    
    for event in &initial_events {
        assert!(!dedup.is_duplicate(&event.id));
        cache.insert(event.clone());
    }
    
    // Simulate reconnection - server resends some events
    let recovery_events: Vec<GameEvent> = (5..15)
        .map(|i| create_test_event(&format!("evt_{}", i), "game1", 1, i))
        .collect();
    
    let mut new_events = 0;
    let mut duplicates = 0;
    
    for event in &recovery_events {
        if dedup.is_duplicate(&event.id) {
            duplicates += 1;
        } else {
            cache.insert(event.clone());
            new_events += 1;
        }
    }
    
    // Events 5-9 should be duplicates, 10-14 should be new
    assert_eq!(duplicates, 5);
    assert_eq!(new_events, 5);
    assert_eq!(cache.len(), 15);
}

// ============================================================================
// 3. Multi-Peer Network Tests
// ============================================================================

/// Test 4 peers all connected.
#[tokio::test]
async fn test_four_peers_all_connected() {
    let pool = ConnectionPool::new(PoolConfig {
        max_connections: 10,
        ..Default::default()
    });
    
    // Add 4 peers
    for i in 0..4 {
        pool.add_connection(format!("peer_{}", i), format!("192.168.1.{}:4433", i)).await.unwrap();
        pool.mark_connected(&format!("peer_{}", i)).await;
    }
    
    // Verify all connected
    assert_eq!(pool.connection_count().await, 4);
    assert_eq!(pool.healthy_count().await, 4);
    
    let status = pool.status().await;
    assert_eq!(status.total_connections, 4);
    assert_eq!(status.healthy_connections, 4);
}

/// Test messages reach all peers.
#[test]
fn test_messages_reach_all_peers() {
    let mut network = SimulatedNetwork::new();
    let game_id = "multi_peer_game";
    
    // Add 4 peers
    network.add_peer("host", game_id, true);
    network.add_peer("peer1", game_id, false);
    network.add_peer("peer2", game_id, false);
    network.add_peer("peer3", game_id, false);
    
    // Host broadcasts an event
    let event = create_test_event("broadcast_evt", game_id, 1, 1);
    network.broadcast("host", event);
    
    // Deliver messages
    let delivered = network.deliver_messages();
    assert_eq!(delivered, 3); // 3 other peers
    
    // Verify all peers received the event
    for peer_id in ["peer1", "peer2", "peer3"] {
        let peer = network.get_peer(peer_id).unwrap();
        assert_eq!(peer.received_events.len(), 1);
        assert_eq!(peer.received_events[0].id, "broadcast_evt");
    }
}

/// Test peer discovery works.
#[tokio::test]
async fn test_peer_discovery_works() {
    let host = PeerManager::new("host".to_string(), "game1".to_string(), true);
    
    // Create discovery ticket
    let ticket = host.create_ticket(
        vec![
            "192.168.1.1:4433".to_string(),
            "10.0.0.1:4433".to_string(),
        ],
        3600,
    );
    
    // Verify ticket can be serialized and deserialized
    let ticket_str = ticket.to_string().unwrap();
    let parsed = nostr_nations_network::ConnectionTicket::from_string(&ticket_str).unwrap();
    
    assert_eq!(parsed.node_id, "host");
    assert_eq!(parsed.game_id, "game1");
    assert_eq!(parsed.addresses.len(), 2);
    
    // Simulate multiple peers joining via discovery
    for i in 0..4 {
        let peer_id = format!("discovered_peer_{}", i);
        host.add_peer(peer_id.clone()).await;
        host.peer_joined(&peer_id, format!("Player{}", i), i as u32).await;
    }
    
    assert_eq!(host.peer_count().await, 4);
}

// ============================================================================
// 4. Network Partition Tests
// ============================================================================

/// Test handling network split.
#[test]
fn test_handle_network_split() {
    let mut network = SimulatedNetwork::new();
    let game_id = "partition_game";
    
    // Add peers
    network.add_peer("host", game_id, true);
    network.add_peer("peer1", game_id, false);
    network.add_peer("peer2", game_id, false);
    
    // Normal broadcast works
    let event1 = create_test_event("evt1", game_id, 1, 1);
    network.broadcast("host", event1);
    let delivered = network.deliver_messages();
    assert_eq!(delivered, 2);
    
    // Partition peer1
    network.partition_peer("peer1");
    
    // Broadcast during partition
    let event2 = create_test_event("evt2", game_id, 1, 2);
    network.broadcast("host", event2);
    let delivered = network.deliver_messages();
    assert_eq!(delivered, 1); // Only peer2 receives
    
    // Verify peer1 didn't receive event2
    let peer1 = network.get_peer("peer1").unwrap();
    assert_eq!(peer1.received_events.len(), 1);
    assert_eq!(peer1.received_events[0].id, "evt1");
    
    // peer2 received both
    let peer2 = network.get_peer("peer2").unwrap();
    assert_eq!(peer2.received_events.len(), 2);
}

/// Test detecting when peers are unreachable.
#[tokio::test]
async fn test_detect_peers_unreachable() {
    let pool = ConnectionPool::new(PoolConfig {
        max_consecutive_failures: 3,
        auto_reconnect: true,
        ..Default::default()
    });
    
    pool.add_connection("peer1".to_string(), "192.168.1.1:4433".to_string()).await.unwrap();
    pool.mark_connected("peer1").await;
    
    // Simulate failures
    pool.record_failure("peer1", "Connection timeout".to_string()).await;
    pool.record_failure("peer1", "Connection timeout".to_string()).await;
    pool.record_failure("peer1", "Connection timeout".to_string()).await;
    
    // Peer should be marked disconnected after 3 failures
    let conn = pool.get_connection("peer1").await.unwrap();
    assert_eq!(conn.health.state, PooledConnectionState::Disconnected);
    assert_eq!(conn.health.consecutive_failures, 3);
}

/// Test reconnection when partition heals.
#[test]
fn test_reconnect_when_partition_heals() {
    let mut network = SimulatedNetwork::new();
    let game_id = "heal_partition_game";
    
    network.add_peer("host", game_id, true);
    network.add_peer("peer1", game_id, false);
    
    // Initial connection
    let event1 = create_test_event("evt1", game_id, 1, 1);
    network.broadcast("host", event1);
    network.deliver_messages();
    
    // Partition
    network.partition_peer("peer1");
    
    // Events during partition
    let event2 = create_test_event("evt2", game_id, 1, 2);
    network.broadcast("host", event2);
    network.deliver_messages();
    
    // Heal partition
    network.heal_partition("peer1");
    
    // Events after healing
    let event3 = create_test_event("evt3", game_id, 1, 3);
    network.broadcast("host", event3);
    network.deliver_messages();
    
    // peer1 should have received evt1 and evt3 (missed evt2 during partition)
    let peer1 = network.get_peer("peer1").unwrap();
    assert_eq!(peer1.received_events.len(), 2);
    assert_eq!(peer1.received_events[0].id, "evt1");
    assert_eq!(peer1.received_events[1].id, "evt3");
}

// ============================================================================
// 5. Message Delivery Tests
// ============================================================================

/// Test messages are delivered in order.
#[test]
fn test_messages_delivered_in_order() {
    let mut batcher = EventBatcher::new(BatchConfig {
        max_batch_size: 100,
        ..Default::default()
    });
    let mut unbatcher = EventUnbatcher::new();
    
    // Add events in order
    let events: Vec<GameEvent> = (0..50)
        .map(|i| create_test_event(&format!("evt_{}", i), "order_game", (i / 10) as u32, (i % 10) as u32))
        .collect();
    
    for event in &events {
        batcher.add_event(event.clone());
    }
    
    // Flush and process
    let batch = batcher.flush().unwrap();
    let received = unbatcher.process_batch(batch).unwrap();
    
    // Verify order is preserved
    for (i, event) in received.iter().enumerate() {
        assert_eq!(event.id, format!("evt_{}", i));
    }
}

/// Test no message loss.
#[test]
fn test_no_message_loss() {
    let mut network = SimulatedNetwork::new();
    let game_id = "no_loss_game";
    
    network.add_peer("host", game_id, true);
    network.add_peer("peer1", game_id, false);
    
    // No drop rate
    network.drop_rate = 0.0;
    
    // Send many events
    let event_count = 100;
    for i in 0..event_count {
        let event = create_test_event(&format!("evt_{}", i), game_id, 1, i);
        network.broadcast("host", event);
    }
    
    // Deliver all messages
    let mut total_delivered = 0;
    loop {
        let delivered = network.deliver_messages();
        if delivered == 0 {
            break;
        }
        total_delivered += delivered;
    }
    
    // Verify no loss
    assert_eq!(total_delivered, event_count as usize);
    
    let peer1 = network.get_peer("peer1").unwrap();
    assert_eq!(peer1.received_events.len(), event_count as usize);
}

/// Test duplicate detection works.
#[test]
fn test_duplicate_detection_works() {
    let mut dedup = EventDeduplicator::new(1000);
    
    // First occurrence
    assert!(!dedup.is_duplicate("evt_1"));
    assert!(!dedup.is_duplicate("evt_2"));
    assert!(!dedup.is_duplicate("evt_3"));
    
    // Duplicates
    assert!(dedup.is_duplicate("evt_1"));
    assert!(dedup.is_duplicate("evt_2"));
    assert!(dedup.is_duplicate("evt_3"));
    
    // New events still work
    assert!(!dedup.is_duplicate("evt_4"));
    assert!(dedup.is_duplicate("evt_4"));
    
    // Verify stats
    let stats = dedup.stats();
    assert_eq!(stats.unique, 4);
    assert_eq!(stats.duplicates, 4);
    assert_eq!(stats.events_checked, 8);
}

// ============================================================================
// 6. Sync Protocol Tests
// ============================================================================

/// Test new peer syncs full state.
#[test]
fn test_new_peer_syncs_full_state() {
    let game_id = "full_sync_game";
    
    // Build event chain on host
    let mut chain = EventChain::new();
    for i in 0..20 {
        let prev = if i > 0 { Some(format!("evt_{}", i - 1)) } else { None };
        let event = create_linked_event(
            &format!("evt_{}", i),
            prev,
            game_id,
            (i / 5) as u32 + 1,
            (i % 5) as u32 + 1,
        );
        chain.add(event).unwrap();
    }
    
    // New peer requests full sync
    let mut new_peer_sync = SyncManager::new(game_id.to_string(), 0);
    let request = new_peer_sync.create_request();
    
    assert_eq!(request.last_turn, 0);
    assert_eq!(request.last_sequence, 0);
    assert!(request.last_event_id.is_none());
    
    // Host sends all events
    let response = SyncResponse {
        game_id: game_id.to_string(),
        has_more: false,
        events: chain.events().to_vec(),
        current_turn: 4,
        chain_hash: None,
    };
    
    let result = new_peer_sync.handle_response(response);
    assert_eq!(result.events_received, 20);
    
    // Apply all events
    let mut applied = 0;
    while let Some(event) = new_peer_sync.next_event() {
        new_peer_sync.confirm_event(&event);
        applied += 1;
    }
    
    assert_eq!(applied, 20);
    assert!(new_peer_sync.is_synced());
}

/// Test incremental sync after reconnect.
#[test]
fn test_incremental_sync_after_reconnect() {
    let game_id = "incremental_sync_game";
    
    // Build event chain
    let mut chain = EventChain::new();
    for i in 0..30 {
        let prev = if i > 0 { Some(format!("evt_{}", i - 1)) } else { None };
        let event = create_linked_event(
            &format!("evt_{}", i),
            prev,
            game_id,
            (i / 10) as u32 + 1,
            (i % 10) as u32 + 1,
        );
        chain.add(event).unwrap();
    }
    
    // Peer already has first 20 events
    let mut peer_sync = SyncManager::new(game_id.to_string(), 0);
    
    // Initial sync of first 20
    let initial_response = SyncResponse {
        game_id: game_id.to_string(),
        has_more: false,
        events: chain.events()[..20].to_vec(),
        current_turn: 2,
        chain_hash: None,
    };
    
    peer_sync.create_request();
    peer_sync.handle_response(initial_response);
    while let Some(event) = peer_sync.next_event() {
        peer_sync.confirm_event(&event);
    }
    
    // Now request incremental sync
    let request = peer_sync.create_request();
    assert_eq!(request.last_turn, 2);
    assert_eq!(request.last_sequence, 10);
    assert_eq!(request.last_event_id, Some("evt_19".to_string()));
    
    // Host sends only new events
    let incremental_response = SyncResponse {
        game_id: game_id.to_string(),
        has_more: false,
        events: chain.events()[20..].to_vec(),
        current_turn: 3,
        chain_hash: None,
    };
    
    let result = peer_sync.handle_response(incremental_response);
    assert_eq!(result.events_received, 10);
    
    while let Some(event) = peer_sync.next_event() {
        peer_sync.confirm_event(&event);
    }
    
    assert!(peer_sync.is_synced());
}

/// Test conflict detection.
#[test]
fn test_conflict_detection() {
    let game_id = "conflict_game";
    
    // Peer expects game1, receives response for game2
    let mut peer_sync = SyncManager::new(game_id.to_string(), 0);
    peer_sync.create_request();
    
    let wrong_game_response = SyncResponse {
        game_id: "wrong_game".to_string(),
        has_more: false,
        events: vec![],
        current_turn: 1,
        chain_hash: None,
    };
    
    let result = peer_sync.handle_response(wrong_game_response);
    
    assert!(matches!(peer_sync.state(), SyncState::Failed(_)));
    assert!(!result.errors.is_empty());
}

/// Test peer sync tracker tracks progress correctly.
#[test]
fn test_peer_sync_tracker() {
    let mut tracker = PeerSyncTracker::new();
    
    // Update progress for multiple peers
    tracker.update_progress("peer1", 5, 10);
    tracker.update_progress("peer2", 5, 5);
    tracker.update_progress("peer3", 4, 15);
    
    // Check individual progress
    assert_eq!(tracker.get_progress("peer1"), Some((5, 10)));
    assert_eq!(tracker.get_progress("peer2"), Some((5, 5)));
    assert_eq!(tracker.get_progress("peer3"), Some((4, 15)));
    
    // Minimum progress is peer3 at turn 4
    let min = tracker.min_progress().unwrap();
    assert_eq!(min, (4, 15));
    
    // Check all_at_least
    assert!(tracker.all_at_least(4, 5));
    assert!(!tracker.all_at_least(5, 1)); // peer3 is behind
    
    // Remove a peer
    tracker.remove_peer("peer3");
    assert!(tracker.all_at_least(5, 5));
}

// ============================================================================
// 7. Latency Handling Tests
// ============================================================================

/// Test system works with high latency.
#[tokio::test]
async fn test_works_with_high_latency() {
    let latencies_ms = [50, 100, 200, 500];
    
    for latency_ms in latencies_ms {
        let mut sync_manager = SyncManager::new("latency_game".to_string(), 0);
        
        let events: Vec<GameEvent> = (0..20)
            .map(|i| create_test_event(&format!("lat_evt_{}", i), "latency_game", 1, i))
            .collect();
        
        let start = Instant::now();
        
        // Simulate latency for request
        tokio::time::sleep(Duration::from_millis(latency_ms)).await;
        
        sync_manager.create_request();
        
        // Simulate latency for response
        tokio::time::sleep(Duration::from_millis(latency_ms)).await;
        
        let response = SyncResponse {
            game_id: "latency_game".to_string(),
            has_more: false,
            events,
            current_turn: 1,
            chain_hash: None,
        };
        
        let result = sync_manager.handle_response(response);
        
        // Apply events
        while let Some(event) = sync_manager.next_event() {
            sync_manager.confirm_event(&event);
        }
        
        let elapsed = start.elapsed();
        
        // Verify sync completed despite latency
        assert!(sync_manager.is_synced());
        assert_eq!(result.events_received, 20);
        
        // Total time should be approximately 2x latency (round trip) plus processing
        assert!(elapsed >= Duration::from_millis(latency_ms * 2));
    }
}

/// Test timeout handling.
#[tokio::test]
async fn test_timeout_handling() {
    let config = PoolConfig {
        connect_timeout: Duration::from_millis(100),
        max_consecutive_failures: 3,
        ..Default::default()
    };
    let pool = ConnectionPool::new(config);
    
    pool.add_connection("peer1".to_string(), "192.168.1.1:4433".to_string()).await.unwrap();
    
    // Simulate timeout by recording failures
    for _ in 0..3 {
        pool.record_failure("peer1", "Connection timeout".to_string()).await;
    }
    
    let conn = pool.get_connection("peer1").await.unwrap();
    assert_eq!(conn.health.state, PooledConnectionState::Disconnected);
    assert_eq!(conn.health.last_error, Some("Connection timeout".to_string()));
}

/// Test retry logic with exponential backoff.
#[test]
fn test_retry_logic_with_backoff() {
    let config = BackoffConfig {
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        multiplier: 2.0,
        max_attempts: 5,
        jitter: false, // Disable jitter for predictable testing
    };
    
    let mut backoff = BackoffState::new(config);
    
    // First failure
    let delay1 = backoff.record_failure().unwrap();
    assert_eq!(delay1, Duration::from_millis(100));
    assert_eq!(backoff.attempts(), 1);
    
    // Second failure - should double
    let delay2 = backoff.record_failure().unwrap();
    assert_eq!(delay2, Duration::from_millis(200));
    
    // Third failure - should double again
    let delay3 = backoff.record_failure().unwrap();
    assert_eq!(delay3, Duration::from_millis(400));
    
    // Fourth failure
    let delay4 = backoff.record_failure().unwrap();
    assert_eq!(delay4, Duration::from_millis(800));
    
    // Fifth failure - at max attempts
    let delay5 = backoff.record_failure();
    assert!(delay5.is_some());
    assert!(backoff.max_reached());
    
    // Sixth failure - beyond max attempts
    let delay6 = backoff.record_failure();
    assert!(delay6.is_none());
    
    // Reset and verify
    backoff.reset();
    assert_eq!(backoff.attempts(), 0);
    assert!(!backoff.max_reached());
}

// ============================================================================
// 8. Error Condition Tests
// ============================================================================

/// Test invalid messages are rejected.
#[test]
fn test_invalid_messages_rejected() {
    // Test invalid JSON
    let invalid_json = b"not valid json";
    let result = PeerMessage::from_bytes(invalid_json);
    assert!(result.is_err());
    
    // Test empty bytes
    let empty = b"";
    let result = PeerMessage::from_bytes(empty);
    assert!(result.is_err());
    
    // Test malformed message (missing required fields)
    let malformed = br#"{"type":"Hello"}"#; // Missing peer_id, game_id, player_name
    let result = PeerMessage::from_bytes(malformed);
    assert!(result.is_err());
}

/// Test malformed data is handled.
#[test]
fn test_malformed_data_handled() {
    // Test malformed batch
    let invalid_batch_bytes = b"not a batch";
    let result = EventBatch::from_bytes(invalid_batch_bytes);
    assert!(result.is_err());
    
    // Test invalid ticket
    let invalid_ticket = "definitely not a valid ticket!@#$%";
    let result = nostr_nations_network::ConnectionTicket::from_string(invalid_ticket);
    assert!(result.is_err());
    
    // Test valid base64 but invalid JSON
    // The base64 encode of "not json" 
    let invalid_json_ticket = "bm90IGpzb24="; // base64 of "not json"
    let result = nostr_nations_network::ConnectionTicket::from_string(invalid_json_ticket);
    assert!(result.is_err());
}

/// Test resource exhaustion protection.
#[tokio::test]
async fn test_resource_exhaustion_protection() {
    // Test pool capacity limit
    let config = PoolConfig {
        max_connections: 5,
        ..Default::default()
    };
    let pool = ConnectionPool::new(config);
    
    // Fill the pool
    for i in 0..5 {
        pool.add_connection(format!("peer_{}", i), format!("192.168.1.{}:4433", i)).await.unwrap();
    }
    
    // Try to add beyond capacity
    let result = pool.add_connection("peer_overflow".to_string(), "192.168.1.100:4433".to_string()).await;
    assert!(result.is_err());
    
    // Test cache capacity limit
    let mut cache = EventCache::new(CacheConfig {
        max_events: 10,
        ..Default::default()
    });
    
    for i in 0..20 {
        cache.insert(create_test_event(&format!("evt_{}", i), "game1", 1, i));
    }
    
    // Cache should not exceed max_events
    assert_eq!(cache.len(), 10);
    
    // Test deduplicator capacity limit
    let mut dedup = EventDeduplicator::new(10);
    
    for i in 0..20 {
        dedup.is_duplicate(&format!("evt_{}", i));
    }
    
    // Deduplicator should evict old entries
    assert_eq!(dedup.len(), 10);
}

/// Test handling of duplicate connection attempts.
#[tokio::test]
async fn test_duplicate_connection_rejected() {
    let pool = ConnectionPool::new(PoolConfig::default());
    
    // Add a connection
    pool.add_connection("peer1".to_string(), "192.168.1.1:4433".to_string()).await.unwrap();
    
    // Try to add duplicate
    let result = pool.add_connection("peer1".to_string(), "192.168.1.2:4433".to_string()).await;
    
    assert!(result.is_err());
    assert_eq!(pool.connection_count().await, 1);
}

/// Test handling sync failure.
#[test]
fn test_sync_failure_handling() {
    let mut sync_manager = SyncManager::new("game1".to_string(), 0);
    
    sync_manager.create_request();
    
    let response = SyncResponse {
        game_id: "game1".to_string(),
        has_more: false,
        events: vec![create_test_event("evt1", "game1", 1, 1)],
        current_turn: 1,
        chain_hash: None,
    };
    
    sync_manager.handle_response(response);
    
    // Simulate failure while applying event
    sync_manager.report_failure("Failed to apply event: invalid signature".to_string());
    
    assert!(matches!(sync_manager.state(), SyncState::Failed(_)));
    assert_eq!(sync_manager.pending_count(), 0); // Pending events cleared
    
    // Reset and retry
    sync_manager.reset();
    assert_eq!(*sync_manager.state(), SyncState::Idle);
}

// ============================================================================
// Integration Tests
// ============================================================================

/// Full integration test of multi-peer game session.
#[tokio::test]
async fn test_full_game_session_flow() {
    let game_id = "integration_game";
    
    // 1. Host creates game
    let host = PeerManager::new("host".to_string(), game_id.to_string(), true);
    
    // 2. Host creates ticket
    let ticket = host.create_ticket(vec!["192.168.1.1:4433".to_string()], 3600);
    assert!(!ticket.is_expired());
    
    // 3. Three clients join
    for i in 0..3 {
        let peer_id = format!("client_{}", i);
        host.add_peer(peer_id.clone()).await;
        
        // Handshake
        let hello = PeerMessage::Hello {
            peer_id: peer_id.clone(),
            game_id: game_id.to_string(),
            player_name: format!("Player{}", i),
        };
        host.handle_message(&peer_id, hello).await;
        
        let join = PeerMessage::JoinRequest {
            player_name: format!("Player{}", i),
            civilization_id: format!("civ_{}", i),
        };
        host.handle_message(&peer_id, join).await;
        
        host.peer_joined(&peer_id, format!("Player{}", i), i as u32).await;
    }
    
    assert_eq!(host.peer_count().await, 3);
    
    // 4. Verify all peers are in correct state
    for peer in host.get_peers().await {
        assert_eq!(peer.state, ConnectionState::Joined);
    }
    
    // 5. Simulate game events via batching
    let mut batcher = EventBatcher::with_defaults();
    
    for turn in 1..=5 {
        for player in 0..4 {
            let event = create_test_event(
                &format!("t{}_p{}", turn, player),
                game_id,
                turn,
                player,
            );
            batcher.add_event(event);
        }
    }
    
    let batch = batcher.flush().unwrap();
    assert_eq!(batch.len(), 20);
    
    // 6. Simulate receiving and deduplicating
    let mut unbatcher = EventUnbatcher::new();
    let mut cache = EventCache::with_defaults();
    
    let events = unbatcher.process_batch(batch).unwrap();
    for event in events {
        cache.insert(event);
    }
    
    assert_eq!(cache.len(), 20);
    
    // 7. Simulate client disconnect and reconnect
    let goodbye = PeerMessage::Goodbye { reason: "Network error".to_string() };
    host.handle_message("client_0", goodbye).await;
    assert_eq!(host.peer_count().await, 2);
    
    // 8. Client reconnects
    host.add_peer("client_0".to_string()).await;
    host.peer_joined("client_0", "Player0".to_string(), 0).await;
    assert_eq!(host.peer_count().await, 3);
}

/// Test sync protocol end-to-end.
#[test]
fn test_sync_protocol_end_to_end() {
    let game_id = "sync_e2e_game";
    
    // Host has full state
    let mut host_chain = EventChain::new();
    for i in 0..50 {
        let prev = if i > 0 { Some(format!("evt_{}", i - 1)) } else { None };
        let event = create_linked_event(
            &format!("evt_{}", i),
            prev,
            game_id,
            (i / 10) as u32 + 1,
            (i % 10) as u32 + 1,
        );
        host_chain.add(event).unwrap();
    }
    
    // New client joins with no state
    let mut client_sync = SyncManager::new(game_id.to_string(), 0);
    let mut client_cache = EventCache::with_defaults();
    
    // Client requests sync
    let _request = client_sync.create_request();
    
    // Host responds (paginated - first 25 events)
    let response1 = SyncResponse {
        game_id: game_id.to_string(),
        has_more: true,
        events: host_chain.events()[..25].to_vec(),
        current_turn: 3,
        chain_hash: None,
    };
    
    let result1 = client_sync.handle_response(response1);
    assert_eq!(result1.events_received, 25);
    assert_eq!(*client_sync.state(), SyncState::Requesting); // Needs more
    
    // Apply first batch
    while let Some(event) = client_sync.next_event() {
        client_cache.insert(event.clone());
        client_sync.confirm_event(&event);
    }
    
    // Request more
    let request2 = client_sync.create_request();
    assert_eq!(request2.last_event_id, Some("evt_24".to_string()));
    
    // Host responds with remaining events
    let response2 = SyncResponse {
        game_id: game_id.to_string(),
        has_more: false,
        events: host_chain.events()[25..].to_vec(),
        current_turn: 5,
        chain_hash: None,
    };
    
    let result2 = client_sync.handle_response(response2);
    assert_eq!(result2.events_received, 25);
    
    // Apply second batch
    while let Some(event) = client_sync.next_event() {
        client_cache.insert(event.clone());
        client_sync.confirm_event(&event);
    }
    
    // Verify fully synced
    assert!(client_sync.is_synced());
    assert_eq!(client_cache.len(), 50);
}

// ============================================================================
// Stress/Performance Tests (marked as ignored)
// ============================================================================

/// Stress test: Many peers connecting simultaneously.
#[tokio::test]
#[ignore]
async fn stress_test_many_peers_connecting() {
    let pool = ConnectionPool::new(PoolConfig {
        max_connections: 100,
        ..Default::default()
    });
    
    let start = Instant::now();
    
    // Add 100 connections
    for i in 0..100 {
        pool.add_connection(format!("peer_{}", i), format!("192.168.{}.{}:4433", i / 256, i % 256))
            .await
            .unwrap();
        pool.mark_connected(&format!("peer_{}", i)).await;
    }
    
    let elapsed = start.elapsed();
    
    assert_eq!(pool.connection_count().await, 100);
    assert_eq!(pool.healthy_count().await, 100);
    
    println!("100 connections established in {:?}", elapsed);
    assert!(elapsed < Duration::from_secs(1), "Should be fast");
}

/// Stress test: High event throughput through sync.
#[test]
#[ignore]
fn stress_test_high_event_throughput() {
    let game_id = "throughput_game";
    let event_count = 10000;
    
    // Create many events
    let events: Vec<GameEvent> = (0..event_count)
        .map(|i| create_test_event(&format!("evt_{}", i), game_id, (i / 100) as u32, (i % 100) as u32))
        .collect();
    
    let mut sync_manager = SyncManager::new(game_id.to_string(), 0);
    let mut cache = EventCache::new(CacheConfig {
        max_events: 15000,
        ..Default::default()
    });
    
    let start = Instant::now();
    
    sync_manager.create_request();
    
    // Receive all events
    let response = SyncResponse {
        game_id: game_id.to_string(),
        has_more: false,
        events,
        current_turn: 100,
        chain_hash: None,
    };
    
    sync_manager.handle_response(response);
    
    // Apply all events
    let mut applied = 0;
    while let Some(event) = sync_manager.next_event() {
        cache.insert(event.clone());
        sync_manager.confirm_event(&event);
        applied += 1;
    }
    
    let elapsed = start.elapsed();
    
    assert_eq!(applied, event_count);
    assert!(sync_manager.is_synced());
    
    let rate = event_count as f64 / elapsed.as_secs_f64();
    println!("{} events synced in {:?} ({:.0} events/sec)", event_count, elapsed, rate);
    
    assert!(rate > 10000.0, "Should process at least 10k events/sec");
}

/// Stress test: Rapid connect/disconnect cycles.
#[tokio::test]
#[ignore]
async fn stress_test_connection_churn() {
    let pool = ConnectionPool::new(PoolConfig {
        max_connections: 50,
        auto_reconnect: true,
        ..Default::default()
    });
    
    let start = Instant::now();
    let cycles = 500;
    
    for cycle in 0..cycles {
        let peer_id = format!("churn_peer_{}", cycle % 50);
        
        // Remove if exists
        pool.remove_connection(&peer_id).await;
        
        // Add and connect
        if pool.add_connection(peer_id.clone(), "localhost:4433".to_string()).await.is_ok() {
            pool.mark_connected(&peer_id).await;
            pool.record_success(&peer_id).await;
        }
    }
    
    let elapsed = start.elapsed();
    let rate = cycles as f64 / elapsed.as_secs_f64();
    
    println!("{} connection cycles in {:?} ({:.0} cycles/sec)", cycles, elapsed, rate);
    
    assert!(rate > 100.0, "Should handle rapid connection changes");
}

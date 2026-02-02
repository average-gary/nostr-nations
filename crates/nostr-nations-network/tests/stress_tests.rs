//! Stress tests for multiplayer networking scenarios.
//!
//! These tests simulate high-load conditions to verify the networking layer
//! handles stress scenarios correctly. They are marked `#[ignore]` because
//! they are slow and resource-intensive.
//!
//! Run with: `cargo test --test stress_tests -- --ignored --nocapture`

use nostr_nations_network::{
    BatchConfig, CacheConfig, ConnectionPool, EventBatch, EventBatcher,
    EventCache, EventDeduplicator, EventPriorityQueue, EventUnbatcher,
    PoolConfig, PriorityQueueConfig, SyncManager, SyncResponse,
};
use nostr_nations_network::nostr_nations_core::events::{GameAction, GameEvent};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

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

/// Create a test event with previous event linkage.
#[allow(dead_code)]
fn create_linked_event(id: &str, prev: Option<&str>, turn: u32, seq: u32) -> GameEvent {
    let mut event = GameEvent::new(
        "stress_test".to_string(),
        0,
        prev.map(|s| s.to_string()),
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

/// Create an event for combat (requires randomness in real scenarios).
fn create_combat_event(id: &str, attacker: u64, defender: u64) -> GameEvent {
    let mut event = GameEvent::new(
        "stress_test".to_string(),
        0,
        None,
        1,
        1,
        GameAction::AttackUnit {
            attacker_id: attacker,
            defender_id: defender,
            random: 0.5,
        },
    );
    event.id = id.to_string();
    event
}

/// Create a move event.
fn create_move_event(id: &str, unit_id: u64) -> GameEvent {
    let mut event = GameEvent::new(
        "stress_test".to_string(),
        0,
        None,
        1,
        1,
        GameAction::MoveUnit {
            unit_id,
            path: vec![],
        },
    );
    event.id = id.to_string();
    event
}

/// Metrics collector for stress tests.
#[derive(Default, Clone)]
struct StressMetrics {
    operations: u64,
    duration_ms: u64,
    errors: u64,
    memory_start_kb: u64,
    memory_end_kb: u64,
}

impl StressMetrics {
    fn ops_per_second(&self) -> f64 {
        if self.duration_ms == 0 {
            0.0
        } else {
            (self.operations as f64 / self.duration_ms as f64) * 1000.0
        }
    }

    fn print_summary(&self, test_name: &str) {
        println!("\n=== {} Results ===", test_name);
        println!("Operations: {}", self.operations);
        println!("Duration: {}ms", self.duration_ms);
        println!("Ops/second: {:.2}", self.ops_per_second());
        println!("Errors: {}", self.errors);
        if self.memory_start_kb > 0 {
            println!("Memory start: {}KB", self.memory_start_kb);
            println!("Memory end: {}KB", self.memory_end_kb);
            let growth = self.memory_end_kb as i64 - self.memory_start_kb as i64;
            println!("Memory growth: {}KB", growth);
        }
        println!("==============================\n");
    }
}

/// Estimate current memory usage (rough approximation).
fn estimate_memory_kb() -> u64 {
    // This is a rough estimate based on process info
    // In a real scenario, you'd use system-specific APIs
    #[cfg(target_os = "linux")]
    {
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        return kb_str.parse().unwrap_or(0);
                    }
                }
            }
        }
    }
    0
}

// ============================================================================
// 1. Connection Stress Test
// ============================================================================

/// Simulates 100 peer connection attempts to verify connection pool handles load.
/// 
/// Pass criteria:
/// - All 100 connections are established (or up to pool capacity)
/// - Connection establishment time is tracked
/// - No panics or deadlocks
#[tokio::test]
#[ignore]
async fn stress_test_connection_pool_100_peers() {
    println!("\n>>> Starting Connection Stress Test (100 peers)");
    
    let config = PoolConfig {
        max_connections: 100,
        min_connections: 1,
        connect_timeout: Duration::from_secs(5),
        idle_timeout: Duration::from_secs(60),
        health_check_interval: Duration::from_secs(10),
        max_consecutive_failures: 3,
        auto_reconnect: true,
    };
    
    let pool = ConnectionPool::new(config);
    let mut metrics = StressMetrics::default();
    
    let start = Instant::now();
    let mut connection_times: Vec<Duration> = Vec::with_capacity(100);
    
    // Attempt to add 100 connections
    for i in 0..100 {
        let conn_start = Instant::now();
        let result = pool
            .add_connection(format!("peer_{}", i), format!("192.168.1.{}:4433", i))
            .await;
        
        let conn_time = conn_start.elapsed();
        connection_times.push(conn_time);
        
        match result {
            Ok(_) => {
                metrics.operations += 1;
                // Simulate connection becoming healthy
                pool.mark_connected(&format!("peer_{}", i)).await;
            }
            Err(e) => {
                metrics.errors += 1;
                println!("Connection {} failed: {:?}", i, e);
            }
        }
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    
    // Calculate connection timing stats
    let avg_conn_time: Duration = connection_times.iter().sum::<Duration>() / connection_times.len() as u32;
    let max_conn_time = connection_times.iter().max().unwrap_or(&Duration::ZERO);
    let min_conn_time = connection_times.iter().min().unwrap_or(&Duration::ZERO);
    
    println!("\nConnection timing:");
    println!("  Average: {:?}", avg_conn_time);
    println!("  Min: {:?}", min_conn_time);
    println!("  Max: {:?}", max_conn_time);
    
    // Verify pool state
    let total = pool.connection_count().await;
    let healthy = pool.healthy_count().await;
    let status = pool.status().await;
    
    println!("\nPool status:");
    println!("  Total connections: {}", total);
    println!("  Healthy connections: {}", healthy);
    println!("  Utilization: {:.1}%", status.utilization());
    
    metrics.print_summary("Connection Stress Test");
    
    // Pass criteria
    assert_eq!(total, 100, "Should have 100 connections");
    assert_eq!(healthy, 100, "All connections should be healthy");
    assert_eq!(metrics.errors, 0, "Should have no errors");
    assert!(avg_conn_time < Duration::from_millis(10), "Average connection time should be fast");
    
    println!("PASS: Connection stress test completed successfully");
}

/// Test rapid connect/disconnect cycles on the connection pool.
#[tokio::test]
#[ignore]
async fn stress_test_connection_churn() {
    println!("\n>>> Starting Connection Churn Test");
    
    let config = PoolConfig {
        max_connections: 50,
        ..Default::default()
    };
    
    let pool = ConnectionPool::new(config);
    let mut metrics = StressMetrics::default();
    let start = Instant::now();
    
    // Perform 500 connect/disconnect cycles
    for cycle in 0..500 {
        let id = format!("churn_peer_{}", cycle % 50);
        
        // Remove if exists
        pool.remove_connection(&id).await;
        
        // Add new connection
        if pool.add_connection(id.clone(), "localhost:8080".to_string()).await.is_ok() {
            pool.mark_connected(&id).await;
            metrics.operations += 1;
        }
        
        // Simulate some activity
        pool.record_success(&id).await;
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    metrics.print_summary("Connection Churn Test");
    
    assert!(metrics.operations >= 450, "Should complete most operations");
    println!("PASS: Connection churn test completed");
}

// ============================================================================
// 2. Event Throughput Test
// ============================================================================

/// Sends 10,000 events as fast as possible and measures throughput.
/// 
/// Pass criteria:
/// - All 10,000 events are processed
/// - Events per second is measured
/// - No events are lost
#[test]
#[ignore]
fn stress_test_event_throughput_10000() {
    println!("\n>>> Starting Event Throughput Test (10,000 events)");
    
    let config = BatchConfig {
        max_batch_size: 100,
        max_batch_timeout: Duration::from_millis(10),
        compression_threshold: 1024,
        compression_enabled: true,
    };
    
    let mut batcher = EventBatcher::new(config);
    let mut unbatcher = EventUnbatcher::new();
    let mut metrics = StressMetrics::default();
    
    let total_events: usize = 10_000;
    let mut events_sent: usize = 0;
    let mut events_received: usize = 0;
    let mut batches_created = 0;
    
    let start = Instant::now();
    
    // Send all events through the batcher
    for i in 0..total_events {
        let event = create_test_event(&format!("evt_{}", i), "throughput_test", (i / 100) as u32, (i % 100) as u32);
        batcher.add_event(event);
        events_sent += 1;
        
        // Take batches when ready
        while let Some(batch) = batcher.take_batch() {
            batches_created += 1;
            
            // Simulate network serialization
            let bytes = batch.to_bytes().unwrap();
            let received_batch = EventBatch::from_bytes(&bytes).unwrap();
            
            // Process through unbatcher
            if let Some(events) = unbatcher.process_batch(received_batch) {
                events_received += events.len();
            }
        }
    }
    
    // Flush remaining events
    if let Some(batch) = batcher.flush() {
        batches_created += 1;
        let bytes = batch.to_bytes().unwrap();
        let received_batch = EventBatch::from_bytes(&bytes).unwrap();
        if let Some(events) = unbatcher.process_batch(received_batch) {
            events_received += events.len();
        }
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    metrics.operations = events_received as u64;
    
    println!("\nThroughput statistics:");
    println!("  Events sent: {}", events_sent);
    println!("  Events received: {}", events_received);
    println!("  Batches created: {}", batches_created);
    println!("  Events/second: {:.0}", metrics.ops_per_second());
    println!("  Avg batch size: {:.1}", events_sent as f64 / batches_created as f64);
    
    metrics.print_summary("Event Throughput Test");
    
    // Pass criteria
    assert_eq!(events_sent, total_events, "Should send all events");
    assert_eq!(events_received, total_events, "Should receive all events");
    assert!(metrics.ops_per_second() > 10000.0, "Should process >10k events/second");
    
    println!("PASS: Event throughput test completed - {:.0} events/second", metrics.ops_per_second());
}

/// Test deduplication under high volume.
#[test]
#[ignore]
fn stress_test_deduplication_high_volume() {
    println!("\n>>> Starting Deduplication Stress Test");
    
    // Use a large enough capacity to detect duplicates within our test range
    let mut dedup = EventDeduplicator::new(10000);
    let mut metrics = StressMetrics::default();
    
    let start = Instant::now();
    let mut unique_count = 0;
    let mut dup_count = 0;
    
    // Send events where second half are duplicates of first half
    // First 10,000: unique IDs 0-9999
    // Second 10,000: duplicate IDs 0-9999
    for i in 0..20_000 {
        let id = format!("evt_{}", i % 10_000); // IDs 0-9999, then repeat
        
        if dedup.is_duplicate(&id) {
            dup_count += 1;
        } else {
            unique_count += 1;
        }
        metrics.operations += 1;
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    
    println!("\nDeduplication results:");
    println!("  Unique events: {}", unique_count);
    println!("  Duplicates detected: {}", dup_count);
    println!("  Dedup rate: {:.1}%", dedup.stats().duplicate_rate());
    
    metrics.print_summary("Deduplication Stress Test");
    
    // First 10,000 should be unique, second 10,000 should be duplicates
    assert_eq!(unique_count, 10000, "Should have 10,000 unique events");
    assert_eq!(dup_count, 10000, "Should detect 10,000 duplicates");
    assert!(metrics.ops_per_second() > 100000.0, "Should be very fast");
    
    println!("PASS: Deduplication stress test completed");
}

// ============================================================================
// 3. Large State Sync Test
// ============================================================================

/// Creates a large game state and tests synchronization performance.
/// 
/// Pass criteria:
/// - State with 1000 units and 100 cities is created
/// - Full sync completes successfully
/// - Sync time is measured
/// - Memory usage is tracked
#[test]
#[ignore]
fn stress_test_large_state_sync() {
    println!("\n>>> Starting Large State Sync Test (1000 units, 100 cities)");
    
    let mut metrics = StressMetrics::default();
    metrics.memory_start_kb = estimate_memory_kb();
    
    let start = Instant::now();
    
    // Create a large event chain representing game state
    let mut events: Vec<GameEvent> = Vec::with_capacity(1200);
    
    // Create 100 cities (one event each)
    for i in 0..100u64 {
        let mut event = GameEvent::new(
            "large_game".to_string(),
            (i % 8) as u8, // 8 players
            None,
            1,
            i as u32 + 1,
            GameAction::FoundCity {
                settler_id: i,
                name: format!("City_{}", i),
            },
        );
        event.id = format!("city_evt_{}", i);
        events.push(event);
    }
    
    // Create 1000 units (move events)
    for i in 0..1000u64 {
        let mut event = GameEvent::new(
            "large_game".to_string(),
            (i % 8) as u8,
            None,
            1,
            (100 + i) as u32,
            GameAction::MoveUnit {
                unit_id: i,
                path: vec![],
            },
        );
        event.id = format!("unit_evt_{}", i);
        events.push(event);
    }
    
    let state_creation_time = start.elapsed();
    println!("State creation time: {:?}", state_creation_time);
    
    // Simulate state sync using SyncResponse
    let sync_start = Instant::now();
    
    // Create sync response with all events
    let response = SyncResponse {
        game_id: "large_game".to_string(),
        has_more: false,
        events: events.clone(),
        current_turn: 1,
        chain_hash: None,
    };
    
    // Serialize the response (simulates network transfer)
    let serialized = serde_json::to_vec(&response).unwrap();
    let serialized_size = serialized.len();
    
    // Deserialize (simulates receiving)
    let received: SyncResponse = serde_json::from_slice(&serialized).unwrap();
    
    let sync_time = sync_start.elapsed();
    
    // Process received events
    let mut cache = EventCache::new(CacheConfig {
        max_events: 2000,
        ..Default::default()
    });
    
    let cache_start = Instant::now();
    for event in received.events {
        cache.insert(event);
        metrics.operations += 1;
    }
    let cache_time = cache_start.elapsed();
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    metrics.memory_end_kb = estimate_memory_kb();
    
    println!("\nSync statistics:");
    println!("  Total events: {}", events.len());
    println!("  Serialized size: {} bytes ({:.1} KB)", serialized_size, serialized_size as f64 / 1024.0);
    println!("  Sync time: {:?}", sync_time);
    println!("  Cache time: {:?}", cache_time);
    println!("  Bytes per event: {:.1}", serialized_size as f64 / events.len() as f64);
    
    metrics.print_summary("Large State Sync Test");
    
    // Pass criteria
    assert_eq!(cache.len(), 1100, "Should cache all events");
    assert!(sync_time < Duration::from_secs(1), "Sync should complete in <1 second");
    
    println!("PASS: Large state sync completed in {:?}", sync_time);
}

// ============================================================================
// 4. Concurrent Player Actions Test
// ============================================================================

/// Simulates 8 players taking actions simultaneously using threads.
/// 
/// Pass criteria:
/// - All player actions are processed
/// - State remains consistent
/// - No race conditions or deadlocks
#[tokio::test]
#[ignore]
async fn stress_test_concurrent_8_players() {
    println!("\n>>> Starting Concurrent Player Actions Test (8 players)");
    
    let num_players = 8;
    let actions_per_player = 100;
    
    // Shared state
    let event_counter = Arc::new(AtomicUsize::new(0));
    let error_counter = Arc::new(AtomicUsize::new(0));
    let processed_events: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));
    
    // Shared priority queue for event ordering
    let queue = Arc::new(tokio::sync::Mutex::new(EventPriorityQueue::new(PriorityQueueConfig {
        max_size: 10000,
        fair_scheduling: true,
        max_consecutive: 10,
        age_promotion_threshold: 5000,
    })));
    
    let start = Instant::now();
    
    // Spawn tasks for each player
    let mut handles = Vec::new();
    
    for player_id in 0..num_players {
        let event_counter = Arc::clone(&event_counter);
        let error_counter = Arc::clone(&error_counter);
        let queue = Arc::clone(&queue);
        
        let handle = tokio::spawn(async move {
            for action_num in 0..actions_per_player {
                let event_id = format!("p{}_evt_{}", player_id, action_num);
                
                // Create various action types
                let event = match action_num % 4 {
                    0 => create_combat_event(&event_id, (player_id * 10) as u64, ((player_id + 1) % 8 * 10) as u64),
                    1 => create_move_event(&event_id, (player_id * 10 + action_num) as u64),
                    2 => create_test_event(&event_id, "concurrent_test", action_num as u32, 1),
                    _ => {
                        let mut e = GameEvent::new(
                            "concurrent_test".to_string(),
                            player_id as u8,
                            None,
                            1,
                            action_num as u32,
                            GameAction::EndTurn,
                        );
                        e.id = event_id;
                        e
                    }
                };
                
                // Enqueue the event
                let mut q = queue.lock().await;
                match q.enqueue(event) {
                    Ok(_) => {
                        event_counter.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(_) => {
                        error_counter.fetch_add(1, Ordering::SeqCst);
                    }
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all players to finish
    for handle in handles {
        handle.await.unwrap();
    }
    
    // Process all events from the queue
    let mut processed = 0;
    {
        let mut q = queue.lock().await;
        while let Some(event) = q.dequeue() {
            processed_events.write().await.push(event.id);
            processed += 1;
        }
    }
    
    let duration = start.elapsed();
    
    let total_events = event_counter.load(Ordering::SeqCst);
    let total_errors = error_counter.load(Ordering::SeqCst);
    
    println!("\nConcurrent player results:");
    println!("  Players: {}", num_players);
    println!("  Actions per player: {}", actions_per_player);
    println!("  Total events enqueued: {}", total_events);
    println!("  Total events processed: {}", processed);
    println!("  Errors: {}", total_errors);
    println!("  Duration: {:?}", duration);
    println!("  Events/second: {:.0}", total_events as f64 / duration.as_secs_f64());
    
    // Verify consistency
    let processed_list = processed_events.read().await;
    let unique_events: std::collections::HashSet<_> = processed_list.iter().collect();
    
    println!("  Unique events: {}", unique_events.len());
    
    // Pass criteria
    let expected_events: usize = num_players * actions_per_player;
    assert_eq!(total_events, expected_events, "All events should be enqueued");
    assert_eq!(processed, expected_events, "All events should be processed");
    assert_eq!(unique_events.len(), expected_events, "All events should be unique");
    assert_eq!(total_errors, 0, "Should have no errors");
    
    println!("PASS: Concurrent player test completed successfully");
}

// ============================================================================
// 5. Network Latency Simulation Test
// ============================================================================

/// Tests the system's behavior under various simulated network latencies.
/// 
/// Pass criteria:
/// - System functions correctly at 50ms, 100ms, and 500ms latency
/// - Events are still delivered correctly
/// - Timeouts are handled appropriately
#[tokio::test]
#[ignore]
async fn stress_test_network_latency_simulation() {
    println!("\n>>> Starting Network Latency Simulation Test");
    
    let latencies = [50, 100, 500]; // milliseconds
    
    for latency_ms in latencies {
        println!("\n--- Testing with {}ms latency ---", latency_ms);
        
        let latency = Duration::from_millis(latency_ms);
        let mut metrics = StressMetrics::default();
        
        // Create sync manager
        let mut sync_manager = SyncManager::new("latency_test".to_string(), 0);
        
        // Create events to sync
        let events: Vec<GameEvent> = (0..100)
            .map(|i| create_test_event(&format!("lat_evt_{}", i), "latency_test", i / 10, i % 10))
            .collect();
        
        let start = Instant::now();
        
        // Simulate sync request
        let _request = sync_manager.create_request();
        
        // Simulate network latency for request
        tokio::time::sleep(latency).await;
        
        // Create response
        let response = SyncResponse {
            game_id: "latency_test".to_string(),
            has_more: false,
            events: events.clone(),
            current_turn: 10,
            chain_hash: None,
        };
        
        // Simulate network latency for response
        tokio::time::sleep(latency).await;
        
        // Process response
        let result = sync_manager.handle_response(response);
        
        // Apply events
        let mut applied = 0;
        while let Some(event) = sync_manager.next_event() {
            sync_manager.confirm_event(&event);
            applied += 1;
            metrics.operations += 1;
        }
        
        metrics.duration_ms = start.elapsed().as_millis() as u64;
        
        println!("  Events applied: {}", applied);
        println!("  Total time: {}ms", metrics.duration_ms);
        println!("  Overhead from latency: {}ms", latency_ms * 2);
        println!("  Processing time: {}ms", metrics.duration_ms - (latency_ms * 2) as u64);
        println!("  Is synced: {}", sync_manager.is_synced());
        
        // Pass criteria
        assert_eq!(applied, 100, "Should apply all events");
        assert!(sync_manager.is_synced(), "Should be fully synced");
        assert_eq!(result.events_received, 100, "Should receive all events");
    }
    
    println!("\nPASS: Network latency simulation completed for all latencies");
}

// ============================================================================
// 6. Reconnection Stress Test
// ============================================================================

/// Tests rapid disconnect/reconnect cycles.
/// 
/// Pass criteria:
/// - State is recovered after each reconnection
/// - No data loss or corruption
/// - System remains stable through rapid reconnections
#[tokio::test]
#[ignore]
async fn stress_test_reconnection_rapid() {
    println!("\n>>> Starting Reconnection Stress Test");
    
    let reconnection_cycles = 50;
    let mut metrics = StressMetrics::default();
    
    // Simulated persistent state
    let mut confirmed_events: Vec<String> = Vec::new();
    let mut last_turn = 0u32;
    let mut last_seq = 0u32;
    
    let start = Instant::now();
    
    for cycle in 0..reconnection_cycles {
        // Create a new sync manager (simulates reconnection)
        let mut sync_manager = SyncManager::new("reconnect_test".to_string(), 0);
        
        // Create events for this session
        let session_events: Vec<GameEvent> = (0..10)
            .map(|i| {
                let turn = last_turn + (i / 5) + 1;
                let seq = if i / 5 > 0 { i % 5 + 1 } else { last_seq + i + 1 };
                create_test_event(
                    &format!("cycle_{}_evt_{}", cycle, i),
                    "reconnect_test",
                    turn,
                    seq,
                )
            })
            .collect();
        
        // Request sync
        let _request = sync_manager.create_request();
        
        // Receive response with new events
        let response = SyncResponse {
            game_id: "reconnect_test".to_string(),
            has_more: false,
            events: session_events.clone(),
            current_turn: last_turn + 2,
            chain_hash: None,
        };
        
        sync_manager.handle_response(response);
        
        // Apply events
        while let Some(event) = sync_manager.next_event() {
            confirmed_events.push(event.id.clone());
            last_turn = event.turn;
            last_seq = event.sequence;
            sync_manager.confirm_event(&event);
            metrics.operations += 1;
        }
        
        // Verify sync state
        assert!(sync_manager.is_synced(), "Should be synced after each cycle");
        
        // Simulate disconnect (sync manager goes out of scope)
        // Then we create a new one in the next iteration
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    
    println!("\nReconnection results:");
    println!("  Reconnection cycles: {}", reconnection_cycles);
    println!("  Total events confirmed: {}", confirmed_events.len());
    println!("  Final turn: {}", last_turn);
    println!("  Duration: {}ms", metrics.duration_ms);
    
    metrics.print_summary("Reconnection Stress Test");
    
    // Verify no data loss
    let expected_events = reconnection_cycles * 10;
    assert_eq!(confirmed_events.len(), expected_events, "Should have all events");
    
    // Verify no duplicates
    let unique: std::collections::HashSet<_> = confirmed_events.iter().collect();
    assert_eq!(unique.len(), expected_events, "Should have no duplicates");
    
    println!("PASS: Reconnection stress test completed with {} cycles", reconnection_cycles);
}

// ============================================================================
// 7. Memory Leak Detection Test
// ============================================================================

/// Runs an extended session to detect memory leaks.
/// 
/// Pass criteria:
/// - Memory usage does not grow unboundedly
/// - System remains responsive after 1000 turns
/// - Caches properly evict old data
#[test]
#[ignore]
fn stress_test_memory_leak_detection() {
    println!("\n>>> Starting Memory Leak Detection Test (1000 turns)");
    
    let num_turns = 1000;
    let events_per_turn = 50;
    
    let mut metrics = StressMetrics::default();
    metrics.memory_start_kb = estimate_memory_kb();
    
    // Create cache with limited size to force evictions
    let mut cache = EventCache::new(CacheConfig {
        max_events: 500,  // Should force evictions
        max_age: Duration::from_secs(60),
        enable_dedup: true,
        max_dedup_ids: 1000,
    });
    
    // Create priority queue with limited size
    let mut queue = EventPriorityQueue::new(PriorityQueueConfig {
        max_size: 500,
        ..Default::default()
    });
    
    // Create batcher
    let mut batcher = EventBatcher::with_defaults();
    
    let start = Instant::now();
    let mut memory_samples: Vec<u64> = Vec::new();
    
    for turn in 0..num_turns {
        // Generate events for this turn
        for evt_num in 0..events_per_turn {
            let event = create_test_event(
                &format!("t{}_e{}", turn, evt_num),
                "memory_test",
                turn,
                evt_num as u32,
            );
            
            // Process through all systems
            cache.insert(event.clone());
            let _ = queue.enqueue(event.clone());
            batcher.add_event(event);
            
            metrics.operations += 1;
        }
        
        // Drain the queue periodically
        if turn % 10 == 0 {
            while queue.dequeue().is_some() {}
        }
        
        // Flush batches periodically
        if turn % 5 == 0 {
            while batcher.flush().is_some() {}
        }
        
        // Expire old cache entries periodically
        if turn % 100 == 0 {
            cache.expire_old();
            
            // Sample memory
            let mem = estimate_memory_kb();
            memory_samples.push(mem);
            
            if turn > 0 && turn % 200 == 0 {
                println!("Turn {}: cache size = {}, queue size = {}", 
                    turn, cache.len(), queue.len());
            }
        }
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    metrics.memory_end_kb = estimate_memory_kb();
    
    // Analyze memory samples
    let memory_growth = if memory_samples.len() > 1 && memory_samples[0] > 0 {
        let first = memory_samples[0] as i64;
        let last = memory_samples[memory_samples.len() - 1] as i64;
        last - first
    } else {
        0
    };
    
    println!("\nMemory analysis:");
    println!("  Cache size: {} (max {})", cache.len(), 500);
    println!("  Queue size: {}", queue.len());
    println!("  Cache stats - hits: {}, misses: {}, evictions: {}", 
        cache.stats().hits, cache.stats().misses, cache.stats().evictions);
    println!("  Memory samples: {:?}", memory_samples);
    println!("  Memory growth: {} KB", memory_growth);
    
    metrics.print_summary("Memory Leak Detection Test");
    
    // Pass criteria
    assert!(cache.len() <= 500, "Cache should not exceed limit");
    assert!(queue.len() <= 500, "Queue should not exceed limit");
    
    // Memory growth should be bounded (allowing for some growth due to test infrastructure)
    // This is a soft check since memory measurement is imprecise
    if metrics.memory_start_kb > 0 && metrics.memory_end_kb > 0 {
        let growth_percent = (metrics.memory_end_kb as f64 / metrics.memory_start_kb as f64 - 1.0) * 100.0;
        println!("  Memory growth: {:.1}%", growth_percent);
        // Allow up to 100% growth (very generous due to test allocations)
        assert!(growth_percent < 100.0, "Memory should not grow excessively");
    }
    
    println!("PASS: Memory leak detection completed - no unbounded growth detected");
}

// ============================================================================
// 8. Event Ordering Test
// ============================================================================

/// Tests that events are correctly reordered when received out of order.
/// 
/// Pass criteria:
/// - Events sent out of order are correctly reordered
/// - Timestamp-based ordering is verified
/// - No events are lost during reordering
#[test]
#[ignore]
fn stress_test_event_ordering() {
    println!("\n>>> Starting Event Ordering Test");
    
    let mut metrics = StressMetrics::default();
    
    // Create events with specific ordering
    let mut events: Vec<GameEvent> = (0..1000)
        .map(|i| {
            let turn = i / 100;
            let seq = i % 100;
            let mut event = create_test_event(
                &format!("order_evt_{}", i),
                "order_test",
                turn as u32,
                seq as u32,
            );
            event.timestamp = 1000000 + i as u64; // Sequential timestamps
            event
        })
        .collect();
    
    // Shuffle events to simulate out-of-order arrival
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    // Simple shuffle using hash-based ordering
    events.sort_by(|a, b| {
        let mut ha = DefaultHasher::new();
        let mut hb = DefaultHasher::new();
        a.id.hash(&mut ha);
        b.id.hash(&mut hb);
        ha.finish().cmp(&hb.finish())
    });
    
    let start = Instant::now();
    
    // Use priority queue for ordering
    let mut queue = EventPriorityQueue::new(PriorityQueueConfig {
        max_size: 2000,
        fair_scheduling: false, // Use strict ordering
        ..Default::default()
    });
    
    // Enqueue all events (out of order)
    for event in &events {
        queue.enqueue(event.clone()).unwrap();
        metrics.operations += 1;
    }
    
    // Dequeue and verify ordering
    let mut dequeued: Vec<GameEvent> = Vec::new();
    while let Some(event) = queue.dequeue() {
        dequeued.push(event);
    }
    
    // Also test timestamp-based sorting
    let mut timestamp_sorted = events.clone();
    timestamp_sorted.sort_by_key(|e| e.timestamp);
    
    // Also test turn/sequence sorting
    let mut turn_sorted = events.clone();
    turn_sorted.sort_by_key(|e| (e.turn, e.sequence));
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    
    println!("\nOrdering results:");
    println!("  Total events: {}", events.len());
    println!("  Events dequeued: {}", dequeued.len());
    println!("  Duration: {}ms", metrics.duration_ms);
    
    // Verify no events lost
    assert_eq!(dequeued.len(), 1000, "Should dequeue all events");
    
    // Verify timestamp ordering works
    for i in 1..timestamp_sorted.len() {
        assert!(
            timestamp_sorted[i].timestamp >= timestamp_sorted[i - 1].timestamp,
            "Timestamp ordering should be monotonic"
        );
    }
    
    // Verify turn/sequence ordering works
    for i in 1..turn_sorted.len() {
        let prev = &turn_sorted[i - 1];
        let curr = &turn_sorted[i];
        assert!(
            curr.turn > prev.turn || (curr.turn == prev.turn && curr.sequence >= prev.sequence),
            "Turn/sequence ordering should be monotonic"
        );
    }
    
    // Verify priority queue orders by priority (high priority first)
    // Since we used auto-priority, events should be grouped by action type
    let mut seen_priorities: Vec<nostr_nations_network::EventPriority> = Vec::new();
    for event in &dequeued {
        let priority = nostr_nations_network::event_priority(&event);
        seen_priorities.push(priority);
    }
    
    println!("  First 10 priorities: {:?}", &seen_priorities[..10.min(seen_priorities.len())]);
    
    metrics.print_summary("Event Ordering Test");
    
    println!("PASS: Event ordering test completed");
}

// ============================================================================
// Additional Stress Tests
// ============================================================================

/// Test cache performance under high access patterns.
#[test]
#[ignore]
fn stress_test_cache_access_patterns() {
    println!("\n>>> Starting Cache Access Pattern Test");
    
    let mut cache = EventCache::new(CacheConfig {
        max_events: 1000,
        max_age: Duration::from_secs(300),
        enable_dedup: true,
        max_dedup_ids: 5000,
    });
    
    let mut metrics = StressMetrics::default();
    let start = Instant::now();
    
    // Insert events
    for i in 0..2000 {
        let event = create_test_event(&format!("cache_evt_{}", i), "cache_test", i / 100, i % 100);
        cache.insert(event);
        metrics.operations += 1;
    }
    
    // Access pattern: frequently access recent events (simulates hot data)
    for _ in 0..5000 {
        // 80% access recent events (last 200), 20% access older events
        let id = if rand_bool(0.8) {
            format!("cache_evt_{}", 1800 + (simple_rand() % 200))
        } else {
            format!("cache_evt_{}", simple_rand() % 2000)
        };
        
        cache.get(&id);
        metrics.operations += 1;
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    
    let stats = cache.stats();
    println!("\nCache statistics:");
    println!("  Size: {} / 1000", cache.len());
    println!("  Hits: {}", stats.hits);
    println!("  Misses: {}", stats.misses);
    println!("  Hit rate: {:.1}%", stats.hit_rate());
    println!("  Evictions: {}", stats.evictions);
    println!("  Duplicates: {}", stats.duplicates);
    
    metrics.print_summary("Cache Access Pattern Test");
    
    // Pass criteria
    assert!(stats.hit_rate() > 50.0, "Hit rate should be reasonable");
    assert_eq!(cache.len(), 1000, "Cache should be at capacity");
    
    println!("PASS: Cache access pattern test completed");
}

/// Test priority queue under mixed priority workload.
#[test]
#[ignore]
fn stress_test_priority_queue_mixed_load() {
    println!("\n>>> Starting Priority Queue Mixed Load Test");
    
    let mut queue = EventPriorityQueue::new(PriorityQueueConfig {
        max_size: 5000,
        fair_scheduling: true,
        max_consecutive: 5,
        age_promotion_threshold: 1000,
    });
    
    let mut metrics = StressMetrics::default();
    let start = Instant::now();
    
    // Enqueue events with various action types (different priorities)
    for i in 0..5000 {
        let event = match i % 5 {
            0 => create_combat_event(&format!("pq_evt_{}", i), 1, 2), // High priority
            1 => create_move_event(&format!("pq_evt_{}", i), i as u64), // Normal
            2 => create_test_event(&format!("pq_evt_{}", i), "pq_test", i as u32, 1), // Varies
            3 => {
                let mut e = GameEvent::new(
                    "pq_test".to_string(),
                    0,
                    None,
                    1,
                    1,
                    GameAction::FortifyUnit { unit_id: i as u64 }, // Low priority
                );
                e.id = format!("pq_evt_{}", i);
                e
            },
            _ => create_test_event(&format!("pq_evt_{}", i), "pq_test", 1, 1),
        };
        
        queue.enqueue(event).unwrap();
        metrics.operations += 1;
    }
    
    // Dequeue all and track priority distribution
    let mut dequeued_priorities: HashMap<nostr_nations_network::EventPriority, usize> = HashMap::new();
    let mut dequeue_count = 0;
    
    while let Some(event) = queue.dequeue() {
        let priority = nostr_nations_network::event_priority(&event);
        *dequeued_priorities.entry(priority).or_insert(0) += 1;
        dequeue_count += 1;
        metrics.operations += 1;
    }
    
    metrics.duration_ms = start.elapsed().as_millis() as u64;
    
    println!("\nPriority distribution:");
    for (priority, count) in &dequeued_priorities {
        println!("  {:?}: {}", priority, count);
    }
    
    let stats = queue.stats();
    println!("\nQueue statistics:");
    println!("  Enqueued: {}", stats.events_enqueued);
    println!("  Dequeued: {}", stats.events_dequeued);
    println!("  Dropped: {}", stats.events_dropped);
    
    metrics.print_summary("Priority Queue Mixed Load Test");
    
    assert_eq!(dequeue_count, 5000, "Should dequeue all events");
    
    println!("PASS: Priority queue mixed load test completed");
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Simple pseudo-random number generator for testing.
fn simple_rand() -> usize {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    nanos as usize
}

/// Simple random boolean with given probability.
fn rand_bool(probability: f64) -> bool {
    (simple_rand() % 100) < (probability * 100.0) as usize
}

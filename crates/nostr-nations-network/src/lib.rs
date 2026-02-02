//! Nostr Nations Networking Layer
//!
//! This crate handles all networking for Nostr Nations:
//! - **Iroh P2P**: Direct peer-to-peer connections for low-latency gameplay
//! - **Nostr Relays**: Event broadcasting and storage for game state
//! - **Local Relay**: Embedded Nostr relay for offline/local play
//!
//! # Architecture
//!
//! The networking layer supports multiple modes:
//! 1. **Full Client**: Local relay + P2P + optional remote relays
//! 2. **Light Client**: Connects to full client or remote relays only
//!
//! Game events are signed Nostr events that can be:
//! - Broadcast to relays for persistence
//! - Sent directly via Iroh for real-time sync
//! - Stored locally for offline play
//!
//! # Modules
//!
//! - [`peer`]: Peer connection management and messaging
//! - [`sync`]: Game state synchronization protocol
//! - [`discovery`]: Peer discovery and QR code generation
//! - [`batch`]: Event batching for reduced network overhead
//! - [`compression`]: Optional payload compression
//! - [`delta`]: Delta synchronization for incremental updates
//! - [`pool`]: Connection pooling with health monitoring
//! - [`priority`]: Priority queue for event scheduling
//! - [`cache`]: Event caching and deduplication
//! - [`conflict`]: Conflict detection and resolution for multiplayer sync
//! - [`encryption`]: NIP-04 style encryption for secure peer communication
//! - [`offline`]: Offline scenario handling, event queuing, and reconnection
//! - [`randomness`]: Verifiable randomness protocol for fair multiplayer games

// Re-export core types
pub use nostr_nations_core;

// Networking modules
pub mod peer;
pub mod sync;
pub mod discovery;
pub mod relay;
pub mod conflict;
pub mod encryption;
pub mod offline;
pub mod randomness;

// Optimization modules
pub mod batch;
pub mod compression;
pub mod delta;
pub mod pool;
pub mod priority;
pub mod cache;

// Re-exports for convenience
pub use peer::{
    ConnectionTicket, PeerManager, PeerMessage, PeerEvent, PeerInfo,
    ConnectionState, PeerId, TicketError,
};
pub use sync::{
    SyncManager, SyncResponder, SyncRequest, SyncResponse, SyncResult,
    SyncState, PeerSyncTracker,
};
pub use discovery::{
    QrCodeData, QrCodeMatrix, QrGenerator, QrParseError,
    DiscoveryService, ErrorCorrection,
};
pub use relay::{
    Filter, LocalRelay, RelayStorage, StorageError,
    Subscription, SubscriptionBuilder, SubscriptionManager,
};

// Optimization re-exports
pub use batch::{
    BatchConfig, EventBatch, EventBatcher, EventUnbatcher,
    BatchStats, UnbatchStats,
};
pub use compression::{
    CompressionAlgorithm, CompressionConfig, CompressedPayload,
    PayloadCompressor, CompressionStats, CompressionError,
};
pub use delta::{
    EntityType, EntityId, DirtyTracker, StateDelta, EntityChange,
    ChangeType, DeltaSyncManager, DeltaSyncStats, DeltaSyncError,
};
pub use pool::{
    PooledConnectionState, ConnectionHealth, PoolConfig, BackoffConfig,
    BackoffState, PooledConnection, ConnectionPool, PoolStats, PoolStatus, PoolError,
};
pub use priority::{
    EventPriority, PrioritizedEvent, PriorityQueueConfig,
    EventPriorityQueue, PriorityQueueStats, QueueError, event_priority,
};
pub use cache::{
    CacheConfig, CachedEvent, EventCache, CacheStats,
    EventDeduplicator, DedupStats, EventIndex,
};
pub use conflict::{
    ConflictType, ConflictDetector, ResolutionStrategy, Resolution,
    ConflictResolver, auto_resolve_conflicts,
};
pub use encryption::{
    EncryptionManager, EncryptedPayload, EncryptedGameEvent, EncryptionError,
    encrypt_for_player, decrypt_from_player, encrypt_event, decrypt_event,
    compute_shared_secret,
};
pub use offline::{
    OfflineManager, OfflineStorage, OfflineSyncStrategy, ConnectionMonitor,
    StorageError as OfflineStorageError,
};
pub use randomness::{
    RandomnessRequest, RandomnessResponse, RandomnessProof, RandomnessPurpose,
    RandomnessProvider, RandomnessClient, RandomnessError, RandomnessMessage,
    PlayerId as RandomnessPlayerId,
};

/// Network configuration
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    /// Enable Iroh P2P networking
    pub enable_p2p: bool,
    /// Nostr relay URLs to connect to
    pub relay_urls: Vec<String>,
    /// Enable local embedded relay
    pub enable_local_relay: bool,
    /// P2P listen port (0 for random)
    pub p2p_port: u16,
    /// Connection ticket TTL in seconds
    pub ticket_ttl_secs: u64,
    /// Maximum peers to connect to
    pub max_peers: usize,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            enable_p2p: true,
            relay_urls: Vec::new(),
            enable_local_relay: true,
            p2p_port: 0,
            ticket_ttl_secs: 3600, // 1 hour
            max_peers: 8,
        }
    }
}

/// Network mode for a client.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NetworkMode {
    /// Full client with local relay and P2P.
    Full,
    /// Light client that connects to a full client.
    Light,
    /// Offline mode with local relay only.
    Offline,
}

/// Network statistics.
#[derive(Clone, Debug, Default)]
pub struct NetworkStats {
    /// Number of connected peers.
    pub peer_count: usize,
    /// Total bytes sent.
    pub bytes_sent: u64,
    /// Total bytes received.
    pub bytes_received: u64,
    /// Number of events broadcast.
    pub events_broadcast: u64,
    /// Number of events received.
    pub events_received: u64,
    /// Average latency to peers in ms.
    pub avg_latency_ms: Option<u32>,
}

/// Initialize the networking subsystem.
///
/// This is a placeholder that will be implemented when Iroh integration
/// is completed.
pub fn init(config: &NetworkConfig) -> Result<NetworkHandle, NetworkError> {
    // Validate config
    if config.max_peers == 0 {
        return Err(NetworkError::InvalidConfig("max_peers must be > 0".to_string()));
    }

    Ok(NetworkHandle {
        config: config.clone(),
        mode: if config.enable_p2p {
            NetworkMode::Full
        } else if config.enable_local_relay {
            NetworkMode::Offline
        } else {
            NetworkMode::Light
        },
        stats: NetworkStats::default(),
    })
}

/// Handle to the network subsystem.
#[derive(Clone, Debug)]
pub struct NetworkHandle {
    /// Current configuration.
    pub config: NetworkConfig,
    /// Network mode.
    pub mode: NetworkMode,
    /// Statistics.
    pub stats: NetworkStats,
}

impl NetworkHandle {
    /// Get current network mode.
    pub fn mode(&self) -> &NetworkMode {
        &self.mode
    }

    /// Get network statistics.
    pub fn stats(&self) -> &NetworkStats {
        &self.stats
    }

    /// Check if P2P is enabled.
    pub fn is_p2p_enabled(&self) -> bool {
        self.config.enable_p2p
    }

    /// Check if local relay is enabled.
    pub fn is_local_relay_enabled(&self) -> bool {
        self.config.enable_local_relay
    }
}

/// Network errors.
#[derive(Clone, Debug)]
pub enum NetworkError {
    /// Invalid configuration.
    InvalidConfig(String),
    /// Connection failed.
    ConnectionFailed(String),
    /// Peer not found.
    PeerNotFound(String),
    /// Sync failed.
    SyncFailed(String),
    /// Iroh error.
    IrohError(String),
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
            NetworkError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            NetworkError::PeerNotFound(msg) => write!(f, "Peer not found: {}", msg),
            NetworkError::SyncFailed(msg) => write!(f, "Sync failed: {}", msg),
            NetworkError::IrohError(msg) => write!(f, "Iroh error: {}", msg),
        }
    }
}

impl std::error::Error for NetworkError {}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== NetworkConfig Tests ====================

    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert!(config.enable_p2p);
        assert!(config.enable_local_relay);
        assert_eq!(config.p2p_port, 0);
        assert_eq!(config.max_peers, 8);
        assert_eq!(config.ticket_ttl_secs, 3600);
        assert!(config.relay_urls.is_empty());
    }

    #[test]
    fn test_network_config_clone() {
        let config = NetworkConfig {
            enable_p2p: true,
            relay_urls: vec!["wss://relay.example.com".to_string()],
            enable_local_relay: false,
            p2p_port: 4433,
            ticket_ttl_secs: 7200,
            max_peers: 16,
        };

        let cloned = config.clone();
        assert_eq!(cloned.enable_p2p, config.enable_p2p);
        assert_eq!(cloned.relay_urls, config.relay_urls);
        assert_eq!(cloned.enable_local_relay, config.enable_local_relay);
        assert_eq!(cloned.p2p_port, config.p2p_port);
        assert_eq!(cloned.ticket_ttl_secs, config.ticket_ttl_secs);
        assert_eq!(cloned.max_peers, config.max_peers);
    }

    #[test]
    fn test_network_config_debug() {
        let config = NetworkConfig::default();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("NetworkConfig"));
        assert!(debug_str.contains("enable_p2p"));
    }

    #[test]
    fn test_network_config_with_relay_urls() {
        let config = NetworkConfig {
            relay_urls: vec![
                "wss://relay1.example.com".to_string(),
                "wss://relay2.example.com".to_string(),
            ],
            ..Default::default()
        };

        assert_eq!(config.relay_urls.len(), 2);
    }

    // ==================== NetworkMode Tests ====================

    #[test]
    fn test_network_mode_equality() {
        assert_eq!(NetworkMode::Full, NetworkMode::Full);
        assert_eq!(NetworkMode::Light, NetworkMode::Light);
        assert_eq!(NetworkMode::Offline, NetworkMode::Offline);

        assert_ne!(NetworkMode::Full, NetworkMode::Light);
        assert_ne!(NetworkMode::Full, NetworkMode::Offline);
        assert_ne!(NetworkMode::Light, NetworkMode::Offline);
    }

    #[test]
    fn test_network_mode_clone() {
        let mode = NetworkMode::Full;
        let cloned = mode.clone();
        assert_eq!(mode, cloned);
    }

    #[test]
    fn test_network_mode_debug() {
        assert!(format!("{:?}", NetworkMode::Full).contains("Full"));
        assert!(format!("{:?}", NetworkMode::Light).contains("Light"));
        assert!(format!("{:?}", NetworkMode::Offline).contains("Offline"));
    }

    // ==================== NetworkStats Tests ====================

    #[test]
    fn test_network_stats_default() {
        let stats = NetworkStats::default();
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.bytes_sent, 0);
        assert_eq!(stats.bytes_received, 0);
        assert_eq!(stats.events_broadcast, 0);
        assert_eq!(stats.events_received, 0);
        assert!(stats.avg_latency_ms.is_none());
    }

    #[test]
    fn test_network_stats_clone() {
        let stats = NetworkStats {
            peer_count: 5,
            bytes_sent: 1000,
            bytes_received: 2000,
            events_broadcast: 50,
            events_received: 100,
            avg_latency_ms: Some(25),
        };

        let cloned = stats.clone();
        assert_eq!(cloned.peer_count, stats.peer_count);
        assert_eq!(cloned.bytes_sent, stats.bytes_sent);
        assert_eq!(cloned.bytes_received, stats.bytes_received);
        assert_eq!(cloned.events_broadcast, stats.events_broadcast);
        assert_eq!(cloned.events_received, stats.events_received);
        assert_eq!(cloned.avg_latency_ms, stats.avg_latency_ms);
    }

    #[test]
    fn test_network_stats_debug() {
        let stats = NetworkStats::default();
        let debug_str = format!("{:?}", stats);
        assert!(debug_str.contains("NetworkStats"));
        assert!(debug_str.contains("peer_count"));
    }

    // ==================== init() Tests ====================

    #[test]
    fn test_network_init() {
        let config = NetworkConfig::default();
        let handle = init(&config).unwrap();
        assert_eq!(handle.mode(), &NetworkMode::Full);
    }

    #[test]
    fn test_network_init_offline() {
        let config = NetworkConfig {
            enable_p2p: false,
            enable_local_relay: true,
            ..Default::default()
        };
        let handle = init(&config).unwrap();
        assert_eq!(handle.mode(), &NetworkMode::Offline);
    }

    #[test]
    fn test_network_init_light() {
        let config = NetworkConfig {
            enable_p2p: false,
            enable_local_relay: false,
            relay_urls: vec!["wss://relay.example.com".to_string()],
            ..Default::default()
        };
        let handle = init(&config).unwrap();
        assert_eq!(handle.mode(), &NetworkMode::Light);
    }

    #[test]
    fn test_network_init_invalid() {
        let config = NetworkConfig {
            max_peers: 0,
            ..Default::default()
        };
        assert!(init(&config).is_err());
    }

    #[test]
    fn test_network_init_preserves_config() {
        let config = NetworkConfig {
            enable_p2p: true,
            relay_urls: vec!["wss://test.relay".to_string()],
            enable_local_relay: true,
            p2p_port: 12345,
            ticket_ttl_secs: 1800,
            max_peers: 4,
        };

        let handle = init(&config).unwrap();

        assert_eq!(handle.config.enable_p2p, config.enable_p2p);
        assert_eq!(handle.config.relay_urls, config.relay_urls);
        assert_eq!(handle.config.enable_local_relay, config.enable_local_relay);
        assert_eq!(handle.config.p2p_port, config.p2p_port);
        assert_eq!(handle.config.ticket_ttl_secs, config.ticket_ttl_secs);
        assert_eq!(handle.config.max_peers, config.max_peers);
    }

    // ==================== NetworkHandle Tests ====================

    #[test]
    fn test_network_handle_mode() {
        let config = NetworkConfig::default();
        let handle = init(&config).unwrap();

        assert_eq!(handle.mode(), &NetworkMode::Full);
    }

    #[test]
    fn test_network_handle_stats() {
        let config = NetworkConfig::default();
        let handle = init(&config).unwrap();

        let stats = handle.stats();
        assert_eq!(stats.peer_count, 0);
    }

    #[test]
    fn test_network_handle_is_p2p_enabled() {
        let config_p2p = NetworkConfig {
            enable_p2p: true,
            ..Default::default()
        };
        let handle_p2p = init(&config_p2p).unwrap();
        assert!(handle_p2p.is_p2p_enabled());

        let config_no_p2p = NetworkConfig {
            enable_p2p: false,
            ..Default::default()
        };
        let handle_no_p2p = init(&config_no_p2p).unwrap();
        assert!(!handle_no_p2p.is_p2p_enabled());
    }

    #[test]
    fn test_network_handle_is_local_relay_enabled() {
        let config_relay = NetworkConfig {
            enable_local_relay: true,
            ..Default::default()
        };
        let handle_relay = init(&config_relay).unwrap();
        assert!(handle_relay.is_local_relay_enabled());

        let config_no_relay = NetworkConfig {
            enable_local_relay: false,
            ..Default::default()
        };
        let handle_no_relay = init(&config_no_relay).unwrap();
        assert!(!handle_no_relay.is_local_relay_enabled());
    }

    #[test]
    fn test_network_handle_clone() {
        let config = NetworkConfig::default();
        let handle = init(&config).unwrap();
        let cloned = handle.clone();

        assert_eq!(handle.mode(), cloned.mode());
        assert_eq!(handle.config.max_peers, cloned.config.max_peers);
    }

    // ==================== NetworkError Tests ====================

    #[test]
    fn test_network_error_display() {
        let invalid_config = NetworkError::InvalidConfig("test error".to_string());
        assert_eq!(format!("{}", invalid_config), "Invalid config: test error");

        let connection_failed = NetworkError::ConnectionFailed("timeout".to_string());
        assert_eq!(format!("{}", connection_failed), "Connection failed: timeout");

        let peer_not_found = NetworkError::PeerNotFound("peer123".to_string());
        assert_eq!(format!("{}", peer_not_found), "Peer not found: peer123");

        let sync_failed = NetworkError::SyncFailed("chain mismatch".to_string());
        assert_eq!(format!("{}", sync_failed), "Sync failed: chain mismatch");

        let iroh_error = NetworkError::IrohError("connection reset".to_string());
        assert_eq!(format!("{}", iroh_error), "Iroh error: connection reset");
    }

    #[test]
    fn test_network_error_debug() {
        let error = NetworkError::InvalidConfig("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("InvalidConfig"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_network_error_clone() {
        let error = NetworkError::ConnectionFailed("reason".to_string());
        let cloned = error.clone();

        match cloned {
            NetworkError::ConnectionFailed(msg) => assert_eq!(msg, "reason"),
            _ => panic!("Wrong error type after clone"),
        }
    }

    #[test]
    fn test_network_error_is_error() {
        // Verify that NetworkError implements std::error::Error
        let error = NetworkError::InvalidConfig("test".to_string());
        let error_ref: &dyn std::error::Error = &error;
        assert!(!error_ref.to_string().is_empty());
    }

    // ==================== Mode Switching Tests ====================

    #[test]
    fn test_mode_determined_by_config() {
        // Full mode: P2P enabled
        let full_config = NetworkConfig {
            enable_p2p: true,
            enable_local_relay: true,
            ..Default::default()
        };
        assert_eq!(init(&full_config).unwrap().mode(), &NetworkMode::Full);

        // Full mode: P2P enabled, no local relay (still Full because P2P is primary)
        let full_no_relay = NetworkConfig {
            enable_p2p: true,
            enable_local_relay: false,
            ..Default::default()
        };
        assert_eq!(init(&full_no_relay).unwrap().mode(), &NetworkMode::Full);

        // Offline mode: No P2P, local relay enabled
        let offline_config = NetworkConfig {
            enable_p2p: false,
            enable_local_relay: true,
            ..Default::default()
        };
        assert_eq!(init(&offline_config).unwrap().mode(), &NetworkMode::Offline);

        // Light mode: No P2P, no local relay
        let light_config = NetworkConfig {
            enable_p2p: false,
            enable_local_relay: false,
            ..Default::default()
        };
        assert_eq!(init(&light_config).unwrap().mode(), &NetworkMode::Light);
    }

    // ==================== Validation Tests ====================

    #[test]
    fn test_config_validation_max_peers_zero() {
        let config = NetworkConfig {
            max_peers: 0,
            ..Default::default()
        };

        let result = init(&config);
        assert!(result.is_err());

        match result {
            Err(NetworkError::InvalidConfig(msg)) => {
                assert!(msg.contains("max_peers"));
            }
            _ => panic!("Expected InvalidConfig error"),
        }
    }

    #[test]
    fn test_config_validation_max_peers_one() {
        let config = NetworkConfig {
            max_peers: 1,
            ..Default::default()
        };

        let result = init(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation_large_max_peers() {
        let config = NetworkConfig {
            max_peers: 1000,
            ..Default::default()
        };

        let result = init(&config);
        assert!(result.is_ok());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_network_setup() {
        // Simulate a complete network setup
        let config = NetworkConfig {
            enable_p2p: true,
            relay_urls: vec![
                "wss://relay1.nostr.net".to_string(),
                "wss://relay2.nostr.net".to_string(),
            ],
            enable_local_relay: true,
            p2p_port: 0, // Random port
            ticket_ttl_secs: 3600,
            max_peers: 8,
        };

        let handle = init(&config).unwrap();

        // Verify state
        assert_eq!(handle.mode(), &NetworkMode::Full);
        assert!(handle.is_p2p_enabled());
        assert!(handle.is_local_relay_enabled());

        // Verify config preserved
        assert_eq!(handle.config.relay_urls.len(), 2);
        assert_eq!(handle.config.max_peers, 8);

        // Verify initial stats
        let stats = handle.stats();
        assert_eq!(stats.peer_count, 0);
        assert_eq!(stats.bytes_sent, 0);
    }

    #[test]
    fn test_light_client_setup() {
        // Simulate a light client that connects to remote relays only
        let config = NetworkConfig {
            enable_p2p: false,
            relay_urls: vec!["wss://public.relay.nostr".to_string()],
            enable_local_relay: false,
            p2p_port: 0,
            ticket_ttl_secs: 1800,
            max_peers: 4,
        };

        let handle = init(&config).unwrap();

        assert_eq!(handle.mode(), &NetworkMode::Light);
        assert!(!handle.is_p2p_enabled());
        assert!(!handle.is_local_relay_enabled());
    }

    #[test]
    fn test_offline_mode_setup() {
        // Simulate offline/local play setup
        let config = NetworkConfig {
            enable_p2p: false,
            relay_urls: vec![],
            enable_local_relay: true,
            p2p_port: 0,
            ticket_ttl_secs: 3600,
            max_peers: 2,
        };

        let handle = init(&config).unwrap();

        assert_eq!(handle.mode(), &NetworkMode::Offline);
        assert!(!handle.is_p2p_enabled());
        assert!(handle.is_local_relay_enabled());
    }

    // ==================== Re-export Tests ====================

    #[test]
    fn test_peer_module_reexports() {
        // Verify that peer module types are properly re-exported
        let ticket = ConnectionTicket::new(
            "node".to_string(),
            vec![],
            "game".to_string(),
            3600,
        );
        assert!(!ticket.is_expired());

        let info = PeerInfo::new("peer".to_string());
        assert_eq!(info.state, ConnectionState::Connecting);
    }

    #[test]
    fn test_sync_module_reexports() {
        // Verify that sync module types are properly re-exported
        let manager = SyncManager::new("game".to_string(), 0);
        assert_eq!(*manager.state(), SyncState::Idle);

        let tracker = PeerSyncTracker::new();
        assert!(tracker.min_progress().is_none());
    }

    #[test]
    fn test_discovery_module_reexports() {
        // Verify that discovery module types are properly re-exported
        let generator = QrGenerator::new();
        let matrix = generator.generate("test");
        assert!(matrix.size >= 21);

        let service = DiscoveryService::new();
        assert!(service.list_games().is_empty());

        let level = ErrorCorrection::default();
        assert!(matches!(level, ErrorCorrection::Medium));
    }
}

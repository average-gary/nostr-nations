//! Connection pooling and health monitoring.
//!
//! This module provides connection management with automatic reconnection,
//! health monitoring, and exponential backoff for failed connections.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Connection state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PooledConnectionState {
    /// Connection is healthy and ready.
    Connected,
    /// Connection is being established.
    Connecting,
    /// Connection failed, waiting for retry.
    Disconnected,
    /// Connection is being closed.
    Closing,
    /// Connection is permanently closed.
    Closed,
}

/// Health status of a connection.
#[derive(Clone, Debug)]
pub struct ConnectionHealth {
    /// Current state.
    pub state: PooledConnectionState,
    /// Last successful ping time.
    pub last_ping: Option<Instant>,
    /// Last ping round-trip time in milliseconds.
    pub last_rtt_ms: Option<u32>,
    /// Number of consecutive failures.
    pub consecutive_failures: u32,
    /// Last error message.
    pub last_error: Option<String>,
    /// Time of last state change.
    pub last_state_change: Instant,
    /// Total successful requests.
    pub total_requests: u64,
    /// Total failed requests.
    pub total_failures: u64,
}

impl Default for ConnectionHealth {
    fn default() -> Self {
        Self {
            state: PooledConnectionState::Disconnected,
            last_ping: None,
            last_rtt_ms: None,
            consecutive_failures: 0,
            last_error: None,
            last_state_change: Instant::now(),
            total_requests: 0,
            total_failures: 0,
        }
    }
}

impl ConnectionHealth {
    /// Create a new connection health tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if the connection is healthy.
    pub fn is_healthy(&self) -> bool {
        self.state == PooledConnectionState::Connected && self.consecutive_failures == 0
    }

    /// Get success rate as a percentage.
    pub fn success_rate(&self) -> f64 {
        let total = self.total_requests + self.total_failures;
        if total == 0 {
            100.0
        } else {
            (self.total_requests as f64 / total as f64) * 100.0
        }
    }

    /// Record a successful operation.
    pub fn record_success(&mut self) {
        self.consecutive_failures = 0;
        self.total_requests += 1;
        self.last_error = None;
    }

    /// Record a failed operation.
    pub fn record_failure(&mut self, error: String) {
        self.consecutive_failures += 1;
        self.total_failures += 1;
        self.last_error = Some(error);
    }

    /// Record a ping result.
    pub fn record_ping(&mut self, rtt_ms: u32) {
        self.last_ping = Some(Instant::now());
        self.last_rtt_ms = Some(rtt_ms);
        self.record_success();
    }

    /// Update the connection state.
    pub fn set_state(&mut self, state: PooledConnectionState) {
        if self.state != state {
            self.state = state;
            self.last_state_change = Instant::now();
        }
    }
}

/// Configuration for connection pooling.
#[derive(Clone, Debug)]
pub struct PoolConfig {
    /// Maximum number of connections in the pool.
    pub max_connections: usize,
    /// Minimum number of connections to maintain.
    pub min_connections: usize,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Idle timeout before closing unused connections.
    pub idle_timeout: Duration,
    /// Health check interval.
    pub health_check_interval: Duration,
    /// Maximum consecutive failures before marking unhealthy.
    pub max_consecutive_failures: u32,
    /// Enable automatic reconnection.
    pub auto_reconnect: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300),
            health_check_interval: Duration::from_secs(30),
            max_consecutive_failures: 3,
            auto_reconnect: true,
        }
    }
}

/// Backoff configuration for reconnection attempts.
#[derive(Clone, Debug)]
pub struct BackoffConfig {
    /// Initial delay before first retry.
    pub initial_delay: Duration,
    /// Maximum delay between retries.
    pub max_delay: Duration,
    /// Multiplier for exponential backoff.
    pub multiplier: f64,
    /// Maximum number of retry attempts (0 = infinite).
    pub max_attempts: u32,
    /// Add random jitter to delays.
    pub jitter: bool,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_attempts: 10,
            jitter: true,
        }
    }
}

/// Tracks backoff state for a connection.
#[derive(Clone, Debug)]
pub struct BackoffState {
    /// Configuration.
    config: BackoffConfig,
    /// Current attempt number.
    attempts: u32,
    /// Next retry time.
    next_retry: Option<Instant>,
}

impl BackoffState {
    /// Create a new backoff state.
    pub fn new(config: BackoffConfig) -> Self {
        Self {
            config,
            attempts: 0,
            next_retry: None,
        }
    }

    /// Reset the backoff state after a successful connection.
    pub fn reset(&mut self) {
        self.attempts = 0;
        self.next_retry = None;
    }

    /// Record a failure and calculate next retry time.
    pub fn record_failure(&mut self) -> Option<Duration> {
        self.attempts += 1;

        // Check max attempts
        if self.config.max_attempts > 0 && self.attempts > self.config.max_attempts {
            return None;
        }

        // Calculate delay with exponential backoff
        let delay_ms = self.config.initial_delay.as_millis() as f64
            * self.config.multiplier.powi((self.attempts - 1) as i32);
        let delay_ms = delay_ms.min(self.config.max_delay.as_millis() as f64);

        // Add jitter if enabled
        let delay_ms = if self.config.jitter {
            let jitter = delay_ms * 0.2 * rand_factor();
            delay_ms + jitter
        } else {
            delay_ms
        };

        let delay = Duration::from_millis(delay_ms as u64);
        self.next_retry = Some(Instant::now() + delay);

        Some(delay)
    }

    /// Check if we should retry now.
    pub fn should_retry(&self) -> bool {
        match self.next_retry {
            Some(time) => Instant::now() >= time,
            None => true, // No retry scheduled, can try immediately
        }
    }

    /// Get the number of attempts made.
    pub fn attempts(&self) -> u32 {
        self.attempts
    }

    /// Check if max attempts reached.
    pub fn max_reached(&self) -> bool {
        self.config.max_attempts > 0 && self.attempts >= self.config.max_attempts
    }
}

/// Simple pseudo-random factor for jitter (0.0 to 1.0).
fn rand_factor() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    (nanos % 1000) as f64 / 1000.0
}

/// A pooled connection entry.
#[derive(Clone)]
pub struct PooledConnection {
    /// Connection identifier.
    pub id: String,
    /// Target address/endpoint.
    pub endpoint: String,
    /// Health information.
    pub health: ConnectionHealth,
    /// Backoff state for reconnection.
    pub backoff: BackoffState,
    /// Last activity time.
    pub last_activity: Instant,
    /// Creation time.
    pub created_at: Instant,
}

impl PooledConnection {
    /// Create a new pooled connection.
    pub fn new(id: String, endpoint: String, backoff_config: BackoffConfig) -> Self {
        Self {
            id,
            endpoint,
            health: ConnectionHealth::new(),
            backoff: BackoffState::new(backoff_config),
            last_activity: Instant::now(),
            created_at: Instant::now(),
        }
    }

    /// Update last activity time.
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if the connection is idle.
    pub fn is_idle(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() >= timeout
    }

    /// Get connection age.
    pub fn age(&self) -> Duration {
        self.created_at.elapsed()
    }
}

/// Connection pool manager.
pub struct ConnectionPool {
    /// Configuration.
    config: PoolConfig,
    /// Backoff configuration for new connections.
    backoff_config: BackoffConfig,
    /// Active connections.
    connections: Arc<RwLock<HashMap<String, PooledConnection>>>,
    /// Statistics.
    stats: Arc<RwLock<PoolStats>>,
}

/// Pool statistics.
#[derive(Clone, Debug, Default)]
pub struct PoolStats {
    /// Total connections created.
    pub connections_created: u64,
    /// Total connections closed.
    pub connections_closed: u64,
    /// Total reconnection attempts.
    pub reconnection_attempts: u64,
    /// Successful reconnections.
    pub reconnection_successes: u64,
    /// Connections closed due to idle timeout.
    pub idle_timeouts: u64,
    /// Health checks performed.
    pub health_checks: u64,
}

impl ConnectionPool {
    /// Create a new connection pool.
    pub fn new(config: PoolConfig) -> Self {
        Self {
            config,
            backoff_config: BackoffConfig::default(),
            connections: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PoolStats::default())),
        }
    }

    /// Create a pool with custom backoff configuration.
    pub fn with_backoff(config: PoolConfig, backoff_config: BackoffConfig) -> Self {
        Self {
            config,
            backoff_config,
            connections: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(PoolStats::default())),
        }
    }

    /// Get the pool configuration.
    pub fn config(&self) -> &PoolConfig {
        &self.config
    }

    /// Get the number of active connections.
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get the number of healthy connections.
    pub async fn healthy_count(&self) -> usize {
        self.connections
            .read()
            .await
            .values()
            .filter(|c| c.health.is_healthy())
            .count()
    }

    /// Add a new connection to the pool.
    pub async fn add_connection(&self, id: String, endpoint: String) -> Result<(), PoolError> {
        let mut connections = self.connections.write().await;

        // Check pool capacity
        if connections.len() >= self.config.max_connections {
            return Err(PoolError::PoolFull);
        }

        // Check for duplicate
        if connections.contains_key(&id) {
            return Err(PoolError::DuplicateConnection(id));
        }

        let connection = PooledConnection::new(id.clone(), endpoint, self.backoff_config.clone());
        connections.insert(id, connection);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.connections_created += 1;

        Ok(())
    }

    /// Remove a connection from the pool.
    pub async fn remove_connection(&self, id: &str) -> Option<PooledConnection> {
        let mut connections = self.connections.write().await;
        let removed = connections.remove(id);

        if removed.is_some() {
            let mut stats = self.stats.write().await;
            stats.connections_closed += 1;
        }

        removed
    }

    /// Get a connection by ID.
    pub async fn get_connection(&self, id: &str) -> Option<PooledConnection> {
        self.connections.read().await.get(id).cloned()
    }

    /// Get all connection IDs.
    pub async fn connection_ids(&self) -> Vec<String> {
        self.connections.read().await.keys().cloned().collect()
    }

    /// Get all healthy connections.
    pub async fn healthy_connections(&self) -> Vec<PooledConnection> {
        self.connections
            .read()
            .await
            .values()
            .filter(|c| c.health.is_healthy())
            .cloned()
            .collect()
    }

    /// Update connection health.
    pub async fn update_health(&self, id: &str, health: ConnectionHealth) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.health = health;
            conn.touch();
        }
    }

    /// Record a successful operation on a connection.
    pub async fn record_success(&self, id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.health.record_success();
            conn.backoff.reset();
            conn.touch();
        }
    }

    /// Record a failed operation on a connection.
    pub async fn record_failure(&self, id: &str, error: String) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.health.record_failure(error);
            conn.backoff.record_failure();

            // Update state if too many failures
            if conn.health.consecutive_failures >= self.config.max_consecutive_failures {
                conn.health.set_state(PooledConnectionState::Disconnected);
            }
        }
    }

    /// Mark a connection as connected.
    pub async fn mark_connected(&self, id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.health.set_state(PooledConnectionState::Connected);
            conn.backoff.reset();
            conn.touch();
        }
    }

    /// Mark a connection as disconnected.
    pub async fn mark_disconnected(&self, id: &str) {
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(id) {
            conn.health.set_state(PooledConnectionState::Disconnected);
        }
    }

    /// Get connections that should attempt reconnection.
    pub async fn get_reconnection_candidates(&self) -> Vec<String> {
        if !self.config.auto_reconnect {
            return Vec::new();
        }

        self.connections
            .read()
            .await
            .iter()
            .filter(|(_, conn)| {
                conn.health.state == PooledConnectionState::Disconnected
                    && conn.backoff.should_retry()
                    && !conn.backoff.max_reached()
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get idle connections that should be closed.
    pub async fn get_idle_connections(&self) -> Vec<String> {
        let min_connections = self.config.min_connections;
        let idle_timeout = self.config.idle_timeout;

        let connections = self.connections.read().await;

        // Don't close if at minimum
        if connections.len() <= min_connections {
            return Vec::new();
        }

        connections
            .iter()
            .filter(|(_, conn)| conn.is_idle(idle_timeout))
            .take(connections.len() - min_connections)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Run health checks on all connections.
    pub async fn health_check(&self) -> Vec<(String, bool)> {
        let connections = self.connections.read().await;
        let mut stats = self.stats.write().await;

        stats.health_checks += 1;

        connections
            .iter()
            .map(|(id, conn)| (id.clone(), conn.health.is_healthy()))
            .collect()
    }

    /// Get pool statistics.
    pub async fn stats(&self) -> PoolStats {
        self.stats.read().await.clone()
    }

    /// Get a summary of pool status.
    pub async fn status(&self) -> PoolStatus {
        let connections = self.connections.read().await;

        let total = connections.len();
        let healthy = connections.values().filter(|c| c.health.is_healthy()).count();
        let connecting = connections
            .values()
            .filter(|c| c.health.state == PooledConnectionState::Connecting)
            .count();
        let disconnected = connections
            .values()
            .filter(|c| c.health.state == PooledConnectionState::Disconnected)
            .count();

        PoolStatus {
            total_connections: total,
            healthy_connections: healthy,
            connecting_connections: connecting,
            disconnected_connections: disconnected,
            capacity: self.config.max_connections,
        }
    }
}

/// Summary of pool status.
#[derive(Clone, Debug)]
pub struct PoolStatus {
    /// Total connections in the pool.
    pub total_connections: usize,
    /// Number of healthy connections.
    pub healthy_connections: usize,
    /// Number of connections being established.
    pub connecting_connections: usize,
    /// Number of disconnected connections.
    pub disconnected_connections: usize,
    /// Maximum pool capacity.
    pub capacity: usize,
}

impl PoolStatus {
    /// Get utilization percentage.
    pub fn utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.total_connections as f64 / self.capacity as f64) * 100.0
        }
    }
}

/// Pool errors.
#[derive(Clone, Debug)]
pub enum PoolError {
    /// Pool is at capacity.
    PoolFull,
    /// Connection already exists.
    DuplicateConnection(String),
    /// Connection not found.
    ConnectionNotFound(String),
    /// Connection failed.
    ConnectionFailed(String),
    /// Pool is closed.
    PoolClosed,
}

impl std::fmt::Display for PoolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PoolError::PoolFull => write!(f, "Connection pool is full"),
            PoolError::DuplicateConnection(id) => write!(f, "Connection already exists: {}", id),
            PoolError::ConnectionNotFound(id) => write!(f, "Connection not found: {}", id),
            PoolError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            PoolError::PoolClosed => write!(f, "Connection pool is closed"),
        }
    }
}

impl std::error::Error for PoolError {}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ConnectionHealth Tests ====================

    #[test]
    fn test_connection_health_default() {
        let health = ConnectionHealth::default();
        assert_eq!(health.state, PooledConnectionState::Disconnected);
        assert!(health.last_ping.is_none());
        assert_eq!(health.consecutive_failures, 0);
    }

    #[test]
    fn test_connection_health_is_healthy() {
        let mut health = ConnectionHealth::new();
        health.state = PooledConnectionState::Connected;
        assert!(health.is_healthy());

        health.consecutive_failures = 1;
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_connection_health_success_rate() {
        let mut health = ConnectionHealth::new();
        assert!((health.success_rate() - 100.0).abs() < 0.001);

        health.total_requests = 8;
        health.total_failures = 2;
        assert!((health.success_rate() - 80.0).abs() < 0.001);
    }

    #[test]
    fn test_connection_health_record_success() {
        let mut health = ConnectionHealth::new();
        health.consecutive_failures = 5;
        health.last_error = Some("error".to_string());

        health.record_success();

        assert_eq!(health.consecutive_failures, 0);
        assert!(health.last_error.is_none());
        assert_eq!(health.total_requests, 1);
    }

    #[test]
    fn test_connection_health_record_failure() {
        let mut health = ConnectionHealth::new();

        health.record_failure("error1".to_string());
        assert_eq!(health.consecutive_failures, 1);
        assert_eq!(health.total_failures, 1);
        assert_eq!(health.last_error, Some("error1".to_string()));

        health.record_failure("error2".to_string());
        assert_eq!(health.consecutive_failures, 2);
        assert_eq!(health.last_error, Some("error2".to_string()));
    }

    #[test]
    fn test_connection_health_record_ping() {
        let mut health = ConnectionHealth::new();

        health.record_ping(50);

        assert!(health.last_ping.is_some());
        assert_eq!(health.last_rtt_ms, Some(50));
        assert_eq!(health.total_requests, 1);
    }

    // ==================== BackoffState Tests ====================

    #[test]
    fn test_backoff_state_new() {
        let config = BackoffConfig::default();
        let state = BackoffState::new(config);

        assert_eq!(state.attempts(), 0);
        assert!(!state.max_reached());
        assert!(state.should_retry());
    }

    #[test]
    fn test_backoff_state_record_failure() {
        let config = BackoffConfig {
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            max_attempts: 5,
            jitter: false,
        };
        let mut state = BackoffState::new(config);

        let delay1 = state.record_failure();
        assert!(delay1.is_some());
        assert_eq!(state.attempts(), 1);

        let delay2 = state.record_failure();
        assert!(delay2.is_some());
        assert!(delay2.unwrap() > delay1.unwrap()); // Exponential increase
    }

    #[test]
    fn test_backoff_state_max_attempts() {
        let config = BackoffConfig {
            max_attempts: 2,
            ..Default::default()
        };
        let mut state = BackoffState::new(config);

        state.record_failure();
        assert!(!state.max_reached());

        state.record_failure();
        assert!(state.max_reached());

        let delay = state.record_failure();
        assert!(delay.is_none()); // Max reached
    }

    #[test]
    fn test_backoff_state_reset() {
        let mut state = BackoffState::new(BackoffConfig::default());

        state.record_failure();
        state.record_failure();
        assert_eq!(state.attempts(), 2);

        state.reset();
        assert_eq!(state.attempts(), 0);
    }

    // ==================== PooledConnection Tests ====================

    #[test]
    fn test_pooled_connection_new() {
        let conn = PooledConnection::new(
            "conn1".to_string(),
            "localhost:8080".to_string(),
            BackoffConfig::default(),
        );

        assert_eq!(conn.id, "conn1");
        assert_eq!(conn.endpoint, "localhost:8080");
        assert!(!conn.is_idle(Duration::from_secs(1)));
    }

    #[test]
    fn test_pooled_connection_touch() {
        let mut conn = PooledConnection::new(
            "conn1".to_string(),
            "localhost:8080".to_string(),
            BackoffConfig::default(),
        );

        let before = conn.last_activity;
        std::thread::sleep(Duration::from_millis(10));
        conn.touch();

        assert!(conn.last_activity > before);
    }

    // ==================== ConnectionPool Tests ====================

    #[tokio::test]
    async fn test_connection_pool_new() {
        let pool = ConnectionPool::new(PoolConfig::default());

        assert_eq!(pool.connection_count().await, 0);
        assert_eq!(pool.healthy_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_pool_add_connection() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();

        assert_eq!(pool.connection_count().await, 1);
    }

    #[tokio::test]
    async fn test_connection_pool_add_duplicate() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();

        let result = pool
            .add_connection("conn1".to_string(), "localhost:8081".to_string())
            .await;

        assert!(matches!(result, Err(PoolError::DuplicateConnection(_))));
    }

    #[tokio::test]
    async fn test_connection_pool_full() {
        let config = PoolConfig {
            max_connections: 2,
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();
        pool.add_connection("conn2".to_string(), "localhost:8081".to_string())
            .await
            .unwrap();

        let result = pool
            .add_connection("conn3".to_string(), "localhost:8082".to_string())
            .await;

        assert!(matches!(result, Err(PoolError::PoolFull)));
    }

    #[tokio::test]
    async fn test_connection_pool_remove() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();

        let removed = pool.remove_connection("conn1").await;
        assert!(removed.is_some());
        assert_eq!(pool.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_pool_mark_connected() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();

        pool.mark_connected("conn1").await;

        let conn = pool.get_connection("conn1").await.unwrap();
        assert_eq!(conn.health.state, PooledConnectionState::Connected);
    }

    #[tokio::test]
    async fn test_connection_pool_record_success() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();
        pool.mark_connected("conn1").await;
        pool.record_success("conn1").await;

        let conn = pool.get_connection("conn1").await.unwrap();
        assert_eq!(conn.health.total_requests, 1);
    }

    #[tokio::test]
    async fn test_connection_pool_record_failure() {
        let config = PoolConfig {
            max_consecutive_failures: 2,
            ..Default::default()
        };
        let pool = ConnectionPool::new(config);

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();
        pool.mark_connected("conn1").await;

        pool.record_failure("conn1", "error1".to_string()).await;
        pool.record_failure("conn1", "error2".to_string()).await;

        let conn = pool.get_connection("conn1").await.unwrap();
        assert_eq!(conn.health.state, PooledConnectionState::Disconnected);
    }

    #[tokio::test]
    async fn test_connection_pool_healthy_connections() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();
        pool.add_connection("conn2".to_string(), "localhost:8081".to_string())
            .await
            .unwrap();

        pool.mark_connected("conn1").await;
        // conn2 stays disconnected

        let healthy = pool.healthy_connections().await;
        assert_eq!(healthy.len(), 1);
        assert_eq!(healthy[0].id, "conn1");
    }

    #[tokio::test]
    async fn test_connection_pool_status() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();
        pool.add_connection("conn2".to_string(), "localhost:8081".to_string())
            .await
            .unwrap();

        pool.mark_connected("conn1").await;

        let status = pool.status().await;
        assert_eq!(status.total_connections, 2);
        assert_eq!(status.healthy_connections, 1);
        assert_eq!(status.disconnected_connections, 1);
    }

    #[tokio::test]
    async fn test_connection_pool_stats() {
        let pool = ConnectionPool::new(PoolConfig::default());

        pool.add_connection("conn1".to_string(), "localhost:8080".to_string())
            .await
            .unwrap();
        pool.remove_connection("conn1").await;

        let stats = pool.stats().await;
        assert_eq!(stats.connections_created, 1);
        assert_eq!(stats.connections_closed, 1);
    }

    // ==================== PoolError Tests ====================

    #[test]
    fn test_pool_error_display() {
        assert!(format!("{}", PoolError::PoolFull).contains("full"));
        assert!(format!("{}", PoolError::DuplicateConnection("c1".to_string())).contains("c1"));
        assert!(format!("{}", PoolError::ConnectionNotFound("c1".to_string())).contains("not found"));
        assert!(format!("{}", PoolError::ConnectionFailed("timeout".to_string())).contains("timeout"));
        assert!(format!("{}", PoolError::PoolClosed).contains("closed"));
    }
}

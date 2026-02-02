//! Peer connection management using Iroh.
//!
//! This module handles:
//! - Creating and accepting P2P connections
//! - Managing connection tickets for easy pairing
//! - Sending and receiving game events over P2P
//!
//! # Connection Flow
//!
//! 1. Host creates a game and generates a connection ticket
//! 2. Host displays ticket as QR code
//! 3. Client scans QR code and extracts ticket
//! 4. Client uses ticket to connect to host
//! 5. Both peers can now exchange game events

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// Unique identifier for a peer (derived from their Iroh node ID).
pub type PeerId = String;

/// Connection ticket for peer discovery.
///
/// This ticket contains all information needed to connect to a peer.
/// It can be serialized to a string for QR code generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConnectionTicket {
    /// The host's node ID.
    pub node_id: String,
    /// Direct addresses for connection.
    pub addresses: Vec<String>,
    /// ALPN protocol identifier.
    pub alpn: String,
    /// Game ID this ticket is for.
    pub game_id: String,
    /// Expiration timestamp (Unix seconds).
    pub expires_at: u64,
}

impl ConnectionTicket {
    /// Create a new connection ticket.
    pub fn new(node_id: String, addresses: Vec<String>, game_id: String, ttl_secs: u64) -> Self {
        let expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() + ttl_secs)
            .unwrap_or(0);

        Self {
            node_id,
            addresses,
            alpn: "nostr-nations/1".to_string(),
            game_id,
            expires_at,
        }
    }

    /// Serialize ticket to a string (for QR codes).
    pub fn to_string(&self) -> Result<String, serde_json::Error> {
        // Use base64-encoded JSON for compact representation
        let json = serde_json::to_string(self)?;
        Ok(base64_encode(json.as_bytes()))
    }

    /// Deserialize ticket from a string.
    pub fn from_string(s: &str) -> Result<Self, TicketError> {
        let bytes = base64_decode(s).map_err(|_| TicketError::InvalidFormat)?;
        let json = String::from_utf8(bytes).map_err(|_| TicketError::InvalidFormat)?;
        serde_json::from_str(&json).map_err(|_| TicketError::InvalidFormat)
    }

    /// Check if the ticket has expired.
    pub fn is_expired(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now > self.expires_at
    }
}

/// Errors from ticket operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TicketError {
    InvalidFormat,
    Expired,
    WrongGame,
}

impl std::fmt::Display for TicketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TicketError::InvalidFormat => write!(f, "Invalid ticket format"),
            TicketError::Expired => write!(f, "Ticket has expired"),
            TicketError::WrongGame => write!(f, "Ticket is for a different game"),
        }
    }
}

impl std::error::Error for TicketError {}

/// Message types for peer communication.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum PeerMessage {
    /// Handshake message with game info.
    Hello {
        peer_id: PeerId,
        game_id: String,
        player_name: String,
    },
    /// Request to join the game.
    JoinRequest {
        player_name: String,
        civilization_id: String,
    },
    /// Response to join request.
    JoinResponse {
        accepted: bool,
        player_id: Option<u32>,
        reason: Option<String>,
    },
    /// Game event from Nostr.
    GameEvent { event_json: String },
    /// Request game state sync.
    SyncRequest {
        from_turn: u32,
        from_sequence: u32,
    },
    /// Response with game events.
    SyncResponse { events_json: Vec<String> },
    /// Request randomness (Cashu).
    RandomnessRequest {
        request_id: String,
        context: String,
        blinded_message: Vec<u8>,
    },
    /// Randomness response (Cashu).
    RandomnessResponse {
        request_id: String,
        blinded_signature: Vec<u8>,
    },
    /// Ping for keepalive.
    Ping { timestamp: u64 },
    /// Pong response.
    Pong { timestamp: u64 },
    /// Graceful disconnect.
    Goodbye { reason: String },
}

impl PeerMessage {
    /// Serialize to bytes for network transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// State of a peer connection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connection is being established.
    Connecting,
    /// Connected but not yet joined game.
    Connected,
    /// Fully joined and ready to play.
    Joined,
    /// Connection is closing.
    Disconnecting,
    /// Connection closed.
    Disconnected,
}

/// Information about a connected peer.
#[derive(Clone, Debug)]
pub struct PeerInfo {
    /// Unique peer identifier.
    pub peer_id: PeerId,
    /// Connection state.
    pub state: ConnectionState,
    /// Player name (if joined).
    pub player_name: Option<String>,
    /// Player ID in the game (if joined).
    pub player_id: Option<u32>,
    /// Last ping timestamp.
    pub last_ping: u64,
    /// Round-trip time in milliseconds.
    pub rtt_ms: Option<u32>,
}

impl PeerInfo {
    /// Create new peer info for an incoming connection.
    pub fn new(peer_id: PeerId) -> Self {
        Self {
            peer_id,
            state: ConnectionState::Connecting,
            player_name: None,
            player_id: None,
            last_ping: 0,
            rtt_ms: None,
        }
    }
}

/// Event from the peer manager.
#[derive(Clone, Debug)]
pub enum PeerEvent {
    /// A new peer connected.
    PeerConnected { peer_id: PeerId },
    /// A peer disconnected.
    PeerDisconnected { peer_id: PeerId, reason: String },
    /// A peer wants to join the game.
    JoinRequest {
        peer_id: PeerId,
        player_name: String,
        civilization_id: String,
    },
    /// Received a game event from a peer.
    GameEventReceived {
        peer_id: PeerId,
        event_json: String,
    },
    /// Received a sync request.
    SyncRequested {
        peer_id: PeerId,
        from_turn: u32,
        from_sequence: u32,
    },
    /// Received a randomness request (host only).
    RandomnessRequested {
        peer_id: PeerId,
        request_id: String,
        context: String,
        blinded_message: Vec<u8>,
    },
}

/// Manages peer connections for a game session.
pub struct PeerManager {
    /// Our node ID.
    node_id: String,
    /// Game ID we're managing connections for.
    game_id: String,
    /// Whether we're the host.
    is_host: bool,
    /// Connected peers.
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    /// Channel for outgoing events.
    event_tx: mpsc::Sender<PeerEvent>,
    /// Channel for receiving events.
    event_rx: mpsc::Receiver<PeerEvent>,
}

impl PeerManager {
    /// Create a new peer manager.
    pub fn new(node_id: String, game_id: String, is_host: bool) -> Self {
        let (event_tx, event_rx) = mpsc::channel(100);

        Self {
            node_id,
            game_id,
            is_host,
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            event_rx,
        }
    }

    /// Get our node ID.
    pub fn node_id(&self) -> &str {
        &self.node_id
    }

    /// Get the game ID.
    pub fn game_id(&self) -> &str {
        &self.game_id
    }

    /// Check if we're the host.
    pub fn is_host(&self) -> bool {
        self.is_host
    }

    /// Create a connection ticket for this game.
    pub fn create_ticket(&self, addresses: Vec<String>, ttl_secs: u64) -> ConnectionTicket {
        ConnectionTicket::new(self.node_id.clone(), addresses, self.game_id.clone(), ttl_secs)
    }

    /// Get the number of connected peers.
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Get information about all peers.
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Get information about a specific peer.
    pub async fn get_peer(&self, peer_id: &str) -> Option<PeerInfo> {
        self.peers.read().await.get(peer_id).cloned()
    }

    /// Register a new peer connection.
    pub async fn add_peer(&self, peer_id: PeerId) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_id.clone(), PeerInfo::new(peer_id.clone()));

        let _ = self.event_tx.send(PeerEvent::PeerConnected { peer_id }).await;
    }

    /// Remove a peer connection.
    pub async fn remove_peer(&self, peer_id: &str, reason: String) {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id);

        let _ = self
            .event_tx
            .send(PeerEvent::PeerDisconnected {
                peer_id: peer_id.to_string(),
                reason,
            })
            .await;
    }

    /// Update peer state after successful join.
    pub async fn peer_joined(&self, peer_id: &str, player_name: String, player_id: u32) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            peer.state = ConnectionState::Joined;
            peer.player_name = Some(player_name);
            peer.player_id = Some(player_id);
        }
    }

    /// Handle an incoming message from a peer.
    pub async fn handle_message(&self, peer_id: &str, message: PeerMessage) {
        match message {
            PeerMessage::JoinRequest {
                player_name,
                civilization_id,
            } => {
                let _ = self
                    .event_tx
                    .send(PeerEvent::JoinRequest {
                        peer_id: peer_id.to_string(),
                        player_name,
                        civilization_id,
                    })
                    .await;
            }
            PeerMessage::GameEvent { event_json } => {
                let _ = self
                    .event_tx
                    .send(PeerEvent::GameEventReceived {
                        peer_id: peer_id.to_string(),
                        event_json,
                    })
                    .await;
            }
            PeerMessage::SyncRequest {
                from_turn,
                from_sequence,
            } => {
                let _ = self
                    .event_tx
                    .send(PeerEvent::SyncRequested {
                        peer_id: peer_id.to_string(),
                        from_turn,
                        from_sequence,
                    })
                    .await;
            }
            PeerMessage::RandomnessRequest {
                request_id,
                context,
                blinded_message,
            } => {
                let _ = self
                    .event_tx
                    .send(PeerEvent::RandomnessRequested {
                        peer_id: peer_id.to_string(),
                        request_id,
                        context,
                        blinded_message,
                    })
                    .await;
            }
            PeerMessage::Ping { timestamp } => {
                // Update last ping time
                let mut peers = self.peers.write().await;
                if let Some(peer) = peers.get_mut(peer_id) {
                    peer.last_ping = timestamp;
                }
            }
            PeerMessage::Pong { timestamp } => {
                // Calculate RTT
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                let rtt = (now.saturating_sub(timestamp)) as u32;

                let mut peers = self.peers.write().await;
                if let Some(peer) = peers.get_mut(peer_id) {
                    peer.rtt_ms = Some(rtt);
                }
            }
            PeerMessage::Goodbye { reason } => {
                self.remove_peer(peer_id, reason).await;
            }
            _ => {} // Other messages handled elsewhere
        }
    }

    /// Receive the next peer event.
    pub async fn recv_event(&mut self) -> Option<PeerEvent> {
        self.event_rx.recv().await
    }

    /// Try to receive a peer event without blocking.
    pub fn try_recv_event(&mut self) -> Option<PeerEvent> {
        self.event_rx.try_recv().ok()
    }
}

// Simple base64 encoding/decoding (for ticket serialization)
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut result = String::new();
    let mut i = 0;

    while i < data.len() {
        let b0 = data[i] as usize;
        let b1 = if i + 1 < data.len() {
            data[i + 1] as usize
        } else {
            0
        };
        let b2 = if i + 2 < data.len() {
            data[i + 2] as usize
        } else {
            0
        };

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if i + 1 < data.len() {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        } else {
            result.push('=');
        }

        if i + 2 < data.len() {
            result.push(ALPHABET[b2 & 0x3f] as char);
        } else {
            result.push('=');
        }

        i += 3;
    }

    result
}

fn base64_decode(data: &str) -> Result<Vec<u8>, ()> {
    const DECODE: [i8; 128] = [
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1,
        -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 62, -1, -1,
        -1, 63, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, -1, -1, -1, -1, -1, -1, -1, 0, 1, 2, 3, 4,
        5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, -1, -1, -1,
        -1, -1, -1, 26, 27, 28, 29, 30, 31, 32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45,
        46, 47, 48, 49, 50, 51, -1, -1, -1, -1, -1,
    ];

    let data = data.trim_end_matches('=');
    let mut result = Vec::new();

    let bytes: Vec<u8> = data
        .bytes()
        .filter_map(|b| {
            if b < 128 && DECODE[b as usize] >= 0 {
                Some(DECODE[b as usize] as u8)
            } else {
                None
            }
        })
        .collect();

    let mut i = 0;
    while i + 3 < bytes.len() {
        result.push((bytes[i] << 2) | (bytes[i + 1] >> 4));
        result.push((bytes[i + 1] << 4) | (bytes[i + 2] >> 2));
        result.push((bytes[i + 2] << 6) | bytes[i + 3]);
        i += 4;
    }

    if i + 1 < bytes.len() {
        result.push((bytes[i] << 2) | (bytes[i + 1] >> 4));
    }
    if i + 2 < bytes.len() {
        result.push((bytes[i + 1] << 4) | (bytes[i + 2] >> 2));
    }
    if i + 3 < bytes.len() {
        result.push((bytes[i + 2] << 6) | bytes[i + 3]);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== ConnectionTicket Tests ====================

    #[test]
    fn test_connection_ticket_roundtrip() {
        let ticket = ConnectionTicket::new(
            "node123".to_string(),
            vec!["192.168.1.1:4433".to_string()],
            "game456".to_string(),
            3600,
        );

        let serialized = ticket.to_string().unwrap();
        let deserialized = ConnectionTicket::from_string(&serialized).unwrap();

        assert_eq!(ticket.node_id, deserialized.node_id);
        assert_eq!(ticket.game_id, deserialized.game_id);
        assert_eq!(ticket.addresses, deserialized.addresses);
    }

    #[test]
    fn test_connection_ticket_multiple_addresses() {
        let addresses = vec![
            "192.168.1.1:4433".to_string(),
            "10.0.0.1:4433".to_string(),
            "[::1]:4433".to_string(),
        ];
        let ticket = ConnectionTicket::new(
            "node_multi".to_string(),
            addresses.clone(),
            "game_multi".to_string(),
            7200,
        );

        let serialized = ticket.to_string().unwrap();
        let deserialized = ConnectionTicket::from_string(&serialized).unwrap();

        assert_eq!(deserialized.addresses.len(), 3);
        assert_eq!(deserialized.addresses, addresses);
    }

    #[test]
    fn test_connection_ticket_empty_addresses() {
        let ticket = ConnectionTicket::new(
            "node_empty".to_string(),
            vec![],
            "game_empty".to_string(),
            3600,
        );

        let serialized = ticket.to_string().unwrap();
        let deserialized = ConnectionTicket::from_string(&serialized).unwrap();

        assert!(deserialized.addresses.is_empty());
    }

    #[test]
    fn test_connection_ticket_alpn_protocol() {
        let ticket = ConnectionTicket::new(
            "node1".to_string(),
            vec![],
            "game1".to_string(),
            3600,
        );

        assert_eq!(ticket.alpn, "nostr-nations/1");
    }

    #[test]
    fn test_ticket_expiration() {
        // Create a ticket that is already expired (expires_at = 0 is Unix epoch, always in the past)
        let ticket = ConnectionTicket {
            node_id: "node123".to_string(),
            addresses: vec![],
            alpn: "nostr-nations/1".to_string(),
            game_id: "game456".to_string(),
            expires_at: 0, // Unix epoch - always in the past
        };

        assert!(ticket.is_expired());
    }

    #[test]
    fn test_ticket_not_expired() {
        let ticket = ConnectionTicket::new(
            "node123".to_string(),
            vec![],
            "game456".to_string(),
            3600, // 1 hour TTL
        );

        assert!(!ticket.is_expired());
    }

    #[test]
    fn test_ticket_from_invalid_string() {
        let result = ConnectionTicket::from_string("not valid base64!!!");
        assert!(matches!(result, Err(TicketError::InvalidFormat)));
    }

    #[test]
    fn test_ticket_from_empty_string() {
        let result = ConnectionTicket::from_string("");
        assert!(matches!(result, Err(TicketError::InvalidFormat)));
    }

    #[test]
    fn test_ticket_from_valid_base64_invalid_json() {
        // Base64 encode some non-JSON data
        let invalid = base64_encode(b"not json");
        let result = ConnectionTicket::from_string(&invalid);
        assert!(matches!(result, Err(TicketError::InvalidFormat)));
    }

    // ==================== TicketError Tests ====================

    #[test]
    fn test_ticket_error_display() {
        assert_eq!(format!("{}", TicketError::InvalidFormat), "Invalid ticket format");
        assert_eq!(format!("{}", TicketError::Expired), "Ticket has expired");
        assert_eq!(format!("{}", TicketError::WrongGame), "Ticket is for a different game");
    }

    #[test]
    fn test_ticket_error_equality() {
        assert_eq!(TicketError::InvalidFormat, TicketError::InvalidFormat);
        assert_ne!(TicketError::InvalidFormat, TicketError::Expired);
    }

    // ==================== PeerMessage Tests ====================

    #[test]
    fn test_peer_message_roundtrip() {
        let msg = PeerMessage::Hello {
            peer_id: "peer123".to_string(),
            game_id: "game456".to_string(),
            player_name: "Alice".to_string(),
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::Hello {
                peer_id,
                game_id,
                player_name,
            } => {
                assert_eq!(peer_id, "peer123");
                assert_eq!(game_id, "game456");
                assert_eq!(player_name, "Alice");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_join_request() {
        let msg = PeerMessage::JoinRequest {
            player_name: "Bob".to_string(),
            civilization_id: "rome".to_string(),
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::JoinRequest { player_name, civilization_id } => {
                assert_eq!(player_name, "Bob");
                assert_eq!(civilization_id, "rome");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_join_response_accepted() {
        let msg = PeerMessage::JoinResponse {
            accepted: true,
            player_id: Some(42),
            reason: None,
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::JoinResponse { accepted, player_id, reason } => {
                assert!(accepted);
                assert_eq!(player_id, Some(42));
                assert!(reason.is_none());
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_join_response_rejected() {
        let msg = PeerMessage::JoinResponse {
            accepted: false,
            player_id: None,
            reason: Some("Game is full".to_string()),
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::JoinResponse { accepted, player_id, reason } => {
                assert!(!accepted);
                assert!(player_id.is_none());
                assert_eq!(reason, Some("Game is full".to_string()));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_game_event() {
        let msg = PeerMessage::GameEvent {
            event_json: r#"{"action":"EndTurn"}"#.to_string(),
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::GameEvent { event_json } => {
                assert!(event_json.contains("EndTurn"));
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_sync_request() {
        let msg = PeerMessage::SyncRequest {
            from_turn: 5,
            from_sequence: 10,
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::SyncRequest { from_turn, from_sequence } => {
                assert_eq!(from_turn, 5);
                assert_eq!(from_sequence, 10);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_sync_response() {
        let events = vec![
            r#"{"id":"evt1"}"#.to_string(),
            r#"{"id":"evt2"}"#.to_string(),
        ];
        let msg = PeerMessage::SyncResponse { events_json: events.clone() };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::SyncResponse { events_json } => {
                assert_eq!(events_json.len(), 2);
                assert_eq!(events_json, events);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_randomness_request() {
        let msg = PeerMessage::RandomnessRequest {
            request_id: "rand123".to_string(),
            context: "combat".to_string(),
            blinded_message: vec![1, 2, 3, 4, 5],
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::RandomnessRequest { request_id, context, blinded_message } => {
                assert_eq!(request_id, "rand123");
                assert_eq!(context, "combat");
                assert_eq!(blinded_message, vec![1, 2, 3, 4, 5]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_randomness_response() {
        let msg = PeerMessage::RandomnessResponse {
            request_id: "rand123".to_string(),
            blinded_signature: vec![10, 20, 30],
        };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::RandomnessResponse { request_id, blinded_signature } => {
                assert_eq!(request_id, "rand123");
                assert_eq!(blinded_signature, vec![10, 20, 30]);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_ping_pong() {
        let ping = PeerMessage::Ping { timestamp: 1234567890 };
        let pong = PeerMessage::Pong { timestamp: 1234567890 };

        let ping_bytes = ping.to_bytes().unwrap();
        let pong_bytes = pong.to_bytes().unwrap();

        match PeerMessage::from_bytes(&ping_bytes).unwrap() {
            PeerMessage::Ping { timestamp } => assert_eq!(timestamp, 1234567890),
            _ => panic!("Wrong message type"),
        }

        match PeerMessage::from_bytes(&pong_bytes).unwrap() {
            PeerMessage::Pong { timestamp } => assert_eq!(timestamp, 1234567890),
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_goodbye() {
        let msg = PeerMessage::Goodbye { reason: "Game ended".to_string() };

        let bytes = msg.to_bytes().unwrap();
        let decoded = PeerMessage::from_bytes(&bytes).unwrap();

        match decoded {
            PeerMessage::Goodbye { reason } => {
                assert_eq!(reason, "Game ended");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_peer_message_from_invalid_bytes() {
        let result = PeerMessage::from_bytes(b"not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_peer_message_from_empty_bytes() {
        let result = PeerMessage::from_bytes(&[]);
        assert!(result.is_err());
    }

    // ==================== ConnectionState Tests ====================

    #[test]
    fn test_connection_state_equality() {
        assert_eq!(ConnectionState::Connecting, ConnectionState::Connecting);
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_eq!(ConnectionState::Joined, ConnectionState::Joined);
        assert_eq!(ConnectionState::Disconnecting, ConnectionState::Disconnecting);
        assert_eq!(ConnectionState::Disconnected, ConnectionState::Disconnected);

        assert_ne!(ConnectionState::Connecting, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Joined);
    }

    #[test]
    fn test_connection_state_debug() {
        let state = ConnectionState::Connecting;
        let debug_str = format!("{:?}", state);
        assert!(debug_str.contains("Connecting"));
    }

    // ==================== PeerInfo Tests ====================

    #[test]
    fn test_peer_info_new() {
        let info = PeerInfo::new("peer123".to_string());

        assert_eq!(info.peer_id, "peer123");
        assert_eq!(info.state, ConnectionState::Connecting);
        assert!(info.player_name.is_none());
        assert!(info.player_id.is_none());
        assert_eq!(info.last_ping, 0);
        assert!(info.rtt_ms.is_none());
    }

    #[test]
    fn test_peer_info_clone() {
        let mut info = PeerInfo::new("peer1".to_string());
        info.player_name = Some("Alice".to_string());
        info.player_id = Some(1);
        info.state = ConnectionState::Joined;

        let cloned = info.clone();
        assert_eq!(cloned.peer_id, info.peer_id);
        assert_eq!(cloned.player_name, info.player_name);
        assert_eq!(cloned.player_id, info.player_id);
        assert_eq!(cloned.state, info.state);
    }

    // ==================== Base64 Tests ====================

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World!";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.as_slice(), decoded.as_slice());
    }

    #[test]
    fn test_base64_empty() {
        let data = b"";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert!(decoded.is_empty());
    }

    #[test]
    fn test_base64_binary_data() {
        let data: Vec<u8> = (0..=255).collect();
        let encoded = base64_encode(&data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data, decoded);
    }

    #[test]
    fn test_base64_padding() {
        // Test different lengths that require different padding
        let data1 = b"a"; // 1 byte -> needs 2 padding chars
        let data2 = b"ab"; // 2 bytes -> needs 1 padding char
        let data3 = b"abc"; // 3 bytes -> no padding

        let enc1 = base64_encode(data1);
        let enc2 = base64_encode(data2);
        let enc3 = base64_encode(data3);

        assert!(enc1.ends_with("=="));
        assert!(enc2.ends_with('=') && !enc2.ends_with("=="));
        assert!(!enc3.ends_with('='));

        assert_eq!(base64_decode(&enc1).unwrap(), data1.to_vec());
        assert_eq!(base64_decode(&enc2).unwrap(), data2.to_vec());
        assert_eq!(base64_decode(&enc3).unwrap(), data3.to_vec());
    }

    #[test]
    fn test_base64_unicode_json() {
        let json = r#"{"name":"Alice \u00e9"}"#;
        let encoded = base64_encode(json.as_bytes());
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(json.as_bytes(), decoded.as_slice());
    }

    // ==================== PeerManager Tests ====================

    #[tokio::test]
    async fn test_peer_manager() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        assert_eq!(manager.peer_count().await, 0);
        assert!(manager.is_host());

        manager.add_peer("peer1".to_string()).await;
        assert_eq!(manager.peer_count().await, 1);

        let peer = manager.get_peer("peer1").await.unwrap();
        assert_eq!(peer.state, ConnectionState::Connecting);

        manager.remove_peer("peer1", "test".to_string()).await;
        assert_eq!(manager.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_peer_manager_node_id_and_game_id() {
        let manager = PeerManager::new("my_node".to_string(), "my_game".to_string(), false);

        assert_eq!(manager.node_id(), "my_node");
        assert_eq!(manager.game_id(), "my_game");
        assert!(!manager.is_host());
    }

    #[tokio::test]
    async fn test_peer_manager_create_ticket() {
        let manager = PeerManager::new("node123".to_string(), "game456".to_string(), true);

        let addresses = vec!["192.168.1.1:4433".to_string()];
        let ticket = manager.create_ticket(addresses.clone(), 3600);

        assert_eq!(ticket.node_id, "node123");
        assert_eq!(ticket.game_id, "game456");
        assert_eq!(ticket.addresses, addresses);
        assert!(!ticket.is_expired());
    }

    #[tokio::test]
    async fn test_peer_manager_multiple_peers() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        manager.add_peer("peer1".to_string()).await;
        manager.add_peer("peer2".to_string()).await;
        manager.add_peer("peer3".to_string()).await;

        assert_eq!(manager.peer_count().await, 3);

        let peers = manager.get_peers().await;
        assert_eq!(peers.len(), 3);

        // Verify all peers are present
        let peer_ids: Vec<&str> = peers.iter().map(|p| p.peer_id.as_str()).collect();
        assert!(peer_ids.contains(&"peer1"));
        assert!(peer_ids.contains(&"peer2"));
        assert!(peer_ids.contains(&"peer3"));
    }

    #[tokio::test]
    async fn test_peer_manager_get_nonexistent_peer() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        let peer = manager.get_peer("nonexistent").await;
        assert!(peer.is_none());
    }

    #[tokio::test]
    async fn test_peer_manager_peer_joined() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        manager.add_peer("peer1".to_string()).await;
        manager.peer_joined("peer1", "Alice".to_string(), 42).await;

        let peer = manager.get_peer("peer1").await.unwrap();
        assert_eq!(peer.state, ConnectionState::Joined);
        assert_eq!(peer.player_name, Some("Alice".to_string()));
        assert_eq!(peer.player_id, Some(42));
    }

    #[tokio::test]
    async fn test_peer_manager_peer_joined_nonexistent() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        // Should not panic when peer doesn't exist
        manager.peer_joined("nonexistent", "Alice".to_string(), 42).await;

        assert!(manager.get_peer("nonexistent").await.is_none());
    }

    #[tokio::test]
    async fn test_peer_manager_handle_ping() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        manager.add_peer("peer1".to_string()).await;

        let ping = PeerMessage::Ping { timestamp: 1234567890 };
        manager.handle_message("peer1", ping).await;

        let peer = manager.get_peer("peer1").await.unwrap();
        assert_eq!(peer.last_ping, 1234567890);
    }

    #[tokio::test]
    async fn test_peer_manager_handle_goodbye() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        manager.add_peer("peer1".to_string()).await;
        assert_eq!(manager.peer_count().await, 1);

        let goodbye = PeerMessage::Goodbye { reason: "User disconnected".to_string() };
        manager.handle_message("peer1", goodbye).await;

        assert_eq!(manager.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_peer_manager_remove_peer_twice() {
        let manager = PeerManager::new("node1".to_string(), "game1".to_string(), true);

        manager.add_peer("peer1".to_string()).await;
        manager.remove_peer("peer1", "first removal".to_string()).await;
        manager.remove_peer("peer1", "second removal".to_string()).await;

        assert_eq!(manager.peer_count().await, 0);
    }
}

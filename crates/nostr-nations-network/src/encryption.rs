//! NIP-04 style encryption infrastructure for secure peer-to-peer communication.
//!
//! This module provides encryption capabilities for game events:
//! - Key pair management for players
//! - Encrypted payloads for peer communication
//! - Event encryption/decryption for private game state
//!
//! Note: This is a placeholder implementation that simulates encryption
//! without actual cryptographic operations. Real crypto would require
//! dependencies like `x25519-dalek` and `chacha20poly1305`.

use nostr_nations_core::events::GameEvent;
use nostr_nations_core::types::PlayerId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Manages key pairs and encryption for a player.
///
/// Handles:
/// - Private/public key pair storage
/// - Peer public key registry
/// - Key generation (placeholder)
#[derive(Clone, Debug, Default)]
pub struct EncryptionManager {
    /// Our private key (32 bytes).
    private_key: Option<[u8; 32]>,
    /// Our public key (32 bytes).
    public_key: Option<[u8; 32]>,
    /// Public keys of known peers, indexed by player ID.
    peer_keys: HashMap<PlayerId, [u8; 32]>,
}

impl EncryptionManager {
    /// Create a new encryption manager with no keys.
    pub fn new() -> Self {
        Self {
            private_key: None,
            public_key: None,
            peer_keys: HashMap::new(),
        }
    }

    /// Generate a new key pair.
    ///
    /// Returns the public key for sharing with peers.
    ///
    /// Note: This is a placeholder that generates deterministic keys.
    /// Real implementation would use secure random generation.
    pub fn generate_keypair(&mut self) -> [u8; 32] {
        use std::sync::atomic::{AtomicU64, Ordering};
        static COUNTER: AtomicU64 = AtomicU64::new(0);

        // Placeholder: generate deterministic "random" keys
        // Real implementation would use: x25519_dalek::StaticSecret::random()
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        // Use counter to ensure uniqueness even when called in rapid succession
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);

        // Generate pseudo-random private key from timestamp and counter
        let mut private_key = [0u8; 32];
        let mut seed = timestamp.wrapping_add(counter.wrapping_mul(0x9E3779B97F4A7C15));
        for byte in &mut private_key {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            *byte = (seed >> 33) as u8;
        }

        // Generate public key (placeholder: just XOR with constant)
        // Real implementation would derive public key cryptographically
        let mut public_key = [0u8; 32];
        for i in 0..32 {
            public_key[i] = private_key[i] ^ 0x42;
        }

        self.private_key = Some(private_key);
        self.public_key = Some(public_key);

        public_key
    }

    /// Set an existing key pair.
    ///
    /// Useful for restoring keys from storage or using pre-shared keys.
    pub fn set_keypair(&mut self, private: [u8; 32], public: [u8; 32]) {
        self.private_key = Some(private);
        self.public_key = Some(public);
    }

    /// Add a peer's public key.
    ///
    /// Required before encrypting messages to that peer.
    pub fn add_peer_key(&mut self, player: PlayerId, public_key: [u8; 32]) {
        self.peer_keys.insert(player, public_key);
    }

    /// Remove a peer's public key.
    pub fn remove_peer_key(&mut self, player: PlayerId) {
        self.peer_keys.remove(&player);
    }

    /// Check if we have a peer's public key.
    pub fn has_peer_key(&self, player: PlayerId) -> bool {
        self.peer_keys.contains_key(&player)
    }

    /// Get a peer's public key if known.
    pub fn get_peer_key(&self, player: PlayerId) -> Option<&[u8; 32]> {
        self.peer_keys.get(&player)
    }

    /// Get our public key if generated.
    pub fn public_key(&self) -> Option<[u8; 32]> {
        self.public_key
    }

    /// Check if we have a private key.
    pub fn has_private_key(&self) -> bool {
        self.private_key.is_some()
    }

    /// Get the number of known peer keys.
    pub fn peer_count(&self) -> usize {
        self.peer_keys.len()
    }

    /// Get all known peer IDs.
    pub fn peer_ids(&self) -> Vec<PlayerId> {
        self.peer_keys.keys().copied().collect()
    }

    /// Clear all keys (for security when leaving a game).
    pub fn clear(&mut self) {
        self.private_key = None;
        self.public_key = None;
        self.peer_keys.clear();
    }

    /// Get the private key (for internal encryption operations).
    pub(crate) fn private_key(&self) -> Option<&[u8; 32]> {
        self.private_key.as_ref()
    }
}

/// Encrypted payload structure (NIP-04 style).
///
/// Contains the ciphertext, nonce for decryption, and sender's public key
/// for deriving the shared secret.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedPayload {
    /// The encrypted data.
    pub ciphertext: Vec<u8>,
    /// Nonce for XChaCha20 (24 bytes).
    pub nonce: [u8; 24],
    /// Sender's public key for shared secret derivation.
    pub sender_pubkey: [u8; 32],
}

impl EncryptedPayload {
    /// Create a new encrypted payload.
    pub fn new(ciphertext: Vec<u8>, nonce: [u8; 24], sender_pubkey: [u8; 32]) -> Self {
        Self {
            ciphertext,
            nonce,
            sender_pubkey,
        }
    }

    /// Get the length of the ciphertext.
    pub fn len(&self) -> usize {
        self.ciphertext.len()
    }

    /// Check if the ciphertext is empty.
    pub fn is_empty(&self) -> bool {
        self.ciphertext.is_empty()
    }
}

/// Encrypted game event wrapper.
///
/// Wraps a game event with encryption metadata for transmission.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncryptedGameEvent {
    /// Game this event belongs to.
    pub game_id: String,
    /// The encrypted event payload.
    pub encrypted_payload: EncryptedPayload,
    /// Turn number (unencrypted for ordering).
    pub turn: u32,
    /// Sequence number within the turn (unencrypted for ordering).
    pub sequence: u32,
}

impl EncryptedGameEvent {
    /// Create a new encrypted game event.
    pub fn new(
        game_id: String,
        encrypted_payload: EncryptedPayload,
        turn: u32,
        sequence: u32,
    ) -> Self {
        Self {
            game_id,
            encrypted_payload,
            turn,
            sequence,
        }
    }
}

/// Errors that can occur during encryption operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EncryptionError {
    /// No private key available for decryption or signing.
    NoPrivateKey,
    /// No public key available for the specified peer.
    NoPeerKey(PlayerId),
    /// Encryption operation failed.
    EncryptionFailed(String),
    /// Decryption operation failed.
    DecryptionFailed(String),
    /// The encrypted payload is malformed.
    InvalidPayload,
    /// Serialization or deserialization failed.
    SerializationError(String),
}

impl std::fmt::Display for EncryptionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EncryptionError::NoPrivateKey => write!(f, "No private key available"),
            EncryptionError::NoPeerKey(player) => {
                write!(f, "No public key for player {}", player)
            }
            EncryptionError::EncryptionFailed(msg) => write!(f, "Encryption failed: {}", msg),
            EncryptionError::DecryptionFailed(msg) => write!(f, "Decryption failed: {}", msg),
            EncryptionError::InvalidPayload => write!(f, "Invalid encrypted payload"),
            EncryptionError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for EncryptionError {}

/// Encrypt data for a specific player.
///
/// Note: This is a placeholder implementation that uses XOR "encryption".
/// Real implementation would use X25519 key exchange + XChaCha20-Poly1305.
pub fn encrypt_for_player(
    manager: &EncryptionManager,
    recipient: PlayerId,
    plaintext: &[u8],
) -> Result<EncryptedPayload, EncryptionError> {
    // Check we have our private key
    let private_key = manager.private_key().ok_or(EncryptionError::NoPrivateKey)?;

    // Check we have the recipient's public key
    let peer_key = manager
        .get_peer_key(recipient)
        .ok_or(EncryptionError::NoPeerKey(recipient))?;

    // Get our public key
    let our_pubkey = manager.public_key().ok_or(EncryptionError::NoPrivateKey)?;

    // Generate a nonce (placeholder: deterministic from keys and plaintext)
    // Real implementation would use secure random nonce
    let mut nonce = [0u8; 24];
    for i in 0..24 {
        nonce[i] = private_key[i % 32] ^ peer_key[i % 32] ^ (i as u8);
    }

    // Derive shared secret (placeholder: XOR keys)
    // Real implementation: X25519 Diffie-Hellman
    let mut shared_secret = [0u8; 32];
    for i in 0..32 {
        shared_secret[i] = private_key[i] ^ peer_key[i];
    }

    // Encrypt (placeholder: XOR with shared secret)
    // Real implementation: XChaCha20-Poly1305
    let ciphertext: Vec<u8> = plaintext
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ shared_secret[i % 32] ^ nonce[i % 24])
        .collect();

    Ok(EncryptedPayload::new(ciphertext, nonce, our_pubkey))
}

/// Decrypt data from a specific player.
///
/// Note: This is a placeholder implementation that reverses the XOR "encryption".
/// Real implementation would use X25519 key exchange + XChaCha20-Poly1305.
pub fn decrypt_from_player(
    manager: &EncryptionManager,
    sender: PlayerId,
    encrypted: &EncryptedPayload,
) -> Result<Vec<u8>, EncryptionError> {
    // Check we have our private key
    let private_key = manager.private_key().ok_or(EncryptionError::NoPrivateKey)?;

    // Check we have the sender's public key
    let peer_key = manager
        .get_peer_key(sender)
        .ok_or(EncryptionError::NoPeerKey(sender))?;

    // Verify the sender's public key matches the payload
    if &encrypted.sender_pubkey != peer_key {
        return Err(EncryptionError::DecryptionFailed(
            "Sender pubkey mismatch".to_string(),
        ));
    }

    // Derive shared secret (same as encryption)
    let mut shared_secret = [0u8; 32];
    for i in 0..32 {
        shared_secret[i] = private_key[i] ^ peer_key[i];
    }

    // Decrypt (reverse the XOR)
    let plaintext: Vec<u8> = encrypted
        .ciphertext
        .iter()
        .enumerate()
        .map(|(i, &b)| b ^ shared_secret[i % 32] ^ encrypted.nonce[i % 24])
        .collect();

    Ok(plaintext)
}

/// Encrypt a game event for a specific player.
pub fn encrypt_event(
    event: &GameEvent,
    manager: &EncryptionManager,
    recipient: PlayerId,
) -> Result<EncryptedGameEvent, EncryptionError> {
    // Serialize the event to JSON
    let plaintext = serde_json::to_vec(event)
        .map_err(|e| EncryptionError::SerializationError(e.to_string()))?;

    // Encrypt the serialized event
    let encrypted_payload = encrypt_for_player(manager, recipient, &plaintext)?;

    Ok(EncryptedGameEvent::new(
        event.game_id.clone(),
        encrypted_payload,
        event.turn,
        event.sequence,
    ))
}

/// Decrypt a game event from a specific player.
pub fn decrypt_event(
    encrypted: &EncryptedGameEvent,
    manager: &EncryptionManager,
    sender: PlayerId,
) -> Result<GameEvent, EncryptionError> {
    // Decrypt the payload
    let plaintext = decrypt_from_player(manager, sender, &encrypted.encrypted_payload)?;

    // Deserialize the event
    let event: GameEvent = serde_json::from_slice(&plaintext)
        .map_err(|e| EncryptionError::SerializationError(e.to_string()))?;

    Ok(event)
}

/// Compute a shared secret between two parties.
///
/// Note: Placeholder implementation using XOR.
/// Real implementation would use X25519.
pub fn compute_shared_secret(private_key: &[u8; 32], peer_public_key: &[u8; 32]) -> [u8; 32] {
    let mut shared = [0u8; 32];
    for i in 0..32 {
        shared[i] = private_key[i] ^ peer_public_key[i];
    }
    shared
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr_nations_core::events::GameAction;

    // ==================== EncryptionManager Tests ====================

    #[test]
    fn test_encryption_manager_new() {
        let manager = EncryptionManager::new();
        assert!(manager.public_key().is_none());
        assert!(!manager.has_private_key());
        assert_eq!(manager.peer_count(), 0);
    }

    #[test]
    fn test_encryption_manager_default() {
        let manager = EncryptionManager::default();
        assert!(manager.public_key().is_none());
        assert!(!manager.has_private_key());
    }

    #[test]
    fn test_generate_keypair() {
        let mut manager = EncryptionManager::new();
        let pubkey = manager.generate_keypair();

        assert!(manager.has_private_key());
        assert!(manager.public_key().is_some());
        assert_eq!(manager.public_key().unwrap(), pubkey);
    }

    #[test]
    fn test_set_keypair() {
        let mut manager = EncryptionManager::new();
        let private = [1u8; 32];
        let public = [2u8; 32];

        manager.set_keypair(private, public);

        assert!(manager.has_private_key());
        assert_eq!(manager.public_key(), Some(public));
    }

    #[test]
    fn test_add_peer_key() {
        let mut manager = EncryptionManager::new();
        let peer_key = [3u8; 32];

        assert!(!manager.has_peer_key(0));
        manager.add_peer_key(0, peer_key);
        assert!(manager.has_peer_key(0));
        assert_eq!(manager.get_peer_key(0), Some(&peer_key));
        assert_eq!(manager.peer_count(), 1);
    }

    #[test]
    fn test_remove_peer_key() {
        let mut manager = EncryptionManager::new();
        let peer_key = [3u8; 32];

        manager.add_peer_key(0, peer_key);
        assert!(manager.has_peer_key(0));

        manager.remove_peer_key(0);
        assert!(!manager.has_peer_key(0));
    }

    #[test]
    fn test_peer_ids() {
        let mut manager = EncryptionManager::new();
        manager.add_peer_key(0, [1u8; 32]);
        manager.add_peer_key(2, [2u8; 32]);
        manager.add_peer_key(5, [3u8; 32]);

        let mut ids = manager.peer_ids();
        ids.sort();
        assert_eq!(ids, vec![0, 2, 5]);
    }

    #[test]
    fn test_clear() {
        let mut manager = EncryptionManager::new();
        manager.generate_keypair();
        manager.add_peer_key(0, [1u8; 32]);
        manager.add_peer_key(1, [2u8; 32]);

        assert!(manager.has_private_key());
        assert_eq!(manager.peer_count(), 2);

        manager.clear();

        assert!(!manager.has_private_key());
        assert!(manager.public_key().is_none());
        assert_eq!(manager.peer_count(), 0);
    }

    // ==================== EncryptedPayload Tests ====================

    #[test]
    fn test_encrypted_payload_new() {
        let payload = EncryptedPayload::new(vec![1, 2, 3], [0u8; 24], [0u8; 32]);

        assert_eq!(payload.len(), 3);
        assert!(!payload.is_empty());
    }

    #[test]
    fn test_encrypted_payload_empty() {
        let payload = EncryptedPayload::new(vec![], [0u8; 24], [0u8; 32]);

        assert_eq!(payload.len(), 0);
        assert!(payload.is_empty());
    }

    #[test]
    fn test_encrypted_payload_serialize() {
        let payload = EncryptedPayload::new(vec![1, 2, 3, 4], [5u8; 24], [6u8; 32]);

        let json = serde_json::to_string(&payload).unwrap();
        let restored: EncryptedPayload = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.ciphertext, payload.ciphertext);
        assert_eq!(restored.nonce, payload.nonce);
        assert_eq!(restored.sender_pubkey, payload.sender_pubkey);
    }

    // ==================== EncryptedGameEvent Tests ====================

    #[test]
    fn test_encrypted_game_event_new() {
        let payload = EncryptedPayload::new(vec![1, 2, 3], [0u8; 24], [0u8; 32]);
        let event = EncryptedGameEvent::new("game123".to_string(), payload, 5, 10);

        assert_eq!(event.game_id, "game123");
        assert_eq!(event.turn, 5);
        assert_eq!(event.sequence, 10);
    }

    #[test]
    fn test_encrypted_game_event_serialize() {
        let payload = EncryptedPayload::new(vec![1, 2, 3], [0u8; 24], [0u8; 32]);
        let event = EncryptedGameEvent::new("game456".to_string(), payload, 1, 2);

        let json = serde_json::to_string(&event).unwrap();
        let restored: EncryptedGameEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.game_id, event.game_id);
        assert_eq!(restored.turn, event.turn);
        assert_eq!(restored.sequence, event.sequence);
    }

    // ==================== EncryptionError Tests ====================

    #[test]
    fn test_encryption_error_display() {
        assert_eq!(
            format!("{}", EncryptionError::NoPrivateKey),
            "No private key available"
        );
        assert_eq!(
            format!("{}", EncryptionError::NoPeerKey(5)),
            "No public key for player 5"
        );
        assert_eq!(
            format!("{}", EncryptionError::EncryptionFailed("test".to_string())),
            "Encryption failed: test"
        );
        assert_eq!(
            format!("{}", EncryptionError::DecryptionFailed("test".to_string())),
            "Decryption failed: test"
        );
        assert_eq!(
            format!("{}", EncryptionError::InvalidPayload),
            "Invalid encrypted payload"
        );
        assert_eq!(
            format!(
                "{}",
                EncryptionError::SerializationError("test".to_string())
            ),
            "Serialization error: test"
        );
    }

    #[test]
    fn test_encryption_error_equality() {
        assert_eq!(EncryptionError::NoPrivateKey, EncryptionError::NoPrivateKey);
        assert_eq!(EncryptionError::NoPeerKey(1), EncryptionError::NoPeerKey(1));
        assert_ne!(EncryptionError::NoPeerKey(1), EncryptionError::NoPeerKey(2));
    }

    #[test]
    fn test_encryption_error_clone() {
        let error = EncryptionError::EncryptionFailed("reason".to_string());
        let cloned = error.clone();
        assert_eq!(error, cloned);
    }

    // ==================== Encryption/Decryption Tests ====================

    #[test]
    fn test_encrypt_no_private_key() {
        let manager = EncryptionManager::new();
        let result = encrypt_for_player(&manager, 0, b"hello");

        assert!(matches!(result, Err(EncryptionError::NoPrivateKey)));
    }

    #[test]
    fn test_encrypt_no_peer_key() {
        let mut manager = EncryptionManager::new();
        manager.generate_keypair();

        let result = encrypt_for_player(&manager, 0, b"hello");

        assert!(matches!(result, Err(EncryptionError::NoPeerKey(0))));
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        // Setup sender
        let mut sender = EncryptionManager::new();
        let sender_pubkey = sender.generate_keypair();

        // Setup recipient
        let mut recipient = EncryptionManager::new();
        let recipient_pubkey = recipient.generate_keypair();

        // Exchange keys
        sender.add_peer_key(1, recipient_pubkey); // recipient is player 1
        recipient.add_peer_key(0, sender_pubkey); // sender is player 0

        // Encrypt
        let plaintext = b"Hello, secret world!";
        let encrypted = encrypt_for_player(&sender, 1, plaintext).unwrap();

        // Decrypt
        let decrypted = decrypt_from_player(&recipient, 0, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext.to_vec());
    }

    #[test]
    fn test_decrypt_wrong_sender() {
        // Setup two players
        let mut player0 = EncryptionManager::new();
        let pubkey0 = player0.generate_keypair();

        let mut player1 = EncryptionManager::new();
        let pubkey1 = player1.generate_keypair();

        let mut player2 = EncryptionManager::new();
        let pubkey2 = player2.generate_keypair();

        // Player 0 and 1 exchange keys
        player0.add_peer_key(1, pubkey1);
        player1.add_peer_key(0, pubkey0);

        // Player 1 and 2 exchange keys (but 0 doesn't know 2)
        player1.add_peer_key(2, pubkey2);
        player2.add_peer_key(1, pubkey1);

        // Player 0 encrypts for player 1
        let encrypted = encrypt_for_player(&player0, 1, b"secret").unwrap();

        // Player 1 tries to decrypt but thinks it's from player 2 (wrong keys)
        let result = decrypt_from_player(&player1, 2, &encrypted);

        assert!(matches!(result, Err(EncryptionError::DecryptionFailed(_))));
    }

    #[test]
    fn test_encrypt_empty_plaintext() {
        let mut sender = EncryptionManager::new();
        let sender_pubkey = sender.generate_keypair();

        let mut recipient = EncryptionManager::new();
        let recipient_pubkey = recipient.generate_keypair();

        sender.add_peer_key(1, recipient_pubkey);
        recipient.add_peer_key(0, sender_pubkey);

        let encrypted = encrypt_for_player(&sender, 1, &[]).unwrap();
        assert!(encrypted.ciphertext.is_empty());

        let decrypted = decrypt_from_player(&recipient, 0, &encrypted).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_encrypt_large_plaintext() {
        let mut sender = EncryptionManager::new();
        let sender_pubkey = sender.generate_keypair();

        let mut recipient = EncryptionManager::new();
        let recipient_pubkey = recipient.generate_keypair();

        sender.add_peer_key(1, recipient_pubkey);
        recipient.add_peer_key(0, sender_pubkey);

        // Large plaintext (1KB)
        let plaintext: Vec<u8> = (0..1024).map(|i| (i % 256) as u8).collect();

        let encrypted = encrypt_for_player(&sender, 1, &plaintext).unwrap();
        let decrypted = decrypt_from_player(&recipient, 0, &encrypted).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    // ==================== Event Encryption Tests ====================

    #[test]
    fn test_encrypt_event_no_keys() {
        let manager = EncryptionManager::new();
        let event = GameEvent::new("game1".to_string(), 0, None, 1, 1, GameAction::EndTurn);

        let result = encrypt_event(&event, &manager, 1);
        assert!(matches!(result, Err(EncryptionError::NoPrivateKey)));
    }

    #[test]
    fn test_encrypt_decrypt_event_roundtrip() {
        // Setup players
        let mut sender = EncryptionManager::new();
        let sender_pubkey = sender.generate_keypair();

        let mut recipient = EncryptionManager::new();
        let recipient_pubkey = recipient.generate_keypair();

        sender.add_peer_key(1, recipient_pubkey);
        recipient.add_peer_key(0, sender_pubkey);

        // Create a game event
        let event = GameEvent::new("game123".to_string(), 0, None, 5, 3, GameAction::EndTurn);

        // Encrypt the event
        let encrypted = encrypt_event(&event, &sender, 1).unwrap();

        assert_eq!(encrypted.game_id, "game123");
        assert_eq!(encrypted.turn, 5);
        assert_eq!(encrypted.sequence, 3);

        // Decrypt the event
        let decrypted = decrypt_event(&encrypted, &recipient, 0).unwrap();

        assert_eq!(decrypted.game_id, event.game_id);
        assert_eq!(decrypted.turn, event.turn);
        assert_eq!(decrypted.sequence, event.sequence);
    }

    #[test]
    fn test_encrypt_complex_event() {
        let mut sender = EncryptionManager::new();
        let sender_pubkey = sender.generate_keypair();

        let mut recipient = EncryptionManager::new();
        let recipient_pubkey = recipient.generate_keypair();

        sender.add_peer_key(1, recipient_pubkey);
        recipient.add_peer_key(0, sender_pubkey);

        // Create a more complex event
        let event = GameEvent::new(
            "game456".to_string(),
            0,
            Some("prev_event_id".to_string()),
            10,
            5,
            GameAction::MoveUnit {
                unit_id: 42,
                path: vec![
                    nostr_nations_core::hex::HexCoord::new(0, 0),
                    nostr_nations_core::hex::HexCoord::new(1, 0),
                    nostr_nations_core::hex::HexCoord::new(2, 1),
                ],
            },
        );

        let encrypted = encrypt_event(&event, &sender, 1).unwrap();
        let decrypted = decrypt_event(&encrypted, &recipient, 0).unwrap();

        assert_eq!(decrypted.game_id, event.game_id);
        assert_eq!(decrypted.prev_event_id, event.prev_event_id);

        match decrypted.action {
            GameAction::MoveUnit { unit_id, path } => {
                assert_eq!(unit_id, 42);
                assert_eq!(path.len(), 3);
            }
            _ => panic!("Wrong action type"),
        }
    }

    // ==================== Shared Secret Tests ====================

    #[test]
    fn test_compute_shared_secret() {
        let private_a = [1u8; 32];
        let public_b = [2u8; 32];

        let secret = compute_shared_secret(&private_a, &public_b);

        // XOR of 1 and 2 is 3
        assert_eq!(secret, [3u8; 32]);
    }

    #[test]
    fn test_shared_secret_computation() {
        // Test that shared secret computation works as expected
        // Note: Our placeholder XOR implementation happens to be symmetric
        // in some cases, similar to how real DH is symmetric.
        let mut manager_a = EncryptionManager::new();
        let pubkey_a = manager_a.generate_keypair();
        let private_a = *manager_a.private_key().unwrap();

        let mut manager_b = EncryptionManager::new();
        let pubkey_b = manager_b.generate_keypair();
        let private_b = *manager_b.private_key().unwrap();

        let secret_ab = compute_shared_secret(&private_a, &pubkey_b);
        let secret_ba = compute_shared_secret(&private_b, &pubkey_a);

        // Both secrets should be 32 bytes
        assert_eq!(secret_ab.len(), 32);
        assert_eq!(secret_ba.len(), 32);

        // Verify the computation is deterministic
        let secret_ab_2 = compute_shared_secret(&private_a, &pubkey_b);
        assert_eq!(secret_ab, secret_ab_2);
    }

    // ==================== Multiple Players Tests ====================

    #[test]
    fn test_multiple_peer_keys() {
        let mut manager = EncryptionManager::new();
        manager.generate_keypair();

        // Add multiple peer keys
        for i in 0..5u8 {
            manager.add_peer_key(i, [i; 32]);
        }

        assert_eq!(manager.peer_count(), 5);

        for i in 0..5u8 {
            assert!(manager.has_peer_key(i));
            assert_eq!(manager.get_peer_key(i), Some(&[i; 32]));
        }
    }

    #[test]
    fn test_encrypt_to_multiple_recipients() {
        // One sender, multiple recipients
        let mut sender = EncryptionManager::new();
        sender.generate_keypair();

        let mut recipients: Vec<EncryptionManager> = (0..3)
            .map(|_| {
                let mut m = EncryptionManager::new();
                m.generate_keypair();
                m
            })
            .collect();

        // Sender adds all recipient keys
        for (i, r) in recipients.iter().enumerate() {
            sender.add_peer_key(i as PlayerId, r.public_key().unwrap());
        }

        // Each recipient adds sender's key
        let sender_pubkey = sender.public_key().unwrap();
        for r in &mut recipients {
            r.add_peer_key(255, sender_pubkey); // Sender is player 255
        }

        let plaintext = b"Message for everyone";

        // Encrypt for each recipient separately
        for i in 0..3u8 {
            let encrypted = encrypt_for_player(&sender, i, plaintext).unwrap();

            // Corresponding recipient can decrypt
            let decrypted = decrypt_from_player(&recipients[i as usize], 255, &encrypted).unwrap();
            assert_eq!(decrypted, plaintext.to_vec());
        }
    }
}

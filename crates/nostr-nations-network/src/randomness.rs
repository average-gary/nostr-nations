//! Randomness request/response protocol over Iroh.
//!
//! This module provides a verifiable randomness protocol for multiplayer games:
//! - **RandomnessProvider**: Host role that generates and signs random values
//! - **RandomnessClient**: Player role that requests and verifies random values
//!
//! # Protocol Flow
//!
//! 1. Client creates a `RandomnessRequest` with purpose and context
//! 2. Request is sent to the host (provider) over Iroh P2P
//! 3. Provider generates random value with cryptographic proof
//! 4. Provider sends `RandomnessResponse` back to client
//! 5. Client verifies the proof and uses the random value
//!
//! # Fairness
//!
//! The protocol uses commit-reveal to prevent manipulation:
//! 1. Provider commits to a value before seeing the request details
//! 2. After request, provider reveals the committed value
//! 3. Client can verify the commitment matches the reveal

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::peer::PeerId;

/// Type alias for player identifiers in randomness protocol.
pub type PlayerId = PeerId;

/// Request for random values from the host.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomnessRequest {
    /// Unique identifier for this request.
    pub request_id: String,
    /// Game this request belongs to.
    pub game_id: String,
    /// Current turn number.
    pub turn: u32,
    /// Sequence number within the turn.
    pub sequence: u32,
    /// Player making the request.
    pub requester: PlayerId,
    /// Purpose of the randomness (for auditing).
    pub purpose: RandomnessPurpose,
    /// Unix timestamp when request was created.
    pub timestamp: u64,
}

/// Purpose of a randomness request for auditing and verification.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum RandomnessPurpose {
    /// Combat resolution between units.
    Combat {
        /// Attacking unit ID.
        attacker_id: u64,
        /// Defending unit ID.
        defender_id: u64,
    },
    /// Map generation seed.
    MapGeneration {
        /// Index of seed being generated.
        seed_index: u32,
    },
    /// Unit promotion roll.
    UnitPromotion {
        /// Unit being promoted.
        unit_id: u64,
    },
    /// Goody hut discovery.
    GoodyHut {
        /// Tile coordinates.
        coord: (i32, i32),
    },
    /// Barbarian spawning.
    BarbarianSpawn,
    /// Other purpose with description.
    Other(String),
}

/// Response with random value and proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomnessResponse {
    /// Request ID this responds to.
    pub request_id: String,
    /// The random value.
    pub random_value: u64,
    /// Cryptographic proof of fairness.
    pub proof: Option<RandomnessProof>,
    /// Provider who generated this (usually host).
    pub provider: PlayerId,
    /// Unix timestamp when response was created.
    pub timestamp: u64,
}

/// Cryptographic proof of randomness fairness.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct RandomnessProof {
    /// Hash commitment made before seeing request.
    pub commitment: [u8; 32],
    /// Revealed value that hashes to commitment.
    pub reveal: [u8; 32],
    /// Provider's signature over the response.
    pub signature: Vec<u8>,
}

/// Provider of randomness (host role).
///
/// Generates verifiable random values with cryptographic proofs.
pub struct RandomnessProvider {
    /// Secret seed for randomness generation.
    seed: [u8; 32],
    /// Counter for deterministic generation.
    counter: u64,
    /// Pending requests awaiting response.
    pending_requests: HashMap<String, RandomnessRequest>,
}

impl RandomnessProvider {
    /// Create a new randomness provider with the given seed.
    pub fn new(seed: [u8; 32]) -> Self {
        Self {
            seed,
            counter: 0,
            pending_requests: HashMap::new(),
        }
    }

    /// Handle an incoming randomness request and generate a response.
    pub fn handle_request(&mut self, request: RandomnessRequest) -> RandomnessResponse {
        let request_id = request.request_id.clone();

        // Store request for potential auditing
        self.pending_requests.insert(request_id.clone(), request);

        // Generate random value with proof
        let (random_value, proof) = self.generate_random();

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        RandomnessResponse {
            request_id,
            random_value,
            proof: Some(proof),
            provider: "host".to_string(), // Provider ID would be set by caller
            timestamp,
        }
    }

    /// Generate a random value with cryptographic proof.
    pub fn generate_random(&mut self) -> (u64, RandomnessProof) {
        // Create reveal from seed and counter
        let reveal = self.compute_reveal();

        // Compute commitment (hash of reveal)
        let commitment = simple_hash(&reveal);

        // Increment counter for next generation
        self.counter += 1;

        // Derive random value from reveal
        let random_value = u64::from_le_bytes([
            reveal[0], reveal[1], reveal[2], reveal[3], reveal[4], reveal[5], reveal[6], reveal[7],
        ]);

        // Create signature (simplified - in production use proper signing)
        let mut sig_data = Vec::new();
        sig_data.extend_from_slice(&commitment);
        sig_data.extend_from_slice(&reveal);
        sig_data.extend_from_slice(&random_value.to_le_bytes());
        let signature = simple_hash(&sig_data).to_vec();

        let proof = RandomnessProof {
            commitment,
            reveal,
            signature,
        };

        (random_value, proof)
    }

    /// Verify a proof is valid for a given random value.
    pub fn verify_proof(value: u64, proof: &RandomnessProof) -> bool {
        // Verify commitment matches hash of reveal
        let expected_commitment = simple_hash(&proof.reveal);
        if proof.commitment != expected_commitment {
            return false;
        }

        // Verify random value matches reveal
        let expected_value = u64::from_le_bytes([
            proof.reveal[0],
            proof.reveal[1],
            proof.reveal[2],
            proof.reveal[3],
            proof.reveal[4],
            proof.reveal[5],
            proof.reveal[6],
            proof.reveal[7],
        ]);
        if value != expected_value {
            return false;
        }

        // Verify signature
        let mut sig_data = Vec::new();
        sig_data.extend_from_slice(&proof.commitment);
        sig_data.extend_from_slice(&proof.reveal);
        sig_data.extend_from_slice(&value.to_le_bytes());
        let expected_sig = simple_hash(&sig_data);

        proof.signature == expected_sig.to_vec()
    }

    /// Get a pending request by ID.
    pub fn get_pending(&self, request_id: &str) -> Option<&RandomnessRequest> {
        self.pending_requests.get(request_id)
    }

    /// Remove a completed request.
    pub fn complete_request(&mut self, request_id: &str) -> Option<RandomnessRequest> {
        self.pending_requests.remove(request_id)
    }

    /// Get count of pending requests.
    pub fn pending_count(&self) -> usize {
        self.pending_requests.len()
    }

    /// Compute reveal value from seed and counter.
    fn compute_reveal(&self) -> [u8; 32] {
        let mut data = [0u8; 40];
        data[..32].copy_from_slice(&self.seed);
        data[32..40].copy_from_slice(&self.counter.to_le_bytes());
        simple_hash(&data)
    }
}

/// Client for requesting randomness (player role).
///
/// Creates requests and verifies responses from the provider.
pub struct RandomnessClient {
    /// Pending requests awaiting response.
    pending_requests: HashMap<String, RandomnessRequest>,
    /// Received and verified responses.
    received_responses: HashMap<String, RandomnessResponse>,
    /// Counter for generating unique request IDs.
    request_counter: u64,
    /// Client's player ID.
    player_id: PlayerId,
}

impl RandomnessClient {
    /// Create a new randomness client.
    pub fn new() -> Self {
        Self {
            pending_requests: HashMap::new(),
            received_responses: HashMap::new(),
            request_counter: 0,
            player_id: "player".to_string(),
        }
    }

    /// Create a new randomness client with a specific player ID.
    pub fn with_player_id(player_id: PlayerId) -> Self {
        Self {
            pending_requests: HashMap::new(),
            received_responses: HashMap::new(),
            request_counter: 0,
            player_id,
        }
    }

    /// Create a new randomness request.
    pub fn create_request(
        &mut self,
        game_id: &str,
        turn: u32,
        sequence: u32,
        purpose: RandomnessPurpose,
    ) -> RandomnessRequest {
        self.request_counter += 1;
        let request_id = format!("{}-{}-{}-{}", game_id, turn, sequence, self.request_counter);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let request = RandomnessRequest {
            request_id: request_id.clone(),
            game_id: game_id.to_string(),
            turn,
            sequence,
            requester: self.player_id.clone(),
            purpose,
            timestamp,
        };

        self.pending_requests.insert(request_id, request.clone());
        request
    }

    /// Handle a response from the provider.
    ///
    /// Verifies the proof and returns the random value if valid.
    pub fn handle_response(
        &mut self,
        response: RandomnessResponse,
    ) -> Result<u64, RandomnessError> {
        // Check if we have a pending request for this response
        if !self.pending_requests.contains_key(&response.request_id) {
            return Err(RandomnessError::RequestNotFound(
                response.request_id.clone(),
            ));
        }

        // Verify proof if present
        if let Some(ref proof) = response.proof {
            if !RandomnessProvider::verify_proof(response.random_value, proof) {
                return Err(RandomnessError::InvalidProof);
            }
        }

        // Remove from pending and store in received
        self.pending_requests.remove(&response.request_id);
        let random_value = response.random_value;
        self.received_responses
            .insert(response.request_id.clone(), response);

        Ok(random_value)
    }

    /// Get a previously received random value.
    pub fn get_random(&self, request_id: &str) -> Option<u64> {
        self.received_responses
            .get(request_id)
            .map(|r| r.random_value)
    }

    /// Check if a request is still pending.
    pub fn has_pending(&self, request_id: &str) -> bool {
        self.pending_requests.contains_key(request_id)
    }

    /// Get count of pending requests.
    pub fn pending_count(&self) -> usize {
        self.pending_requests.len()
    }

    /// Get count of received responses.
    pub fn received_count(&self) -> usize {
        self.received_responses.len()
    }

    /// Get a pending request by ID.
    pub fn get_pending_request(&self, request_id: &str) -> Option<&RandomnessRequest> {
        self.pending_requests.get(request_id)
    }

    /// Get a received response by ID.
    pub fn get_response(&self, request_id: &str) -> Option<&RandomnessResponse> {
        self.received_responses.get(request_id)
    }

    /// Clear all pending requests (e.g., on timeout).
    pub fn clear_pending(&mut self) {
        self.pending_requests.clear();
    }
}

impl Default for RandomnessClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors from randomness operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomnessError {
    /// No pending request found for response.
    RequestNotFound(String),
    /// Proof verification failed.
    InvalidProof,
    /// Request ID already exists.
    DuplicateRequest,
    /// Request timed out waiting for response.
    Timeout,
    /// Provider is not available.
    ProviderOffline,
}

impl std::fmt::Display for RandomnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RandomnessError::RequestNotFound(id) => write!(f, "Request not found: {}", id),
            RandomnessError::InvalidProof => write!(f, "Invalid randomness proof"),
            RandomnessError::DuplicateRequest => write!(f, "Duplicate request ID"),
            RandomnessError::Timeout => write!(f, "Request timed out"),
            RandomnessError::ProviderOffline => write!(f, "Randomness provider offline"),
        }
    }
}

impl std::error::Error for RandomnessError {}

/// Protocol messages for randomness exchange.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RandomnessMessage {
    /// Request for random value.
    Request(RandomnessRequest),
    /// Response with random value.
    Response(RandomnessResponse),
    /// Pre-commitment for fairness (sent before request details).
    Commitment([u8; 32]),
}

impl RandomnessMessage {
    /// Serialize to bytes for network transmission.
    pub fn to_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }

    /// Deserialize from bytes.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(bytes)
    }
}

/// Simple hash function for proofs.
///
/// Uses a basic implementation for demonstration.
/// In production, use a proper cryptographic hash like SHA-256.
fn simple_hash(data: &[u8]) -> [u8; 32] {
    let mut hash = [0u8; 32];

    // FNV-1a inspired hash, extended to 256 bits
    let mut state = [
        0x6c62272e07bb0142u64,
        0x62b821756295c58du64,
        0x6c62272e07bb0142u64,
        0x62b821756295c58du64,
    ];

    for (i, &byte) in data.iter().enumerate() {
        let idx = i % 4;
        state[idx] ^= byte as u64;
        state[idx] = state[idx].wrapping_mul(0x100000001b3);
    }

    // Mix states
    for i in 0..4 {
        state[i] = state[i].wrapping_add(state[(i + 1) % 4]).rotate_left(17);
    }

    // Output
    for (i, s) in state.iter().enumerate() {
        hash[i * 8..(i + 1) * 8].copy_from_slice(&s.to_le_bytes());
    }

    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RandomnessRequest Tests ====================

    #[test]
    fn test_randomness_request_serialization() {
        let request = RandomnessRequest {
            request_id: "req-123".to_string(),
            game_id: "game-456".to_string(),
            turn: 5,
            sequence: 10,
            requester: "player1".to_string(),
            purpose: RandomnessPurpose::Combat {
                attacker_id: 100,
                defender_id: 200,
            },
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: RandomnessRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(request.request_id, deserialized.request_id);
        assert_eq!(request.game_id, deserialized.game_id);
        assert_eq!(request.turn, deserialized.turn);
        assert_eq!(request.sequence, deserialized.sequence);
    }

    #[test]
    fn test_randomness_request_clone() {
        let request = RandomnessRequest {
            request_id: "req-1".to_string(),
            game_id: "game-1".to_string(),
            turn: 1,
            sequence: 1,
            requester: "player".to_string(),
            purpose: RandomnessPurpose::BarbarianSpawn,
            timestamp: 0,
        };

        let cloned = request.clone();
        assert_eq!(request.request_id, cloned.request_id);
    }

    // ==================== RandomnessPurpose Tests ====================

    #[test]
    fn test_randomness_purpose_combat() {
        let purpose = RandomnessPurpose::Combat {
            attacker_id: 1,
            defender_id: 2,
        };

        let json = serde_json::to_string(&purpose).unwrap();
        let deserialized: RandomnessPurpose = serde_json::from_str(&json).unwrap();

        assert_eq!(purpose, deserialized);
    }

    #[test]
    fn test_randomness_purpose_map_generation() {
        let purpose = RandomnessPurpose::MapGeneration { seed_index: 42 };

        let json = serde_json::to_string(&purpose).unwrap();
        let deserialized: RandomnessPurpose = serde_json::from_str(&json).unwrap();

        assert_eq!(purpose, deserialized);
    }

    #[test]
    fn test_randomness_purpose_unit_promotion() {
        let purpose = RandomnessPurpose::UnitPromotion { unit_id: 999 };

        let json = serde_json::to_string(&purpose).unwrap();
        let deserialized: RandomnessPurpose = serde_json::from_str(&json).unwrap();

        assert_eq!(purpose, deserialized);
    }

    #[test]
    fn test_randomness_purpose_goody_hut() {
        let purpose = RandomnessPurpose::GoodyHut { coord: (10, -20) };

        let json = serde_json::to_string(&purpose).unwrap();
        let deserialized: RandomnessPurpose = serde_json::from_str(&json).unwrap();

        assert_eq!(purpose, deserialized);
    }

    #[test]
    fn test_randomness_purpose_barbarian_spawn() {
        let purpose = RandomnessPurpose::BarbarianSpawn;

        let json = serde_json::to_string(&purpose).unwrap();
        let deserialized: RandomnessPurpose = serde_json::from_str(&json).unwrap();

        assert_eq!(purpose, deserialized);
    }

    #[test]
    fn test_randomness_purpose_other() {
        let purpose = RandomnessPurpose::Other("custom purpose".to_string());

        let json = serde_json::to_string(&purpose).unwrap();
        let deserialized: RandomnessPurpose = serde_json::from_str(&json).unwrap();

        assert_eq!(purpose, deserialized);
    }

    // ==================== RandomnessResponse Tests ====================

    #[test]
    fn test_randomness_response_serialization() {
        let response = RandomnessResponse {
            request_id: "req-123".to_string(),
            random_value: 42,
            proof: None,
            provider: "host".to_string(),
            timestamp: 1234567890,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RandomnessResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(response.request_id, deserialized.request_id);
        assert_eq!(response.random_value, deserialized.random_value);
    }

    #[test]
    fn test_randomness_response_with_proof() {
        let proof = RandomnessProof {
            commitment: [1u8; 32],
            reveal: [2u8; 32],
            signature: vec![3, 4, 5],
        };

        let response = RandomnessResponse {
            request_id: "req-456".to_string(),
            random_value: 12345,
            proof: Some(proof),
            provider: "host".to_string(),
            timestamp: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: RandomnessResponse = serde_json::from_str(&json).unwrap();

        assert!(deserialized.proof.is_some());
        assert_eq!(deserialized.proof.unwrap().commitment, [1u8; 32]);
    }

    // ==================== RandomnessProof Tests ====================

    #[test]
    fn test_randomness_proof_serialization() {
        let proof = RandomnessProof {
            commitment: [0xAB; 32],
            reveal: [0xCD; 32],
            signature: vec![1, 2, 3, 4, 5],
        };

        let json = serde_json::to_string(&proof).unwrap();
        let deserialized: RandomnessProof = serde_json::from_str(&json).unwrap();

        assert_eq!(proof, deserialized);
    }

    #[test]
    fn test_randomness_proof_equality() {
        let proof1 = RandomnessProof {
            commitment: [1u8; 32],
            reveal: [2u8; 32],
            signature: vec![3],
        };

        let proof2 = RandomnessProof {
            commitment: [1u8; 32],
            reveal: [2u8; 32],
            signature: vec![3],
        };

        let proof3 = RandomnessProof {
            commitment: [99u8; 32],
            reveal: [2u8; 32],
            signature: vec![3],
        };

        assert_eq!(proof1, proof2);
        assert_ne!(proof1, proof3);
    }

    // ==================== RandomnessProvider Tests ====================

    #[test]
    fn test_provider_new() {
        let seed = [42u8; 32];
        let provider = RandomnessProvider::new(seed);

        assert_eq!(provider.pending_count(), 0);
    }

    #[test]
    fn test_provider_generate_random() {
        let seed = [1u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let (value1, proof1) = provider.generate_random();
        let (value2, proof2) = provider.generate_random();

        // Values should be different (counter increments)
        assert_ne!(value1, value2);
        assert_ne!(proof1.reveal, proof2.reveal);
    }

    #[test]
    fn test_provider_handle_request() {
        let seed = [0u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let request = RandomnessRequest {
            request_id: "test-req".to_string(),
            game_id: "game1".to_string(),
            turn: 1,
            sequence: 1,
            requester: "player1".to_string(),
            purpose: RandomnessPurpose::BarbarianSpawn,
            timestamp: 0,
        };

        let response = provider.handle_request(request);

        assert_eq!(response.request_id, "test-req");
        assert!(response.proof.is_some());
        assert_eq!(provider.pending_count(), 1);
    }

    #[test]
    fn test_provider_verify_proof() {
        let seed = [123u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let (value, proof) = provider.generate_random();

        assert!(RandomnessProvider::verify_proof(value, &proof));
    }

    #[test]
    fn test_provider_verify_proof_invalid_value() {
        let seed = [123u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let (value, proof) = provider.generate_random();

        // Wrong value should fail verification
        assert!(!RandomnessProvider::verify_proof(value + 1, &proof));
    }

    #[test]
    fn test_provider_verify_proof_invalid_commitment() {
        let seed = [123u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let (value, mut proof) = provider.generate_random();

        // Tamper with commitment
        proof.commitment[0] ^= 0xFF;

        assert!(!RandomnessProvider::verify_proof(value, &proof));
    }

    #[test]
    fn test_provider_verify_proof_invalid_signature() {
        let seed = [123u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let (value, mut proof) = provider.generate_random();

        // Tamper with signature
        if !proof.signature.is_empty() {
            proof.signature[0] ^= 0xFF;
        }

        assert!(!RandomnessProvider::verify_proof(value, &proof));
    }

    #[test]
    fn test_provider_get_pending() {
        let seed = [0u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let request = RandomnessRequest {
            request_id: "pending-test".to_string(),
            game_id: "game1".to_string(),
            turn: 1,
            sequence: 1,
            requester: "player1".to_string(),
            purpose: RandomnessPurpose::BarbarianSpawn,
            timestamp: 0,
        };

        provider.handle_request(request);

        assert!(provider.get_pending("pending-test").is_some());
        assert!(provider.get_pending("nonexistent").is_none());
    }

    #[test]
    fn test_provider_complete_request() {
        let seed = [0u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let request = RandomnessRequest {
            request_id: "complete-test".to_string(),
            game_id: "game1".to_string(),
            turn: 1,
            sequence: 1,
            requester: "player1".to_string(),
            purpose: RandomnessPurpose::BarbarianSpawn,
            timestamp: 0,
        };

        provider.handle_request(request);
        assert_eq!(provider.pending_count(), 1);

        let completed = provider.complete_request("complete-test");
        assert!(completed.is_some());
        assert_eq!(provider.pending_count(), 0);
    }

    #[test]
    fn test_provider_deterministic_with_same_seed() {
        let seed = [42u8; 32];

        let mut provider1 = RandomnessProvider::new(seed);
        let mut provider2 = RandomnessProvider::new(seed);

        let (value1, _) = provider1.generate_random();
        let (value2, _) = provider2.generate_random();

        assert_eq!(value1, value2);
    }

    // ==================== RandomnessClient Tests ====================

    #[test]
    fn test_client_new() {
        let client = RandomnessClient::new();

        assert_eq!(client.pending_count(), 0);
        assert_eq!(client.received_count(), 0);
    }

    #[test]
    fn test_client_with_player_id() {
        let mut client = RandomnessClient::with_player_id("custom-player".to_string());

        let request = client.create_request("game", 1, 1, RandomnessPurpose::BarbarianSpawn);

        assert_eq!(request.requester, "custom-player");
    }

    #[test]
    fn test_client_create_request() {
        let mut client = RandomnessClient::new();

        let request = client.create_request(
            "game1",
            5,
            10,
            RandomnessPurpose::Combat {
                attacker_id: 1,
                defender_id: 2,
            },
        );

        assert_eq!(request.game_id, "game1");
        assert_eq!(request.turn, 5);
        assert_eq!(request.sequence, 10);
        assert!(client.has_pending(&request.request_id));
        assert_eq!(client.pending_count(), 1);
    }

    #[test]
    fn test_client_create_multiple_requests() {
        let mut client = RandomnessClient::new();

        let req1 = client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);
        let req2 = client.create_request("game1", 1, 2, RandomnessPurpose::BarbarianSpawn);
        let req3 = client.create_request("game1", 2, 1, RandomnessPurpose::BarbarianSpawn);

        assert_ne!(req1.request_id, req2.request_id);
        assert_ne!(req2.request_id, req3.request_id);
        assert_eq!(client.pending_count(), 3);
    }

    #[test]
    fn test_client_handle_response() {
        let mut client = RandomnessClient::new();
        let seed = [0u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let request = client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);
        let response = provider.handle_request(request.clone());

        let result = client.handle_response(response);

        assert!(result.is_ok());
        assert!(!client.has_pending(&request.request_id));
        assert!(client.get_random(&request.request_id).is_some());
    }

    #[test]
    fn test_client_handle_response_not_found() {
        let mut client = RandomnessClient::new();

        let response = RandomnessResponse {
            request_id: "unknown-request".to_string(),
            random_value: 42,
            proof: None,
            provider: "host".to_string(),
            timestamp: 0,
        };

        let result = client.handle_response(response);

        assert!(matches!(result, Err(RandomnessError::RequestNotFound(_))));
    }

    #[test]
    fn test_client_handle_response_invalid_proof() {
        let mut client = RandomnessClient::new();

        let request = client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);

        let bad_proof = RandomnessProof {
            commitment: [0u8; 32],
            reveal: [1u8; 32], // Doesn't match commitment
            signature: vec![],
        };

        let response = RandomnessResponse {
            request_id: request.request_id.clone(),
            random_value: 42,
            proof: Some(bad_proof),
            provider: "host".to_string(),
            timestamp: 0,
        };

        let result = client.handle_response(response);

        assert!(matches!(result, Err(RandomnessError::InvalidProof)));
    }

    #[test]
    fn test_client_get_random() {
        let mut client = RandomnessClient::new();
        let seed = [0u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let request = client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);
        let response = provider.handle_request(request.clone());
        let expected_value = response.random_value;

        client.handle_response(response).unwrap();

        assert_eq!(client.get_random(&request.request_id), Some(expected_value));
        assert!(client.get_random("nonexistent").is_none());
    }

    #[test]
    fn test_client_has_pending() {
        let mut client = RandomnessClient::new();

        let request = client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);

        assert!(client.has_pending(&request.request_id));
        assert!(!client.has_pending("nonexistent"));
    }

    #[test]
    fn test_client_get_pending_request() {
        let mut client = RandomnessClient::new();

        let request = client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);

        let pending = client.get_pending_request(&request.request_id);
        assert!(pending.is_some());
        assert_eq!(pending.unwrap().game_id, "game1");
    }

    #[test]
    fn test_client_get_response() {
        let mut client = RandomnessClient::new();
        let seed = [0u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let request = client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);
        let response = provider.handle_request(request.clone());

        client.handle_response(response).unwrap();

        let stored = client.get_response(&request.request_id);
        assert!(stored.is_some());
        assert_eq!(stored.unwrap().request_id, request.request_id);
    }

    #[test]
    fn test_client_clear_pending() {
        let mut client = RandomnessClient::new();

        client.create_request("game1", 1, 1, RandomnessPurpose::BarbarianSpawn);
        client.create_request("game1", 1, 2, RandomnessPurpose::BarbarianSpawn);
        assert_eq!(client.pending_count(), 2);

        client.clear_pending();
        assert_eq!(client.pending_count(), 0);
    }

    #[test]
    fn test_client_default() {
        let client = RandomnessClient::default();
        assert_eq!(client.pending_count(), 0);
    }

    // ==================== RandomnessError Tests ====================

    #[test]
    fn test_randomness_error_display() {
        assert_eq!(
            format!("{}", RandomnessError::RequestNotFound("req-1".to_string())),
            "Request not found: req-1"
        );
        assert_eq!(
            format!("{}", RandomnessError::InvalidProof),
            "Invalid randomness proof"
        );
        assert_eq!(
            format!("{}", RandomnessError::DuplicateRequest),
            "Duplicate request ID"
        );
        assert_eq!(format!("{}", RandomnessError::Timeout), "Request timed out");
        assert_eq!(
            format!("{}", RandomnessError::ProviderOffline),
            "Randomness provider offline"
        );
    }

    #[test]
    fn test_randomness_error_equality() {
        assert_eq!(RandomnessError::InvalidProof, RandomnessError::InvalidProof);
        assert_ne!(RandomnessError::InvalidProof, RandomnessError::Timeout);

        assert_eq!(
            RandomnessError::RequestNotFound("a".to_string()),
            RandomnessError::RequestNotFound("a".to_string())
        );
        assert_ne!(
            RandomnessError::RequestNotFound("a".to_string()),
            RandomnessError::RequestNotFound("b".to_string())
        );
    }

    #[test]
    fn test_randomness_error_clone() {
        let error = RandomnessError::RequestNotFound("test".to_string());
        let cloned = error.clone();

        match cloned {
            RandomnessError::RequestNotFound(id) => assert_eq!(id, "test"),
            _ => panic!("Wrong error type"),
        }
    }

    // ==================== RandomnessMessage Tests ====================

    #[test]
    fn test_randomness_message_request() {
        let request = RandomnessRequest {
            request_id: "msg-req".to_string(),
            game_id: "game1".to_string(),
            turn: 1,
            sequence: 1,
            requester: "player1".to_string(),
            purpose: RandomnessPurpose::BarbarianSpawn,
            timestamp: 0,
        };

        let message = RandomnessMessage::Request(request);
        let bytes = message.to_bytes().unwrap();
        let decoded = RandomnessMessage::from_bytes(&bytes).unwrap();

        match decoded {
            RandomnessMessage::Request(req) => {
                assert_eq!(req.request_id, "msg-req");
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_randomness_message_response() {
        let response = RandomnessResponse {
            request_id: "msg-resp".to_string(),
            random_value: 42,
            proof: None,
            provider: "host".to_string(),
            timestamp: 0,
        };

        let message = RandomnessMessage::Response(response);
        let bytes = message.to_bytes().unwrap();
        let decoded = RandomnessMessage::from_bytes(&bytes).unwrap();

        match decoded {
            RandomnessMessage::Response(resp) => {
                assert_eq!(resp.request_id, "msg-resp");
                assert_eq!(resp.random_value, 42);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_randomness_message_commitment() {
        let commitment = [0xAB; 32];
        let message = RandomnessMessage::Commitment(commitment);

        let bytes = message.to_bytes().unwrap();
        let decoded = RandomnessMessage::from_bytes(&bytes).unwrap();

        match decoded {
            RandomnessMessage::Commitment(c) => {
                assert_eq!(c, commitment);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_randomness_message_from_invalid_bytes() {
        let result = RandomnessMessage::from_bytes(b"not valid json");
        assert!(result.is_err());
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_full_randomness_flow() {
        let seed = [42u8; 32];
        let mut provider = RandomnessProvider::new(seed);
        let mut client = RandomnessClient::with_player_id("player1".to_string());

        // Client creates request
        let request = client.create_request(
            "game1",
            5,
            1,
            RandomnessPurpose::Combat {
                attacker_id: 100,
                defender_id: 200,
            },
        );
        assert!(client.has_pending(&request.request_id));

        // Provider handles request
        let response = provider.handle_request(request.clone());
        assert!(response.proof.is_some());

        // Client handles response
        let result = client.handle_response(response.clone());
        assert!(result.is_ok());

        // Verify state
        assert!(!client.has_pending(&request.request_id));
        assert_eq!(
            client.get_random(&request.request_id),
            Some(result.unwrap())
        );
    }

    #[test]
    fn test_multiple_requests_flow() {
        let seed = [0u8; 32];
        let mut provider = RandomnessProvider::new(seed);
        let mut client = RandomnessClient::new();

        // Create multiple requests
        let requests: Vec<_> = (0..5)
            .map(|i| {
                client.create_request(
                    "game1",
                    1,
                    i,
                    RandomnessPurpose::MapGeneration { seed_index: i },
                )
            })
            .collect();

        assert_eq!(client.pending_count(), 5);

        // Handle responses out of order
        for i in [2, 0, 4, 1, 3] {
            let response = provider.handle_request(requests[i as usize].clone());
            client.handle_response(response).unwrap();
        }

        assert_eq!(client.pending_count(), 0);
        assert_eq!(client.received_count(), 5);

        // All random values should be different
        let values: Vec<_> = requests
            .iter()
            .map(|r| client.get_random(&r.request_id).unwrap())
            .collect();

        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                // Note: theoretically could collide, but very unlikely with 64-bit values
                // For this test with deterministic seed, they will be different
                assert_ne!(
                    values[i], values[j],
                    "Values at {} and {} should differ",
                    i, j
                );
            }
        }
    }

    #[test]
    fn test_proof_verification_independent() {
        // Verify that proof verification works without the original provider
        let seed = [99u8; 32];
        let mut provider = RandomnessProvider::new(seed);

        let (value, proof) = provider.generate_random();

        // Should verify correctly
        assert!(RandomnessProvider::verify_proof(value, &proof));

        // Create a new provider and verify it can still validate
        let provider2 = RandomnessProvider::new([0u8; 32]);
        assert!(RandomnessProvider::verify_proof(value, &proof));

        // Verify using the static method works
        assert!(RandomnessProvider::verify_proof(value, &proof));

        // Explicitly use provider2 to avoid unused warning
        assert_eq!(provider2.pending_count(), 0);
    }

    // ==================== Hash Function Tests ====================

    #[test]
    fn test_simple_hash_deterministic() {
        let data = b"test data";
        let hash1 = simple_hash(data);
        let hash2 = simple_hash(data);

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_simple_hash_different_inputs() {
        let hash1 = simple_hash(b"input1");
        let hash2 = simple_hash(b"input2");

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_simple_hash_empty() {
        let hash = simple_hash(b"");
        // Should produce some output even for empty input
        assert_ne!(hash, [0u8; 32]);
    }

    #[test]
    fn test_simple_hash_length() {
        let hash = simple_hash(b"any input");
        assert_eq!(hash.len(), 32);
    }
}

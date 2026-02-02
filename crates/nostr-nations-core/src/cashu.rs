//! Cashu integration for verifiable randomness.
//!
//! This module provides unbiased randomness for game mechanics using Cashu
//! blinded signatures. The protocol ensures that neither the game host nor
//! players can predict or manipulate random outcomes.
//!
//! # Protocol Overview
//!
//! 1. **Commitment Phase**: Player/game creates a random nonce (blinding factor)
//! 2. **Blinding Phase**: Combine nonce with request data to create blinded message
//! 3. **Signing Phase**: Cashu mint signs the blinded message (without seeing content)
//! 4. **Unblinding Phase**: Remove blinding factor to get deterministic random value
//! 5. **Verification**: Anyone can verify the signature matches the mint's public key
//!
//! # Randomness Applications
//!
//! - **Combat**: Determines damage variance (±20%)
//! - **Map Generation**: Seeds the procedural generator
//! - **Exploration**: Goody hut outcomes, barbarian spawns
//! - **Diplomacy**: AI decision variance

use serde::{Deserialize, Serialize};

/// A proof of randomness from a Cashu mint.
///
/// This proof can be verified by anyone to confirm the randomness
/// was generated fairly by a third-party mint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomnessProof {
    /// The mint's public key (keyset ID).
    pub mint_keyset_id: String,
    /// The blinded message that was signed.
    pub blinded_message: Vec<u8>,
    /// The mint's signature on the blinded message.
    pub blinded_signature: Vec<u8>,
    /// The unblinded signature (proof).
    pub signature: Vec<u8>,
    /// The derived random bytes (32 bytes for seeds, or hashed for floats).
    pub random_bytes: [u8; 32],
    /// Context string describing what this randomness is for.
    pub context: String,
    /// Timestamp when the randomness was generated.
    pub timestamp: u64,
}

impl RandomnessProof {
    /// Convert the random bytes to a float in range [0.0, 1.0).
    pub fn to_f32(&self) -> f32 {
        // Use first 4 bytes as u32, then normalize
        let value = u32::from_le_bytes([
            self.random_bytes[0],
            self.random_bytes[1],
            self.random_bytes[2],
            self.random_bytes[3],
        ]);
        (value as f32) / (u32::MAX as f32)
    }

    /// Convert the random bytes to a u64.
    pub fn to_u64(&self) -> u64 {
        u64::from_le_bytes([
            self.random_bytes[0],
            self.random_bytes[1],
            self.random_bytes[2],
            self.random_bytes[3],
            self.random_bytes[4],
            self.random_bytes[5],
            self.random_bytes[6],
            self.random_bytes[7],
        ])
    }

    /// Get the full 32-byte seed (for map generation).
    pub fn to_seed(&self) -> [u8; 32] {
        self.random_bytes
    }

    /// Get a random value in range [0, max).
    pub fn to_range(&self, max: u32) -> u32 {
        if max == 0 {
            return 0;
        }
        let value = u32::from_le_bytes([
            self.random_bytes[0],
            self.random_bytes[1],
            self.random_bytes[2],
            self.random_bytes[3],
        ]);
        value % max
    }
}

/// Request for randomness from a Cashu mint.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RandomnessRequest {
    /// Unique identifier for this request.
    pub request_id: String,
    /// Context describing what the randomness is for.
    pub context: RandomnessContext,
    /// The player requesting (their Nostr pubkey).
    pub requester_pubkey: String,
    /// Game-specific data to bind the randomness to.
    pub binding_data: Vec<u8>,
}

/// Context for a randomness request.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum RandomnessContext {
    /// Map generation seed.
    MapGeneration { game_id: String },
    /// Combat resolution.
    Combat {
        game_id: String,
        turn: u32,
        attacker_id: u64,
        defender_id: u64,
    },
    /// Exploration/goody hut.
    Exploration {
        game_id: String,
        turn: u32,
        hex_q: i32,
        hex_r: i32,
    },
    /// Barbarian spawn.
    BarbarianSpawn { game_id: String, turn: u32 },
    /// Generic game event.
    GameEvent {
        game_id: String,
        turn: u32,
        event_type: String,
    },
}

impl RandomnessContext {
    /// Convert context to bytes for binding.
    pub fn to_bytes(&self) -> Vec<u8> {
        // Use JSON serialization for deterministic binding
        serde_json::to_vec(self).unwrap_or_default()
    }
}

/// Trait for providing randomness (implemented by Cashu client or fallback).
pub trait RandomnessProvider {
    /// Request randomness for the given context.
    ///
    /// Returns a proof that can be verified and stored in Nostr events.
    fn request_randomness(
        &mut self,
        context: RandomnessContext,
    ) -> Result<RandomnessProof, RandomnessError>;

    /// Verify a randomness proof is valid.
    fn verify_proof(&self, proof: &RandomnessProof) -> Result<bool, RandomnessError>;
}

/// Errors from randomness operations.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RandomnessError {
    /// Mint is not available.
    MintUnavailable,
    /// Invalid keyset.
    InvalidKeyset,
    /// Signature verification failed.
    InvalidSignature,
    /// Network error.
    NetworkError(String),
    /// Protocol error.
    ProtocolError(String),
}

impl std::fmt::Display for RandomnessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RandomnessError::MintUnavailable => write!(f, "Cashu mint is unavailable"),
            RandomnessError::InvalidKeyset => write!(f, "Invalid mint keyset"),
            RandomnessError::InvalidSignature => write!(f, "Signature verification failed"),
            RandomnessError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            RandomnessError::ProtocolError(msg) => write!(f, "Protocol error: {}", msg),
        }
    }
}

impl std::error::Error for RandomnessError {}

/// Deterministic fallback randomness provider for offline/local play.
///
/// This uses a seeded PRNG that produces verifiable (but not unbiased)
/// randomness. Use only for local/trusted games.
#[derive(Clone, Debug)]
pub struct DeterministicRandomness {
    /// Current state of the PRNG.
    state: u64,
    /// Initial seed for verification.
    seed: [u8; 32],
}

impl DeterministicRandomness {
    /// Create a new deterministic randomness provider with the given seed.
    pub fn new(seed: [u8; 32]) -> Self {
        let mut state: u64 = 0;
        for (i, &byte) in seed.iter().enumerate() {
            state ^= (byte as u64) << ((i % 8) * 8);
        }
        if state == 0 {
            state = 0x853c49e6748fea9b;
        }
        Self { state, seed }
    }

    /// Generate next random bytes.
    fn next_bytes(&mut self) -> [u8; 32] {
        let mut result = [0u8; 32];
        for chunk in result.chunks_mut(8) {
            // xorshift64*
            self.state ^= self.state >> 12;
            self.state ^= self.state << 25;
            self.state ^= self.state >> 27;
            let value = self.state.wrapping_mul(0x2545F4914F6CDD1D);
            let bytes = value.to_le_bytes();
            chunk.copy_from_slice(&bytes[..chunk.len()]);
        }
        result
    }

    /// Get current timestamp (seconds since Unix epoch).
    fn current_timestamp() -> u64 {
        // In a real implementation, use std::time::SystemTime
        // For now, return 0 (tests don't depend on timestamp)
        0
    }
}

impl RandomnessProvider for DeterministicRandomness {
    fn request_randomness(
        &mut self,
        context: RandomnessContext,
    ) -> Result<RandomnessProof, RandomnessError> {
        // Mix context into state for binding
        let context_bytes = context.to_bytes();
        for (i, &byte) in context_bytes.iter().enumerate() {
            self.state ^= (byte as u64) << ((i % 8) * 8);
        }

        let random_bytes = self.next_bytes();

        Ok(RandomnessProof {
            mint_keyset_id: "deterministic".to_string(),
            blinded_message: Vec::new(),
            blinded_signature: Vec::new(),
            signature: self.seed.to_vec(),
            random_bytes,
            context: format!("{:?}", context),
            timestamp: Self::current_timestamp(),
        })
    }

    fn verify_proof(&self, proof: &RandomnessProof) -> Result<bool, RandomnessError> {
        // Deterministic proofs are always "valid" if they match the seed
        Ok(proof.mint_keyset_id == "deterministic" && proof.signature == self.seed.to_vec())
    }
}

/// Configuration for Cashu mint connection.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CashuConfig {
    /// Mint URL.
    pub mint_url: String,
    /// Preferred keyset ID (or None for default).
    pub keyset_id: Option<String>,
    /// Request timeout in seconds.
    pub timeout_secs: u32,
    /// Whether to fall back to deterministic randomness on mint failure.
    pub allow_fallback: bool,
}

impl Default for CashuConfig {
    fn default() -> Self {
        Self {
            mint_url: String::new(),
            keyset_id: None,
            timeout_secs: 30,
            allow_fallback: true,
        }
    }
}

/// Manager for randomness requests during a game.
///
/// Handles batching, caching, and fallback logic.
#[derive(Debug)]
pub struct RandomnessManager {
    /// Configuration.
    config: CashuConfig,
    /// Fallback provider.
    fallback: DeterministicRandomness,
    /// Cache of generated proofs (by request context hash).
    proof_cache: std::collections::HashMap<String, RandomnessProof>,
    /// Whether we're in offline mode.
    offline_mode: bool,
}

impl RandomnessManager {
    /// Create a new randomness manager.
    pub fn new(config: CashuConfig, fallback_seed: [u8; 32]) -> Self {
        let offline_mode = config.mint_url.is_empty();
        Self {
            config,
            fallback: DeterministicRandomness::new(fallback_seed),
            proof_cache: std::collections::HashMap::new(),
            offline_mode,
        }
    }

    /// Check if operating in offline mode.
    pub fn is_offline(&self) -> bool {
        self.offline_mode
    }

    /// Request randomness for combat.
    pub fn combat_random(
        &mut self,
        game_id: &str,
        turn: u32,
        attacker_id: u64,
        defender_id: u64,
    ) -> Result<RandomnessProof, RandomnessError> {
        let context = RandomnessContext::Combat {
            game_id: game_id.to_string(),
            turn,
            attacker_id,
            defender_id,
        };
        self.request_with_cache(context)
    }

    /// Request randomness for map generation.
    pub fn map_seed(&mut self, game_id: &str) -> Result<RandomnessProof, RandomnessError> {
        let context = RandomnessContext::MapGeneration {
            game_id: game_id.to_string(),
        };
        self.request_with_cache(context)
    }

    /// Request randomness for exploration.
    pub fn exploration_random(
        &mut self,
        game_id: &str,
        turn: u32,
        hex_q: i32,
        hex_r: i32,
    ) -> Result<RandomnessProof, RandomnessError> {
        let context = RandomnessContext::Exploration {
            game_id: game_id.to_string(),
            turn,
            hex_q,
            hex_r,
        };
        self.request_with_cache(context)
    }

    /// Request randomness with caching.
    fn request_with_cache(
        &mut self,
        context: RandomnessContext,
    ) -> Result<RandomnessProof, RandomnessError> {
        // Generate cache key from context
        let cache_key = format!("{:?}", context);

        // Check cache first
        if let Some(proof) = self.proof_cache.get(&cache_key) {
            return Ok(proof.clone());
        }

        // Request new randomness
        let proof = if self.offline_mode {
            self.fallback.request_randomness(context)?
        } else {
            // In a full implementation, this would call the Cashu mint
            // For now, fall back to deterministic if configured
            if self.config.allow_fallback {
                self.fallback.request_randomness(context)?
            } else {
                return Err(RandomnessError::MintUnavailable);
            }
        };

        // Cache the result
        self.proof_cache.insert(cache_key, proof.clone());

        Ok(proof)
    }

    /// Clear the proof cache.
    pub fn clear_cache(&mut self) {
        self.proof_cache.clear();
    }

    /// Get all cached proofs (for storing in Nostr events).
    pub fn get_cached_proofs(&self) -> Vec<&RandomnessProof> {
        self.proof_cache.values().collect()
    }
}

/// Helper to create a combat random value from a proof.
pub fn combat_random_from_proof(proof: &RandomnessProof) -> f32 {
    proof.to_f32()
}

/// Helper to create a map seed from a proof.
pub fn map_seed_from_proof(proof: &RandomnessProof) -> [u8; 32] {
    proof.to_seed()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_randomness() {
        let seed = [42u8; 32];
        let mut rng1 = DeterministicRandomness::new(seed);
        let mut rng2 = DeterministicRandomness::new(seed);

        let context = RandomnessContext::Combat {
            game_id: "test".to_string(),
            turn: 1,
            attacker_id: 1,
            defender_id: 2,
        };

        let proof1 = rng1.request_randomness(context.clone()).unwrap();
        let proof2 = rng2.request_randomness(context).unwrap();

        // Same seed and context should produce same randomness
        assert_eq!(proof1.random_bytes, proof2.random_bytes);
    }

    #[test]
    fn test_different_contexts_different_results() {
        let seed = [42u8; 32];
        let mut rng = DeterministicRandomness::new(seed);

        let context1 = RandomnessContext::Combat {
            game_id: "test".to_string(),
            turn: 1,
            attacker_id: 1,
            defender_id: 2,
        };

        let context2 = RandomnessContext::Combat {
            game_id: "test".to_string(),
            turn: 1,
            attacker_id: 1,
            defender_id: 3, // Different defender
        };

        let proof1 = rng.request_randomness(context1).unwrap();

        let mut rng2 = DeterministicRandomness::new(seed);
        let proof2 = rng2.request_randomness(context2).unwrap();

        // Different contexts should produce different randomness
        assert_ne!(proof1.random_bytes, proof2.random_bytes);
    }

    #[test]
    fn test_proof_to_f32_range() {
        let seed = [0u8; 32];
        let mut rng = DeterministicRandomness::new(seed);

        for i in 0..100 {
            let context = RandomnessContext::GameEvent {
                game_id: "test".to_string(),
                turn: i,
                event_type: "test".to_string(),
            };
            let proof = rng.request_randomness(context).unwrap();
            let value = proof.to_f32();

            assert!(value >= 0.0, "Value should be >= 0.0, got {}", value);
            assert!(value < 1.0, "Value should be < 1.0, got {}", value);
        }
    }

    #[test]
    fn test_proof_to_range() {
        let seed = [123u8; 32];
        let mut rng = DeterministicRandomness::new(seed);

        for i in 0..100 {
            let context = RandomnessContext::GameEvent {
                game_id: "test".to_string(),
                turn: i,
                event_type: "test".to_string(),
            };
            let proof = rng.request_randomness(context).unwrap();
            let value = proof.to_range(10);

            assert!(value < 10, "Value should be < 10, got {}", value);
        }
    }

    #[test]
    fn test_randomness_manager_caching() {
        let config = CashuConfig::default();
        let seed = [1u8; 32];
        let mut manager = RandomnessManager::new(config, seed);

        // Request same randomness twice
        let proof1 = manager.combat_random("game1", 1, 1, 2).unwrap();
        let proof2 = manager.combat_random("game1", 1, 1, 2).unwrap();

        // Should return cached result
        assert_eq!(proof1.random_bytes, proof2.random_bytes);

        // Different request should be different
        let proof3 = manager.combat_random("game1", 1, 1, 3).unwrap();
        assert_ne!(proof1.random_bytes, proof3.random_bytes);
    }

    #[test]
    fn test_randomness_manager_offline() {
        let config = CashuConfig::default(); // Empty mint_url = offline
        let seed = [2u8; 32];
        let manager = RandomnessManager::new(config, seed);

        assert!(manager.is_offline());
    }

    #[test]
    fn test_map_seed() {
        let config = CashuConfig::default();
        let seed = [3u8; 32];
        let mut manager = RandomnessManager::new(config, seed);

        let proof = manager.map_seed("game1").unwrap();
        let map_seed = proof.to_seed();

        // Should produce 32-byte seed
        assert_eq!(map_seed.len(), 32);

        // Same game should produce same seed
        let proof2 = manager.map_seed("game1").unwrap();
        assert_eq!(proof.to_seed(), proof2.to_seed());
    }

    #[test]
    fn test_proof_verification() {
        let seed = [4u8; 32];
        let rng = DeterministicRandomness::new(seed);

        let mut rng_clone = rng.clone();
        let context = RandomnessContext::MapGeneration {
            game_id: "test".to_string(),
        };
        let proof = rng_clone.request_randomness(context).unwrap();

        // Verification should pass for deterministic proofs
        assert!(rng.verify_proof(&proof).unwrap());
    }

    #[test]
    fn test_context_to_bytes_deterministic() {
        let context1 = RandomnessContext::Combat {
            game_id: "test".to_string(),
            turn: 5,
            attacker_id: 10,
            defender_id: 20,
        };

        let context2 = RandomnessContext::Combat {
            game_id: "test".to_string(),
            turn: 5,
            attacker_id: 10,
            defender_id: 20,
        };

        // Same context should produce same bytes
        assert_eq!(context1.to_bytes(), context2.to_bytes());
    }
}

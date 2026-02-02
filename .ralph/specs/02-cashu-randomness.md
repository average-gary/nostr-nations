# Nostr Nations - Cashu Randomness System

## Overview

Cashu ecash provides unbiased, verifiable randomness for all game mechanics. The game host operates a Cashu mint, and players redeem tokens to generate random values that neither party can predict or manipulate.

## Why Cashu for Randomness?

### The Problem
In multiplayer games, randomness must be:
- **Unbiased**: Neither player can influence the outcome
- **Verifiable**: All players can confirm the result is legitimate
- **Unpredictable**: Cannot be known before commitment
- **Deterministic**: Same inputs always produce same outputs (for replay)

### The Solution
Cashu blind signatures provide these properties:
1. Player creates a blinded secret
2. Mint signs the blinded secret (without knowing the value)
3. Player unblinds to get a unique signature
4. The unblinded signature serves as a random seed

The mint cannot bias the randomness because it never sees the unblinded value.
The player cannot bias it because they cannot predict the signature.

## Cashu Integration

### Game Host as Mint

The player who hosts the game runs a Cashu mint:
- Mint is created when game starts
- Single denomination tokens (1 sat equivalent)
- Tokens are not real money - just randomness sources
- Mint state is part of game state (for replay)

### CDK Integration

Using the Cashu Development Kit (CDK) Rust library:

```rust
// Host initializes mint
let mint = Mint::new(
    mint_url,
    seed,           // Derived from game seed
    mint_info,
    localstore,
);

// Player requests randomness
let blinded_message = BlindedMessage::new(secret);
let blind_signature = mint.blind_sign(blinded_message)?;

// Player unblinds
let proof = unblind(blind_signature, blinding_factor);
let random_bytes = proof.secret.as_bytes();
```

## Randomness Protocol

### Request Flow

```
Player                          Host (Mint)
   |                                |
   |  1. Create blinded secret      |
   |                                |
   |  2. Send BlindedMessage        |
   |  ----------------------------> |
   |                                |
   |                                | 3. Sign blindly
   |                                |
   |  4. Receive BlindSignature     |
   |  <---------------------------- |
   |                                |
   |  5. Unblind to get Proof       |
   |                                |
   |  6. Use proof.secret as seed   |
   |                                |
   |  7. Broadcast proof in event   |
   |  ----------------------------> |
   |                                |
```

### Nostr Event Integration

Random requests and responses are recorded as Nostr events:

**Random Request (Kind 30150)**
```json
{
  "kind": 30150,
  "tags": [
    ["g", "<game-id>"],
    ["p", "<prev-event>"],
    ["purpose", "combat|exploration|map_gen|event"],
    ["context", "<related-event-id>"]
  ],
  "content": "{\"blinded_message\":\"<base64>\"}",
}
```

**Random Response (Kind 30151)**
```json
{
  "kind": 30151,
  "tags": [
    ["g", "<game-id>"],
    ["p", "<prev-event>"],
    ["request", "<request-event-id>"]
  ],
  "content": "{\"blind_signature\":\"<base64>\"}",
}
```

**Proof Reveal (in action event)**
```json
{
  "kind": 30111,  // e.g., combat event
  "tags": [
    ["random", "<request-event-id>"],
    ["proof", "<cashu-proof-base64>"]
  ],
  "content": "{...action details...}",
}
```

## Randomness Applications

### Map Generation

When a new game starts:
1. Host generates initial Cashu token
2. Token's unblinded signature = map seed
3. Map generator uses seed for deterministic terrain generation
4. All players can verify map by checking the proof

```rust
fn generate_map(proof: &Proof, settings: &MapSettings) -> Map {
    let seed = derive_seed(proof.secret.as_bytes());
    let mut rng = ChaCha20Rng::from_seed(seed);
    
    generate_terrain(&mut rng, settings)
}
```

### Combat Resolution

Combat uses Cashu for dice rolls:

1. Attacker commits to attack (creates blinded message)
2. Host signs blindly
3. Attacker unblinds and reveals proof
4. Combat result derived from proof

```rust
fn resolve_combat(
    attacker: &Unit,
    defender: &Unit,
    proof: &Proof,
) -> CombatResult {
    let seed = derive_seed(proof.secret.as_bytes());
    let mut rng = ChaCha20Rng::from_seed(seed);
    
    let roll: f32 = rng.gen();
    let attacker_power = calculate_power(attacker);
    let defender_power = calculate_power(defender);
    
    // Combat formula using random roll
    let threshold = attacker_power / (attacker_power + defender_power);
    
    if roll < threshold {
        CombatResult::AttackerWins
    } else {
        CombatResult::DefenderWins
    }
}
```

### Exploration Events

When units explore new tiles:
- Goody hut contents
- Resource discovery
- Barbarian spawns
- Ancient ruins rewards

### Random Events

Global events during gameplay:
- Natural disasters
- Barbarian invasions
- Resource depletion/discovery

### AI Decisions (Phase 2)

AI opponents use Cashu randomness for:
- Decision making (adds unpredictability)
- Combat rolls (same as players)
- Diplomatic moods

## Verification System

### During Gameplay

Every action using randomness includes the Cashu proof:
1. Other players receive the action event
2. Extract the proof from event tags
3. Verify proof signature against mint's public key
4. Derive the same random values
5. Confirm the action outcome matches

### During Replay

Replay validation:
1. Load game's mint public key
2. For each random action, extract and verify proof
3. Recompute random values
4. Confirm game state matches recorded state

```rust
fn verify_random_action(
    event: &NostrEvent,
    mint_pubkey: &PublicKey,
) -> Result<bool> {
    let proof = extract_proof(event)?;
    
    // Verify Cashu proof
    if !verify_proof(&proof, mint_pubkey) {
        return Ok(false);
    }
    
    // Recompute random outcome
    let expected_outcome = compute_outcome(&proof, &event)?;
    let recorded_outcome = extract_outcome(&event)?;
    
    Ok(expected_outcome == recorded_outcome)
}
```

## Token Management

### Token Lifecycle

1. **Minting**: Host creates tokens as needed
2. **Distribution**: Tokens given to players for their turns
3. **Redemption**: Players use tokens for random actions
4. **Recording**: Used proofs stored in event chain
5. **Verification**: Any party can verify proofs

### Token Economy

- Tokens are free (not real money)
- Players receive tokens per turn (or on demand)
- No scarcity mechanic - just verification tool
- Used tokens cannot be reused (prevents replay attacks)

## Security Considerations

### Preventing Manipulation

1. **Blinding**: Mint cannot see what it's signing
2. **Commitment**: Player commits before seeing signature
3. **Uniqueness**: Each proof can only be used once
4. **Verification**: All players verify all proofs

### Edge Cases

**Host Disconnection**
- Game pauses until host reconnects
- Or: Backup host takes over mint (requires key handoff)

**Proof Replay Attack**
- Track used proof IDs in game state
- Reject any proof that's been used before

**Timing Attacks**
- Players must commit to action before receiving randomness
- Action + blinded message submitted together

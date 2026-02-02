# Nostr Nations - Nostr Integration

## Overview

Nostr provides the backbone for game state persistence, event chaining, and multiplayer synchronization. Every definitive game action is recorded as a Nostr event, creating a verifiable and replayable game history similar to chess notation.

## Core Principles

### Event Chaining

Game events are cryptographically chained together:
- Each event references the previous event's ID in its tags
- This creates an immutable, ordered sequence of game actions
- Any client can validate the entire game by replaying the event chain
- Forks in the chain can be detected and resolved

### Deterministic Replay

Given:
1. The initial game seed (from Cashu)
2. The ordered chain of game events
3. The game rules (core library)

Any client can reconstruct the exact game state at any point in history.

## Leveraging Existing NIPs

### NIP-28: Public Chat (Game Channels)

Each game operates as a channel:
- **Kind 40**: Channel creation (game lobby)
- **Kind 41**: Channel metadata (game settings)
- **Kind 42**: Channel messages (game events/chat)

### NIP-04: Encrypted Direct Messages

For private communications:
- Treaty negotiations
- Private diplomatic messages
- Fog of war information (what each player sees)

### NIP-01: Basic Protocol

Standard event structure for all game events:
- `pubkey`: Player's Nostr public key
- `created_at`: Event timestamp
- `kind`: Event type
- `tags`: References, metadata
- `content`: Event payload (JSON)
- `sig`: Player's signature

## Event Kinds

Using the 30000-39999 range for parameterized replaceable events:

| Kind | Name | Description |
|------|------|-------------|
| 30100 | Game Created | New game initialization |
| 30101 | Game Settings | Map size, victory conditions, players |
| 30102 | Player Joined | Player joining a game |
| 30103 | Game Started | Game beginning (all players ready) |
| 30104 | Turn Start | Beginning of a player's turn |
| 30105 | Turn End | End of a player's turn |
| 30110 | Unit Move | Unit movement action |
| 30111 | Unit Attack | Combat initiation |
| 30112 | Unit Created | New unit produced |
| 30113 | Unit Destroyed | Unit death/removal |
| 30120 | City Founded | New city established |
| 30121 | City Production | Production queue change |
| 30122 | City Growth | Population change |
| 30123 | City Improvement | Building constructed |
| 30130 | Tech Researched | Technology completed |
| 30131 | Tech Selected | Research target changed |
| 30140 | Diplomacy Propose | Treaty/agreement proposed |
| 30141 | Diplomacy Accept | Proposal accepted |
| 30142 | Diplomacy Reject | Proposal rejected |
| 30143 | War Declared | Declaration of war |
| 30150 | Random Request | Request for Cashu randomness |
| 30151 | Random Response | Cashu token redemption result |
| 30160 | Map Reveal | Fog of war update |
| 30170 | Victory | Game victory condition met |
| 30171 | Defeat | Player eliminated |
| 30172 | Game End | Game concluded |

## Event Structure

### Common Tags

All game events include:
- `["g", "<game-id>"]` - Game identifier
- `["p", "<prev-event-id>"]` - Previous event in chain (for chaining)
- `["t", "<turn-number>"]` - Current turn number
- `["player", "<player-pubkey>"]` - Acting player

### Example Events

#### Game Creation (Kind 30100)
```json
{
  "kind": 30100,
  "pubkey": "<host-pubkey>",
  "created_at": 1706400000,
  "tags": [
    ["d", "<game-uuid>"],
    ["g", "<game-uuid>"],
    ["settings", "map_size:standard,players:4,victory:all"]
  ],
  "content": "{\"name\":\"Epic Battle\",\"description\":\"4 player FFA\"}",
  "sig": "<signature>"
}
```

#### Unit Move (Kind 30110)
```json
{
  "kind": 30110,
  "pubkey": "<player-pubkey>",
  "created_at": 1706400100,
  "tags": [
    ["g", "<game-uuid>"],
    ["p", "<previous-event-id>"],
    ["t", "5"],
    ["player", "<player-pubkey>"],
    ["unit", "<unit-id>"],
    ["from", "12,8"],
    ["to", "13,9"]
  ],
  "content": "{}",
  "sig": "<signature>"
}
```

#### Combat (Kind 30111)
```json
{
  "kind": 30111,
  "pubkey": "<attacker-pubkey>",
  "created_at": 1706400200,
  "tags": [
    ["g", "<game-uuid>"],
    ["p", "<previous-event-id>"],
    ["t", "5"],
    ["player", "<attacker-pubkey>"],
    ["attacker", "<unit-id>"],
    ["defender", "<unit-id>"],
    ["random", "<cashu-token-id>"]
  ],
  "content": "{\"attacker_strength\":5,\"defender_strength\":3}",
  "sig": "<signature>"
}
```

#### Random Request (Kind 30150)
```json
{
  "kind": 30150,
  "pubkey": "<player-pubkey>",
  "created_at": 1706400150,
  "tags": [
    ["g", "<game-uuid>"],
    ["p", "<previous-event-id>"],
    ["t", "5"],
    ["purpose", "combat"],
    ["context", "<combat-event-id>"]
  ],
  "content": "{\"blinded_message\":\"<blinded-secret>\"}",
  "sig": "<signature>"
}
```

## Event Chain Validation

### Chain Integrity Rules

1. **Continuous Chain**: Every event (except game creation) must reference a valid previous event
2. **Temporal Order**: `created_at` must be >= previous event's `created_at`
3. **Turn Consistency**: Turn numbers must increment properly
4. **Player Authorization**: Only the current turn's player can submit actions
5. **Action Validity**: Actions must be legal according to game rules
6. **Randomness Verification**: Combat outcomes must match Cashu token results

### Validation Algorithm

```
function validateChain(events):
    state = initialGameState()
    
    for event in events:
        # Check chain linkage
        if event.prev != previousEvent.id:
            return INVALID("broken chain")
        
        # Check signature
        if !verifySignature(event):
            return INVALID("bad signature")
        
        # Check game rules
        if !isValidAction(state, event):
            return INVALID("illegal action")
        
        # Apply event to state
        state = applyEvent(state, event)
        previousEvent = event
    
    return VALID(state)
```

## Storage Architecture

### Local Nostr Relay

Each client runs a local Nostr relay for:
- Storing game events for active and saved games
- Fast local queries without network latency
- Offline play capability
- Game state persistence across sessions

### Relay Sync

For multiplayer:
1. Host's local relay is authoritative for the game
2. Clients sync events from host via Iroh P2P
3. Light clients can use public relays as intermediaries
4. Conflict resolution: host's chain is canonical

### Data Retention

- Active game: Full event chain in memory + local relay
- Saved games: Compressed event chain on local relay
- Replay files: Exportable event chain (JSON or binary)

## Privacy Considerations

### Fog of War

Players should not see opponents' hidden information:
- Use NIP-04 encryption for player-specific events
- Host maintains full state, sends filtered views to players
- Combat reveals only what both parties can see

### Event Visibility

| Event Type | Visibility |
|------------|------------|
| Game Setup | All players |
| Public Actions | All players |
| Fog Reveals | Affected player only (encrypted) |
| Diplomacy | Involved parties only (encrypted) |
| Combat Results | All players (after resolution) |

## Replay System

### Game Recording

Every game automatically creates a complete replay:
1. Export event chain as JSON
2. Include game settings and player info
3. Optionally include Cashu proofs for randomness verification

### Replay Playback

1. Load event chain
2. Initialize game state from settings
3. Apply events sequentially
4. Allow pause, rewind, fast-forward
5. Validate chain integrity during playback

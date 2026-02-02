# Nostr Nations - Networking Architecture

## Overview

Nostr Nations uses a hybrid networking model combining Iroh for P2P connectivity, local Nostr relays for storage, and optional public relays for light clients.

## Network Topology

```
┌─────────────────────────────────────────────────────────────┐
│                      Game Session                            │
│                                                              │
│    ┌──────────────┐        Iroh P2P        ┌──────────────┐ │
│    │  Full Client │◄──────────────────────►│  Full Client │ │
│    │   (Host)     │                        │  (Player 2)  │ │
│    │              │                        │              │ │
│    │ ┌──────────┐ │                        │ ┌──────────┐ │ │
│    │ │  Cashu   │ │                        │ │  Local   │ │ │
│    │ │  Mint    │ │                        │ │  Relay   │ │ │
│    │ └──────────┘ │                        │ └──────────┘ │ │
│    │ ┌──────────┐ │                        └──────────────┘ │
│    │ │  Local   │ │                               ▲         │
│    │ │  Relay   │ │                               │         │
│    │ └──────────┘ │                               │         │
│    └──────────────┘                               │         │
│           ▲                                       │         │
│           │              ┌──────────────┐         │         │
│           │              │ Public Relay │         │         │
│           │              │  (Optional)  │─────────┘         │
│           │              └──────────────┘                   │
│           │                     ▲                           │
│           │                     │                           │
│    ┌──────┴───────┐      ┌──────┴───────┐                  │
│    │ Light Client │      │ Light Client │                  │
│    │  (Mobile)    │      │  (Mobile)    │                  │
│    └──────────────┘      └──────────────┘                  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Iroh P2P Connectivity

### Why Iroh?

- **NAT Traversal**: Works through firewalls and NATs
- **Peer Discovery**: Easy to find and connect to peers
- **Reliable Streams**: Built-in reliability over QUIC
- **Rust Native**: Integrates well with our stack

### Connection Setup

#### QR Code Flow

1. Host starts game, Iroh generates a ticket
2. Host displays QR code containing:
   - Iroh node ID
   - Connection ticket
   - Game ID
3. Joining players scan QR code
4. Iroh establishes direct P2P connection

```rust
// Host generates invite
let ticket = node.ticket()?;
let qr_data = QrInvite {
    node_id: node.node_id(),
    ticket: ticket.to_string(),
    game_id: game.id.clone(),
};
let qr_code = generate_qr(&qr_data)?;

// Player joins via scanned QR
let invite: QrInvite = parse_qr(scanned_data)?;
let conn = node.connect(invite.ticket).await?;
```

#### Manual Connection

For cases where QR scanning isn't available:
- Display connection string (base58 encoded)
- Player enters string manually
- Same connection flow as QR

### Iroh Protocols

#### Game Sync Protocol

Custom Iroh protocol for game state synchronization:

```rust
#[derive(Debug)]
pub struct GameSyncProtocol;

impl Protocol for GameSyncProtocol {
    const ALPN: &'static [u8] = b"nostr-nations/game-sync/1";
    
    async fn handle(&self, conn: Connection) -> Result<()> {
        // Handle sync requests
    }
}
```

**Message Types:**
- `SyncRequest`: Request events since last known event
- `SyncResponse`: Stream of events
- `EventBroadcast`: New event from a player
- `StateQuery`: Request current game state
- `StateResponse`: Full or partial game state

#### Cashu Protocol

For randomness requests between players and host:

```rust
pub struct CashuProtocol;

impl Protocol for CashuProtocol {
    const ALPN: &'static [u8] = b"nostr-nations/cashu/1";
}
```

**Message Types:**
- `MintRequest`: Blinded message for signing
- `MintResponse`: Blind signature

## Local Nostr Relay

### Purpose

Each full client runs a local Nostr relay:
- Persistent storage of game events
- Fast local queries
- Offline capability
- Multi-game storage

### Implementation

Based on patterns from Damus Notedeck:

```rust
pub struct LocalRelay {
    db: Database,  // SQLite or similar
    subscriptions: HashMap<SubscriptionId, Subscription>,
}

impl LocalRelay {
    pub async fn store_event(&self, event: NostrEvent) -> Result<()>;
    pub async fn query(&self, filter: Filter) -> Vec<NostrEvent>;
    pub fn subscribe(&self, filter: Filter) -> Receiver<NostrEvent>;
}
```

### Storage Schema

```sql
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    pubkey TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    kind INTEGER NOT NULL,
    tags TEXT NOT NULL,  -- JSON array
    content TEXT NOT NULL,
    sig TEXT NOT NULL,
    game_id TEXT,  -- Indexed for game queries
    
    INDEX idx_game_id (game_id),
    INDEX idx_kind (kind),
    INDEX idx_created_at (created_at)
);

CREATE TABLE games (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    last_played INTEGER NOT NULL,
    status TEXT NOT NULL,  -- active, paused, completed
    host_pubkey TEXT NOT NULL,
    settings TEXT NOT NULL  -- JSON
);
```

## State Synchronization

### Initial Sync

When a player joins a game:

1. Connect to host via Iroh
2. Request game state from creation
3. Receive and validate event chain
4. Build local game state
5. Store events in local relay
6. Ready to play

```rust
async fn initial_sync(conn: &Connection, game_id: &str) -> Result<GameState> {
    // Request all events
    let request = SyncRequest {
        game_id: game_id.to_string(),
        since: None,  // From beginning
    };
    
    conn.send(&request).await?;
    
    let mut events = Vec::new();
    while let Some(event) = conn.recv::<NostrEvent>().await? {
        validate_event(&event)?;
        events.push(event);
    }
    
    // Build state from events
    let state = replay_events(&events)?;
    
    // Store locally
    local_relay.store_events(&events).await?;
    
    Ok(state)
}
```

### Incremental Sync

During gameplay:

1. New events broadcast immediately via Iroh
2. Each client validates and applies events
3. Events stored in local relay
4. Periodic sync checks for missed events

### Conflict Resolution

Since games are turn-based with a single authority (host):

1. Host's event chain is canonical
2. If client has divergent state, resync from host
3. Invalid events from clients are rejected by host
4. Clients can detect invalid host events (cheating)

## Light Client Mode

### Characteristics

Light clients (mobile):
- No local Nostr relay
- Minimal storage
- Connect to full client or public relay
- Cannot host games

### Connection Options

**Direct to Full Client:**
```
Light Client ◄──── Iroh P2P ────► Full Client (Host)
```

**Via Public Relay:**
```
Light Client ◄──── WebSocket ────► Public Relay ◄────► Full Client
```

### Data Flow

Light clients receive:
- Filtered game state (fog of war applied)
- Events relevant to their view
- Turn notifications

Light clients send:
- Player actions (validated by host)
- Acknowledgments

## Public Relay Integration

### Use Cases

1. **Light Client Support**: Bridge for mobile players
2. **Backup Storage**: Redundant event storage
3. **Spectator Mode**: Watch games without connecting directly
4. **Async Play**: Leave events for disconnected players

### Relay Selection

Game settings can specify:
- Required relays (all players must use)
- Optional relays (for redundancy)
- No relays (direct P2P only)

### Event Filtering

Not all events go to public relays:
- Encrypted events (diplomacy) stay encrypted
- Fog of war events filtered per player
- Full history available to game participants only

## Connection Management

### Connection States

```rust
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Syncing,
    Ready,
    Reconnecting,
}
```

### Reconnection

On disconnect:
1. Attempt Iroh reconnection (exponential backoff)
2. If direct fails, try relay bridge
3. On reconnect, perform incremental sync
4. Notify UI of connection status

### Offline Play

Full clients can continue playing while disconnected:
- Make moves locally
- Queue events for sync
- On reconnect, submit queued events
- If conflicts, may need to revert (rare in turn-based)

## Security

### Authentication

- Players identified by Nostr pubkey
- All events signed with player's key
- Invalid signatures rejected

### Encryption

- Iroh connections are encrypted (QUIC)
- Sensitive events use NIP-04 encryption
- Local relay can be encrypted at rest

### Anti-Cheat

- All game logic validated by core library
- Invalid moves rejected
- Event chain prevents history modification
- Cashu proofs prevent random manipulation

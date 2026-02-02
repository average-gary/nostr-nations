# Nostr Nations - Build Instructions

## Project Overview

Nostr Nations is a Civilization-style 4X strategy game built with:
- **Backend**: Rust (Bevy game engine + Tauri)
- **Frontend**: React + TypeScript + Three.js
- **Networking**: Iroh P2P + Nostr events
- **Randomness**: Cashu ecash blind signatures

## Project Structure

```
nostr-nations/
├── crates/
│   ├── nostr-nations-core/    # Core game library (no UI deps)
│   ├── nostr-nations-bevy/    # Bevy ECS integration
│   └── nostr-nations-network/ # Iroh + Nostr relay
├── src-tauri/                 # Tauri backend
├── src/                       # React frontend
├── .ralph/                    # Ralph configuration
└── Cargo.toml                 # Workspace manifest
```

## Prerequisites

### System Dependencies

```bash
# macOS
brew install rust node

# Ubuntu/Debian
sudo apt install build-essential libwebkit2gtk-4.0-dev \
    libssl-dev libgtk-3-dev libayatana-appindicator3-dev \
    librsvg2-dev

# Windows
# Install Rust via rustup.rs
# Install Node.js from nodejs.org
```

### Rust Setup

```bash
# Install Rust (if not installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm target (for potential future use)
rustup target add wasm32-unknown-unknown

# Install Tauri CLI
cargo install tauri-cli
```

### Node.js Setup

```bash
# Using nvm (recommended)
nvm install 20
nvm use 20

# Or install Node.js 20+ directly
```

## Project Setup

```bash
# Clone repository
git clone <repo-url>
cd nostr-nations

# Install Node dependencies
npm install

# Build Rust dependencies (first time)
cargo build
```

## Development

### Start Development Server

```bash
# Start Tauri development (runs both frontend and backend)
cargo tauri dev

# Or run frontend only (for UI development)
npm run dev

# Or run backend only (for Rust development)
cargo run -p nostr-nations-tauri
```

### Running Tests

```bash
# Run all Rust tests
cargo test

# Run tests for specific crate
cargo test -p nostr-nations-core

# Run with output
cargo test -- --nocapture

# Run frontend tests
npm test

# Run specific test
cargo test test_hex_neighbors
```

### Code Quality

```bash
# Format Rust code
cargo fmt

# Lint Rust code
cargo clippy

# Format TypeScript/React
npm run lint
npm run format
```

## Building for Production

```bash
# Build release binaries
cargo tauri build

# Output locations:
# macOS: target/release/bundle/dmg/
# Windows: target/release/bundle/msi/
# Linux: target/release/bundle/deb/
```

## Key Crates and Dependencies

### Rust Dependencies

| Crate | Purpose |
|-------|---------|
| `tauri` | Desktop framework |
| `bevy` | Game engine (ECS) |
| `nostr-sdk` | Nostr protocol |
| `cdk` | Cashu Development Kit |
| `iroh` | P2P networking |
| `serde` | Serialization |
| `tokio` | Async runtime |

### Frontend Dependencies

| Package | Purpose |
|---------|---------|
| `react` | UI framework |
| `three` | WebGL rendering |
| `@tauri-apps/api` | Tauri IPC |
| `zustand` | State management |
| `tailwindcss` | Styling |

## Architecture Notes

### Core Library Separation

The `nostr-nations-core` crate must:
1. Have NO dependencies on Bevy, Tauri, or UI frameworks
2. Be fully deterministic (same inputs = same outputs)
3. Be serializable (all state can be saved/loaded)
4. Be thoroughly tested

### Tauri IPC Commands

Commands are defined in `src-tauri/src/commands/` and exposed to React:

```typescript
// Frontend usage
import { invoke } from '@tauri-apps/api/tauri';
const state = await invoke('get_game_state', { gameId });
```

### Bevy Integration

Bevy runs in the Tauri backend, managing game state via ECS:
- Components: Game entities (units, cities, tiles)
- Systems: Game logic (combat, movement, production)
- Resources: Shared state (current game, settings)

## Troubleshooting

### Common Issues

**Tauri build fails on macOS:**
```bash
# Ensure Xcode CLI tools are installed
xcode-select --install
```

**Cargo build fails with OpenSSL errors:**
```bash
# macOS
brew install openssl
export OPENSSL_DIR=$(brew --prefix openssl)

# Ubuntu
sudo apt install pkg-config libssl-dev
```

**Node modules issues:**
```bash
rm -rf node_modules package-lock.json
npm install
```

## Development Workflow

1. Pick a task from `.ralph/fix_plan.md`
2. Create feature branch: `git checkout -b feature/task-name`
3. Implement with tests
4. Run `cargo test` and `npm test`
5. Run `cargo fmt` and `npm run lint`
6. Commit with conventional commit message
7. Update `.ralph/fix_plan.md`
8. Push and create PR

## Key Learnings

*Update this section as you discover project-specific patterns:*

- Hex math uses offset (odd-q) coordinates - see `HexCoord` in core lib
- All game actions must produce Nostr events for replay
- Cashu randomness must be requested BEFORE knowing the outcome
- Fog of war events use NIP-04 encryption per player

---

## IPC Commands Reference

This section documents all Tauri IPC commands available to the React frontend. Commands are invoked using the `@tauri-apps/api` package.

### TypeScript Type Definitions

```typescript
// Common Types
interface GameStateResponse {
  game_id: string;
  phase: string;        // "Setup" | "Playing" | "Finished"
  turn: number;
  current_player: number;
  player_count: number;
  map_width: number;
  map_height: number;
}

interface ActionResult {
  success: boolean;
  message: string | null;
  effects: string[];
}

interface ConnectionStatus {
  connected: boolean;
  peer_count: number;
  ticket: string | null;
}

interface TicketInfo {
  node_id: string;
  addresses: string[];
  game_id: string | null;
  expires_at: number;
}

interface CreateGameOptions {
  name: string;
  player_name: string;
  civilization: string;
  map_size: "duel" | "small" | "standard" | "large" | "huge";
  difficulty: "settler" | "chieftain" | "normal" | "prince" | "king" | "emperor" | "deity" | "immortal";
  game_speed: "quick" | "normal" | "standard" | "epic" | "marathon";
  seed?: string;
}

// Improvement types for build_improvement command
type ImprovementType =
  | "farm"
  | "mine"
  | "pasture"
  | "plantation"
  | "camp"
  | "quarry"
  | "lumber_mill"
  | "trading_post"
  | "fort";

// Error type returned by all commands on failure
interface AppError {
  message: string;
}
```

### Error Types

All commands may return one of the following errors:

| Error Type | Description |
|------------|-------------|
| `NoActiveGame` | No game is currently active |
| `GameAlreadyActive` | A game is already in progress |
| `InvalidState` | Invalid game state or action |
| `NetworkError` | P2P network operation failed |
| `SerializationError` | Failed to serialize/deserialize data |

---

### Game Management Commands

Commands in `src-tauri/src/commands/game.rs` handle game lifecycle operations.

#### `create_game`

Creates a new game with the specified settings.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `options` | `CreateGameOptions` | Game configuration options |

**Returns:** `GameStateResponse`

**Possible Errors:**
- `GameAlreadyActive` - A game is already in progress
- `InvalidState` - Lock poisoned or game creation failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const gameState = await invoke<GameStateResponse>('create_game', {
  options: {
    name: 'My Game',
    player_name: 'Player 1',
    civilization: 'romans',
    map_size: 'standard',
    difficulty: 'normal',
    game_speed: 'normal',
    seed: 'optional_seed_string'
  }
});

console.log(`Created game: ${gameState.game_id}`);
console.log(`Map size: ${gameState.map_width}x${gameState.map_height}`);
```

---

#### `join_game`

Joins an existing game that is in the Setup phase.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `player_name` | `string` | Display name for the player |
| `civilization` | `string` | Civilization ID to play as |

**Returns:** `GameStateResponse`

**Possible Errors:**
- `NoActiveGame` - No game exists to join
- `InvalidState` - Game is not in setup phase or join failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const gameState = await invoke<GameStateResponse>('join_game', {
  player_name: 'Player 2',
  civilization: 'greeks'
});

console.log(`Joined game with ${gameState.player_count} players`);
```

---

#### `start_game`

Starts the game, transitioning from Setup to Playing phase.

**Parameters:** None

**Returns:** `GameStateResponse`

**Possible Errors:**
- `NoActiveGame` - No game exists
- `InvalidState` - Game is not in setup phase

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const gameState = await invoke<GameStateResponse>('start_game');

console.log(`Game started! Phase: ${gameState.phase}`);
console.log(`Turn ${gameState.turn}, Player ${gameState.current_player}'s turn`);
```

---

#### `get_game_state`

Retrieves the current game state.

**Parameters:** None

**Returns:** `GameStateResponse`

**Possible Errors:**
- `NoActiveGame` - No game exists

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const gameState = await invoke<GameStateResponse>('get_game_state');

console.log(`Game ID: ${gameState.game_id}`);
console.log(`Phase: ${gameState.phase}`);
console.log(`Turn: ${gameState.turn}`);
console.log(`Current Player: ${gameState.current_player}`);
console.log(`Players: ${gameState.player_count}`);
console.log(`Map: ${gameState.map_width}x${gameState.map_height}`);
```

---

#### `end_turn`

Ends the current player's turn and advances to the next player.

**Parameters:** None

**Returns:** `GameStateResponse`

**Possible Errors:**
- `NoActiveGame` - No game exists
- `InvalidState` - Cannot end turn in current state

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const gameState = await invoke<GameStateResponse>('end_turn');

console.log(`Turn ended. Now turn ${gameState.turn}, Player ${gameState.current_player}`);
```

---

### Action Commands

Commands in `src-tauri/src/commands/actions.rs` handle in-game actions.

#### `move_unit`

Moves a unit along a specified path.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `unit_id` | `number` | Unique identifier of the unit to move |
| `path` | `[number, number][]` | Array of hex coordinates `[q, r]` forming the movement path |

**Returns:** `ActionResult`

**Possible Errors:**
- `NoActiveGame` - No game exists
- `InvalidState` - Unit not found, invalid path, or insufficient movement points

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke<ActionResult>('move_unit', {
  unit_id: 42,
  path: [
    [5, 3],   // Starting adjacent hex
    [6, 3],   // Next hex
    [7, 4]    // Destination
  ]
});

if (result.success) {
  console.log('Unit moved successfully');
  console.log('Effects:', result.effects);
} else {
  console.error('Move failed:', result.message);
}
```

---

#### `attack_unit`

Attacks an enemy unit.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `attacker_id` | `number` | Unique identifier of the attacking unit |
| `defender_id` | `number` | Unique identifier of the defending unit |
| `random` | `number` | Random value (0.0-1.0) for combat resolution |

**Returns:** `ActionResult`

**Possible Errors:**
- `NoActiveGame` - No game exists
- `InvalidState` - Units not found, out of range, or cannot attack

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// In a real implementation, the random value would come from
// Cashu blind signature for verifiable randomness
const result = await invoke<ActionResult>('attack_unit', {
  attacker_id: 42,
  defender_id: 99,
  random: 0.65
});

if (result.success) {
  console.log('Attack resolved');
  console.log('Combat effects:', result.effects);
} else {
  console.error('Attack failed:', result.message);
}
```

---

#### `found_city`

Founds a new city using a settler unit.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `settler_id` | `number` | Unique identifier of the settler unit |
| `name` | `string` | Name for the new city |

**Returns:** `ActionResult`

**Possible Errors:**
- `NoActiveGame` - No game exists
- `InvalidState` - Settler not found, invalid location, or too close to another city

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke<ActionResult>('found_city', {
  settler_id: 1,
  name: 'Rome'
});

if (result.success) {
  console.log('City founded!');
  console.log('Effects:', result.effects);
} else {
  console.error('Cannot found city:', result.message);
}
```

---

#### `build_improvement`

Orders a worker unit to build an improvement on its current tile.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `unit_id` | `number` | Unique identifier of the worker unit |
| `improvement` | `ImprovementType` | Type of improvement to build |

**Improvement Types:**
| Value | Description |
|-------|-------------|
| `"farm"` | Increases food output |
| `"mine"` | Increases production output |
| `"pasture"` | For animal resources |
| `"plantation"` | For luxury resources |
| `"camp"` | For hunting resources |
| `"quarry"` | For stone resources |
| `"lumber_mill"` | For forest tiles |
| `"trading_post"` | Increases gold output |
| `"fort"` | Defensive structure |

**Returns:** `ActionResult`

**Possible Errors:**
- `NoActiveGame` - No game exists
- `InvalidState` - Unknown improvement type, worker not found, or invalid tile

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke<ActionResult>('build_improvement', {
  unit_id: 5,
  improvement: 'farm'
});

if (result.success) {
  console.log('Improvement started');
} else {
  console.error('Cannot build:', result.message);
}
```

---

#### `set_research`

Sets the current research target for the player.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `tech_id` | `string` | Identifier of the technology to research |

**Returns:** `ActionResult`

**Possible Errors:**
- `NoActiveGame` - No game exists
- `InvalidState` - Technology not available or prerequisites not met

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const result = await invoke<ActionResult>('set_research', {
  tech_id: 'writing'
});

if (result.success) {
  console.log('Now researching Writing');
} else {
  console.error('Cannot research:', result.message);
}
```

---

### Network Commands

Commands in `src-tauri/src/commands/network.rs` handle P2P networking.

#### `connect_peer`

Connects to a peer using a connection ticket.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `ticket` | `string` | Base64-encoded connection ticket |

**Returns:** `ConnectionStatus`

**Possible Errors:**
- `NetworkError` - Invalid ticket format or connection failed

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const status = await invoke<ConnectionStatus>('connect_peer', {
  ticket: 'base64EncodedTicketString...'
});

if (status.connected) {
  console.log(`Connected! Total peers: ${status.peer_count}`);
}
```

---

#### `disconnect_peer`

Disconnects from a specific peer.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `peer_id` | `string` | Identifier of the peer to disconnect |

**Returns:** `ConnectionStatus`

**Possible Errors:**
- `NetworkError` - Not connected to any peers

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const status = await invoke<ConnectionStatus>('disconnect_peer', {
  peer_id: 'peer_node_id_here'
});

console.log(`Disconnected. Remaining peers: ${status.peer_count}`);
```

---

#### `get_connection_ticket`

Generates a connection ticket that other clients can use to connect.

**Parameters:** None

**Returns:** `string` - Base64-encoded connection ticket

**Possible Errors:**
- `SerializationError` - Failed to encode ticket

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

const ticket = await invoke<string>('get_connection_ticket');

// Display as QR code for easy sharing
console.log('Share this ticket:', ticket);

// Or with the nn: prefix for QR codes
const qrData = `nn:${ticket}`;
```

---

#### `scan_qr_code`

Parses a scanned QR code and extracts connection ticket information.

**Parameters:**
| Name | Type | Description |
|------|------|-------------|
| `qr_data` | `string` | Raw data from QR code (with or without `nn:` prefix) |

**Returns:** `TicketInfo`

**Possible Errors:**
- `NetworkError` - Invalid QR code format

**Example:**
```typescript
import { invoke } from '@tauri-apps/api/tauri';

// QR code data from camera scan
const qrData = 'nn:base64EncodedTicketString...';

const ticketInfo = await invoke<TicketInfo>('scan_qr_code', {
  qr_data: qrData
});

console.log('Node ID:', ticketInfo.node_id);
console.log('Addresses:', ticketInfo.addresses);
console.log('Game ID:', ticketInfo.game_id);
console.log('Expires:', new Date(ticketInfo.expires_at * 1000));

// Now use the original ticket string to connect
// await invoke('connect_peer', { ticket: qrData.replace('nn:', '') });
```

---

### Complete Usage Example

```typescript
import { invoke } from '@tauri-apps/api/tauri';

// Game flow example
async function playGame() {
  try {
    // 1. Create a new game
    const game = await invoke<GameStateResponse>('create_game', {
      options: {
        name: 'Test Game',
        player_name: 'Alice',
        civilization: 'romans',
        map_size: 'small',
        difficulty: 'normal',
        game_speed: 'quick'
      }
    });
    console.log(`Created game ${game.game_id}`);

    // 2. Generate connection ticket for other players
    const ticket = await invoke<string>('get_connection_ticket');
    console.log('Share this ticket:', ticket);

    // 3. Wait for other players to join...
    // (In real app, listen for player join events)

    // 4. Start the game
    const started = await invoke<GameStateResponse>('start_game');
    console.log(`Game started! Turn ${started.turn}`);

    // 5. Game loop
    while (started.phase === 'Playing') {
      const state = await invoke<GameStateResponse>('get_game_state');

      // Perform actions (move units, found cities, etc.)
      const moveResult = await invoke<ActionResult>('move_unit', {
        unit_id: 1,
        path: [[5, 5], [6, 5]]
      });

      if (moveResult.success) {
        console.log('Unit moved');
      }

      // End turn
      await invoke<GameStateResponse>('end_turn');
    }
  } catch (error) {
    console.error('Game error:', error);
  }
}
```

# Nostr Nations Architecture

## Overview

Nostr Nations is a Civilization-style 4X strategy game built with:

- **Rust** backend for game logic and networking
- **React/TypeScript** frontend with Three.js for 3D rendering
- **Tauri** for desktop application packaging
- **Nostr** protocol for deterministic game replay and multiplayer
- **Cashu** for verifiable randomness

## Project Structure

```
nostr-nations/
├── crates/                          # Rust workspace crates
│   ├── nostr-nations-core/          # Core game logic (no UI dependencies)
│   ├── nostr-nations-bevy/          # Bevy ECS integration (optional)
│   └── nostr-nations-network/       # P2P networking (Iroh + Nostr)
├── src-tauri/                       # Tauri application
│   └── src/
│       ├── main.rs                  # Tauri entry point
│       ├── commands/                # IPC command handlers
│       │   ├── game.rs              # Game lifecycle commands
│       │   ├── actions.rs           # In-game action commands
│       │   └── network.rs           # P2P networking commands
│       ├── events.rs                # Backend-to-frontend events
│       └── state.rs                 # Application state management
├── src/                             # React frontend
│   ├── components/                  # React components
│   │   ├── game/                    # Game view components (HexMap, Units, Cities)
│   │   ├── hud/                     # HUD elements (TopBar, BottomBar)
│   │   ├── menu/                    # Menu screens
│   │   ├── overlays/                # Modal overlays (TechTree, Diplomacy)
│   │   └── ui/                      # Reusable UI components
│   ├── hooks/                       # React hooks
│   │   └── useTauri.ts              # Tauri IPC hooks
│   ├── stores/                      # Zustand state stores
│   ├── shaders/                     # GLSL shaders (fog of war, etc.)
│   └── types/                       # TypeScript type definitions
├── Cargo.toml                       # Workspace configuration
└── package.json                     # Frontend dependencies
```

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                           FRONTEND (React)                          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────────┐ │
│  │   Zustand   │◄───│    Hooks    │◄───│   React Components      │ │
│  │   Stores    │    │  useTauri   │    │  (HexMap, Units, HUD)   │ │
│  └──────┬──────┘    └──────┬──────┘    └─────────────────────────┘ │
│         │                  │                                        │
│         │    invoke()      │           listen()                     │
└─────────┼──────────────────┼────────────────────────────────────────┘
          │                  │
          ▼                  ▼
┌─────────────────────────────────────────────────────────────────────┐
│                         TAURI IPC BRIDGE                            │
│  ┌─────────────────────┐              ┌───────────────────────────┐ │
│  │  Commands (invoke)  │              │    Events (listen)        │ │
│  │  - create_game      │              │  - game_state_updated     │ │
│  │  - move_unit        │              │  - turn_event             │ │
│  │  - attack_unit      │              │  - combat_resolved        │ │
│  │  - end_turn         │              │  - network_event          │ │
│  │  - connect_peer     │              │  - notification           │ │
│  └──────────┬──────────┘              └───────────────────────────┘ │
└─────────────┼───────────────────────────────────────────────────────┘
              │
              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        BACKEND (Rust)                               │
│  ┌─────────────┐    ┌─────────────────┐    ┌────────────────────┐  │
│  │  AppState   │◄───│   GameEngine    │◄───│  nostr-nations-    │  │
│  │  (Mutex)    │    │   (replay.rs)   │    │  core              │  │
│  └─────────────┘    └────────┬────────┘    └────────────────────┘  │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                    Event Chain (Nostr)                       │   │
│  │  GameEvent → GameEvent → GameEvent → ...                     │   │
│  │     ↓           ↓           ↓                                │   │
│  │  Cashu Proof  (optional - for combat/map gen)                │   │
│  └─────────────────────────────────────────────────────────────┘   │
│                              │                                      │
│                              ▼                                      │
│  ┌─────────────────────────────────────────────────────────────┐   │
│  │                 P2P Network (Iroh/Nostr)                     │   │
│  │  Peer ◄──────► Peer ◄──────► Peer                            │   │
│  └─────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────┘
```

## Key Components

### Core Crate (`nostr-nations-core`)

The heart of the game, containing all game logic with no UI dependencies:

| Module              | Responsibility                                   |
| ------------------- | ------------------------------------------------ |
| `game_state.rs`     | Root game state, player management, diplomacy    |
| `map.rs` / `hex.rs` | Hexagonal map representation and coordinate math |
| `mapgen.rs`         | Procedural map generation with seeded RNG        |
| `unit.rs`           | Unit types, stats, movement, promotions          |
| `combat.rs`         | Combat resolution formulas                       |
| `city.rs`           | City mechanics, production, growth               |
| `technology.rs`     | Tech tree with 40+ technologies                  |
| `victory.rs`        | Victory condition checking                       |
| `events.rs`         | Nostr event types and chain validation           |
| `replay.rs`         | GameEngine for deterministic replay              |
| `cashu.rs`          | Verifiable randomness integration                |

### Tauri Application (`src-tauri`)

Bridges the Rust backend with the React frontend:

| Module                | Responsibility                                 |
| --------------------- | ---------------------------------------------- |
| `commands/game.rs`    | Game lifecycle (create, join, start, end turn) |
| `commands/actions.rs` | In-game actions (move, attack, build)          |
| `commands/network.rs` | P2P connections and sync                       |
| `events.rs`           | Event emission to frontend                     |
| `state.rs`            | Managed application state                      |

### Frontend (`src/`)

React application with Three.js rendering:

| Directory              | Responsibility                                        |
| ---------------------- | ----------------------------------------------------- |
| `components/game/`     | 3D game rendering (HexMap, UnitMesh, CityMesh)        |
| `components/hud/`      | UI overlay (resources, turn info, notifications)      |
| `components/overlays/` | Modal screens (tech tree, diplomacy, city management) |
| `stores/`              | Zustand state management                              |
| `hooks/useTauri.ts`    | Tauri command/event hooks                             |

## Nostr Event System

All game actions are serialized as Nostr events, enabling deterministic replay:

### Event Kinds

| Kind  | Name            | Description                           |
| ----- | --------------- | ------------------------------------- |
| 30100 | GAME_CREATE     | Game creation with settings and seed  |
| 30101 | PLAYER_JOIN     | Player joining a game                 |
| 30102 | GAME_START      | Game start signal                     |
| 30103 | GAME_ACTION     | In-game actions (move, attack, build) |
| 30104 | TURN_END        | Player ending their turn              |
| 30105 | GAME_END        | Game completion with winner           |
| 30106 | RANDOM_REQUEST  | Request for Cashu randomness          |
| 30107 | RANDOM_RESPONSE | Cashu randomness proof                |

### Event Chain

Events form a linked chain where each event references the previous:

```
Event1 (prev: null)
   ↓
Event2 (prev: Event1.id)
   ↓
Event3 (prev: Event2.id)
   ↓
  ...
```

This enables:

- **Validation**: Any node can verify the chain is unbroken
- **Replay**: Reconstruct game state by replaying events
- **Sync**: Missing events can be requested from peers

## Cashu Randomness Integration

Combat and map generation use Cashu blinded signatures for verifiable randomness:

```
┌──────────────┐     ┌─────────────────┐     ┌──────────────┐
│    Player    │     │   Cashu Mint    │     │    Verify    │
│  (request)   │     │   (sign)        │     │   (anyone)   │
└──────┬───────┘     └────────┬────────┘     └──────────────┘
       │                      │
       │ 1. Create blinded    │
       │    message + nonce   │
       │                      │
       │ 2. Send blinded ───► │
       │    message           │
       │                      │
       │ 3. Receive blind ◄── │
       │    signature         │
       │                      │
       │ 4. Unblind to get    │
       │    final random      │
       │                      │
       │ 5. Include proof     │ ◄──────────────────────────┐
       │    in Nostr event    │                            │
       │                      │                            │
       └──────────────────────┘                   Anyone can verify
                                                  the randomness
```

### Randomness Applications

- **Combat**: Determines damage variance (±20%)
- **Map Generation**: Seeds the procedural generator
- **Exploration**: Goody hut outcomes, barbarian spawns

### Fallback

For offline/local games, `DeterministicRandomness` provides a seeded PRNG fallback.

## Network Architecture

### P2P via Iroh

Peer-to-peer connections use [Iroh](https://iroh.computer/):

1. **Connection Tickets**: Base64-encoded connection info (node ID, addresses)
2. **QR Codes**: Tickets can be shared as QR codes for mobile
3. **Direct Connections**: Iroh handles NAT traversal

### State Synchronization

1. Peers exchange event chains
2. Missing events are requested
3. Game state is reconstructed by replaying events
4. All peers converge to the same state (determinism)

## Design Principles

1. **Deterministic**: Same inputs always produce same outputs
2. **Serializable**: All state can be saved/loaded via serde
3. **No UI in Core**: `nostr-nations-core` has no rendering dependencies
4. **Event-Sourced**: Game state is derived from event chain
5. **Verifiable Randomness**: Cashu ensures fair outcomes
6. **P2P First**: No central server required

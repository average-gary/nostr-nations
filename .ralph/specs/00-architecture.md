# Nostr Nations - System Architecture

## Overview

Nostr Nations is a Civilization-style 4X strategy game built on a decentralized architecture using Nostr for game state, Cashu for verifiable randomness, and Iroh for P2P networking.

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Desktop Runtime | Tauri 2.x | Cross-platform desktop app (macOS, Windows, Linux) |
| Frontend | React 18+ | UI components, game interface |
| Map Rendering | Three.js / WebGL | Hardware-accelerated hex map visualization |
| Backend Engine | Bevy 0.14+ | Game logic, state management, ECS architecture |
| Networking | Iroh | P2P peer discovery and direct connections |
| Events | nostr-rs / rust-nostr | Nostr event creation, signing, validation |
| Randomness | CDK (Cashu Dev Kit) | Cashu mint for unbiased randomness tokens |
| Local Storage | Local Nostr Relay | Game state persistence (inspired by Damus Notedeck) |

## Architecture Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                       Tauri Application                          │
├─────────────────────────────────────────────────────────────────┤
│  ┌───────────────────────┐    ┌───────────────────────────────┐ │
│  │    React Frontend     │    │        Bevy Backend           │ │
│  │                       │    │                               │ │
│  │  ┌─────────────────┐  │    │  ┌─────────────────────────┐ │ │
│  │  │  Three.js Map   │  │◄──►│  │    Game Engine (ECS)    │ │ │
│  │  │  Renderer       │  │IPC │  │    - Systems            │ │ │
│  │  └─────────────────┘  │    │  │    - Components         │ │ │
│  │                       │    │  │    - Resources          │ │ │
│  │  ┌─────────────────┐  │    │  └─────────────────────────┘ │ │
│  │  │  UI Components  │  │    │            │                  │ │
│  │  │  - Game HUD     │  │    │  ┌─────────▼───────────────┐ │ │
│  │  │  - Menus        │  │    │  │     Core Library        │ │ │
│  │  │  - Diplomacy    │  │    │  │  (nostr-nations-core)   │ │ │
│  │  │  - Tech Tree    │  │    │  │                         │ │ │
│  │  └─────────────────┘  │    │  │  - Game Rules           │ │ │
│  └───────────────────────┘    │  │  - State Validation     │ │ │
│                               │  │  - Event Processing     │ │ │
│                               │  │  - Deterministic Logic  │ │ │
│                               │  └─────────────────────────┘ │ │
│                               └───────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│                        Networking Layer                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐ │
│  │    Iroh     │  │   Cashu     │  │   Local Nostr Relay     │ │
│  │  P2P Mesh   │  │   Mint      │  │   (Game State Store)    │ │
│  └─────────────┘  └─────────────┘  └─────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
nostr-nations/
├── .ralph/                    # Ralph agent configuration
│   ├── specs/                 # Project specifications (this folder)
│   ├── PROMPT.md              # Agent instructions
│   ├── AGENT.md               # Build instructions
│   └── fix_plan.md            # Task tracking
│
├── crates/                    # Rust workspace
│   ├── nostr-nations-core/    # Core game library (no UI dependencies)
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── game/          # Game state, rules, mechanics
│   │   │   ├── map/           # Hex grid, terrain, generation
│   │   │   ├── units/         # Unit types, movement, combat
│   │   │   ├── cities/        # City management, production
│   │   │   ├── tech/          # Technology tree
│   │   │   ├── diplomacy/     # Treaties, alliances
│   │   │   ├── events/        # Nostr event handling
│   │   │   └── randomness/    # Cashu integration
│   │   └── Cargo.toml
│   │
│   ├── nostr-nations-bevy/    # Bevy game engine integration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── systems/       # Bevy ECS systems
│   │   │   ├── components/    # Bevy components
│   │   │   ├── resources/     # Bevy resources
│   │   │   └── plugins/       # Bevy plugins
│   │   └── Cargo.toml
│   │
│   └── nostr-nations-network/ # Networking (Iroh + Nostr relay)
│       ├── src/
│       │   ├── lib.rs
│       │   ├── iroh/          # P2P connectivity
│       │   ├── relay/         # Local Nostr relay
│       │   └── sync/          # Game state synchronization
│       └── Cargo.toml
│
├── src-tauri/                 # Tauri backend
│   ├── src/
│   │   ├── main.rs
│   │   ├── commands/          # Tauri IPC commands
│   │   └── state/             # Application state
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── src/                       # React frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   │   ├── game/              # Game UI components
│   │   ├── map/               # Three.js map renderer
│   │   ├── hud/               # Heads-up display
│   │   ├── menus/             # Menu screens
│   │   └── common/            # Shared components
│   ├── hooks/                 # React hooks
│   ├── stores/                # State management
│   ├── types/                 # TypeScript types
│   └── utils/                 # Utility functions
│
├── Cargo.toml                 # Workspace manifest
├── package.json               # Node dependencies
└── README.md
```

## Frontend-Backend Communication

### Tauri IPC Protocol

The React frontend communicates with the Bevy backend via Tauri's command/event system.

**Commands (Frontend → Backend):**
- Synchronous request/response pattern
- Used for game actions, queries, configuration

**Events (Backend → Frontend):**
- Asynchronous push notifications
- Used for game state updates, turn notifications, network events

### Command Categories

| Category | Commands | Description |
|----------|----------|-------------|
| Game Setup | `create_game`, `join_game`, `load_game` | Game lifecycle |
| Turn Actions | `end_turn`, `move_unit`, `found_city`, `attack` | Player actions |
| Production | `set_production`, `buy_item`, `manage_workers` | City management |
| Diplomacy | `propose_treaty`, `accept_treaty`, `declare_war` | Player relations |
| Tech | `research_tech`, `get_tech_tree` | Technology |
| Query | `get_game_state`, `get_unit_info`, `get_city_info` | State queries |
| Network | `connect_peer`, `disconnect`, `sync_state` | Networking |

### Event Categories

| Category | Events | Description |
|----------|--------|-------------|
| State | `game_state_updated`, `map_revealed` | State changes |
| Turn | `turn_started`, `turn_ended`, `player_turn` | Turn flow |
| Combat | `combat_started`, `combat_resolved` | Battle events |
| Diplomacy | `treaty_proposed`, `treaty_accepted`, `war_declared` | Diplomatic events |
| Network | `peer_connected`, `peer_disconnected`, `sync_complete` | Network events |

## Core Library Principles

The `nostr-nations-core` crate must:

1. **Be UI-agnostic**: No dependencies on Bevy, Tauri, or any rendering framework
2. **Be deterministic**: Same inputs always produce same outputs
3. **Be serializable**: All state can be serialized to/from Nostr events
4. **Be validatable**: Any game state can be validated by replaying events
5. **Be testable**: Comprehensive unit tests without UI dependencies

## Client Types

### Full Client (Desktop)
- Runs complete game engine locally
- Can host games (run Cashu mint)
- Stores game data on local Nostr relay
- Direct P2P connections via Iroh

### Light Client (Mobile - Future)
- Minimal local processing
- Connects to full client or public relays
- Renders game state received from host
- Submits actions to host for validation

## Session Management

- **One active game session** per client instance
- **Multiple stored games** persisted on local relay
- Games can be paused and resumed
- Turn-based nature allows asynchronous play

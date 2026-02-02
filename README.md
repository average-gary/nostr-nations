# Nostr Nations

<!-- ![Nostr Nations Logo](docs/assets/logo.png) -->

**A turn-based strategy game built on Nostr**

[![Build Status](https://img.shields.io/github/actions/workflow/status/nostr-nations/nostr-nations/ci.yml?branch=main)](https://github.com/nostr-nations/nostr-nations/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0-green.svg)](Cargo.toml)

## Overview

Nostr Nations is a multiplayer turn-based strategy game where players build civilizations, conduct diplomacy, wage war, and race toward victory—all powered by the decentralized Nostr protocol. The game features deterministic gameplay with verifiable randomness provided by Cashu tokens, ensuring fair and transparent mechanics for all players.

<!-- ![Game Screenshot](docs/assets/screenshot.png) -->

### Key Features

- Decentralized multiplayer via the Nostr protocol
- Verifiable randomness using Cashu ecash
- Cross-platform desktop application (macOS, Windows, Linux)
- Hex-based map with diverse terrain types
- Deep strategic systems: combat, diplomacy, technology, and economics

## Features

### Multiplayer via Nostr Protocol

Connect and play with anyone on the Nostr network. No central servers required—games are coordinated through Nostr relays, enabling truly decentralized multiplayer.

### Deterministic Gameplay with Cashu Randomness

All game mechanics are deterministic and verifiable. Random elements (combat outcomes, exploration rewards) use Cashu token secrets as entropy, ensuring fairness that any player can independently verify.

### Cross-Platform Support

Built with Tauri, Nostr Nations runs natively on macOS, Windows, and Linux with a single codebase, delivering excellent performance and a native feel on every platform.

### Hex-Based Map System

Explore procedurally generated worlds with varied terrain including plains, forests, mountains, deserts, and oceans. Each terrain type affects movement, combat, and resource production.

### Strategic Depth

- **Combat**: Tactical unit battles with terrain bonuses and unit counters
- **Diplomacy**: Forge alliances, declare wars, negotiate treaties
- **Technology**: Research trees unlocking new units, buildings, and abilities
- **Economics**: Manage resources, trade with other nations, build infrastructure

### Multiple Victory Conditions

Achieve victory through military conquest, technological supremacy, diplomatic unity, or economic dominance.

## Quick Start

### Prerequisites

- [Rust](https://rustup.rs/) (1.75 or later)
- [Node.js](https://nodejs.org/) (18 or later)
- [pnpm](https://pnpm.io/) (recommended) or npm

### Installation

```bash
# Clone the repository
git clone https://github.com/nostr-nations/nostr-nations.git
cd nostr-nations

# Install frontend dependencies
pnpm install

# Build the Rust backend
cargo build
```

### Running the App

```bash
# Development mode (hot-reload enabled)
pnpm tauri dev

# Production build
pnpm tauri build
```

## Documentation

| Document                                 | Description                          |
| ---------------------------------------- | ------------------------------------ |
| [Architecture](docs/ARCHITECTURE.md)     | System design and component overview |
| [API Documentation](docs/API.md)         | Backend API reference                |
| [Game Mechanics](docs/GAME_MECHANICS.md) | Detailed game rules and systems      |
| [Development Guide](docs/DEVELOPMENT.md) | Setup and contribution guidelines    |
| [User Manual](docs/USER_MANUAL.md)       | How to play the game                 |

## Development

### Setup

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js dependencies
pnpm install

# Install Tauri CLI
cargo install tauri-cli
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run frontend linting
pnpm lint

# Format code
pnpm format
```

### Project Structure

```
nostr-nations/
├── crates/
│   ├── nostr-nations-core/     # Core game logic
│   ├── nostr-nations-bevy/     # Bevy game engine integration
│   └── nostr-nations-network/  # Nostr networking layer
├── src/                        # React frontend
├── src-tauri/                  # Tauri application
└── docs/                       # Documentation
```

### Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

Please read the [Development Guide](docs/DEVELOPMENT.md) for detailed contribution guidelines.

## Technology Stack

| Layer                 | Technology                       |
| --------------------- | -------------------------------- |
| **Backend**           | Rust                             |
| **Desktop Framework** | Tauri 2.0                        |
| **Frontend**          | React 18 + TypeScript            |
| **3D Rendering**      | Three.js (via React Three Fiber) |
| **State Management**  | Zustand                          |
| **Styling**           | Tailwind CSS                     |
| **Networking**        | Nostr (nostr-sdk) + Iroh (P2P)   |
| **Randomness**        | Cashu (cdk)                      |
| **Game Engine**       | Bevy (optional native client)    |

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Nostr Protocol](https://nostr.com/) - Decentralized social networking protocol
- [Cashu](https://cashu.space/) - Chaumian ecash system for Bitcoin
- [Tauri](https://tauri.app/) - Framework for building desktop applications
- [Bevy](https://bevyengine.org/) - Data-driven game engine in Rust

Inspired by classic strategy games including Civilization, Age of Empires, and Polytopia.

---

<p align="center">
  <sub>Built with Rust and powered by Nostr</sub>
</p>

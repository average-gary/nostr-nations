# Changelog

All notable changes to Nostr Nations will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Particle effects for visual polish (planned)
- Network sync optimization (planned)
- Multiplayer stress testing (planned)
- NIP-04 encryption for fog of war events (planned)

## [0.1.0] - 2026-02-02

### Added

#### Core Game Engine

- Turn-based game loop with configurable turn limits
- Hex coordinate system with comprehensive hex math utilities
- Terrain types (plains, forest, mountains, desert, ocean, tundra) with yield calculations
- Procedural map generation with deterministic seeding
- Resource system (strategic, luxury, bonus resources)

#### Unit System

- Unit types with combat stats, movement points, and abilities
- A\* pathfinding on hex grid with terrain costs
- Zone of control mechanics
- Combat system with terrain defense bonuses
- Experience and promotion system

#### City System

- City founding and population growth mechanics
- Citizen tile assignment and working radius
- Production queue for units and buildings
- Building effects and bonuses
- Border expansion through culture accumulation

#### Technology Tree

- 60+ technologies with prerequisites
- Research mechanics with science point accumulation
- Tech unlocks for units, buildings, and abilities
- Technology tree visualization UI

#### Diplomacy System

- Relationship tracking between civilizations
- Treaty system (open borders, defensive pact, alliance, trade agreement)
- War and peace mechanics with war weariness
- Trade deal evaluation and builder UI
- Diplomatic notifications

#### Victory Conditions

- Domination victory (capture all capitals)
- Science victory (spaceship construction)
- Economic victory (treasury threshold)
- Diplomatic victory (UN votes)
- Score victory (turn limit reached)

#### Nostr Integration

- Local relay with SQLite storage
- Game event kinds (30100-30199) for all game actions
- Event signing with Nostr keys
- Event chain validation
- State reconstruction from events
- Deterministic game replay

#### Cashu Randomness

- CDK integration for verifiable randomness
- Blinded message creation and proof verification
- Integration with map generation
- Integration with combat resolution
- Proof recording in Nostr events

#### Networking

- Iroh P2P library integration
- Peer discovery and connection tickets
- QR code generation for game invites
- Sync protocol messages (initial and incremental)

#### React Frontend

- Tauri 2.0 desktop application
- React 18 + TypeScript + Vite
- Three.js rendering via React Three Fiber
- Zustand state management
- Tailwind CSS styling

#### Map Renderer

- Hex geometry generation with instanced rendering
- Terrain textures and colors
- Camera controls (pan, zoom, rotation)
- Tile selection and highlighting
- Unit and city rendering
- Fog of war shader with optimized rendering

#### User Interface

- Main menu with new game wizard
- Load game and settings screens
- Join game screen with QR scanner
- Top bar (resources, turn counter)
- Minimap with viewport indicator
- Selection panel for units and cities
- Action buttons and notifications
- Tech tree viewer overlay
- Diplomacy screen overlay
- City management screen overlay
- Military overview overlay
- Victory screen overlay

#### Audio System

- AudioManager for sound effects and music
- useAudio hooks for React integration
- AudioContext provider
- Music track support

#### Visual Feedback Components

- LoadingSpinner
- ProgressBar
- Toast notifications
- Tooltip system
- Modal dialogs
- ConfirmDialog

#### Documentation

- Architecture documentation (ARCHITECTURE.md)
- API reference (API.md)
- Game mechanics guide (GAME_MECHANICS.md)
- Development guide (DEVELOPMENT.md)
- User manual (USER_MANUAL.md)
- Deployment guide (DEPLOYMENT.md)

#### Testing

- 1330+ passing tests across all crates
- 50 integration tests covering all game flows
- 55 fog of war visibility tests
- 63 protocol/replay tests
- Unit tests for hex math, combat, cities, diplomacy, victory conditions
- Tauri event flow tests

#### Development Infrastructure

- Rust workspace with multiple crates
- GitHub Actions CI/CD pipeline
- ESLint and Prettier configuration
- Rustfmt and Clippy configuration

### Changed

- Optimized map rendering with frustum culling
- Shared geometries and materials for instanced rendering
- Memoized React components for performance
- Fixed all Clippy warnings across crates

### Fixed

- Bevy test failures (query conflicts, input resources, civilian attack logic)
- Core library test failures (serialization, RNG seeding, map determinism)
- Network test failures (expiration timing tests)
- Encryption key generation uniqueness issue causing test failures
- Clippy warnings: manual slice size calculation, unnecessary map_or, abs_diff pattern
- Clippy warnings: derivable_impls for Default trait, needless_range_loop
- Clippy warnings: unnecessary casts in conflict resolution code
- 1330+ tests fixed and passing

### Security

- Nostr event signing ensures action authenticity
- Cashu proofs provide verifiable randomness
- Local-first architecture with optional relay sync
- No central server required for gameplay

[Unreleased]: https://github.com/nostr-nations/nostr-nations/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/nostr-nations/nostr-nations/releases/tag/v0.1.0

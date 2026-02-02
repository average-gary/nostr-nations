//! Nostr Nations Core Library
//!
//! This crate contains the core game logic for Nostr Nations, a Civilization-style
//! 4X strategy game that uses Nostr events for deterministic replay and Cashu
//! for verifiable randomness.
//!
//! # Design Principles
//!
//! - **No UI dependencies**: This crate is purely game logic
//! - **Deterministic**: Same inputs always produce same outputs
//! - **Serializable**: All state can be saved/loaded via serde
//! - **Thoroughly tested**: Comprehensive test coverage

// Core modules
pub mod hex;
pub mod map;
pub mod terrain;
pub mod types;
pub mod yields;

// Game state modules
pub mod game_state;
pub mod player;
pub mod settings;

// Map generation
pub mod mapgen;

// Units and combat
pub mod combat;
pub mod pathfinding;
pub mod unit;

// Cities and buildings
pub mod city;

// Technology
pub mod technology;

// Trading system
pub mod trading;

// Victory conditions
pub mod victory;

// Memory optimization utilities
pub mod memory;

// Nostr events and replay
pub mod events;
pub mod replay;

// Visibility and fog of war
pub mod visibility;

// Cashu randomness
pub mod cashu;

// Re-exports for convenience
pub use cashu::{
    combat_random_from_proof, map_seed_from_proof, CashuConfig, DeterministicRandomness,
    RandomnessContext, RandomnessError, RandomnessManager, RandomnessProof, RandomnessProvider,
    RandomnessRequest,
};
pub use city::{BuildingType, City, ProductionItem, WonderType};
pub use combat::{resolve_combat, CombatContext, CombatResult};
pub use events::{EventBuilder, EventChain, GameAction, GameEvent};
pub use game_state::{DiplomacyState, DiplomaticStatus, GameError, GamePhase, GameState};
pub use hex::HexCoord;
pub use map::{Map, Tile};
pub use mapgen::{MapGenConfig, MapGenerator, SeededRng};
pub use pathfinding::{find_path, find_reachable, PathConfig, PathResult};
pub use player::{Civilization, Player, Score};
pub use replay::{ActionEffect, ActionResult, GameEngine, ReplayConfig, ReplayError};
pub use settings::{Difficulty, GameSettings, GameSpeed};
pub use technology::{TechTree, TechUnlocks, Technology};
pub use terrain::{Feature, Improvement, Resource, ResourceCategory, Road, Terrain};
pub use trading::{
    calculate_trade_value, execute_trade, TradeError, TradeFairness, TradeItems, TradeManager,
    TradeOffer, TradeStatus,
};
pub use types::*;
pub use unit::{Promotion, Unit, UnitCategory, UnitStats, UnitType};
pub use victory::{SpaceshipProgress, VictoryChecker};
pub use yields::Yields;

// Memory optimization re-exports
pub use memory::{Arena, InternPool, InternedString, MemoryStats, ObjectPool, PackedCoord};

// Visibility re-exports
pub use visibility::{
    redact_event_for_player, FilteredEvent, FilteredGameState, PlayerSummary, TileVisibility,
    VisibilityFilter,
};

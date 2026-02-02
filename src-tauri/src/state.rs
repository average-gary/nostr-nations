//! Application state management for Tauri.
//!
//! This module manages the global application state that is shared
//! across all Tauri commands.

use nostr_nations_core::{GameEngine, GameSettings, GameState};
use std::collections::HashMap;

/// Main application state.
#[allow(dead_code)]
pub struct AppState {
    /// Current game engine (if a game is active).
    pub game_engine: Option<GameEngine>,
    /// Network peer count (simplified for now).
    pub peer_count: usize,
    /// Saved games list.
    pub saved_games: HashMap<String, String>,
    /// User preferences.
    pub preferences: Preferences,
}

impl AppState {
    /// Create a new application state.
    pub fn new() -> Self {
        Self {
            game_engine: None,
            peer_count: 0,
            saved_games: HashMap::new(),
            preferences: Preferences::default(),
        }
    }

    /// Create a new game with the given settings.
    pub fn create_game(&mut self, settings: GameSettings, seed: [u8; 32]) -> Result<(), AppError> {
        let engine = GameEngine::new(settings, seed);
        self.game_engine = Some(engine);
        Ok(())
    }

    /// Get the current game state.
    pub fn get_game_state(&self) -> Result<&GameState, AppError> {
        self.game_engine
            .as_ref()
            .map(|e| &e.state)
            .ok_or(AppError::NoActiveGame)
    }

    /// Get mutable access to the game engine.
    pub fn get_engine_mut(&mut self) -> Result<&mut GameEngine, AppError> {
        self.game_engine.as_mut().ok_or(AppError::NoActiveGame)
    }

    /// Check if a game is currently active.
    pub fn has_active_game(&self) -> bool {
        self.game_engine.is_some()
    }

    /// End the current game.
    #[allow(dead_code)]
    pub fn end_game(&mut self) {
        self.game_engine = None;
    }

    /// Get peer count.
    pub fn connected_peers(&self) -> usize {
        self.peer_count
    }

    /// Add a peer.
    pub fn add_peer(&mut self) {
        self.peer_count += 1;
    }

    /// Remove a peer.
    pub fn remove_peer(&mut self) {
        self.peer_count = self.peer_count.saturating_sub(1);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// User preferences.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Preferences {
    /// Enable sound effects.
    pub sound_enabled: bool,
    /// Enable music.
    pub music_enabled: bool,
    /// Music volume (0.0 - 1.0).
    pub music_volume: f32,
    /// Sound effects volume (0.0 - 1.0).
    pub sfx_volume: f32,
    /// Auto-save interval in turns (0 = disabled).
    pub auto_save_turns: u32,
    /// Show grid overlay on map.
    pub show_grid: bool,
    /// Show yield icons on tiles.
    pub show_yields: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            sound_enabled: true,
            music_enabled: true,
            music_volume: 0.7,
            sfx_volume: 0.8,
            auto_save_turns: 5,
            show_grid: true,
            show_yields: true,
        }
    }
}

/// Application errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum AppError {
    #[error("No active game")]
    NoActiveGame,
    #[error("Game already in progress")]
    GameAlreadyActive,
    #[error("Invalid game state: {0}")]
    InvalidState(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

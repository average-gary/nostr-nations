//! Game settings and configuration.

use crate::types::{Era, MapSize, VictoryConditions};
use serde::{Deserialize, Serialize};

/// Configuration for a game session.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSettings {
    /// Display name for the game.
    pub name: String,
    /// Map size preset.
    pub map_size: MapSize,
    /// Number of players (2-8).
    pub player_count: u8,
    /// Enabled victory conditions.
    pub victory_conditions: VictoryConditions,
    /// Turn time limit in seconds (0 = unlimited).
    pub turn_timer: u32,
    /// Maximum number of turns (0 = unlimited).
    pub max_turns: u32,
    /// Allow technology trading between players.
    pub tech_trading: bool,
    /// Starting era for all players.
    pub starting_era: Era,
    /// Does the map wrap horizontally?
    pub map_wraps: bool,
    /// Fog of war enabled?
    pub fog_of_war: bool,
    /// Barbarians enabled? (AI hostile units)
    pub barbarians: bool,
    /// Game speed multiplier (1.0 = normal).
    pub game_speed: GameSpeed,
    /// Difficulty level.
    pub difficulty: Difficulty,
}

impl GameSettings {
    /// Create default settings for a new game.
    pub fn new(name: String) -> Self {
        Self {
            name,
            map_size: MapSize::Standard,
            player_count: 2,
            victory_conditions: VictoryConditions::all_enabled(),
            turn_timer: 0,
            max_turns: 500,
            tech_trading: true,
            starting_era: Era::Ancient,
            map_wraps: false,
            fog_of_war: true,
            barbarians: false, // Disabled for now as we're focusing on multiplayer
            game_speed: GameSpeed::Normal,
            difficulty: Difficulty::Normal,
        }
    }

    /// Create settings for a quick 2-player duel.
    pub fn duel(name: String) -> Self {
        Self {
            name,
            map_size: MapSize::Duel,
            player_count: 2,
            victory_conditions: VictoryConditions::domination_only(),
            turn_timer: 60,
            max_turns: 250,
            tech_trading: false,
            starting_era: Era::Ancient,
            map_wraps: false,
            fog_of_war: true,
            barbarians: false,
            game_speed: GameSpeed::Quick,
            difficulty: Difficulty::Normal,
        }
    }

    /// Validate settings and return any errors.
    pub fn validate(&self) -> Result<(), SettingsError> {
        if self.name.is_empty() {
            return Err(SettingsError::EmptyName);
        }
        if self.name.len() > 64 {
            return Err(SettingsError::NameTooLong);
        }
        if self.player_count < 2 {
            return Err(SettingsError::TooFewPlayers);
        }
        if self.player_count > 8 {
            return Err(SettingsError::TooManyPlayers);
        }
        let (width, height) = self.map_size.dimensions();
        let min_tiles_per_player: u32 = 100;
        let total_tiles = width * height;
        let tiles_per_player = total_tiles / (self.player_count as u32);
        if tiles_per_player < min_tiles_per_player {
            return Err(SettingsError::MapTooSmallForPlayers);
        }
        Ok(())
    }

    /// Get the map dimensions based on settings.
    pub fn map_dimensions(&self) -> (u32, u32) {
        self.map_size.dimensions()
    }

    /// Calculate production multiplier based on game speed.
    pub fn production_multiplier(&self) -> f32 {
        self.game_speed.production_multiplier()
    }

    /// Calculate research multiplier based on game speed.
    pub fn research_multiplier(&self) -> f32 {
        self.game_speed.research_multiplier()
    }
}

impl Default for GameSettings {
    fn default() -> Self {
        Self::new("New Game".to_string())
    }
}

/// Game speed affects how fast various game mechanics progress.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum GameSpeed {
    /// Fast game - reduced costs.
    Quick,
    /// Standard game pace.
    #[default]
    Normal,
    /// Slower game - increased costs.
    Epic,
    /// Very slow game.
    Marathon,
}

impl GameSpeed {
    /// Get the production cost multiplier.
    pub const fn production_multiplier(&self) -> f32 {
        match self {
            GameSpeed::Quick => 0.67,
            GameSpeed::Normal => 1.0,
            GameSpeed::Epic => 1.5,
            GameSpeed::Marathon => 3.0,
        }
    }

    /// Get the research cost multiplier.
    pub const fn research_multiplier(&self) -> f32 {
        match self {
            GameSpeed::Quick => 0.67,
            GameSpeed::Normal => 1.0,
            GameSpeed::Epic => 1.5,
            GameSpeed::Marathon => 3.0,
        }
    }

    /// Get the growth rate multiplier.
    pub const fn growth_multiplier(&self) -> f32 {
        match self {
            GameSpeed::Quick => 0.67,
            GameSpeed::Normal => 1.0,
            GameSpeed::Epic => 1.5,
            GameSpeed::Marathon => 3.0,
        }
    }
}

/// AI difficulty level (affects bonuses and behavior).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Difficulty {
    /// Easier for new players.
    Settler,
    /// Slightly easier.
    Chieftain,
    /// Balanced.
    #[default]
    Normal,
    /// AI gets small bonuses.
    King,
    /// AI gets moderate bonuses.
    Emperor,
    /// AI gets large bonuses.
    Deity,
}

impl Difficulty {
    /// Get the AI yield bonus percentage.
    pub const fn ai_yield_bonus(&self) -> i32 {
        match self {
            Difficulty::Settler => -20,
            Difficulty::Chieftain => -10,
            Difficulty::Normal => 0,
            Difficulty::King => 10,
            Difficulty::Emperor => 25,
            Difficulty::Deity => 50,
        }
    }

    /// Get the AI combat bonus percentage.
    pub const fn ai_combat_bonus(&self) -> i32 {
        match self {
            Difficulty::Settler => -10,
            Difficulty::Chieftain => -5,
            Difficulty::Normal => 0,
            Difficulty::King => 5,
            Difficulty::Emperor => 10,
            Difficulty::Deity => 20,
        }
    }
}

/// Errors from invalid game settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SettingsError {
    EmptyName,
    NameTooLong,
    TooFewPlayers,
    TooManyPlayers,
    MapTooSmallForPlayers,
}

impl std::fmt::Display for SettingsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SettingsError::EmptyName => write!(f, "Game name cannot be empty"),
            SettingsError::NameTooLong => write!(f, "Game name must be 64 characters or less"),
            SettingsError::TooFewPlayers => write!(f, "Need at least 2 players"),
            SettingsError::TooManyPlayers => write!(f, "Maximum 8 players allowed"),
            SettingsError::MapTooSmallForPlayers => {
                write!(f, "Map is too small for this many players")
            }
        }
    }
}

impl std::error::Error for SettingsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let settings = GameSettings::default();
        assert_eq!(settings.player_count, 2);
        assert_eq!(settings.map_size, MapSize::Standard);
        assert!(settings.fog_of_war);
    }

    #[test]
    fn test_duel_settings() {
        let settings = GameSettings::duel("Quick Match".to_string());
        assert_eq!(settings.map_size, MapSize::Duel);
        assert_eq!(settings.player_count, 2);
        assert_eq!(settings.turn_timer, 60);
    }

    #[test]
    fn test_validation_valid() {
        let settings = GameSettings::new("Test Game".to_string());
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn test_validation_empty_name() {
        let settings = GameSettings {
            name: String::new(),
            ..Default::default()
        };
        assert_eq!(settings.validate(), Err(SettingsError::EmptyName));
    }

    #[test]
    fn test_validation_too_few_players() {
        let settings = GameSettings {
            player_count: 1,
            ..Default::default()
        };
        assert_eq!(settings.validate(), Err(SettingsError::TooFewPlayers));
    }

    #[test]
    fn test_game_speed_multipliers() {
        assert_eq!(GameSpeed::Quick.production_multiplier(), 0.67);
        assert_eq!(GameSpeed::Normal.production_multiplier(), 1.0);
        assert_eq!(GameSpeed::Marathon.production_multiplier(), 3.0);
    }

    #[test]
    fn test_difficulty_bonuses() {
        assert_eq!(Difficulty::Normal.ai_yield_bonus(), 0);
        assert_eq!(Difficulty::Deity.ai_yield_bonus(), 50);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = GameSettings::new("Test".to_string());
        let json = serde_json::to_string(&settings).unwrap();
        let restored: GameSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.name, settings.name);
        assert_eq!(restored.map_size, settings.map_size);
    }
}

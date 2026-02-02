//! Core type aliases used throughout the crate.

use serde::{Deserialize, Serialize};

/// Unique identifier for a game session.
pub type GameId = String;

/// Player index (0-3 for 4 player games).
pub type PlayerId = u8;

/// Unique identifier for a unit.
pub type UnitId = u64;

/// Unique identifier for a city.
pub type CityId = u64;

/// Nostr event ID for chain validation.
pub type EventId = String;

/// Technology identifier.
pub type TechId = String;

/// Game era progression.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Era {
    #[default]
    Ancient,
    Classical,
    Medieval,
    Renaissance,
    Industrial,
    Modern,
}

impl Era {
    /// Get the next era in progression.
    pub const fn next(&self) -> Option<Era> {
        match self {
            Era::Ancient => Some(Era::Classical),
            Era::Classical => Some(Era::Medieval),
            Era::Medieval => Some(Era::Renaissance),
            Era::Renaissance => Some(Era::Industrial),
            Era::Industrial => Some(Era::Modern),
            Era::Modern => None,
        }
    }

    /// Get the era index (0-5).
    pub const fn index(&self) -> usize {
        match self {
            Era::Ancient => 0,
            Era::Classical => 1,
            Era::Medieval => 2,
            Era::Renaissance => 3,
            Era::Industrial => 4,
            Era::Modern => 5,
        }
    }

    /// Get all era variants.
    pub const fn all() -> &'static [Era] {
        &[
            Era::Ancient,
            Era::Classical,
            Era::Medieval,
            Era::Renaissance,
            Era::Industrial,
            Era::Modern,
        ]
    }
}

impl std::fmt::Display for Era {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Era::Ancient => write!(f, "Ancient Era"),
            Era::Classical => write!(f, "Classical Era"),
            Era::Medieval => write!(f, "Medieval Era"),
            Era::Renaissance => write!(f, "Renaissance Era"),
            Era::Industrial => write!(f, "Industrial Era"),
            Era::Modern => write!(f, "Modern Era"),
        }
    }
}

/// Map size presets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum MapSize {
    Duel,
    Small,
    #[default]
    Standard,
    Large,
    Huge,
}

impl MapSize {
    /// Get the dimensions (width, height) for this map size.
    pub const fn dimensions(&self) -> (u32, u32) {
        match self {
            MapSize::Duel => (40, 25),
            MapSize::Small => (60, 38),
            MapSize::Standard => (80, 50),
            MapSize::Large => (100, 63),
            MapSize::Huge => (120, 75),
        }
    }

    /// Get the recommended number of players for this map size.
    pub const fn recommended_players(&self) -> u8 {
        match self {
            MapSize::Duel => 2,
            MapSize::Small => 3,
            MapSize::Standard => 4,
            MapSize::Large => 6,
            MapSize::Huge => 8,
        }
    }

    /// Get all map size variants.
    pub const fn all() -> &'static [MapSize] {
        &[
            MapSize::Duel,
            MapSize::Small,
            MapSize::Standard,
            MapSize::Large,
            MapSize::Huge,
        ]
    }
}

impl std::fmt::Display for MapSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (w, h) = self.dimensions();
        match self {
            MapSize::Duel => write!(f, "Duel ({}x{})", w, h),
            MapSize::Small => write!(f, "Small ({}x{})", w, h),
            MapSize::Standard => write!(f, "Standard ({}x{})", w, h),
            MapSize::Large => write!(f, "Large ({}x{})", w, h),
            MapSize::Huge => write!(f, "Huge ({}x{})", w, h),
        }
    }
}

/// Victory conditions that can be enabled for a game.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VictoryConditions {
    /// Capture all capitals
    pub domination: bool,
    /// Build and launch spaceship
    pub science: bool,
    /// Accumulate wealth threshold
    pub economic: bool,
    /// Win world leader vote
    pub diplomatic: bool,
    /// Highest score at turn limit
    pub score: bool,
}

impl Default for VictoryConditions {
    fn default() -> Self {
        Self {
            domination: true,
            science: true,
            economic: true,
            diplomatic: true,
            score: true,
        }
    }
}

impl VictoryConditions {
    /// Create with all victories enabled.
    pub const fn all_enabled() -> Self {
        Self {
            domination: true,
            science: true,
            economic: true,
            diplomatic: true,
            score: true,
        }
    }

    /// Create with only domination victory.
    pub const fn domination_only() -> Self {
        Self {
            domination: true,
            science: false,
            economic: false,
            diplomatic: false,
            score: false,
        }
    }
}

/// Types of victory a player can achieve.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VictoryType {
    Domination,
    Science,
    Economic,
    Diplomatic,
    Score,
}

/// RGB color for player identification.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl PlayerColor {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Default colors for players 0-7.
    pub const fn default_for_player(player_id: PlayerId) -> Self {
        match player_id {
            0 => Self::new(255, 0, 0),     // Red
            1 => Self::new(0, 0, 255),     // Blue
            2 => Self::new(255, 255, 0),   // Yellow
            3 => Self::new(0, 255, 0),     // Green
            4 => Self::new(128, 0, 128),   // Purple
            5 => Self::new(255, 165, 0),   // Orange
            6 => Self::new(0, 255, 255),   // Cyan
            _ => Self::new(255, 192, 203), // Pink
        }
    }

    /// Convert to hex string (e.g., "#FF0000").
    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
}

impl Default for PlayerColor {
    fn default() -> Self {
        Self::new(128, 128, 128) // Gray
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_era_progression() {
        assert_eq!(Era::Ancient.next(), Some(Era::Classical));
        assert_eq!(Era::Modern.next(), None);
    }

    #[test]
    fn test_era_index() {
        assert_eq!(Era::Ancient.index(), 0);
        assert_eq!(Era::Modern.index(), 5);
    }

    #[test]
    fn test_map_dimensions() {
        assert_eq!(MapSize::Duel.dimensions(), (40, 25));
        assert_eq!(MapSize::Standard.dimensions(), (80, 50));
    }

    #[test]
    fn test_player_colors() {
        let red = PlayerColor::default_for_player(0);
        assert_eq!(red.r, 255);
        assert_eq!(red.g, 0);
        assert_eq!(red.b, 0);
        assert_eq!(red.to_hex(), "#FF0000");
    }

    #[test]
    fn test_victory_conditions_default() {
        let vc = VictoryConditions::default();
        assert!(vc.domination);
        assert!(vc.science);
    }
}

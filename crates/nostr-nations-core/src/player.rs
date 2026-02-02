//! Player state and civilization data.

use crate::hex::HexCoord;
use crate::types::{CityId, Era, PlayerColor, PlayerId, TechId};
use crate::victory::SpaceshipProgress;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A player in the game.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    /// Player index (0-3 typically).
    pub id: PlayerId,
    /// Nostr public key (hex encoded).
    pub pubkey: String,
    /// Display name.
    pub name: String,
    /// Chosen civilization.
    pub civilization: Civilization,
    /// Player's color for map display.
    pub color: PlayerColor,
    /// Current gold in treasury.
    pub gold: i32,
    /// Gold income per turn (can be negative).
    pub gold_per_turn: i32,
    /// Science output per turn.
    pub science_per_turn: i32,
    /// Culture output per turn.
    pub culture_per_turn: i32,
    /// Currently researching technology.
    pub current_research: Option<TechId>,
    /// Accumulated research progress on current tech.
    pub research_progress: u32,
    /// Set of completed technologies.
    pub technologies: HashSet<TechId>,
    /// ID of this player's capital city.
    pub capital: Option<CityId>,
    /// Whether this player has been eliminated.
    pub eliminated: bool,
    /// Set of tiles this player has explored (can see terrain).
    pub explored_tiles: HashSet<HexCoord>,
    /// Player's current score breakdown.
    pub score: Score,
    /// Is this player the game host (Cashu mint operator)?
    pub is_host: bool,
    /// Progress toward building the spaceship (for science victory).
    pub spaceship: SpaceshipProgress,
}

impl Player {
    /// Create a new player with default values.
    pub fn new(id: PlayerId, pubkey: String, name: String, civilization: Civilization) -> Self {
        Self {
            id,
            pubkey,
            name,
            color: PlayerColor::default_for_player(id),
            civilization,
            gold: 0,
            gold_per_turn: 0,
            science_per_turn: 0,
            culture_per_turn: 0,
            current_research: None,
            research_progress: 0,
            technologies: HashSet::new(),
            capital: None,
            eliminated: false,
            explored_tiles: HashSet::new(),
            score: Score::default(),
            is_host: false,
            spaceship: SpaceshipProgress::default(),
        }
    }

    /// Check if the player has researched a specific technology.
    pub fn has_tech(&self, tech_id: &TechId) -> bool {
        self.technologies.contains(tech_id)
    }

    /// Add a researched technology.
    pub fn add_tech(&mut self, tech_id: TechId) {
        self.technologies.insert(tech_id);
        self.score.techs += 1;
        self.score.recalculate();
    }

    /// Mark a tile as explored.
    pub fn explore_tile(&mut self, coord: HexCoord) {
        if self.explored_tiles.insert(coord) {
            self.score.land += 1;
            self.score.recalculate();
        }
    }

    /// Check if a tile has been explored.
    pub fn has_explored(&self, coord: &HexCoord) -> bool {
        self.explored_tiles.contains(coord)
    }

    /// Calculate the total number of explored tiles.
    pub fn explored_count(&self) -> usize {
        self.explored_tiles.len()
    }

    /// Eliminate this player from the game.
    pub fn eliminate(&mut self) {
        self.eliminated = true;
    }

    /// Check if player can afford a gold purchase.
    pub fn can_afford(&self, cost: i32) -> bool {
        self.gold >= cost
    }

    /// Spend gold (returns false if insufficient).
    pub fn spend_gold(&mut self, amount: i32) -> bool {
        if self.gold >= amount {
            self.gold -= amount;
            true
        } else {
            false
        }
    }

    /// Add gold to treasury.
    pub fn add_gold(&mut self, amount: i32) {
        self.gold += amount;
    }

    /// Add a spaceship part for science victory.
    /// Returns true if the part was successfully added, false if invalid or already built.
    pub fn add_spaceship_part(&mut self, part: &str) -> bool {
        self.spaceship.add_part(part)
    }
}

/// Score breakdown for victory calculations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Score {
    /// Score from population.
    pub population: u32,
    /// Score from owned land tiles.
    pub land: u32,
    /// Score from researched technologies.
    pub techs: u32,
    /// Score from wonders built.
    pub wonders: u32,
    /// Score from cities owned.
    pub cities: u32,
    /// Total computed score.
    pub total: u32,
}

impl Score {
    /// Recalculate total score from components.
    pub fn recalculate(&mut self) {
        // Scoring weights (can be tuned for balance)
        self.total =
            self.population * 3 + self.land + self.techs * 4 + self.wonders * 25 + self.cities * 10;
    }

    /// Create a score from components.
    pub fn new(population: u32, land: u32, techs: u32, wonders: u32, cities: u32) -> Self {
        let mut score = Self {
            population,
            land,
            techs,
            wonders,
            cities,
            total: 0,
        };
        score.recalculate();
        score
    }
}

/// A civilization with unique abilities.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Civilization {
    /// Unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Leader's name.
    pub leader_name: String,
    /// Unique ability description.
    pub ability_name: String,
    /// Detailed ability description.
    pub ability_description: String,
    /// Starting era bonus (if any).
    pub starting_era: Era,
}

impl Civilization {
    /// Create a generic/random civilization.
    pub fn generic() -> Self {
        Self {
            id: "generic".to_string(),
            name: "Generic Nation".to_string(),
            leader_name: "Leader".to_string(),
            ability_name: "Balanced".to_string(),
            ability_description: "No special bonuses or penalties.".to_string(),
            starting_era: Era::Ancient,
        }
    }

    /// Create the Rome civilization.
    pub fn rome() -> Self {
        Self {
            id: "rome".to_string(),
            name: "Rome".to_string(),
            leader_name: "Augustus Caesar".to_string(),
            ability_name: "Glory of Rome".to_string(),
            ability_description:
                "+25% production towards buildings that already exist in the Capital.".to_string(),
            starting_era: Era::Ancient,
        }
    }

    /// Create the Egypt civilization.
    pub fn egypt() -> Self {
        Self {
            id: "egypt".to_string(),
            name: "Egypt".to_string(),
            leader_name: "Ramesses II".to_string(),
            ability_name: "Monument Builders".to_string(),
            ability_description: "+20% production towards Wonders.".to_string(),
            starting_era: Era::Ancient,
        }
    }

    /// Create the Greece civilization.
    pub fn greece() -> Self {
        Self {
            id: "greece".to_string(),
            name: "Greece".to_string(),
            leader_name: "Alexander".to_string(),
            ability_name: "Hellenic League".to_string(),
            ability_description: "City-state influence degrades half as fast.".to_string(),
            starting_era: Era::Ancient,
        }
    }

    /// Create the China civilization.
    pub fn china() -> Self {
        Self {
            id: "china".to_string(),
            name: "China".to_string(),
            leader_name: "Wu Zetian".to_string(),
            ability_name: "Art of War".to_string(),
            ability_description: "Great General spawn rate increased by 50%.".to_string(),
            starting_era: Era::Ancient,
        }
    }

    /// Get all predefined civilizations.
    pub fn all_civilizations() -> Vec<Civilization> {
        vec![
            Self::rome(),
            Self::egypt(),
            Self::greece(),
            Self::china(),
            Self::generic(),
        ]
    }
}

impl Default for Civilization {
    fn default() -> Self {
        Self::generic()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_creation() {
        let player = Player::new(
            0,
            "npub1abc123".to_string(),
            "TestPlayer".to_string(),
            Civilization::rome(),
        );
        assert_eq!(player.id, 0);
        assert_eq!(player.name, "TestPlayer");
        assert!(!player.eliminated);
        assert_eq!(player.gold, 0);
    }

    #[test]
    fn test_player_gold() {
        let mut player = Player::new(
            0,
            "npub".to_string(),
            "P1".to_string(),
            Civilization::generic(),
        );
        player.add_gold(100);
        assert_eq!(player.gold, 100);
        assert!(player.can_afford(50));
        assert!(player.spend_gold(30));
        assert_eq!(player.gold, 70);
        assert!(!player.spend_gold(100)); // Can't afford
        assert_eq!(player.gold, 70); // Unchanged
    }

    #[test]
    fn test_player_tech() {
        let mut player = Player::new(
            0,
            "npub".to_string(),
            "P1".to_string(),
            Civilization::generic(),
        );
        assert!(!player.has_tech(&"mining".to_string()));
        player.add_tech("mining".to_string());
        assert!(player.has_tech(&"mining".to_string()));
        assert_eq!(player.score.techs, 1);
    }

    #[test]
    fn test_player_exploration() {
        let mut player = Player::new(
            0,
            "npub".to_string(),
            "P1".to_string(),
            Civilization::generic(),
        );
        let coord = HexCoord::new(5, 5);
        assert!(!player.has_explored(&coord));
        player.explore_tile(coord);
        assert!(player.has_explored(&coord));
        assert_eq!(player.explored_count(), 1);
    }

    #[test]
    fn test_score_calculation() {
        let score = Score::new(10, 50, 5, 2, 3);
        // 10*3 + 50 + 5*4 + 2*25 + 3*10 = 30 + 50 + 20 + 50 + 30 = 180
        assert_eq!(score.total, 180);
    }

    #[test]
    fn test_civilizations() {
        let civs = Civilization::all_civilizations();
        assert!(civs.len() >= 4);
        assert!(civs.iter().any(|c| c.id == "rome"));
    }

    #[test]
    fn test_player_serialization() {
        let player = Player::new(
            0,
            "npub1test".to_string(),
            "TestPlayer".to_string(),
            Civilization::rome(),
        );
        let json = serde_json::to_string(&player).unwrap();
        let restored: Player = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, player.id);
        assert_eq!(restored.name, player.name);
        assert_eq!(restored.civilization.id, player.civilization.id);
    }
}

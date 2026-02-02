//! Victory condition checking for all victory types.

use crate::game_state::GameState;
use crate::types::{PlayerId, VictoryType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks progress toward science victory via spaceship construction.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpaceshipProgress {
    /// Cockpit module completed.
    pub cockpit: bool,
    /// Fuel tanks module completed.
    pub fuel_tanks: bool,
    /// Thrusters module completed.
    pub thrusters: bool,
    /// Life support module completed.
    pub life_support: bool,
    /// Stasis chamber module completed.
    pub stasis_chamber: bool,
}

impl SpaceshipProgress {
    /// Check if all spaceship parts have been completed.
    pub fn is_complete(&self) -> bool {
        self.cockpit
            && self.fuel_tanks
            && self.thrusters
            && self.life_support
            && self.stasis_chamber
    }

    /// Count the number of completed parts.
    pub fn parts_completed(&self) -> u32 {
        let mut count = 0;
        if self.cockpit {
            count += 1;
        }
        if self.fuel_tanks {
            count += 1;
        }
        if self.thrusters {
            count += 1;
        }
        if self.life_support {
            count += 1;
        }
        if self.stasis_chamber {
            count += 1;
        }
        count
    }

    /// Get the total number of spaceship parts required.
    pub const fn total_parts() -> u32 {
        5
    }

    /// Add a spaceship part by name. Returns true if the part was added successfully.
    pub fn add_part(&mut self, part: &str) -> bool {
        match part.to_lowercase().as_str() {
            "cockpit" => {
                if self.cockpit {
                    return false;
                }
                self.cockpit = true;
                true
            }
            "fuel_tanks" | "fuel tanks" | "fueltanks" => {
                if self.fuel_tanks {
                    return false;
                }
                self.fuel_tanks = true;
                true
            }
            "thrusters" => {
                if self.thrusters {
                    return false;
                }
                self.thrusters = true;
                true
            }
            "life_support" | "life support" | "lifesupport" => {
                if self.life_support {
                    return false;
                }
                self.life_support = true;
                true
            }
            "stasis_chamber" | "stasis chamber" | "stasischamber" => {
                if self.stasis_chamber {
                    return false;
                }
                self.stasis_chamber = true;
                true
            }
            _ => false,
        }
    }
}

/// Configuration and methods for checking victory conditions.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VictoryChecker {
    /// Whether domination victory is enabled (control all original capitals).
    pub domination_enabled: bool,
    /// Whether science victory is enabled (complete spaceship).
    pub science_enabled: bool,
    /// Whether economic victory is enabled (accumulate gold threshold).
    pub economic_enabled: bool,
    /// Whether diplomatic victory is enabled (win UN vote).
    pub diplomatic_enabled: bool,
    /// Whether score victory is enabled (highest score at turn limit).
    pub score_enabled: bool,
    /// Gold needed for economic victory.
    pub economic_threshold: i32,
    /// Turn limit for score victory (None means no limit).
    pub turn_limit: Option<u32>,
}

impl Default for VictoryChecker {
    fn default() -> Self {
        Self {
            domination_enabled: true,
            science_enabled: true,
            economic_enabled: true,
            diplomatic_enabled: true,
            score_enabled: true,
            economic_threshold: 20_000,
            turn_limit: Some(500),
        }
    }
}

impl VictoryChecker {
    /// Create a new VictoryChecker with all victories enabled.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a VictoryChecker with custom settings.
    pub fn with_settings(
        domination: bool,
        science: bool,
        economic: bool,
        diplomatic: bool,
        score: bool,
        economic_threshold: i32,
        turn_limit: Option<u32>,
    ) -> Self {
        Self {
            domination_enabled: domination,
            science_enabled: science,
            economic_enabled: economic,
            diplomatic_enabled: diplomatic,
            score_enabled: score,
            economic_threshold,
            turn_limit,
        }
    }

    /// Check for domination victory: a player controls all original capitals.
    ///
    /// Returns the player ID if they control all capitals that were originally
    /// assigned to players at the start of the game.
    pub fn check_domination(game: &GameState) -> Option<PlayerId> {
        // Get all original capitals (cities that are marked as capitals)
        let original_capitals: Vec<_> = game.cities.values().filter(|c| c.is_capital).collect();

        // If there are no capitals, no domination victory possible
        if original_capitals.is_empty() {
            return None;
        }

        // Check if any single player owns all capitals
        let active_players: Vec<PlayerId> = game
            .players
            .iter()
            .filter(|p| !p.eliminated)
            .map(|p| p.id)
            .collect();

        for player_id in active_players {
            let owns_all = original_capitals.iter().all(|c| c.owner == player_id);
            if owns_all {
                return Some(player_id);
            }
        }

        None
    }

    /// Check for science victory: a player has completed all spaceship parts.
    pub fn check_science(game: &GameState) -> Option<PlayerId> {
        for player in &game.players {
            if !player.eliminated && player.spaceship.is_complete() {
                return Some(player.id);
            }
        }
        None
    }

    /// Check for economic victory: a player has accumulated enough gold.
    pub fn check_economic(game: &GameState, threshold: i32) -> Option<PlayerId> {
        for player in &game.players {
            if !player.eliminated && player.gold >= threshold {
                return Some(player.id);
            }
        }
        None
    }

    /// Check for diplomatic victory based on UN vote results.
    ///
    /// The votes map contains voter -> candidate mappings.
    /// A player wins if they receive more than half of the total votes.
    pub fn check_diplomatic_votes(votes: &HashMap<PlayerId, PlayerId>) -> Option<PlayerId> {
        if votes.is_empty() {
            return None;
        }

        // Count votes for each candidate
        let mut vote_counts: HashMap<PlayerId, u32> = HashMap::new();
        for &candidate in votes.values() {
            *vote_counts.entry(candidate).or_insert(0) += 1;
        }

        // Find majority winner (more than half of voters)
        let total_voters = votes.len() as u32;
        let majority_needed = total_voters / 2 + 1;

        for (candidate, count) in vote_counts {
            if count >= majority_needed {
                return Some(candidate);
            }
        }

        None
    }

    /// Check for score victory: highest score when turn limit is reached.
    ///
    /// Returns the player with the highest score among non-eliminated players.
    pub fn check_score_victory(game: &GameState) -> Option<PlayerId> {
        game.players
            .iter()
            .filter(|p| !p.eliminated)
            .max_by_key(|p| p.score.total)
            .map(|p| p.id)
    }

    /// Check all enabled victory conditions and return the winner if any.
    ///
    /// Checks victories in order of priority:
    /// 1. Domination (immediate when achieved)
    /// 2. Science (immediate when achieved)
    /// 3. Economic (immediate when achieved)
    /// 4. Score (only at turn limit)
    ///
    /// Note: Diplomatic victory requires external vote data and should be
    /// checked separately using `check_diplomatic_votes`.
    pub fn check_all(&self, game: &GameState) -> Option<(PlayerId, VictoryType)> {
        // Check domination victory
        if self.domination_enabled {
            if let Some(winner) = Self::check_domination(game) {
                return Some((winner, VictoryType::Domination));
            }
        }

        // Check science victory
        if self.science_enabled {
            if let Some(winner) = Self::check_science(game) {
                return Some((winner, VictoryType::Science));
            }
        }

        // Check economic victory
        if self.economic_enabled {
            if let Some(winner) = Self::check_economic(game, self.economic_threshold) {
                return Some((winner, VictoryType::Economic));
            }
        }

        // Check score victory (only if turn limit reached)
        if self.score_enabled {
            if let Some(limit) = self.turn_limit {
                if game.turn >= limit {
                    if let Some(winner) = Self::check_score_victory(game) {
                        return Some((winner, VictoryType::Score));
                    }
                }
            }
        }

        None
    }

    /// Check diplomatic victory with provided votes.
    ///
    /// This is separate from `check_all` because diplomatic votes come from
    /// an external source (player voting).
    pub fn check_diplomatic(
        &self,
        votes: &HashMap<PlayerId, PlayerId>,
    ) -> Option<(PlayerId, VictoryType)> {
        if !self.diplomatic_enabled {
            return None;
        }

        Self::check_diplomatic_votes(votes).map(|winner| (winner, VictoryType::Diplomatic))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::city::City;
    use crate::hex::HexCoord;
    use crate::player::{Civilization, Player};
    use crate::settings::GameSettings;

    fn create_test_game() -> GameState {
        let settings = GameSettings::new("Test".to_string());
        GameState::new("game1".to_string(), settings, [0u8; 32])
    }

    fn create_test_player(id: PlayerId, name: &str) -> Player {
        Player::new(
            id,
            format!("npub{}", id),
            name.to_string(),
            Civilization::generic(),
        )
    }

    // ==================== SpaceshipProgress Tests ====================

    #[test]
    fn test_spaceship_default() {
        let progress = SpaceshipProgress::default();
        assert!(!progress.is_complete());
        assert_eq!(progress.parts_completed(), 0);
        assert_eq!(SpaceshipProgress::total_parts(), 5);
    }

    #[test]
    fn test_spaceship_add_parts() {
        let mut progress = SpaceshipProgress::default();

        assert!(progress.add_part("cockpit"));
        assert_eq!(progress.parts_completed(), 1);
        assert!(!progress.is_complete());

        assert!(progress.add_part("fuel_tanks"));
        assert!(progress.add_part("thrusters"));
        assert!(progress.add_part("life_support"));
        assert_eq!(progress.parts_completed(), 4);
        assert!(!progress.is_complete());

        assert!(progress.add_part("stasis_chamber"));
        assert_eq!(progress.parts_completed(), 5);
        assert!(progress.is_complete());
    }

    #[test]
    fn test_spaceship_duplicate_part() {
        let mut progress = SpaceshipProgress::default();

        assert!(progress.add_part("cockpit"));
        assert!(!progress.add_part("cockpit")); // Duplicate should fail
        assert_eq!(progress.parts_completed(), 1);
    }

    #[test]
    fn test_spaceship_invalid_part() {
        let mut progress = SpaceshipProgress::default();

        assert!(!progress.add_part("invalid_part"));
        assert_eq!(progress.parts_completed(), 0);
    }

    #[test]
    fn test_spaceship_part_name_variants() {
        let mut progress = SpaceshipProgress::default();

        // Test various name formats
        assert!(progress.add_part("COCKPIT")); // uppercase
        assert!(progress.add_part("fuel tanks")); // space separator
        assert!(progress.add_part("lifesupport")); // no separator
        assert_eq!(progress.parts_completed(), 3);
    }

    #[test]
    fn test_spaceship_serialization() {
        let progress = SpaceshipProgress {
            cockpit: true,
            thrusters: true,
            ..Default::default()
        };

        let json = serde_json::to_string(&progress).unwrap();
        let restored: SpaceshipProgress = serde_json::from_str(&json).unwrap();

        assert_eq!(restored, progress);
        assert_eq!(restored.parts_completed(), 2);
    }

    // ==================== VictoryChecker Tests ====================

    #[test]
    fn test_victory_checker_default() {
        let checker = VictoryChecker::default();
        assert!(checker.domination_enabled);
        assert!(checker.science_enabled);
        assert!(checker.economic_enabled);
        assert!(checker.diplomatic_enabled);
        assert!(checker.score_enabled);
        assert_eq!(checker.economic_threshold, 20_000);
        assert_eq!(checker.turn_limit, Some(500));
    }

    #[test]
    fn test_victory_checker_with_settings() {
        let checker =
            VictoryChecker::with_settings(true, false, true, false, true, 10_000, Some(300));

        assert!(checker.domination_enabled);
        assert!(!checker.science_enabled);
        assert!(checker.economic_enabled);
        assert!(!checker.diplomatic_enabled);
        assert!(checker.score_enabled);
        assert_eq!(checker.economic_threshold, 10_000);
        assert_eq!(checker.turn_limit, Some(300));
    }

    // ==================== Domination Victory Tests ====================

    #[test]
    fn test_domination_no_capitals() {
        let game = create_test_game();
        assert!(VictoryChecker::check_domination(&game).is_none());
    }

    #[test]
    fn test_domination_single_player_owns_all() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        let mut p2 = create_test_player(1, "P2");
        p1.capital = Some(1);
        p2.capital = Some(2);
        game.players.push(p1);
        game.players.push(p2);

        // Player 0 owns both capitals
        let capital1 = City::new(1, 0, "Capital1".to_string(), HexCoord::new(0, 0), true);
        let capital2 = City::new(2, 0, "Capital2".to_string(), HexCoord::new(5, 5), true);
        game.cities.insert(1, capital1);
        game.cities.insert(2, capital2);

        assert_eq!(VictoryChecker::check_domination(&game), Some(0));
    }

    #[test]
    fn test_domination_no_winner_capitals_split() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        let mut p2 = create_test_player(1, "P2");
        p1.capital = Some(1);
        p2.capital = Some(2);
        game.players.push(p1);
        game.players.push(p2);

        // Each player owns their own capital
        let capital1 = City::new(1, 0, "Capital1".to_string(), HexCoord::new(0, 0), true);
        let capital2 = City::new(2, 1, "Capital2".to_string(), HexCoord::new(5, 5), true);
        game.cities.insert(1, capital1);
        game.cities.insert(2, capital2);

        assert!(VictoryChecker::check_domination(&game).is_none());
    }

    #[test]
    fn test_domination_eliminated_player_ignored() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        let mut p2 = create_test_player(1, "P2");
        p1.capital = Some(1);
        p2.capital = Some(2);
        p2.eliminated = true;
        game.players.push(p1);
        game.players.push(p2);

        // Player 0 owns both capitals (P2 is eliminated)
        let capital1 = City::new(1, 0, "Capital1".to_string(), HexCoord::new(0, 0), true);
        let capital2 = City::new(2, 0, "Capital2".to_string(), HexCoord::new(5, 5), true);
        game.cities.insert(1, capital1);
        game.cities.insert(2, capital2);

        assert_eq!(VictoryChecker::check_domination(&game), Some(0));
    }

    // ==================== Science Victory Tests ====================

    #[test]
    fn test_science_no_winner() {
        let mut game = create_test_game();
        game.players.push(create_test_player(0, "P1"));
        game.players.push(create_test_player(1, "P2"));

        assert!(VictoryChecker::check_science(&game).is_none());
    }

    #[test]
    fn test_science_winner() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.spaceship = SpaceshipProgress {
            cockpit: true,
            fuel_tanks: true,
            thrusters: true,
            life_support: true,
            stasis_chamber: true,
        };
        game.players.push(p1);
        game.players.push(create_test_player(1, "P2"));

        assert_eq!(VictoryChecker::check_science(&game), Some(0));
    }

    #[test]
    fn test_science_partial_progress_no_winner() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.spaceship = SpaceshipProgress {
            cockpit: true,
            fuel_tanks: true,
            thrusters: true,
            life_support: true,
            stasis_chamber: false, // Missing one part
        };
        game.players.push(p1);

        assert!(VictoryChecker::check_science(&game).is_none());
    }

    #[test]
    fn test_science_eliminated_player_ignored() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.eliminated = true;
        p1.spaceship = SpaceshipProgress {
            cockpit: true,
            fuel_tanks: true,
            thrusters: true,
            life_support: true,
            stasis_chamber: true,
        };
        game.players.push(p1);
        game.players.push(create_test_player(1, "P2"));

        assert!(VictoryChecker::check_science(&game).is_none());
    }

    // ==================== Economic Victory Tests ====================

    #[test]
    fn test_economic_no_winner() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.gold = 5000;
        game.players.push(p1);

        assert!(VictoryChecker::check_economic(&game, 20_000).is_none());
    }

    #[test]
    fn test_economic_winner() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.gold = 25_000;
        game.players.push(p1);
        game.players.push(create_test_player(1, "P2"));

        assert_eq!(VictoryChecker::check_economic(&game, 20_000), Some(0));
    }

    #[test]
    fn test_economic_exact_threshold() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.gold = 20_000;
        game.players.push(p1);

        assert_eq!(VictoryChecker::check_economic(&game, 20_000), Some(0));
    }

    #[test]
    fn test_economic_eliminated_player_ignored() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.gold = 30_000;
        p1.eliminated = true;
        game.players.push(p1);
        game.players.push(create_test_player(1, "P2"));

        assert!(VictoryChecker::check_economic(&game, 20_000).is_none());
    }

    // ==================== Diplomatic Victory Tests ====================

    #[test]
    fn test_diplomatic_no_votes() {
        let votes: HashMap<PlayerId, PlayerId> = HashMap::new();
        assert!(VictoryChecker::check_diplomatic_votes(&votes).is_none());
    }

    #[test]
    fn test_diplomatic_majority_winner() {
        let mut votes: HashMap<PlayerId, PlayerId> = HashMap::new();
        votes.insert(0, 1); // Player 0 votes for Player 1
        votes.insert(1, 1); // Player 1 votes for Player 1
        votes.insert(2, 0); // Player 2 votes for Player 0

        // 2 out of 3 votes for Player 1 (majority)
        assert_eq!(VictoryChecker::check_diplomatic_votes(&votes), Some(1));
    }

    #[test]
    fn test_diplomatic_no_majority() {
        let mut votes: HashMap<PlayerId, PlayerId> = HashMap::new();
        votes.insert(0, 0); // Player 0 votes for themselves
        votes.insert(1, 1); // Player 1 votes for themselves
        votes.insert(2, 2); // Player 2 votes for themselves

        // No majority
        assert!(VictoryChecker::check_diplomatic_votes(&votes).is_none());
    }

    #[test]
    fn test_diplomatic_two_player_majority() {
        let mut votes: HashMap<PlayerId, PlayerId> = HashMap::new();
        votes.insert(0, 1);
        votes.insert(1, 1);

        // 2 out of 2 votes for Player 1
        assert_eq!(VictoryChecker::check_diplomatic_votes(&votes), Some(1));
    }

    #[test]
    fn test_diplomatic_four_player_needs_three() {
        let mut votes: HashMap<PlayerId, PlayerId> = HashMap::new();
        votes.insert(0, 1);
        votes.insert(1, 1);
        votes.insert(2, 0);
        votes.insert(3, 0);

        // 2 vs 2, no majority (need 3)
        assert!(VictoryChecker::check_diplomatic_votes(&votes).is_none());

        votes.insert(3, 1); // Change vote to create majority

        // 3 out of 4 for Player 1
        assert_eq!(VictoryChecker::check_diplomatic_votes(&votes), Some(1));
    }

    // ==================== Score Victory Tests ====================

    #[test]
    fn test_score_single_player() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.score.total = 100;
        game.players.push(p1);

        assert_eq!(VictoryChecker::check_score_victory(&game), Some(0));
    }

    #[test]
    fn test_score_highest_wins() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        let mut p2 = create_test_player(1, "P2");
        let mut p3 = create_test_player(2, "P3");
        p1.score.total = 100;
        p2.score.total = 250;
        p3.score.total = 150;
        game.players.push(p1);
        game.players.push(p2);
        game.players.push(p3);

        assert_eq!(VictoryChecker::check_score_victory(&game), Some(1));
    }

    #[test]
    fn test_score_eliminated_ignored() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        let mut p2 = create_test_player(1, "P2");
        p1.score.total = 500;
        p1.eliminated = true;
        p2.score.total = 100;
        game.players.push(p1);
        game.players.push(p2);

        assert_eq!(VictoryChecker::check_score_victory(&game), Some(1));
    }

    // ==================== Check All Victory Tests ====================

    #[test]
    fn test_check_all_no_winner() {
        let mut game = create_test_game();
        game.players.push(create_test_player(0, "P1"));
        game.players.push(create_test_player(1, "P2"));
        game.turn = 1;

        let checker = VictoryChecker::default();
        assert!(checker.check_all(&game).is_none());
    }

    #[test]
    fn test_check_all_domination_priority() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.gold = 30_000; // Also has economic victory
        p1.spaceship = SpaceshipProgress {
            cockpit: true,
            fuel_tanks: true,
            thrusters: true,
            life_support: true,
            stasis_chamber: true,
        }; // Also has science victory
        p1.capital = Some(1);
        game.players.push(p1);

        let mut p2 = create_test_player(1, "P2");
        p2.capital = Some(2);
        game.players.push(p2);

        // P1 owns both capitals
        let capital1 = City::new(1, 0, "Capital1".to_string(), HexCoord::new(0, 0), true);
        let capital2 = City::new(2, 0, "Capital2".to_string(), HexCoord::new(5, 5), true);
        game.cities.insert(1, capital1);
        game.cities.insert(2, capital2);

        let checker = VictoryChecker::default();
        let result = checker.check_all(&game);

        // Domination should be checked first
        assert_eq!(result, Some((0, VictoryType::Domination)));
    }

    #[test]
    fn test_check_all_science_without_domination() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.spaceship = SpaceshipProgress {
            cockpit: true,
            fuel_tanks: true,
            thrusters: true,
            life_support: true,
            stasis_chamber: true,
        };
        game.players.push(p1);
        game.players.push(create_test_player(1, "P2"));

        let checker = VictoryChecker::default();
        let result = checker.check_all(&game);

        assert_eq!(result, Some((0, VictoryType::Science)));
    }

    #[test]
    fn test_check_all_score_at_turn_limit() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        let mut p2 = create_test_player(1, "P2");
        p1.score.total = 100;
        p2.score.total = 200;
        game.players.push(p1);
        game.players.push(p2);
        game.turn = 500; // At turn limit

        let checker = VictoryChecker::default();
        let result = checker.check_all(&game);

        assert_eq!(result, Some((1, VictoryType::Score)));
    }

    #[test]
    fn test_check_all_score_before_turn_limit() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        let mut p2 = create_test_player(1, "P2");
        p1.score.total = 100;
        p2.score.total = 200;
        game.players.push(p1);
        game.players.push(p2);
        game.turn = 100; // Before turn limit

        let checker = VictoryChecker::default();
        let result = checker.check_all(&game);

        // Score victory not triggered before turn limit
        assert!(result.is_none());
    }

    #[test]
    fn test_check_all_disabled_victories() {
        let mut game = create_test_game();
        let mut p1 = create_test_player(0, "P1");
        p1.gold = 30_000;
        game.players.push(p1);
        game.players.push(create_test_player(1, "P2"));

        let checker = VictoryChecker::with_settings(
            true,
            true,
            false, // Economic disabled
            true,
            true,
            20_000,
            Some(500),
        );
        let result = checker.check_all(&game);

        // Economic victory should not trigger even though gold threshold met
        assert!(result.is_none());
    }

    #[test]
    fn test_check_diplomatic_method() {
        let checker = VictoryChecker::default();
        let mut votes: HashMap<PlayerId, PlayerId> = HashMap::new();
        votes.insert(0, 1);
        votes.insert(1, 1);
        votes.insert(2, 0);

        let result = checker.check_diplomatic(&votes);
        assert_eq!(result, Some((1, VictoryType::Diplomatic)));
    }

    #[test]
    fn test_check_diplomatic_disabled() {
        let checker = VictoryChecker::with_settings(
            true,
            true,
            true,
            false, // Diplomatic disabled
            true,
            20_000,
            Some(500),
        );
        let mut votes: HashMap<PlayerId, PlayerId> = HashMap::new();
        votes.insert(0, 1);
        votes.insert(1, 1);

        let result = checker.check_diplomatic(&votes);
        assert!(result.is_none());
    }

    #[test]
    fn test_victory_checker_serialization() {
        let checker =
            VictoryChecker::with_settings(true, false, true, false, true, 15_000, Some(400));

        let json = serde_json::to_string(&checker).unwrap();
        let restored: VictoryChecker = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.domination_enabled, checker.domination_enabled);
        assert_eq!(restored.science_enabled, checker.science_enabled);
        assert_eq!(restored.economic_enabled, checker.economic_enabled);
        assert_eq!(restored.diplomatic_enabled, checker.diplomatic_enabled);
        assert_eq!(restored.score_enabled, checker.score_enabled);
        assert_eq!(restored.economic_threshold, checker.economic_threshold);
        assert_eq!(restored.turn_limit, checker.turn_limit);
    }
}

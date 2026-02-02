//! Unit system - military and civilian units.

use crate::hex::HexCoord;
use crate::types::{Era, PlayerId, UnitId};
use serde::{Deserialize, Serialize};

/// A unit on the game map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    /// Unique identifier.
    pub id: UnitId,
    /// Owning player.
    pub owner: PlayerId,
    /// Type of unit.
    pub unit_type: UnitType,
    /// Current position on the map.
    pub position: HexCoord,
    /// Current health (0-100).
    pub health: u32,
    /// Remaining movement points this turn (x10 for precision).
    pub movement: u32,
    /// Experience points earned.
    pub experience: u32,
    /// Promotions this unit has.
    pub promotions: Vec<Promotion>,
    /// Is the unit fortified?
    pub fortified: bool,
    /// Turns spent fortifying (0-2, affects defense bonus).
    pub fortify_turns: u32,
    /// Is the unit embarked on water?
    pub embarked: bool,
    /// Has the unit used its action this turn?
    pub has_acted: bool,
    /// Is the unit sleeping (skip until enemy in sight)?
    pub sleeping: bool,
    /// Queued orders/path.
    pub queued_path: Option<Vec<HexCoord>>,
}

impl Unit {
    /// Create a new unit.
    pub fn new(id: UnitId, owner: PlayerId, unit_type: UnitType, position: HexCoord) -> Self {
        let stats = unit_type.stats();
        Self {
            id,
            owner,
            unit_type,
            position,
            health: 100,
            movement: stats.movement * 10, // x10 for fractional movement
            experience: 0,
            promotions: Vec::new(),
            fortified: false,
            fortify_turns: 0,
            embarked: false,
            has_acted: false,
            sleeping: false,
            queued_path: None,
        }
    }

    /// Get the unit's stats (base + promotions).
    pub fn effective_stats(&self) -> UnitStats {
        let mut stats = self.unit_type.stats();

        // Apply promotion bonuses
        for promo in &self.promotions {
            stats = promo.apply(stats);
        }

        stats
    }

    /// Get effective combat strength (includes health penalty).
    pub fn effective_combat_strength(&self) -> u32 {
        let base = self.effective_stats().combat_strength;
        if base == 0 {
            return 0;
        }
        // Combat strength scales with health
        (base * self.health / 100).max(1)
    }

    /// Get effective ranged strength.
    pub fn effective_ranged_strength(&self) -> u32 {
        let base = self.effective_stats().ranged_strength;
        if base == 0 {
            return 0;
        }
        (base * self.health / 100).max(1)
    }

    /// Check if unit can move.
    pub fn can_move(&self) -> bool {
        self.movement > 0 && !self.has_acted
    }

    /// Check if unit can attack.
    pub fn can_attack(&self) -> bool {
        !self.has_acted && self.effective_combat_strength() > 0
    }

    /// Check if unit is ranged.
    pub fn is_ranged(&self) -> bool {
        self.unit_type.stats().range > 0
    }

    /// Get attack range.
    pub fn range(&self) -> u32 {
        self.effective_stats().range
    }

    /// Use movement points.
    pub fn use_movement(&mut self, cost: u32) {
        self.movement = self.movement.saturating_sub(cost);
        // Unfortify when moving
        if cost > 0 {
            self.fortified = false;
            self.fortify_turns = 0;
        }
    }

    /// Mark as having acted this turn.
    pub fn mark_acted(&mut self) {
        self.has_acted = true;
    }

    /// Take damage.
    pub fn take_damage(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
    }

    /// Heal the unit.
    pub fn heal(&mut self, amount: u32) {
        self.health = (self.health + amount).min(100);
    }

    /// Check if unit is dead.
    pub fn is_dead(&self) -> bool {
        self.health == 0
    }

    /// Gain experience.
    pub fn gain_experience(&mut self, xp: u32) {
        self.experience += xp;
    }

    /// Check if unit can earn a promotion.
    pub fn can_promote(&self) -> bool {
        let threshold = self.next_promotion_threshold();
        self.experience >= threshold
    }

    /// Get XP needed for next promotion.
    fn next_promotion_threshold(&self) -> u32 {
        // Each promotion requires more XP
        let promo_count = self.promotions.len() as u32;
        10 + promo_count * 10
    }

    /// Add a promotion.
    pub fn add_promotion(&mut self, promotion: Promotion) {
        self.promotions.push(promotion);
    }

    /// Fortify the unit.
    pub fn fortify(&mut self) {
        self.fortified = true;
        self.fortify_turns = 0;
        self.has_acted = true;
    }

    /// Get defense bonus from fortification.
    pub fn fortification_bonus(&self) -> i32 {
        if !self.fortified {
            return 0;
        }
        // 25% per turn, max 50%
        (self.fortify_turns.min(2) * 25) as i32
    }

    /// Reset for new turn.
    pub fn new_turn(&mut self) {
        let stats = self.effective_stats();
        self.movement = stats.movement * 10;
        self.has_acted = false;

        // Increase fortification bonus
        if self.fortified && self.fortify_turns < 2 {
            self.fortify_turns += 1;
        }

        // Heal if fortified or in friendly territory
        if self.fortified {
            self.heal(10);
        }
    }

    /// Check if this is a civilian unit.
    pub fn is_civilian(&self) -> bool {
        self.unit_type.stats().category == UnitCategory::Civilian
    }

    /// Check if this is a military unit.
    pub fn is_military(&self) -> bool {
        !self.is_civilian()
    }
}

/// Types of units available.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitType {
    // Civilian
    Settler,
    Worker,
    GreatScientist,
    GreatEngineer,
    GreatMerchant,
    GreatArtist,
    GreatGeneral,

    // Ancient Military
    Warrior,
    Archer,
    Spearman,
    Slinger,
    Chariot,
    Galley,
    Catapult,

    // Classical Military
    Swordsman,
    CompositeBow,
    Horseman,
    Pikeman,
    Trireme,
    Ballista,

    // Medieval Military
    Longswordsman,
    Crossbow,
    Knight,
    Halberdier,
    Caravel,
    Trebuchet,

    // Renaissance Military
    Musketman,
    Lancer,
    Frigate,
    Cannon,

    // Industrial Military
    Rifleman,
    GatlingGun,
    Cavalry,
    Ironclad,
    Artillery,

    // Modern Military
    Infantry,
    MachineGun,
    Tank,
    Battleship,
    RocketArtillery,
    Fighter,
    Bomber,
}

impl UnitType {
    /// Get the stats for this unit type.
    pub const fn stats(&self) -> UnitStats {
        match self {
            // Civilians
            UnitType::Settler => UnitStats::civilian(2, 106),
            UnitType::Worker => UnitStats::civilian(2, 70),
            UnitType::GreatScientist => UnitStats::civilian(2, 0),
            UnitType::GreatEngineer => UnitStats::civilian(2, 0),
            UnitType::GreatMerchant => UnitStats::civilian(2, 0),
            UnitType::GreatArtist => UnitStats::civilian(2, 0),
            UnitType::GreatGeneral => UnitStats::civilian(2, 0),

            // Ancient
            UnitType::Warrior => UnitStats::melee(8, 2, 40),
            UnitType::Archer => UnitStats::ranged(5, 7, 2, 2, 40),
            UnitType::Spearman => UnitStats::melee(11, 2, 56),
            UnitType::Slinger => UnitStats::ranged(4, 5, 1, 2, 30),
            UnitType::Chariot => UnitStats::cavalry(6, 4, 56),
            UnitType::Galley => UnitStats::naval(8, 0, 0, 3, 50),
            UnitType::Catapult => UnitStats::siege(4, 14, 2, 2, 75),

            // Classical
            UnitType::Swordsman => UnitStats::melee(14, 2, 75),
            UnitType::CompositeBow => UnitStats::ranged(7, 11, 2, 2, 75),
            UnitType::Horseman => UnitStats::cavalry(12, 4, 75),
            UnitType::Pikeman => UnitStats::melee(16, 2, 90),
            UnitType::Trireme => UnitStats::naval(10, 0, 0, 4, 75),
            UnitType::Ballista => UnitStats::siege(6, 18, 2, 2, 100),

            // Medieval
            UnitType::Longswordsman => UnitStats::melee(21, 2, 120),
            UnitType::Crossbow => UnitStats::ranged(13, 18, 2, 2, 120),
            UnitType::Knight => UnitStats::cavalry(20, 4, 120),
            UnitType::Halberdier => UnitStats::melee(25, 2, 150),
            UnitType::Caravel => UnitStats::naval(16, 0, 0, 5, 120),
            UnitType::Trebuchet => UnitStats::siege(12, 26, 2, 2, 150),

            // Renaissance
            UnitType::Musketman => UnitStats::melee(24, 2, 150),
            UnitType::Lancer => UnitStats::cavalry(25, 4, 185),
            UnitType::Frigate => UnitStats::naval(25, 28, 2, 5, 185),
            UnitType::Cannon => UnitStats::siege(14, 35, 2, 2, 185),

            // Industrial
            UnitType::Rifleman => UnitStats::melee(34, 2, 225),
            UnitType::GatlingGun => UnitStats::ranged(30, 38, 1, 2, 225),
            UnitType::Cavalry => UnitStats::cavalry(34, 5, 225),
            UnitType::Ironclad => UnitStats::naval(40, 0, 0, 4, 250),
            UnitType::Artillery => UnitStats::siege(21, 50, 3, 2, 250),

            // Modern
            UnitType::Infantry => UnitStats::melee(50, 2, 300),
            UnitType::MachineGun => UnitStats::ranged(45, 60, 1, 2, 300),
            UnitType::Tank => UnitStats::cavalry(70, 5, 375),
            UnitType::Battleship => UnitStats::naval(55, 65, 3, 5, 375),
            UnitType::RocketArtillery => UnitStats::siege(32, 80, 3, 2, 375),
            UnitType::Fighter => UnitStats::air(45, 0, 0, 8, 375),
            UnitType::Bomber => UnitStats::air(35, 65, 10, 8, 450),
        }
    }

    /// Get the era this unit belongs to.
    pub const fn era(&self) -> Era {
        match self {
            UnitType::Settler
            | UnitType::Worker
            | UnitType::Warrior
            | UnitType::Archer
            | UnitType::Spearman
            | UnitType::Slinger
            | UnitType::Chariot
            | UnitType::Galley
            | UnitType::Catapult => Era::Ancient,

            UnitType::Swordsman
            | UnitType::CompositeBow
            | UnitType::Horseman
            | UnitType::Pikeman
            | UnitType::Trireme
            | UnitType::Ballista => Era::Classical,

            UnitType::Longswordsman
            | UnitType::Crossbow
            | UnitType::Knight
            | UnitType::Halberdier
            | UnitType::Caravel
            | UnitType::Trebuchet => Era::Medieval,

            UnitType::Musketman | UnitType::Lancer | UnitType::Frigate | UnitType::Cannon => {
                Era::Renaissance
            }

            UnitType::Rifleman
            | UnitType::GatlingGun
            | UnitType::Cavalry
            | UnitType::Ironclad
            | UnitType::Artillery => Era::Industrial,

            UnitType::Infantry
            | UnitType::MachineGun
            | UnitType::Tank
            | UnitType::Battleship
            | UnitType::RocketArtillery
            | UnitType::Fighter
            | UnitType::Bomber
            | UnitType::GreatScientist
            | UnitType::GreatEngineer
            | UnitType::GreatMerchant
            | UnitType::GreatArtist
            | UnitType::GreatGeneral => Era::Modern,
        }
    }

    /// Get units available in Ancient era.
    pub fn ancient_units() -> Vec<UnitType> {
        vec![
            UnitType::Settler,
            UnitType::Worker,
            UnitType::Warrior,
            UnitType::Archer,
            UnitType::Spearman,
            UnitType::Slinger,
            UnitType::Chariot,
            UnitType::Galley,
            UnitType::Catapult,
        ]
    }
}

/// Stats for a unit type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnitStats {
    /// Melee/defense combat strength.
    pub combat_strength: u32,
    /// Ranged attack strength (0 if melee only).
    pub ranged_strength: u32,
    /// Attack range in hexes (0 for melee).
    pub range: u32,
    /// Movement points per turn.
    pub movement: u32,
    /// Production cost to build.
    pub cost: u32,
    /// Unit category.
    pub category: UnitCategory,
}

impl UnitStats {
    /// Create civilian unit stats.
    pub const fn civilian(movement: u32, cost: u32) -> Self {
        Self {
            combat_strength: 0,
            ranged_strength: 0,
            range: 0,
            movement,
            cost,
            category: UnitCategory::Civilian,
        }
    }

    /// Create melee unit stats.
    pub const fn melee(combat: u32, movement: u32, cost: u32) -> Self {
        Self {
            combat_strength: combat,
            ranged_strength: 0,
            range: 0,
            movement,
            cost,
            category: UnitCategory::Melee,
        }
    }

    /// Create ranged unit stats.
    pub const fn ranged(combat: u32, ranged: u32, range: u32, movement: u32, cost: u32) -> Self {
        Self {
            combat_strength: combat,
            ranged_strength: ranged,
            range,
            movement,
            cost,
            category: UnitCategory::Ranged,
        }
    }

    /// Create cavalry unit stats.
    pub const fn cavalry(combat: u32, movement: u32, cost: u32) -> Self {
        Self {
            combat_strength: combat,
            ranged_strength: 0,
            range: 0,
            movement,
            cost,
            category: UnitCategory::Cavalry,
        }
    }

    /// Create naval unit stats.
    pub const fn naval(combat: u32, ranged: u32, range: u32, movement: u32, cost: u32) -> Self {
        Self {
            combat_strength: combat,
            ranged_strength: ranged,
            range,
            movement,
            cost,
            category: UnitCategory::Naval,
        }
    }

    /// Create siege unit stats.
    pub const fn siege(combat: u32, ranged: u32, range: u32, movement: u32, cost: u32) -> Self {
        Self {
            combat_strength: combat,
            ranged_strength: ranged,
            range,
            movement,
            cost,
            category: UnitCategory::Siege,
        }
    }

    /// Create air unit stats.
    pub const fn air(combat: u32, ranged: u32, range: u32, movement: u32, cost: u32) -> Self {
        Self {
            combat_strength: combat,
            ranged_strength: ranged,
            range,
            movement,
            cost,
            category: UnitCategory::Air,
        }
    }
}

impl Default for UnitStats {
    fn default() -> Self {
        Self::melee(8, 2, 40) // Warrior as default
    }
}

/// Categories of units.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum UnitCategory {
    Civilian,
    Melee,
    Ranged,
    Cavalry,
    Naval,
    Siege,
    Air,
}

/// Unit promotions (upgrades earned through combat).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Promotion {
    // Melee promotions
    ShockI,
    ShockII,
    ShockIII,
    DrillI,
    DrillII,
    DrillIII,

    // Ranged promotions
    AccuracyI,
    AccuracyII,
    AccuracyIII,
    BarrageI,
    BarrageII,
    BarrageIII,

    // General promotions
    Medic,
    March,
    Blitz,
    Logistics,

    // Movement promotions
    Mobility,
    Sentry,

    // Defense promotions
    CoverI,
    CoverII,
}

impl Promotion {
    /// Apply this promotion to unit stats.
    pub const fn apply(&self, mut stats: UnitStats) -> UnitStats {
        match self {
            // Shock: +15/20/25% vs melee
            Promotion::ShockI | Promotion::ShockII | Promotion::ShockIII => {
                // Implemented in combat calculations
                stats
            }
            // Drill: +15/20/25% in rough terrain
            Promotion::DrillI | Promotion::DrillII | Promotion::DrillIII => stats,
            // Accuracy: +15/20/25% vs units
            Promotion::AccuracyI | Promotion::AccuracyII | Promotion::AccuracyIII => stats,
            // Barrage: +15/20/25% vs cities
            Promotion::BarrageI | Promotion::BarrageII | Promotion::BarrageIII => stats,
            // Medic: Heal adjacent units
            Promotion::Medic => stats,
            // March: Heal every turn even when moving
            Promotion::March => stats,
            // Blitz: Can attack twice per turn
            Promotion::Blitz => stats,
            // Logistics: Extra attack per turn (ranged)
            Promotion::Logistics => stats,
            // Mobility: +1 movement
            Promotion::Mobility => {
                stats.movement += 1;
                stats
            }
            // Sentry: +1 sight range
            Promotion::Sentry => stats,
            // Cover: +25/50% defense vs ranged
            Promotion::CoverI | Promotion::CoverII => stats,
        }
    }

    /// Get the prerequisite promotions for this one.
    pub const fn prerequisites(&self) -> &'static [Promotion] {
        match self {
            Promotion::ShockII => &[Promotion::ShockI],
            Promotion::ShockIII => &[Promotion::ShockII],
            Promotion::DrillII => &[Promotion::DrillI],
            Promotion::DrillIII => &[Promotion::DrillII],
            Promotion::AccuracyII => &[Promotion::AccuracyI],
            Promotion::AccuracyIII => &[Promotion::AccuracyII],
            Promotion::BarrageII => &[Promotion::BarrageI],
            Promotion::BarrageIII => &[Promotion::BarrageII],
            Promotion::CoverII => &[Promotion::CoverI],
            Promotion::March => &[Promotion::Medic],
            _ => &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_creation() {
        let unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(5, 5));
        assert_eq!(unit.id, 1);
        assert_eq!(unit.owner, 0);
        assert_eq!(unit.health, 100);
        assert_eq!(unit.movement, 20); // 2 * 10
    }

    #[test]
    fn test_unit_stats() {
        let warrior = UnitType::Warrior.stats();
        assert_eq!(warrior.combat_strength, 8);
        assert_eq!(warrior.movement, 2);
        assert_eq!(warrior.category, UnitCategory::Melee);

        let archer = UnitType::Archer.stats();
        assert_eq!(archer.ranged_strength, 7);
        assert_eq!(archer.range, 2);
        assert_eq!(archer.category, UnitCategory::Ranged);
    }

    #[test]
    fn test_effective_combat_strength() {
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        assert_eq!(unit.effective_combat_strength(), 8);

        unit.health = 50;
        assert_eq!(unit.effective_combat_strength(), 4); // 8 * 50 / 100
    }

    #[test]
    fn test_unit_movement() {
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        assert!(unit.can_move());

        unit.use_movement(10);
        assert_eq!(unit.movement, 10);
        assert!(unit.can_move());

        unit.use_movement(10);
        assert_eq!(unit.movement, 0);
        assert!(!unit.can_move());
    }

    #[test]
    fn test_unit_damage_and_heal() {
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));

        unit.take_damage(30);
        assert_eq!(unit.health, 70);
        assert!(!unit.is_dead());

        unit.heal(20);
        assert_eq!(unit.health, 90);

        unit.heal(50); // Can't exceed 100
        assert_eq!(unit.health, 100);
    }

    #[test]
    fn test_unit_death() {
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        unit.take_damage(100);
        assert!(unit.is_dead());
    }

    #[test]
    fn test_unit_fortification() {
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        assert_eq!(unit.fortification_bonus(), 0);

        unit.fortify();
        assert!(unit.fortified);
        assert_eq!(unit.fortification_bonus(), 0); // 0 turns

        unit.new_turn();
        assert_eq!(unit.fortification_bonus(), 25); // 1 turn

        unit.new_turn();
        assert_eq!(unit.fortification_bonus(), 50); // 2 turns (max)

        unit.new_turn();
        assert_eq!(unit.fortification_bonus(), 50); // Still 50 (capped)
    }

    #[test]
    fn test_unit_experience() {
        let mut unit = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        assert!(!unit.can_promote());

        unit.gain_experience(10);
        assert!(unit.can_promote()); // 10 XP needed for first

        unit.add_promotion(Promotion::ShockI);
        assert!(!unit.can_promote()); // Now needs 20 XP for second
    }

    #[test]
    fn test_civilian_unit() {
        let settler = Unit::new(1, 0, UnitType::Settler, HexCoord::new(0, 0));
        assert!(settler.is_civilian());
        assert!(!settler.is_military());
        assert_eq!(settler.effective_combat_strength(), 0);
    }

    #[test]
    fn test_ranged_unit() {
        let archer = Unit::new(1, 0, UnitType::Archer, HexCoord::new(0, 0));
        assert!(archer.is_ranged());
        assert_eq!(archer.range(), 2);
        assert_eq!(archer.effective_ranged_strength(), 7);
    }

    #[test]
    fn test_unit_serialization() {
        let unit = Unit::new(1, 0, UnitType::Knight, HexCoord::new(3, 7));
        let json = serde_json::to_string(&unit).unwrap();
        let restored: Unit = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, unit.id);
        assert_eq!(restored.unit_type, unit.unit_type);
        assert_eq!(restored.position, unit.position);
    }

    #[test]
    fn test_promotion_prerequisites() {
        assert!(Promotion::ShockI.prerequisites().is_empty());
        assert_eq!(Promotion::ShockII.prerequisites(), &[Promotion::ShockI]);
        assert_eq!(Promotion::ShockIII.prerequisites(), &[Promotion::ShockII]);
    }
}

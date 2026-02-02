//! City system - settlements, production, and growth.

use crate::hex::HexCoord;
use crate::types::{CityId, PlayerId};
use crate::unit::UnitType;
use crate::yields::Yields;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// A city on the game map.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct City {
    /// Unique identifier.
    pub id: CityId,
    /// Owning player.
    pub owner: PlayerId,
    /// City name.
    pub name: String,
    /// Location on the map.
    pub position: HexCoord,
    /// Current population.
    pub population: u32,
    /// Food stored toward next population.
    pub food_stored: u32,
    /// Is this the player's capital?
    pub is_capital: bool,
    /// Current city health.
    pub health: u32,
    /// Maximum city health.
    pub max_health: u32,
    /// City combat strength.
    pub combat_strength: u32,
    /// Buildings constructed in this city.
    pub buildings: HashSet<BuildingType>,
    /// Current production item.
    pub production: Option<ProductionItem>,
    /// Progress on current production (hammers).
    pub production_progress: u32,
    /// Production queue.
    pub production_queue: Vec<ProductionItem>,
    /// Tiles being worked by citizens.
    pub worked_tiles: HashSet<HexCoord>,
    /// Specialist assignments.
    pub specialists: Specialists,
    /// Tiles owned by this city (borders).
    pub territory: HashSet<HexCoord>,
    /// Culture accumulated toward border expansion.
    pub culture: u32,
    /// Turns since founding.
    pub age: u32,
    /// Was city founded or conquered?
    pub founded: bool,
}

impl City {
    /// Create a new city.
    pub fn new(
        id: CityId,
        owner: PlayerId,
        name: String,
        position: HexCoord,
        is_capital: bool,
    ) -> Self {
        let mut territory = HashSet::new();
        territory.insert(position);

        // Initial territory: adjacent tiles
        for neighbor in position.neighbors() {
            territory.insert(neighbor);
        }

        let mut worked_tiles = HashSet::new();
        worked_tiles.insert(position); // City center is always worked

        Self {
            id,
            owner,
            name,
            position,
            population: 1,
            food_stored: 0,
            is_capital,
            health: 200,
            max_health: 200,
            combat_strength: 10,
            buildings: HashSet::new(),
            production: None,
            production_progress: 0,
            production_queue: Vec::new(),
            worked_tiles,
            specialists: Specialists::default(),
            territory,
            culture: 0,
            age: 0,
            founded: true,
        }
    }

    /// Get the number of citizens available to work tiles or be specialists.
    pub fn available_citizens(&self) -> u32 {
        let assigned = self.worked_tiles.len() as u32 - 1 + self.specialists.total(); // -1 for city center
        self.population.saturating_sub(assigned)
    }

    /// Food required for next population.
    pub fn food_for_growth(&self) -> u32 {
        // Formula: 15 + 6 * (population - 1) + population^1.8
        let base = 15 + 6 * (self.population.saturating_sub(1));
        let exp = (self.population as f32).powf(1.8) as u32;
        base + exp
    }

    /// Process a turn for this city.
    pub fn process_turn(&mut self, yields: &Yields) -> CityTurnResult {
        let mut result = CityTurnResult::default();
        self.age += 1;

        // Process food/growth
        self.process_food(yields.food, &mut result);

        // Process production
        self.process_production(yields.production, &mut result);

        // Process culture
        self.culture += yields.culture as u32;
        if self.should_expand_borders() {
            result.border_expansion = true;
        }

        // Heal city if damaged
        if self.health < self.max_health {
            self.health = (self.health + 10).min(self.max_health);
        }

        result
    }

    /// Process food and population growth.
    fn process_food(&mut self, food: i32, result: &mut CityTurnResult) {
        // Calculate food consumption (2 per citizen)
        let consumption = self.population as i32 * 2;
        let surplus = food - consumption;

        if surplus >= 0 {
            self.food_stored = self.food_stored.saturating_add(surplus as u32);

            // Check for growth
            let growth_threshold = self.food_for_growth();
            if self.food_stored >= growth_threshold {
                self.food_stored -= growth_threshold;
                self.population += 1;
                result.population_grew = true;

                // Keep a portion of food based on buildings
                let keep_ratio = self.food_keep_ratio();
                self.food_stored = (self.food_stored as f32 * keep_ratio) as u32;
            }
        } else {
            // Starvation
            if self.food_stored > 0 {
                self.food_stored = self.food_stored.saturating_sub((-surplus) as u32);
            } else if self.population > 1 {
                self.population -= 1;
                result.population_starved = true;
            }
        }
    }

    /// Process production.
    fn process_production(&mut self, production: i32, result: &mut CityTurnResult) {
        if production <= 0 {
            return;
        }

        if let Some(ref item) = self.production {
            self.production_progress += production as u32;

            let cost = item.cost();
            if self.production_progress >= cost {
                result.completed_production = Some(item.clone());
                self.production_progress = 0;

                // Move to next item in queue
                self.production = self.production_queue.pop();
            }
        }
    }

    /// Get the ratio of food kept on growth.
    fn food_keep_ratio(&self) -> f32 {
        let mut ratio: f32 = 0.0;
        if self.buildings.contains(&BuildingType::Granary) {
            ratio += 0.5;
        }
        if self.buildings.contains(&BuildingType::Aqueduct) {
            ratio += 0.4;
        }
        ratio.min(1.0)
    }

    /// Check if city should expand borders.
    fn should_expand_borders(&self) -> bool {
        // Culture needed increases with territory size
        let tiles = self.territory.len() as u32;
        let needed = 10 + tiles * tiles * 2;
        self.culture >= needed
    }

    /// Expand borders to include a new tile.
    pub fn expand_borders(&mut self, tile: HexCoord) {
        self.territory.insert(tile);
        // Reset culture for next expansion
        let tiles = self.territory.len() as u32;
        self.culture = self.culture.saturating_sub(10 + tiles * tiles * 2);
    }

    /// Set production to a new item.
    pub fn set_production(&mut self, item: ProductionItem) {
        self.production = Some(item);
        self.production_progress = 0;
    }

    /// Add an item to the production queue.
    pub fn queue_production(&mut self, item: ProductionItem) {
        self.production_queue.push(item);
    }

    /// Build a building in this city.
    pub fn add_building(&mut self, building: BuildingType) {
        self.buildings.insert(building);

        // Apply building effects
        match building {
            BuildingType::Walls => {
                self.max_health += 50;
                self.health += 50;
                self.combat_strength += 5;
            }
            BuildingType::Castle => {
                self.max_health += 75;
                self.health += 75;
                self.combat_strength += 8;
            }
            _ => {}
        }
    }

    /// Check if a building can be built.
    pub fn can_build(&self, building: BuildingType) -> bool {
        if self.buildings.contains(&building) {
            return false;
        }

        // Check prerequisites
        match building {
            BuildingType::University => self.buildings.contains(&BuildingType::Library),
            BuildingType::Bank => self.buildings.contains(&BuildingType::Market),
            BuildingType::Castle => self.buildings.contains(&BuildingType::Walls),
            BuildingType::Hospital => self.buildings.contains(&BuildingType::Aqueduct),
            _ => true,
        }
    }

    /// Assign a citizen to work a tile.
    pub fn assign_citizen(&mut self, tile: HexCoord) -> bool {
        if !self.territory.contains(&tile) {
            return false;
        }
        if self.available_citizens() == 0 {
            return false;
        }
        self.worked_tiles.insert(tile);
        true
    }

    /// Unassign a citizen from a tile.
    pub fn unassign_citizen(&mut self, tile: HexCoord) -> bool {
        if tile == self.position {
            return false; // Can't unassign city center
        }
        self.worked_tiles.remove(&tile)
    }

    /// Take damage from combat.
    pub fn take_damage(&mut self, damage: u32) {
        self.health = self.health.saturating_sub(damage);
    }

    /// Check if city can be captured (health at 0 and melee unit adjacent).
    pub fn can_be_captured(&self) -> bool {
        self.health == 0
    }

    /// Calculate total yields from worked tiles.
    pub fn calculate_yields(&self, tile_yields: impl Fn(&HexCoord) -> Yields) -> Yields {
        let mut total = Yields::zero();

        // Sum yields from worked tiles
        for tile in &self.worked_tiles {
            total += tile_yields(tile);
        }

        // Add specialist yields
        total += self.specialists.yields();

        // Apply building modifiers
        total = self.apply_building_modifiers(total);

        total
    }

    /// Apply building modifiers to yields.
    fn apply_building_modifiers(&self, mut yields: Yields) -> Yields {
        for building in &self.buildings {
            let effects = building.effects();

            // Flat bonuses
            yields.food += effects.food;
            yields.production += effects.production;
            yields.gold += effects.gold;
            yields.science += effects.science;
            yields.culture += effects.culture;

            // Per-population bonuses
            yields.science += effects.science_per_2_pop * (self.population as i32 / 2);

            // Percentage modifiers
            if effects.gold_modifier > 0.0 {
                yields.gold = (yields.gold as f32 * (1.0 + effects.gold_modifier)) as i32;
            }
            if effects.science_modifier > 0.0 {
                yields.science = (yields.science as f32 * (1.0 + effects.science_modifier)) as i32;
            }
            if effects.production_modifier > 0.0 {
                yields.production =
                    (yields.production as f32 * (1.0 + effects.production_modifier)) as i32;
            }
        }

        yields
    }
}

/// Result of processing a city turn.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct CityTurnResult {
    pub population_grew: bool,
    pub population_starved: bool,
    pub completed_production: Option<ProductionItem>,
    pub border_expansion: bool,
}

/// Specialist citizen assignments.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Specialists {
    pub scientists: u32,
    pub engineers: u32,
    pub merchants: u32,
    pub artists: u32,
}

impl Specialists {
    /// Total number of specialists.
    pub fn total(&self) -> u32 {
        self.scientists + self.engineers + self.merchants + self.artists
    }

    /// Yields from specialists.
    pub fn yields(&self) -> Yields {
        Yields::new(
            0,
            self.engineers as i32 * 2,
            self.merchants as i32 * 2,
            self.scientists as i32 * 3,
            self.artists as i32 * 3,
        )
    }
}

/// Items that can be produced by a city.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProductionItem {
    Unit(UnitType),
    Building(BuildingType),
    Wonder(WonderType),
    Project(ProjectType),
}

impl ProductionItem {
    /// Get the production cost.
    pub fn cost(&self) -> u32 {
        match self {
            ProductionItem::Unit(ut) => ut.stats().cost,
            ProductionItem::Building(bt) => bt.cost(),
            ProductionItem::Wonder(wt) => wt.cost(),
            ProductionItem::Project(pt) => pt.cost(),
        }
    }

    /// Get the display name.
    pub fn name(&self) -> String {
        match self {
            ProductionItem::Unit(ut) => format!("{:?}", ut),
            ProductionItem::Building(bt) => format!("{:?}", bt),
            ProductionItem::Wonder(wt) => format!("{:?}", wt),
            ProductionItem::Project(pt) => format!("{:?}", pt),
        }
    }
}

/// Building types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildingType {
    Monument,
    Granary,
    Library,
    Barracks,
    Walls,
    Market,
    Aqueduct,
    University,
    Bank,
    Factory,
    Hospital,
    Castle,
    Workshop,
    Amphitheater,
    Temple,
    Colosseum,
    Courthouse,
    Armory,
    Lighthouse,
}

impl BuildingType {
    /// Get the production cost.
    pub const fn cost(&self) -> u32 {
        match self {
            BuildingType::Monument => 40,
            BuildingType::Granary => 60,
            BuildingType::Library => 75,
            BuildingType::Barracks => 75,
            BuildingType::Walls => 75,
            BuildingType::Market => 100,
            BuildingType::Aqueduct => 120,
            BuildingType::University => 160,
            BuildingType::Bank => 200,
            BuildingType::Factory => 300,
            BuildingType::Hospital => 400,
            BuildingType::Castle => 160,
            BuildingType::Workshop => 100,
            BuildingType::Amphitheater => 100,
            BuildingType::Temple => 100,
            BuildingType::Colosseum => 150,
            BuildingType::Courthouse => 120,
            BuildingType::Armory => 130,
            BuildingType::Lighthouse => 75,
        }
    }

    /// Get building effects.
    pub const fn effects(&self) -> BuildingEffects {
        match self {
            BuildingType::Monument => BuildingEffects::culture(2),
            BuildingType::Granary => BuildingEffects::food(2),
            BuildingType::Library => BuildingEffects::science_per_pop(1),
            BuildingType::Barracks => BuildingEffects::xp_bonus(15),
            BuildingType::Walls => BuildingEffects::defense(5),
            BuildingType::Market => BuildingEffects::gold_modifier(0.25),
            BuildingType::Aqueduct => BuildingEffects::food(2),
            BuildingType::University => BuildingEffects::science_modifier(0.33),
            BuildingType::Bank => BuildingEffects::gold_modifier(0.25),
            BuildingType::Factory => BuildingEffects::production_modifier(0.25),
            BuildingType::Hospital => BuildingEffects::food(5),
            BuildingType::Castle => BuildingEffects::defense(8),
            BuildingType::Workshop => BuildingEffects::production(2),
            BuildingType::Amphitheater => BuildingEffects::culture(3),
            BuildingType::Temple => BuildingEffects::culture(2),
            BuildingType::Colosseum => BuildingEffects::culture(4),
            BuildingType::Courthouse => BuildingEffects::default(),
            BuildingType::Armory => BuildingEffects::xp_bonus(15),
            BuildingType::Lighthouse => BuildingEffects::food(1), // Lighthouse provides +1 food from sea tiles
        }
    }
}

/// Effects of a building.
#[derive(Clone, Copy, Debug, Default)]
pub struct BuildingEffects {
    pub food: i32,
    pub production: i32,
    pub gold: i32,
    pub science: i32,
    pub culture: i32,
    pub defense: i32,
    pub xp_bonus: u32,
    pub food_modifier: f32,
    pub gold_modifier: f32,
    pub science_modifier: f32,
    pub production_modifier: f32,
    pub science_per_2_pop: i32,
}

impl BuildingEffects {
    pub const fn default() -> Self {
        Self {
            food: 0,
            production: 0,
            gold: 0,
            science: 0,
            culture: 0,
            defense: 0,
            xp_bonus: 0,
            food_modifier: 0.0,
            gold_modifier: 0.0,
            science_modifier: 0.0,
            production_modifier: 0.0,
            science_per_2_pop: 0,
        }
    }

    pub const fn food(amount: i32) -> Self {
        Self {
            food: amount,
            ..Self::default()
        }
    }

    pub const fn production(amount: i32) -> Self {
        Self {
            production: amount,
            ..Self::default()
        }
    }

    pub const fn culture(amount: i32) -> Self {
        Self {
            culture: amount,
            ..Self::default()
        }
    }

    pub const fn defense(amount: i32) -> Self {
        Self {
            defense: amount,
            ..Self::default()
        }
    }

    pub const fn xp_bonus(amount: u32) -> Self {
        Self {
            xp_bonus: amount,
            ..Self::default()
        }
    }

    pub const fn gold_modifier(amount: f32) -> Self {
        Self {
            gold_modifier: amount,
            ..Self::default()
        }
    }

    pub const fn science_modifier(amount: f32) -> Self {
        Self {
            science_modifier: amount,
            ..Self::default()
        }
    }

    pub const fn production_modifier(amount: f32) -> Self {
        Self {
            production_modifier: amount,
            ..Self::default()
        }
    }

    pub const fn science_per_pop(amount: i32) -> Self {
        Self {
            science_per_2_pop: amount,
            ..Self::default()
        }
    }
}

/// World wonder types.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WonderType {
    Pyramids,
    GreatLibrary,
    Stonehenge,
    HangingGardens,
    Oracle,
    Colossus,
    GreatLighthouse,
    Parthenon,
    TerracottaArmy,
    GreatWall,
    MachuPicchu,
    NotreDame,
}

impl WonderType {
    pub const fn cost(&self) -> u32 {
        match self {
            WonderType::Pyramids => 185,
            WonderType::GreatLibrary => 185,
            WonderType::Stonehenge => 185,
            WonderType::HangingGardens => 250,
            WonderType::Oracle => 250,
            WonderType::Colossus => 250,
            WonderType::GreatLighthouse => 250,
            WonderType::Parthenon => 250,
            WonderType::TerracottaArmy => 250,
            WonderType::GreatWall => 300,
            WonderType::MachuPicchu => 300,
            WonderType::NotreDame => 400,
        }
    }
}

/// Special project types (like spaceship parts).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProjectType {
    SpaceshipEngine,
    SpaceshipFuelTank,
    SpaceshipCockpit,
    SpaceshipHull,
    ManhattanProject,
}

impl ProjectType {
    pub const fn cost(&self) -> u32 {
        match self {
            ProjectType::SpaceshipEngine => 1500,
            ProjectType::SpaceshipFuelTank => 1500,
            ProjectType::SpaceshipCockpit => 1500,
            ProjectType::SpaceshipHull => 1500,
            ProjectType::ManhattanProject => 750,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_city_creation() {
        let city = City::new(1, 0, "Rome".to_string(), HexCoord::new(5, 5), true);

        assert_eq!(city.id, 1);
        assert_eq!(city.owner, 0);
        assert_eq!(city.population, 1);
        assert!(city.is_capital);
        assert!(city.territory.contains(&city.position));
        assert_eq!(city.territory.len(), 7); // Center + 6 neighbors
    }

    #[test]
    fn test_food_for_growth() {
        let city = City::new(1, 0, "Test".to_string(), HexCoord::new(0, 0), false);

        // Population 1: 15 + 0 + 1 = 16
        assert!(city.food_for_growth() >= 15);
    }

    #[test]
    fn test_city_growth() {
        let mut city = City::new(1, 0, "Test".to_string(), HexCoord::new(0, 0), false);

        // Simulate excess food
        let yields = Yields::new(10, 0, 0, 0, 0); // 10 food, 2 consumed = +8

        // Process many turns to grow
        for _ in 0..10 {
            city.process_turn(&yields);
        }

        assert!(city.population > 1);
    }

    #[test]
    fn test_city_production() {
        let mut city = City::new(1, 0, "Test".to_string(), HexCoord::new(0, 0), false);
        city.set_production(ProductionItem::Unit(UnitType::Warrior));

        let yields = Yields::new(2, 10, 0, 0, 0);

        // Warrior costs 40, at 10/turn = 4 turns
        let mut completed = false;
        for _ in 0..5 {
            let result = city.process_turn(&yields);
            if result.completed_production.is_some() {
                completed = true;
                break;
            }
        }

        assert!(completed);
    }

    #[test]
    fn test_city_buildings() {
        let mut city = City::new(1, 0, "Test".to_string(), HexCoord::new(0, 0), false);

        assert!(city.can_build(BuildingType::Library));
        city.add_building(BuildingType::Library);
        assert!(!city.can_build(BuildingType::Library)); // Already built

        assert!(city.can_build(BuildingType::University)); // Requires library
    }

    #[test]
    fn test_specialist_yields() {
        let specialists = Specialists {
            scientists: 2,
            engineers: 1,
            merchants: 0,
            artists: 0,
        };

        let yields = specialists.yields();
        assert_eq!(yields.science, 6); // 2 * 3
        assert_eq!(yields.production, 2); // 1 * 2
    }

    #[test]
    fn test_city_damage() {
        let mut city = City::new(1, 0, "Test".to_string(), HexCoord::new(0, 0), false);

        city.take_damage(150);
        assert_eq!(city.health, 50);
        assert!(!city.can_be_captured());

        city.take_damage(50);
        assert_eq!(city.health, 0);
        assert!(city.can_be_captured());
    }

    #[test]
    fn test_production_item_cost() {
        let unit = ProductionItem::Unit(UnitType::Warrior);
        assert_eq!(unit.cost(), 40);

        let building = ProductionItem::Building(BuildingType::Library);
        assert_eq!(building.cost(), 75);

        let wonder = ProductionItem::Wonder(WonderType::Pyramids);
        assert_eq!(wonder.cost(), 185);
    }

    #[test]
    fn test_city_serialization() {
        let city = City::new(1, 0, "TestCity".to_string(), HexCoord::new(3, 7), true);
        let json = serde_json::to_string(&city).unwrap();
        let restored: City = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.id, city.id);
        assert_eq!(restored.name, city.name);
        assert_eq!(restored.position, city.position);
    }
}

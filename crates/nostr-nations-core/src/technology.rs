//! Technology tree and research system.
//!
//! The tech tree defines the progression of technologies available to players.
//! Each technology can unlock units, buildings, wonders, improvements, and abilities.

use crate::city::BuildingType;
use crate::terrain::Improvement;
use crate::types::{Era, TechId};
use crate::unit::UnitType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A technology in the tech tree.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Technology {
    /// Unique identifier.
    pub id: TechId,
    /// Display name.
    pub name: String,
    /// Era this technology belongs to.
    pub era: Era,
    /// Base research cost (modified by game speed).
    pub cost: u32,
    /// Technologies required before this can be researched.
    pub prerequisites: Vec<TechId>,
    /// What this technology unlocks.
    pub unlocks: TechUnlocks,
    /// Flavor quote for UI.
    pub quote: String,
}

impl Technology {
    /// Create a new technology.
    pub fn new(id: &str, name: &str, era: Era, cost: u32) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            era,
            cost,
            prerequisites: Vec::new(),
            unlocks: TechUnlocks::default(),
            quote: String::new(),
        }
    }

    /// Add prerequisites.
    pub fn with_prerequisites(mut self, prereqs: &[&str]) -> Self {
        self.prerequisites = prereqs.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set quote.
    pub fn with_quote(mut self, quote: &str) -> Self {
        self.quote = quote.to_string();
        self
    }

    /// Add unit unlocks.
    pub fn unlocks_units(mut self, units: &[UnitType]) -> Self {
        self.unlocks.units.extend(units.iter().cloned());
        self
    }

    /// Add building unlocks.
    pub fn unlocks_buildings(mut self, buildings: &[BuildingType]) -> Self {
        self.unlocks.buildings.extend(buildings.iter().cloned());
        self
    }

    /// Add improvement unlocks.
    pub fn unlocks_improvements(mut self, improvements: &[Improvement]) -> Self {
        self.unlocks
            .improvements
            .extend(improvements.iter().cloned());
        self
    }

    /// Add ability unlocks.
    pub fn unlocks_abilities(mut self, abilities: &[&str]) -> Self {
        self.unlocks
            .abilities
            .extend(abilities.iter().map(|s| s.to_string()));
        self
    }
}

/// What a technology unlocks.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TechUnlocks {
    pub units: Vec<UnitType>,
    pub buildings: Vec<BuildingType>,
    pub improvements: Vec<Improvement>,
    pub abilities: Vec<String>,
}

/// The complete technology tree.
#[derive(Clone, Debug)]
pub struct TechTree {
    /// All technologies indexed by ID.
    techs: HashMap<TechId, Technology>,
    /// Techs organized by era.
    by_era: HashMap<Era, Vec<TechId>>,
}

impl TechTree {
    /// Create a new tech tree with all technologies.
    pub fn new() -> Self {
        let mut tree = Self {
            techs: HashMap::new(),
            by_era: HashMap::new(),
        };

        // Add all technologies
        tree.add_ancient_techs();
        tree.add_classical_techs();
        tree.add_medieval_techs();
        tree.add_renaissance_techs();
        tree.add_industrial_techs();
        tree.add_modern_techs();

        tree
    }

    /// Add a technology to the tree.
    fn add(&mut self, tech: Technology) {
        let era = tech.era;
        let id = tech.id.clone();
        self.techs.insert(id.clone(), tech);
        self.by_era.entry(era).or_default().push(id);
    }

    /// Get a technology by ID.
    pub fn get(&self, id: &TechId) -> Option<&Technology> {
        self.techs.get(id)
    }

    /// Get all technologies in an era.
    pub fn get_era(&self, era: Era) -> Vec<&Technology> {
        self.by_era
            .get(&era)
            .map(|ids| ids.iter().filter_map(|id| self.techs.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all technology IDs.
    pub fn all_ids(&self) -> Vec<&TechId> {
        self.techs.keys().collect()
    }

    /// Check if prerequisites are met for a technology.
    pub fn can_research(&self, tech_id: &TechId, researched: &HashSet<TechId>) -> bool {
        if let Some(tech) = self.techs.get(tech_id) {
            tech.prerequisites
                .iter()
                .all(|prereq| researched.contains(prereq))
        } else {
            false
        }
    }

    /// Get available technologies to research.
    pub fn available_techs(&self, researched: &HashSet<TechId>) -> Vec<&Technology> {
        self.techs
            .values()
            .filter(|tech| {
                !researched.contains(&tech.id) && self.can_research(&tech.id, researched)
            })
            .collect()
    }

    /// Get what units are unlocked by a technology.
    pub fn units_unlocked_by(&self, tech_id: &TechId) -> Vec<UnitType> {
        self.techs
            .get(tech_id)
            .map(|t| t.unlocks.units.clone())
            .unwrap_or_default()
    }

    /// Get what buildings are unlocked by a technology.
    pub fn buildings_unlocked_by(&self, tech_id: &TechId) -> Vec<BuildingType> {
        self.techs
            .get(tech_id)
            .map(|t| t.unlocks.buildings.clone())
            .unwrap_or_default()
    }

    /// Add Ancient Era technologies.
    fn add_ancient_techs(&mut self) {
        // Starting tech - Agriculture
        self.add(
            Technology::new("agriculture", "Agriculture", Era::Ancient, 20)
                .unlocks_improvements(&[Improvement::Farm])
                .with_quote("Whoever could make two ears of corn grow where only one grew before would deserve better of mankind."),
        );

        // Pottery - requires Agriculture
        self.add(
            Technology::new("pottery", "Pottery", Era::Ancient, 35)
                .with_prerequisites(&["agriculture"])
                .unlocks_buildings(&[BuildingType::Granary])
                .with_quote("Pottery is a silent voice of the past."),
        );

        // Animal Husbandry - requires Agriculture
        self.add(
            Technology::new("animal_husbandry", "Animal Husbandry", Era::Ancient, 35)
                .with_prerequisites(&["agriculture"])
                .unlocks_units(&[UnitType::Chariot])
                .unlocks_improvements(&[Improvement::Pasture])
                .unlocks_abilities(&["reveal_horses"])
                .with_quote(
                    "The horse, the horse! The symbol of surging potency and power of movement.",
                ),
        );

        // Archery - requires Agriculture
        self.add(
            Technology::new("archery", "Archery", Era::Ancient, 35)
                .with_prerequisites(&["agriculture"])
                .unlocks_units(&[UnitType::Archer])
                .with_quote("The bow is the strongest shield in the world."),
        );

        // Mining
        self.add(
            Technology::new("mining", "Mining", Era::Ancient, 35)
                .unlocks_improvements(&[Improvement::Mine])
                .with_quote(
                    "If you can find a path with no obstacles, it probably doesn't lead anywhere.",
                ),
        );

        // Sailing
        self.add(
            Technology::new("sailing", "Sailing", Era::Ancient, 45)
                .unlocks_units(&[UnitType::Galley])
                .unlocks_abilities(&["embark"])
                .with_quote(
                    "The wind and the waves are always on the side of the ablest navigator.",
                ),
        );

        // Calendar - requires Pottery
        self.add(
            Technology::new("calendar", "Calendar", Era::Ancient, 55)
                .with_prerequisites(&["pottery"])
                .unlocks_improvements(&[Improvement::Plantation])
                .unlocks_buildings(&[BuildingType::Monument])
                .with_quote("Calendars measure the progress of time."),
        );

        // Writing - requires Pottery
        self.add(
            Technology::new("writing", "Writing", Era::Ancient, 55)
                .with_prerequisites(&["pottery"])
                .unlocks_buildings(&[BuildingType::Library])
                .with_quote("Writing is the painting of the voice."),
        );

        // Trapping - requires Animal Husbandry
        self.add(
            Technology::new("trapping", "Trapping", Era::Ancient, 55)
                .with_prerequisites(&["animal_husbandry"])
                .unlocks_improvements(&[Improvement::Camp])
                .with_quote("We should trap what we cannot charm."),
        );

        // The Wheel - requires Animal Husbandry
        self.add(
            Technology::new("the_wheel", "The Wheel", Era::Ancient, 55)
                .with_prerequisites(&["animal_husbandry"])
                .unlocks_abilities(&["roads"])
                .with_quote("The wheel is the greatest invention of all time."),
        );

        // Masonry - requires Mining
        self.add(
            Technology::new("masonry", "Masonry", Era::Ancient, 55)
                .with_prerequisites(&["mining"])
                .unlocks_buildings(&[BuildingType::Walls])
                .unlocks_improvements(&[Improvement::Quarry])
                .with_quote("A stone is hard, but time is harder."),
        );

        // Bronze Working - requires Mining
        self.add(
            Technology::new("bronze_working", "Bronze Working", Era::Ancient, 55)
                .with_prerequisites(&["mining"])
                .unlocks_units(&[UnitType::Spearman])
                .unlocks_buildings(&[BuildingType::Barracks])
                .unlocks_abilities(&["clear_forest"])
                .with_quote("Bronze is the mirror of the form; wine, of the heart."),
        );
    }

    /// Add Classical Era technologies.
    fn add_classical_techs(&mut self) {
        // Optics - requires Sailing
        self.add(
            Technology::new("optics", "Optics", Era::Classical, 85)
                .with_prerequisites(&["sailing"])
                .unlocks_units(&[UnitType::Trireme])
                .unlocks_buildings(&[BuildingType::Lighthouse])
                .with_quote("The eye of man is like a mirror."),
        );

        // Philosophy - requires Writing
        self.add(
            Technology::new("philosophy", "Philosophy", Era::Classical, 100)
                .with_prerequisites(&["writing"])
                .unlocks_buildings(&[BuildingType::Temple])
                .with_quote("The unexamined life is not worth living."),
        );

        // Drama - requires Writing
        self.add(
            Technology::new("drama", "Drama", Era::Classical, 100)
                .with_prerequisites(&["writing"])
                .unlocks_buildings(&[BuildingType::Amphitheater])
                .with_quote("All the world's a stage."),
        );

        // Mathematics - requires Writing, The Wheel
        self.add(
            Technology::new("mathematics", "Mathematics", Era::Classical, 100)
                .with_prerequisites(&["writing", "the_wheel"])
                .unlocks_units(&[UnitType::Catapult])
                .unlocks_buildings(&[BuildingType::Courthouse])
                .with_quote("Mathematics is the queen of the sciences."),
        );

        // Construction - requires Masonry
        self.add(
            Technology::new("construction", "Construction", Era::Classical, 100)
                .with_prerequisites(&["masonry"])
                .unlocks_buildings(&[BuildingType::Colosseum, BuildingType::Aqueduct])
                .with_quote("Give me a lever long enough and I shall move the world."),
        );

        // Iron Working - requires Bronze Working
        self.add(
            Technology::new("iron_working", "Iron Working", Era::Classical, 150)
                .with_prerequisites(&["bronze_working"])
                .unlocks_units(&[UnitType::Swordsman])
                .unlocks_abilities(&["reveal_iron"])
                .with_quote("Iron rusts from disuse; water loses its purity from stagnation."),
        );

        // Horseback Riding - requires Animal Husbandry
        self.add(
            Technology::new("horseback_riding", "Horseback Riding", Era::Classical, 100)
                .with_prerequisites(&["animal_husbandry"])
                .unlocks_units(&[UnitType::Horseman])
                .with_quote("No hour of life is wasted that is spent in the saddle."),
        );

        // Currency - requires Mathematics
        self.add(
            Technology::new("currency", "Currency", Era::Classical, 140)
                .with_prerequisites(&["mathematics"])
                .unlocks_buildings(&[BuildingType::Market])
                .with_quote("Money is a good servant but a bad master."),
        );
    }

    /// Add Medieval Era technologies.
    fn add_medieval_techs(&mut self) {
        // Civil Service - requires Philosophy, Currency
        self.add(
            Technology::new("civil_service", "Civil Service", Era::Medieval, 275)
                .with_prerequisites(&["philosophy", "currency"])
                .unlocks_units(&[UnitType::Pikeman])
                .with_quote("The bureaucracy is a giant mechanism operated by pygmies."),
        );

        // Chivalry - requires Horseback Riding, Civil Service
        self.add(
            Technology::new("chivalry", "Chivalry", Era::Medieval, 340)
                .with_prerequisites(&["horseback_riding", "civil_service"])
                .unlocks_units(&[UnitType::Knight])
                .unlocks_buildings(&[BuildingType::Castle])
                .with_quote(
                    "A true knight is fuller of bravery in the midst of strokes of battle.",
                ),
        );

        // Education - requires Philosophy
        self.add(
            Technology::new("education", "Education", Era::Medieval, 275)
                .with_prerequisites(&["philosophy"])
                .unlocks_buildings(&[BuildingType::University])
                .with_quote(
                    "Education is what survives when what has been learned has been forgotten.",
                ),
        );

        // Steel - requires Iron Working
        self.add(
            Technology::new("steel", "Steel", Era::Medieval, 275)
                .with_prerequisites(&["iron_working"])
                .unlocks_units(&[UnitType::Longswordsman])
                .with_quote("Steel is the metal of civilization."),
        );

        // Machinery - requires Iron Working
        self.add(
            Technology::new("machinery", "Machinery", Era::Medieval, 275)
                .with_prerequisites(&["iron_working"])
                .unlocks_units(&[UnitType::Crossbow])
                .unlocks_improvements(&[Improvement::LumberMill])
                .with_quote("A machine has no mind of its own."),
        );

        // Compass - requires Optics
        self.add(
            Technology::new("compass", "Compass", Era::Medieval, 275)
                .with_prerequisites(&["optics"])
                .unlocks_units(&[UnitType::Caravel])
                .with_quote("He who has the compass knows the way."),
        );

        // Physics - requires Mathematics, Machinery
        self.add(
            Technology::new("physics", "Physics", Era::Medieval, 340)
                .with_prerequisites(&["mathematics", "machinery"])
                .unlocks_units(&[UnitType::Trebuchet])
                .with_quote("For every action, there is an equal and opposite reaction."),
        );

        // Banking - requires Education, Chivalry
        self.add(
            Technology::new("banking", "Banking", Era::Medieval, 400)
                .with_prerequisites(&["education", "chivalry"])
                .unlocks_buildings(&[BuildingType::Bank])
                .with_quote("It is well that the people do not understand our banking system."),
        );
    }

    /// Add Renaissance Era technologies.
    fn add_renaissance_techs(&mut self) {
        // Astronomy - requires Compass, Education
        self.add(
            Technology::new("astronomy", "Astronomy", Era::Renaissance, 485)
                .with_prerequisites(&["compass", "education"])
                .unlocks_abilities(&["ocean_crossing"])
                .with_quote("The cosmos is within us. We are made of star-stuff."),
        );

        // Printing Press - requires Machinery
        self.add(
            Technology::new("printing_press", "Printing Press", Era::Renaissance, 485)
                .with_prerequisites(&["machinery"])
                .with_quote("The printing press is the greatest weapon in the armory of the modern commander."),
        );

        // Gunpowder - requires Physics, Steel
        self.add(
            Technology::new("gunpowder", "Gunpowder", Era::Renaissance, 550)
                .with_prerequisites(&["physics", "steel"])
                .unlocks_units(&[UnitType::Musketman])
                .with_quote("The cannon conquers all."),
        );

        // Metallurgy - requires Gunpowder
        self.add(
            Technology::new("metallurgy", "Metallurgy", Era::Renaissance, 620)
                .with_prerequisites(&["gunpowder"])
                .unlocks_units(&[UnitType::Lancer, UnitType::Cannon])
                .with_quote("Metal is the backbone of civilization."),
        );

        // Navigation - requires Astronomy
        self.add(
            Technology::new("navigation", "Navigation", Era::Renaissance, 550)
                .with_prerequisites(&["astronomy"])
                .unlocks_units(&[UnitType::Frigate])
                .with_quote("He that would learn to pray, let him go to sea."),
        );

        // Economics - requires Banking, Printing Press
        self.add(
            Technology::new("economics", "Economics", Era::Renaissance, 620)
                .with_prerequisites(&["banking", "printing_press"])
                .unlocks_buildings(&[BuildingType::Workshop])
                .with_quote("Economics is not about things and tangible material objects."),
        );

        // Chemistry - requires Gunpowder
        self.add(
            Technology::new("chemistry", "Chemistry", Era::Renaissance, 620)
                .with_prerequisites(&["gunpowder"])
                .with_quote("Chemistry is the dirty part of physics."),
        );

        // Acoustics - requires Printing Press
        self.add(
            Technology::new("acoustics", "Acoustics", Era::Renaissance, 550)
                .with_prerequisites(&["printing_press"])
                .with_quote("Music gives a soul to the universe."),
        );
    }

    /// Add Industrial Era technologies.
    fn add_industrial_techs(&mut self) {
        // Scientific Theory - requires Chemistry, Acoustics
        self.add(
            Technology::new(
                "scientific_theory",
                "Scientific Theory",
                Era::Industrial,
                780,
            )
            .with_prerequisites(&["chemistry", "acoustics"])
            .with_quote(
                "The good thing about science is that it's true whether you believe it or not.",
            ),
        );

        // Industrialization - requires Economics
        self.add(
            Technology::new(
                "industrialization",
                "Industrialization",
                Era::Industrial,
                700,
            )
            .with_prerequisites(&["economics"])
            .unlocks_buildings(&[BuildingType::Factory])
            .unlocks_abilities(&["reveal_coal"])
            .with_quote("The factory is the temple of the machine."),
        );

        // Rifling - requires Metallurgy
        self.add(
            Technology::new("rifling", "Rifling", Era::Industrial, 700)
                .with_prerequisites(&["metallurgy"])
                .unlocks_units(&[UnitType::Rifleman])
                .with_quote("The rifle is a civilizing weapon."),
        );

        // Military Science - requires Chemistry
        self.add(
            Technology::new("military_science", "Military Science", Era::Industrial, 700)
                .with_prerequisites(&["chemistry"])
                .unlocks_units(&[UnitType::Cavalry])
                .with_quote("War is the continuation of politics by other means."),
        );

        // Steam Power - requires Industrialization, Scientific Theory
        self.add(
            Technology::new("steam_power", "Steam Power", Era::Industrial, 860)
                .with_prerequisites(&["industrialization", "scientific_theory"])
                .unlocks_units(&[UnitType::Ironclad])
                .with_quote("Steam is a perfect servant but a terrible master."),
        );

        // Dynamite - requires Rifling, Military Science
        self.add(
            Technology::new("dynamite", "Dynamite", Era::Industrial, 860)
                .with_prerequisites(&["rifling", "military_science"])
                .unlocks_units(&[UnitType::Artillery])
                .with_quote(
                    "My dynamite will sooner lead to peace than a thousand world conventions.",
                ),
        );

        // Electricity - requires Scientific Theory
        self.add(
            Technology::new("electricity", "Electricity", Era::Industrial, 860)
                .with_prerequisites(&["scientific_theory"])
                .with_quote("Electricity is really just organized lightning."),
        );

        // Biology - requires Scientific Theory
        self.add(
            Technology::new("biology", "Biology", Era::Industrial, 860)
                .with_prerequisites(&["scientific_theory"])
                .unlocks_buildings(&[BuildingType::Hospital])
                .with_quote("Nothing in biology makes sense except in the light of evolution."),
        );

        // Telegraph - requires Electricity
        self.add(
            Technology::new("telegraph", "Telegraph", Era::Industrial, 950)
                .with_prerequisites(&["electricity"])
                .with_quote("What hath God wrought?"),
        );
    }

    /// Add Modern Era technologies.
    fn add_modern_techs(&mut self) {
        // Replaceable Parts - requires Steam Power
        self.add(
            Technology::new("replaceable_parts", "Replaceable Parts", Era::Modern, 1040)
                .with_prerequisites(&["steam_power"])
                .unlocks_units(&[UnitType::Infantry])
                .with_quote("The machine does not isolate man from the great problems of nature."),
        );

        // Combustion - requires Dynamite, Replaceable Parts
        self.add(
            Technology::new("combustion", "Combustion", Era::Modern, 1150)
                .with_prerequisites(&["dynamite", "replaceable_parts"])
                .unlocks_units(&[UnitType::Tank])
                .unlocks_abilities(&["reveal_oil"])
                .with_quote("The internal combustion engine opened up the world."),
        );

        // Ballistics - requires Dynamite
        self.add(
            Technology::new("ballistics", "Ballistics", Era::Modern, 1040)
                .with_prerequisites(&["dynamite"])
                .unlocks_units(&[UnitType::MachineGun])
                .with_quote("The bullet is a fool, the bayonet is a fine lad."),
        );

        // Flight - requires Telegraph, Combustion
        self.add(
            Technology::new("flight", "Flight", Era::Modern, 1250)
                .with_prerequisites(&["telegraph", "combustion"])
                .unlocks_units(&[UnitType::Fighter])
                .with_quote("If God had really intended men to fly, He'd make it easier to get to the airport."),
        );

        // Electronics - requires Telegraph
        self.add(
            Technology::new("electronics", "Electronics", Era::Modern, 1150)
                .with_prerequisites(&["telegraph"])
                .with_quote(
                    "Electronics is really easy: all you have to do is connect things with wires.",
                ),
        );

        // Radar - requires Electronics
        self.add(
            Technology::new("radar", "Radar", Era::Modern, 1250)
                .with_prerequisites(&["electronics"])
                .unlocks_units(&[UnitType::Battleship, UnitType::Bomber])
                .with_quote("Radar: the science of detection by reflection."),
        );

        // Rocketry - requires Radar
        self.add(
            Technology::new("rocketry", "Rocketry", Era::Modern, 1350)
                .with_prerequisites(&["radar"])
                .unlocks_units(&[UnitType::RocketArtillery])
                .with_quote(
                    "Earth is the cradle of humanity, but one cannot live in a cradle forever.",
                ),
        );

        // Nuclear Fission - requires Radar
        self.add(
            Technology::new("nuclear_fission", "Nuclear Fission", Era::Modern, 1400)
                .with_prerequisites(&["radar"])
                .unlocks_abilities(&["reveal_uranium", "nuclear_weapons"])
                .with_quote("I am become Death, the destroyer of worlds."),
        );

        // Spaceflight - requires Rocketry
        self.add(
            Technology::new("spaceflight", "Spaceflight", Era::Modern, 1500)
                .with_prerequisites(&["rocketry"])
                .unlocks_abilities(&["spaceship_parts"])
                .with_quote("That's one small step for man, one giant leap for mankind."),
        );
    }
}

impl Default for TechTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tech_tree_creation() {
        let tree = TechTree::new();
        assert!(tree.techs.len() > 40, "Should have 40+ technologies");
    }

    #[test]
    fn test_get_technology() {
        let tree = TechTree::new();
        let agriculture = tree.get(&"agriculture".to_string());
        assert!(agriculture.is_some());
        assert_eq!(agriculture.unwrap().name, "Agriculture");
    }

    #[test]
    fn test_ancient_techs() {
        let tree = TechTree::new();
        let ancient = tree.get_era(Era::Ancient);
        assert!(!ancient.is_empty());
    }

    #[test]
    fn test_prerequisites() {
        let tree = TechTree::new();
        let researched: HashSet<TechId> = HashSet::new();

        // Can research Agriculture (no prereqs)
        assert!(tree.can_research(&"agriculture".to_string(), &researched));

        // Cannot research Pottery (requires Agriculture)
        assert!(!tree.can_research(&"pottery".to_string(), &researched));

        // After researching Agriculture, can research Pottery
        let mut researched = researched;
        researched.insert("agriculture".to_string());
        assert!(tree.can_research(&"pottery".to_string(), &researched));
    }

    #[test]
    fn test_available_techs() {
        let tree = TechTree::new();
        let researched: HashSet<TechId> = HashSet::new();

        let available = tree.available_techs(&researched);

        // Should include starting techs
        assert!(available.iter().any(|t| t.id == "agriculture"));
        assert!(available.iter().any(|t| t.id == "mining"));
        assert!(available.iter().any(|t| t.id == "sailing"));

        // Should not include techs with prereqs
        assert!(!available.iter().any(|t| t.id == "pottery"));
    }

    #[test]
    fn test_tech_unlocks() {
        let tree = TechTree::new();

        let units = tree.units_unlocked_by(&"archery".to_string());
        assert!(units.contains(&UnitType::Archer));

        let buildings = tree.buildings_unlocked_by(&"pottery".to_string());
        assert!(buildings.contains(&BuildingType::Granary));
    }

    #[test]
    fn test_era_progression() {
        let tree = TechTree::new();

        // Each era should have technologies
        assert!(!tree.get_era(Era::Ancient).is_empty());
        assert!(!tree.get_era(Era::Classical).is_empty());
        assert!(!tree.get_era(Era::Medieval).is_empty());
        assert!(!tree.get_era(Era::Renaissance).is_empty());
        assert!(!tree.get_era(Era::Industrial).is_empty());
        assert!(!tree.get_era(Era::Modern).is_empty());
    }
}

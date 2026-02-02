//! Terrain types, features, and resources for the game map.

use crate::yields::Yields;
use serde::{Deserialize, Serialize};

/// Base terrain type for a tile.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Terrain {
    #[default]
    Grassland,
    Plains,
    Desert,
    Tundra,
    Snow,
    Coast,
    Ocean,
}

impl Terrain {
    /// Get the base yields for this terrain type.
    pub const fn base_yields(&self) -> Yields {
        match self {
            Terrain::Grassland => Yields::new(2, 0, 0, 0, 0),
            Terrain::Plains => Yields::new(1, 1, 0, 0, 0),
            Terrain::Desert => Yields::zero(),
            Terrain::Tundra => Yields::new(1, 0, 0, 0, 0),
            Terrain::Snow => Yields::zero(),
            Terrain::Coast => Yields::new(1, 0, 1, 0, 0),
            Terrain::Ocean => Yields::new(1, 0, 0, 0, 0),
        }
    }

    /// Get the base movement cost for this terrain.
    pub const fn movement_cost(&self) -> u32 {
        match self {
            Terrain::Coast | Terrain::Ocean => 1, // Naval units only
            _ => 1,
        }
    }

    /// Check if this is a water terrain type.
    pub const fn is_water(&self) -> bool {
        matches!(self, Terrain::Coast | Terrain::Ocean)
    }

    /// Check if this terrain can support a city.
    pub const fn can_found_city(&self) -> bool {
        !self.is_water()
    }

    /// Get all terrain variants.
    pub const fn all() -> &'static [Terrain] {
        &[
            Terrain::Grassland,
            Terrain::Plains,
            Terrain::Desert,
            Terrain::Tundra,
            Terrain::Snow,
            Terrain::Coast,
            Terrain::Ocean,
        ]
    }
}

/// Features that can appear on tiles (overlay on terrain).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Feature {
    Hills,
    Mountains,
    Forest,
    Jungle,
    Marsh,
    Oasis,
    FloodPlains,
    Ice,
}

impl Feature {
    /// Get the yield modifier for this feature.
    pub const fn yield_modifier(&self) -> Yields {
        match self {
            Feature::Hills => Yields::new(0, 1, 0, 0, 0),
            Feature::Forest => Yields::new(-1, 1, 0, 0, 0),
            Feature::Jungle => Yields::new(0, -1, 0, 0, 0),
            Feature::Marsh => Yields::new(-1, 0, 0, 0, 0),
            Feature::Oasis => Yields::new(3, 0, 1, 0, 0),
            Feature::FloodPlains => Yields::new(2, 0, 0, 0, 0),
            Feature::Mountains => Yields::zero(),
            Feature::Ice => Yields::zero(),
        }
    }

    /// Get the movement cost modifier for this feature.
    pub const fn movement_cost(&self) -> u32 {
        match self {
            Feature::Hills => 2,
            Feature::Forest | Feature::Jungle => 2,
            Feature::Marsh => 3,
            Feature::Mountains => u32::MAX, // Impassable by default
            Feature::Ice => u32::MAX,       // Impassable
            _ => 1,
        }
    }

    /// Get the defense bonus percentage for units on this feature.
    pub const fn defense_bonus(&self) -> i32 {
        match self {
            Feature::Hills | Feature::Forest | Feature::Jungle => 25,
            Feature::Marsh => -10,
            _ => 0,
        }
    }

    /// Check if this feature blocks city founding.
    pub const fn blocks_city(&self) -> bool {
        matches!(self, Feature::Mountains | Feature::Ice)
    }

    /// Check if this feature can be removed by workers.
    pub const fn can_remove(&self) -> bool {
        matches!(self, Feature::Forest | Feature::Jungle | Feature::Marsh)
    }

    /// Get all feature variants.
    pub const fn all() -> &'static [Feature] {
        &[
            Feature::Hills,
            Feature::Mountains,
            Feature::Forest,
            Feature::Jungle,
            Feature::Marsh,
            Feature::Oasis,
            Feature::FloodPlains,
            Feature::Ice,
        ]
    }
}

/// Resources that can appear on tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Resource {
    // Strategic resources
    Iron,
    Horses,
    Coal,
    Oil,
    Uranium,

    // Luxury resources
    Gold,
    Silver,
    Gems,
    Pearls,
    Silk,
    Dyes,
    Spices,
    Incense,
    Wine,
    Furs,
    Ivory,
    Marble,

    // Bonus resources
    Wheat,
    Cattle,
    Sheep,
    Deer,
    Fish,
    Whales,
    Crabs,
    Stone,
    Copper,
    Salt,
}

/// Category of a resource.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceCategory {
    /// Required for certain units/buildings
    Strategic,
    /// Provides happiness when connected
    Luxury,
    /// Provides extra yields when improved
    Bonus,
}

impl Resource {
    /// Get the category of this resource.
    pub const fn category(&self) -> ResourceCategory {
        match self {
            Resource::Iron
            | Resource::Horses
            | Resource::Coal
            | Resource::Oil
            | Resource::Uranium => ResourceCategory::Strategic,

            Resource::Gold
            | Resource::Silver
            | Resource::Gems
            | Resource::Pearls
            | Resource::Silk
            | Resource::Dyes
            | Resource::Spices
            | Resource::Incense
            | Resource::Wine
            | Resource::Furs
            | Resource::Ivory
            | Resource::Marble => ResourceCategory::Luxury,

            _ => ResourceCategory::Bonus,
        }
    }

    /// Get the yield bonus when this resource is improved.
    pub const fn yield_bonus(&self) -> Yields {
        match self {
            // Bonus food resources
            Resource::Wheat | Resource::Cattle => Yields::new(1, 0, 0, 0, 0),
            Resource::Fish | Resource::Deer | Resource::Sheep => Yields::new(1, 0, 0, 0, 0),

            // Strategic
            Resource::Iron | Resource::Coal => Yields::new(0, 1, 0, 0, 0),
            Resource::Horses => Yields::new(0, 1, 0, 0, 0),
            Resource::Oil | Resource::Uranium => Yields::new(0, 1, 0, 0, 0),

            // Luxury (generally gold)
            Resource::Gold | Resource::Silver => Yields::new(0, 0, 2, 0, 0),
            Resource::Gems => Yields::new(0, 0, 3, 0, 0),
            Resource::Pearls => Yields::new(0, 0, 2, 0, 0),
            Resource::Silk | Resource::Dyes | Resource::Spices | Resource::Incense => {
                Yields::new(0, 0, 2, 0, 0)
            }
            Resource::Wine | Resource::Furs | Resource::Ivory => Yields::new(0, 0, 2, 0, 0),
            Resource::Marble => Yields::new(0, 0, 1, 0, 1),

            // Other bonus
            Resource::Whales => Yields::new(1, 0, 1, 0, 0),
            Resource::Crabs => Yields::new(1, 0, 1, 0, 0),
            Resource::Stone => Yields::new(0, 1, 0, 0, 0),
            Resource::Copper => Yields::new(0, 0, 1, 0, 0),
            Resource::Salt => Yields::new(1, 0, 1, 0, 0),
        }
    }

    /// Check if this resource is visible without a tech.
    pub const fn initially_visible(&self) -> bool {
        match self {
            // Hidden until tech
            Resource::Iron | Resource::Coal | Resource::Oil | Resource::Uranium => false,
            _ => true,
        }
    }

    /// Get all resource variants.
    pub const fn all() -> &'static [Resource] {
        &[
            Resource::Iron,
            Resource::Horses,
            Resource::Coal,
            Resource::Oil,
            Resource::Uranium,
            Resource::Gold,
            Resource::Silver,
            Resource::Gems,
            Resource::Pearls,
            Resource::Silk,
            Resource::Dyes,
            Resource::Spices,
            Resource::Incense,
            Resource::Wine,
            Resource::Furs,
            Resource::Ivory,
            Resource::Marble,
            Resource::Wheat,
            Resource::Cattle,
            Resource::Sheep,
            Resource::Deer,
            Resource::Fish,
            Resource::Whales,
            Resource::Crabs,
            Resource::Stone,
            Resource::Copper,
            Resource::Salt,
        ]
    }
}

/// Tile improvements built by workers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Improvement {
    Farm,
    Mine,
    Plantation,
    Pasture,
    Camp,
    Quarry,
    FishingBoats,
    OilWell,
    LumberMill,
    TradingPost,
    Fort,
    Academy,
    Manufactory,
    CustomsHouse,
    Landmark,
}

impl Improvement {
    /// Get the yield bonus from this improvement.
    pub const fn yield_bonus(&self) -> Yields {
        match self {
            Improvement::Farm => Yields::new(1, 0, 0, 0, 0),
            Improvement::Mine => Yields::new(0, 1, 0, 0, 0),
            Improvement::Plantation => Yields::new(0, 0, 1, 0, 0),
            Improvement::Pasture => Yields::new(0, 1, 0, 0, 0),
            Improvement::Camp => Yields::new(0, 0, 1, 0, 0),
            Improvement::Quarry => Yields::new(0, 1, 0, 0, 0),
            Improvement::FishingBoats => Yields::new(1, 0, 0, 0, 0),
            Improvement::OilWell => Yields::new(0, 3, 0, 0, 0),
            Improvement::LumberMill => Yields::new(0, 1, 0, 0, 0),
            Improvement::TradingPost => Yields::new(0, 0, 1, 1, 0),
            Improvement::Fort => Yields::zero(), // Defense only
            Improvement::Academy => Yields::new(0, 0, 0, 8, 0),
            Improvement::Manufactory => Yields::new(0, 4, 0, 0, 0),
            Improvement::CustomsHouse => Yields::new(0, 0, 4, 0, 0),
            Improvement::Landmark => Yields::new(0, 0, 0, 0, 6),
        }
    }

    /// Get the turns required to build this improvement.
    pub const fn build_turns(&self) -> u32 {
        match self {
            Improvement::Farm => 5,
            Improvement::Mine => 6,
            Improvement::Fort => 6,
            Improvement::Academy
            | Improvement::Manufactory
            | Improvement::CustomsHouse
            | Improvement::Landmark => 1, // Great person instant
            _ => 5,
        }
    }
}

/// Road types that can be built on tiles.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Road {
    Road,
    Railroad,
}

impl Road {
    /// Get the movement cost multiplier for this road type.
    pub const fn movement_multiplier(&self) -> f32 {
        match self {
            Road::Road => 0.5,
            Road::Railroad => 0.1,
        }
    }

    /// Get turns to build this road.
    pub const fn build_turns(&self) -> u32 {
        match self {
            Road::Road => 3,
            Road::Railroad => 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terrain_yields() {
        assert_eq!(Terrain::Grassland.base_yields().food, 2);
        assert_eq!(Terrain::Plains.base_yields().food, 1);
        assert_eq!(Terrain::Plains.base_yields().production, 1);
        assert_eq!(Terrain::Desert.base_yields().total(), 0);
    }

    #[test]
    fn test_terrain_is_water() {
        assert!(Terrain::Coast.is_water());
        assert!(Terrain::Ocean.is_water());
        assert!(!Terrain::Grassland.is_water());
    }

    #[test]
    fn test_feature_defense_bonus() {
        assert_eq!(Feature::Hills.defense_bonus(), 25);
        assert_eq!(Feature::Forest.defense_bonus(), 25);
        assert_eq!(Feature::Marsh.defense_bonus(), -10);
        assert_eq!(Feature::Oasis.defense_bonus(), 0);
    }

    #[test]
    fn test_resource_categories() {
        assert_eq!(Resource::Iron.category(), ResourceCategory::Strategic);
        assert_eq!(Resource::Gold.category(), ResourceCategory::Luxury);
        assert_eq!(Resource::Wheat.category(), ResourceCategory::Bonus);
    }

    #[test]
    fn test_combined_yields() {
        // Grassland + Hills should be 2F + 1P
        let base = Terrain::Grassland.base_yields();
        let feature = Feature::Hills.yield_modifier();
        let combined = base + feature;
        assert_eq!(combined.food, 2);
        assert_eq!(combined.production, 1);
    }

    #[test]
    fn test_jungle_reduces_production() {
        let base = Terrain::Plains.base_yields(); // 1F 1P
        let jungle = Feature::Jungle.yield_modifier(); // -1P
        let combined = base + jungle;
        assert_eq!(combined.food, 1);
        assert_eq!(combined.production, 0);
    }
}

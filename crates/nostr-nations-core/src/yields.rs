//! Resource yields from tiles, buildings, and other game elements.

use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Resource yields produced by tiles, buildings, specialists, etc.
///
/// All values can be negative (e.g., jungle reduces production).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Yields {
    /// Food - drives population growth
    pub food: i32,
    /// Production - builds units and buildings
    pub production: i32,
    /// Gold - currency for purchasing and maintenance
    pub gold: i32,
    /// Science - advances research
    pub science: i32,
    /// Culture - expands borders and enables policies
    pub culture: i32,
}

impl Yields {
    /// Create yields with all values set to zero.
    pub const fn zero() -> Self {
        Self {
            food: 0,
            production: 0,
            gold: 0,
            science: 0,
            culture: 0,
        }
    }

    /// Create yields with only food.
    pub const fn food(amount: i32) -> Self {
        Self {
            food: amount,
            ..Self::zero()
        }
    }

    /// Create yields with only production.
    pub const fn production(amount: i32) -> Self {
        Self {
            production: amount,
            ..Self::zero()
        }
    }

    /// Create yields with only gold.
    pub const fn gold(amount: i32) -> Self {
        Self {
            gold: amount,
            ..Self::zero()
        }
    }

    /// Create yields with only science.
    pub const fn science(amount: i32) -> Self {
        Self {
            science: amount,
            ..Self::zero()
        }
    }

    /// Create yields with only culture.
    pub const fn culture(amount: i32) -> Self {
        Self {
            culture: amount,
            ..Self::zero()
        }
    }

    /// Create yields from individual components.
    pub const fn new(food: i32, production: i32, gold: i32, science: i32, culture: i32) -> Self {
        Self {
            food,
            production,
            gold,
            science,
            culture,
        }
    }

    /// Check if all yields are non-negative.
    pub fn is_non_negative(&self) -> bool {
        self.food >= 0
            && self.production >= 0
            && self.gold >= 0
            && self.science >= 0
            && self.culture >= 0
    }

    /// Get the total of all yields (useful for basic comparisons).
    pub fn total(&self) -> i32 {
        self.food + self.production + self.gold + self.science + self.culture
    }

    /// Apply a multiplier to all yields.
    pub fn multiply(&self, factor: f32) -> Self {
        Self {
            food: (self.food as f32 * factor).round() as i32,
            production: (self.production as f32 * factor).round() as i32,
            gold: (self.gold as f32 * factor).round() as i32,
            science: (self.science as f32 * factor).round() as i32,
            culture: (self.culture as f32 * factor).round() as i32,
        }
    }

    /// Clamp all negative values to zero.
    pub fn clamp_non_negative(&self) -> Self {
        Self {
            food: self.food.max(0),
            production: self.production.max(0),
            gold: self.gold.max(0),
            science: self.science.max(0),
            culture: self.culture.max(0),
        }
    }
}

impl Add for Yields {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            food: self.food + other.food,
            production: self.production + other.production,
            gold: self.gold + other.gold,
            science: self.science + other.science,
            culture: self.culture + other.culture,
        }
    }
}

impl AddAssign for Yields {
    fn add_assign(&mut self, other: Self) {
        self.food += other.food;
        self.production += other.production;
        self.gold += other.gold;
        self.science += other.science;
        self.culture += other.culture;
    }
}

impl Sub for Yields {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            food: self.food - other.food,
            production: self.production - other.production,
            gold: self.gold - other.gold,
            science: self.science - other.science,
            culture: self.culture - other.culture,
        }
    }
}

impl SubAssign for Yields {
    fn sub_assign(&mut self, other: Self) {
        self.food -= other.food;
        self.production -= other.production;
        self.gold -= other.gold;
        self.science -= other.science;
        self.culture -= other.culture;
    }
}

impl std::fmt::Display for Yields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if self.food != 0 {
            parts.push(format!("{}F", self.food));
        }
        if self.production != 0 {
            parts.push(format!("{}P", self.production));
        }
        if self.gold != 0 {
            parts.push(format!("{}G", self.gold));
        }
        if self.science != 0 {
            parts.push(format!("{}S", self.science));
        }
        if self.culture != 0 {
            parts.push(format!("{}C", self.culture));
        }
        if parts.is_empty() {
            write!(f, "0")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_zero() {
        let y = Yields::default();
        assert_eq!(y, Yields::zero());
        assert_eq!(y.total(), 0);
    }

    #[test]
    fn test_add() {
        let a = Yields::new(2, 1, 0, 0, 0);
        let b = Yields::new(1, 2, 1, 0, 0);
        let sum = a + b;
        assert_eq!(sum.food, 3);
        assert_eq!(sum.production, 3);
        assert_eq!(sum.gold, 1);
    }

    #[test]
    fn test_sub_can_be_negative() {
        let a = Yields::food(1);
        let b = Yields::food(3);
        let diff = a - b;
        assert_eq!(diff.food, -2);
    }

    #[test]
    fn test_clamp_non_negative() {
        let y = Yields::new(-2, 3, -1, 0, 5);
        let clamped = y.clamp_non_negative();
        assert_eq!(clamped.food, 0);
        assert_eq!(clamped.production, 3);
        assert_eq!(clamped.gold, 0);
        assert_eq!(clamped.science, 0);
        assert_eq!(clamped.culture, 5);
    }

    #[test]
    fn test_multiply() {
        let y = Yields::new(2, 4, 3, 1, 0);
        let doubled = y.multiply(2.0);
        assert_eq!(doubled.food, 4);
        assert_eq!(doubled.production, 8);
    }

    #[test]
    fn test_display() {
        let y = Yields::new(2, 1, 3, 0, 0);
        assert_eq!(format!("{}", y), "2F 1P 3G");
    }

    #[test]
    fn test_display_zero() {
        let y = Yields::zero();
        assert_eq!(format!("{}", y), "0");
    }
}

//! Combat system for unit-to-unit and unit-to-city battles.
//!
//! Combat in Nostr Nations uses deterministic formulas combined with
//! Cashu-based randomness for fairness. The combat resolver takes
//! a random value (from Cashu unblinded signature) to determine outcomes.

use crate::map::Tile;
use crate::unit::{Promotion, Unit, UnitCategory};
use serde::{Deserialize, Serialize};

/// Result of a combat engagement.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatResult {
    /// Damage dealt to the defender.
    pub defender_damage: u32,
    /// Damage dealt to the attacker (counter-attack or ranged defense).
    pub attacker_damage: u32,
    /// Whether the defender was destroyed.
    pub defender_destroyed: bool,
    /// Whether the attacker was destroyed.
    pub attacker_destroyed: bool,
    /// Experience gained by attacker.
    pub attacker_xp: u32,
    /// Experience gained by defender.
    pub defender_xp: u32,
    /// Combat log for replay/display.
    pub log: CombatLog,
}

/// Detailed combat log for UI display and replay.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatLog {
    pub attacker_base_strength: u32,
    pub defender_base_strength: u32,
    pub attacker_modifiers: Vec<CombatModifier>,
    pub defender_modifiers: Vec<CombatModifier>,
    pub attacker_final_strength: f32,
    pub defender_final_strength: f32,
    pub random_factor: f32,
}

/// A modifier that affects combat strength.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatModifier {
    pub name: String,
    pub percentage: i32,
}

/// Context for combat calculations.
pub struct CombatContext<'a> {
    pub attacker: &'a Unit,
    pub defender: &'a Unit,
    pub attacker_tile: &'a Tile,
    pub defender_tile: &'a Tile,
    /// Random value from Cashu (0.0 to 1.0).
    pub random: f32,
    /// Is this a ranged attack?
    pub is_ranged: bool,
}

/// Resolve combat between two units.
///
/// The random value should come from a Cashu unblinded signature
/// to ensure neither player can bias the outcome.
pub fn resolve_combat(ctx: &CombatContext) -> CombatResult {
    let mut attacker_modifiers = Vec::new();
    let mut defender_modifiers = Vec::new();

    // Get base strengths
    let attacker_base = if ctx.is_ranged {
        ctx.attacker.effective_ranged_strength()
    } else {
        ctx.attacker.effective_combat_strength()
    };

    let defender_base = ctx.defender.effective_combat_strength();

    // Calculate attacker modifiers
    let attacker_mod = calculate_attacker_modifiers(ctx, &mut attacker_modifiers);

    // Calculate defender modifiers
    let defender_mod = calculate_defender_modifiers(ctx, &mut defender_modifiers);

    // Apply modifiers to get final strengths
    let attacker_final = attacker_base as f32 * (1.0 + attacker_mod / 100.0);
    let defender_final = defender_base as f32 * (1.0 + defender_mod / 100.0);

    // Calculate damage using combat formula
    let (defender_damage, attacker_damage) =
        calculate_damage(attacker_final, defender_final, ctx.random, ctx.is_ranged);

    // Determine outcomes
    let defender_destroyed = ctx.defender.health <= defender_damage;
    let attacker_destroyed = !ctx.is_ranged && ctx.attacker.health <= attacker_damage;

    // Calculate experience
    let (attacker_xp, defender_xp) = calculate_experience(
        attacker_base,
        defender_base,
        defender_damage,
        attacker_damage,
        defender_destroyed,
    );

    CombatResult {
        defender_damage,
        attacker_damage,
        defender_destroyed,
        attacker_destroyed,
        attacker_xp,
        defender_xp,
        log: CombatLog {
            attacker_base_strength: attacker_base,
            defender_base_strength: defender_base,
            attacker_modifiers,
            defender_modifiers,
            attacker_final_strength: attacker_final,
            defender_final_strength: defender_final,
            random_factor: ctx.random,
        },
    }
}

/// Calculate attacker combat modifiers.
fn calculate_attacker_modifiers(ctx: &CombatContext, mods: &mut Vec<CombatModifier>) -> f32 {
    let mut total = 0.0f32;

    // Great General bonus (if nearby, +15%)
    // This would need to check for nearby great generals

    // Flanking bonus: +10% per adjacent friendly military unit
    // This would need access to all units

    // Promotion bonuses
    for promo in &ctx.attacker.promotions {
        let bonus = get_attacker_promotion_bonus(promo, ctx);
        if bonus != 0 {
            mods.push(CombatModifier {
                name: format!("{:?}", promo),
                percentage: bonus,
            });
            total += bonus as f32;
        }
    }

    // Wounded penalty (attacking with low health)
    // Already factored into effective_combat_strength

    total
}

/// Calculate defender combat modifiers.
fn calculate_defender_modifiers(ctx: &CombatContext, mods: &mut Vec<CombatModifier>) -> f32 {
    let mut total = 0.0f32;

    // Terrain defense bonus
    let terrain_bonus = ctx.defender_tile.defense_bonus();
    if terrain_bonus != 0 {
        mods.push(CombatModifier {
            name: "Terrain".to_string(),
            percentage: terrain_bonus,
        });
        total += terrain_bonus as f32;
    }

    // Fortification bonus
    let fort_bonus = ctx.defender.fortification_bonus();
    if fort_bonus != 0 {
        mods.push(CombatModifier {
            name: "Fortified".to_string(),
            percentage: fort_bonus,
        });
        total += fort_bonus as f32;
    }

    // River crossing penalty for attacker (becomes defender bonus)
    // Would need to check if attacker crossed a river

    // Promotion bonuses
    for promo in &ctx.defender.promotions {
        let bonus = get_defender_promotion_bonus(promo, ctx);
        if bonus != 0 {
            mods.push(CombatModifier {
                name: format!("{:?}", promo),
                percentage: bonus,
            });
            total += bonus as f32;
        }
    }

    total
}

/// Get attacker bonus from a promotion.
fn get_attacker_promotion_bonus(promo: &Promotion, ctx: &CombatContext) -> i32 {
    match promo {
        // Shock: bonus vs melee units
        Promotion::ShockI => {
            if ctx.defender.unit_type.stats().category == UnitCategory::Melee {
                15
            } else {
                0
            }
        }
        Promotion::ShockII => {
            if ctx.defender.unit_type.stats().category == UnitCategory::Melee {
                20
            } else {
                0
            }
        }
        Promotion::ShockIII => {
            if ctx.defender.unit_type.stats().category == UnitCategory::Melee {
                25
            } else {
                0
            }
        }
        // Drill: bonus in rough terrain
        Promotion::DrillI => {
            if ctx.defender_tile.feature.is_some() {
                15
            } else {
                0
            }
        }
        Promotion::DrillII => {
            if ctx.defender_tile.feature.is_some() {
                20
            } else {
                0
            }
        }
        Promotion::DrillIII => {
            if ctx.defender_tile.feature.is_some() {
                25
            } else {
                0
            }
        }
        // Accuracy: ranged bonus vs units
        Promotion::AccuracyI if ctx.is_ranged => 15,
        Promotion::AccuracyII if ctx.is_ranged => 20,
        Promotion::AccuracyIII if ctx.is_ranged => 25,
        // Barrage: ranged bonus vs cities (handled separately)
        _ => 0,
    }
}

/// Get defender bonus from a promotion.
fn get_defender_promotion_bonus(promo: &Promotion, ctx: &CombatContext) -> i32 {
    match promo {
        // Cover: defense vs ranged attacks
        Promotion::CoverI if ctx.is_ranged => 25,
        Promotion::CoverII if ctx.is_ranged => 50,
        _ => 0,
    }
}

/// Calculate damage using the combat formula.
///
/// The formula is based on the strength ratio with randomness:
/// - Equal strength: ~30 damage to each side
/// - 2:1 advantage: ~50 to defender, ~15 to attacker
fn calculate_damage(
    attacker_strength: f32,
    defender_strength: f32,
    random: f32,
    is_ranged: bool,
) -> (u32, u32) {
    if attacker_strength <= 0.0 || defender_strength <= 0.0 {
        return (0, 0);
    }

    // Strength ratio
    let ratio = attacker_strength / defender_strength;

    // Base damage calculation (normalized around 30)
    let base_damage = 30.0;

    // Attacker's damage to defender scales with ratio
    // At 1:1 ratio = 30 damage, at 2:1 = ~45 damage, at 0.5:1 = ~20 damage
    let defender_damage_base = base_damage * ratio.powf(0.5);

    // Add randomness (±20%)
    let random_factor = 0.8 + random * 0.4; // 0.8 to 1.2
    let defender_damage = (defender_damage_base * random_factor).round() as u32;

    // Attacker takes counter-attack damage (unless ranged)
    let attacker_damage = if is_ranged {
        0 // Ranged attacks don't receive counter-attack
    } else {
        let attacker_damage_base = base_damage / ratio.powf(0.5);
        let random_factor_def = 0.8 + (1.0 - random) * 0.4;
        (attacker_damage_base * random_factor_def).round() as u32
    };

    (defender_damage.min(100), attacker_damage.min(100))
}

/// Calculate experience gained from combat.
fn calculate_experience(
    attacker_strength: u32,
    defender_strength: u32,
    defender_damage: u32,
    attacker_damage: u32,
    defender_destroyed: bool,
) -> (u32, u32) {
    // Base XP for combat
    let base_xp = 2u32;

    // Bonus for fighting stronger enemies
    let strength_ratio = defender_strength as f32 / attacker_strength.max(1) as f32;
    let strength_bonus = ((strength_ratio - 1.0).max(0.0) * 2.0) as u32;

    // Attacker XP
    let mut attacker_xp = base_xp + strength_bonus;
    if defender_destroyed {
        attacker_xp += 3; // Bonus for kill
    }
    attacker_xp += defender_damage / 10; // Bonus for damage dealt

    // Defender XP (only if survived and dealt damage)
    let defender_xp = if !defender_destroyed && attacker_damage > 0 {
        base_xp + attacker_damage / 10
    } else {
        0
    };

    (attacker_xp, defender_xp)
}

/// Context for city combat.
pub struct CityCombatContext<'a> {
    pub attacker: &'a Unit,
    pub city_strength: u32,
    pub city_health: u32,
    pub attacker_tile: &'a Tile,
    pub random: f32,
    pub is_ranged: bool,
}

/// Result of attacking a city.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CityCombatResult {
    /// Damage dealt to the city.
    pub city_damage: u32,
    /// Damage dealt to the attacker.
    pub attacker_damage: u32,
    /// Was the city captured?
    pub city_captured: bool,
    /// Was the attacker destroyed?
    pub attacker_destroyed: bool,
    /// Experience gained.
    pub attacker_xp: u32,
}

/// Resolve combat against a city.
pub fn resolve_city_combat(ctx: &CityCombatContext) -> CityCombatResult {
    let attacker_strength = if ctx.is_ranged {
        ctx.attacker.effective_ranged_strength()
    } else {
        ctx.attacker.effective_combat_strength()
    };

    // Cities have inherent combat strength
    let city_strength = ctx.city_strength as f32;
    let attacker_final = attacker_strength as f32;

    // Calculate damage
    let ratio = attacker_final / city_strength.max(1.0);
    let base_damage = 20.0; // Lower base damage vs cities

    // Check for Barrage promotions (bonus vs cities)
    let mut city_bonus = 0;
    for promo in &ctx.attacker.promotions {
        match promo {
            Promotion::BarrageI => city_bonus += 15,
            Promotion::BarrageII => city_bonus += 20,
            Promotion::BarrageIII => city_bonus += 25,
            _ => {}
        }
    }

    let city_damage_base = base_damage * ratio.powf(0.5) * (1.0 + city_bonus as f32 / 100.0);
    let random_factor = 0.8 + ctx.random * 0.4;
    let city_damage = (city_damage_base * random_factor).round() as u32;

    // City counter-attack (unless ranged)
    let attacker_damage = if ctx.is_ranged {
        0
    } else {
        let counter_base = base_damage / ratio.powf(0.5);
        let random_factor_def = 0.8 + (1.0 - ctx.random) * 0.4;
        (counter_base * random_factor_def).round() as u32
    };

    let city_captured = ctx.city_health <= city_damage && !ctx.is_ranged;
    let attacker_destroyed = ctx.attacker.health <= attacker_damage;

    CityCombatResult {
        city_damage: city_damage.min(ctx.city_health),
        attacker_damage: attacker_damage.min(100),
        city_captured,
        attacker_destroyed,
        attacker_xp: 3 + city_damage / 10,
    }
}

/// Calculate the combat preview (expected outcome without randomness).
pub fn preview_combat(
    attacker: &Unit,
    defender: &Unit,
    attacker_tile: &Tile,
    defender_tile: &Tile,
    is_ranged: bool,
) -> (u32, u32) {
    let ctx = CombatContext {
        attacker,
        defender,
        attacker_tile,
        defender_tile,
        random: 0.5, // Use average for preview
        is_ranged,
    };

    let result = resolve_combat(&ctx);
    (result.defender_damage, result.attacker_damage)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hex::HexCoord;
    use crate::terrain::{Feature, Terrain};
    use crate::unit::UnitType;

    fn create_test_tile(terrain: Terrain, feature: Option<Feature>) -> Tile {
        let mut tile = Tile::new(HexCoord::new(0, 0), terrain);
        tile.feature = feature;
        tile
    }

    #[test]
    fn test_equal_strength_combat() {
        let attacker = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let defender = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 0));
        let tile = create_test_tile(Terrain::Grassland, None);

        let ctx = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &tile,
            defender_tile: &tile,
            random: 0.5,
            is_ranged: false,
        };

        let result = resolve_combat(&ctx);

        // Both should take similar damage with equal strength
        assert!(result.defender_damage > 0);
        assert!(result.attacker_damage > 0);
        // Damage should be in reasonable range
        assert!(result.defender_damage <= 50);
        assert!(result.attacker_damage <= 50);
    }

    #[test]
    fn test_stronger_attacker() {
        let attacker = Unit::new(1, 0, UnitType::Swordsman, HexCoord::new(0, 0)); // 14 strength
        let defender = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 0)); // 8 strength
        let tile = create_test_tile(Terrain::Grassland, None);

        let ctx = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &tile,
            defender_tile: &tile,
            random: 0.5,
            is_ranged: false,
        };

        let result = resolve_combat(&ctx);

        // Stronger attacker should deal more damage and take less
        assert!(result.defender_damage > result.attacker_damage);
    }

    #[test]
    fn test_terrain_defense_bonus() {
        let attacker = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let defender = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 0));
        let flat_tile = create_test_tile(Terrain::Grassland, None);
        let hill_tile = create_test_tile(Terrain::Grassland, Some(Feature::Hills));

        // Combat on flat ground
        let ctx_flat = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &flat_tile,
            defender_tile: &flat_tile,
            random: 0.5,
            is_ranged: false,
        };
        let result_flat = resolve_combat(&ctx_flat);

        // Combat with defender on hills
        let ctx_hills = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &flat_tile,
            defender_tile: &hill_tile,
            random: 0.5,
            is_ranged: false,
        };
        let result_hills = resolve_combat(&ctx_hills);

        // Defender on hills should take less damage
        assert!(result_hills.defender_damage <= result_flat.defender_damage);
    }

    #[test]
    fn test_ranged_no_counter() {
        let attacker = Unit::new(1, 0, UnitType::Archer, HexCoord::new(0, 0));
        let defender = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(2, 0));
        let tile = create_test_tile(Terrain::Grassland, None);

        let ctx = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &tile,
            defender_tile: &tile,
            random: 0.5,
            is_ranged: true,
        };

        let result = resolve_combat(&ctx);

        // Ranged attack should deal damage but take none
        assert!(result.defender_damage > 0);
        assert_eq!(result.attacker_damage, 0);
    }

    #[test]
    fn test_fortification_bonus() {
        let attacker = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let mut defender = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 0));
        let tile = create_test_tile(Terrain::Grassland, None);

        // Non-fortified combat
        let ctx1 = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &tile,
            defender_tile: &tile,
            random: 0.5,
            is_ranged: false,
        };
        let result1 = resolve_combat(&ctx1);

        // Fortify defender
        defender.fortify();
        defender.fortify_turns = 2; // Max fortification

        let ctx2 = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &tile,
            defender_tile: &tile,
            random: 0.5,
            is_ranged: false,
        };
        let result2 = resolve_combat(&ctx2);

        // Fortified defender should take less damage
        assert!(result2.defender_damage < result1.defender_damage);
    }

    #[test]
    fn test_experience_gain() {
        let attacker = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let defender = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 0));
        let tile = create_test_tile(Terrain::Grassland, None);

        let ctx = CombatContext {
            attacker: &attacker,
            defender: &defender,
            attacker_tile: &tile,
            defender_tile: &tile,
            random: 0.5,
            is_ranged: false,
        };

        let result = resolve_combat(&ctx);

        // Both should gain XP
        assert!(result.attacker_xp > 0);
        assert!(result.defender_xp > 0);
    }

    #[test]
    fn test_combat_preview() {
        let attacker = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let defender = Unit::new(2, 1, UnitType::Warrior, HexCoord::new(1, 0));
        let tile = create_test_tile(Terrain::Grassland, None);

        let (def_dmg, atk_dmg) = preview_combat(&attacker, &defender, &tile, &tile, false);

        assert!(def_dmg > 0);
        assert!(atk_dmg > 0);
    }

    #[test]
    fn test_city_combat() {
        let attacker = Unit::new(1, 0, UnitType::Warrior, HexCoord::new(0, 0));
        let tile = create_test_tile(Terrain::Grassland, None);

        let ctx = CityCombatContext {
            attacker: &attacker,
            city_strength: 10,
            city_health: 200,
            attacker_tile: &tile,
            random: 0.5,
            is_ranged: false,
        };

        let result = resolve_city_combat(&ctx);

        assert!(result.city_damage > 0);
        assert!(result.attacker_damage > 0);
        assert!(!result.city_captured); // City has too much health
    }
}

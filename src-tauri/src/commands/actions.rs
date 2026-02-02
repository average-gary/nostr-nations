//! Game action commands.
//!
//! These commands handle in-game actions like moving units, attacking, and building.

use crate::events::{
    emit_combat_resolved, emit_game_state_updated, emit_notification,
    CombatResolvedPayload, CombatResults, CombatantInfo, GameStateUpdatedPayload,
    NotificationPayload, NotificationType, UnitUpdate,
};
use crate::state::{AppError, AppState};
use nostr_nations_core::{GameAction, HexCoord, Improvement};
use serde::Serialize;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, State};

/// Result of a game action.
#[derive(Clone, Debug, Serialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: Option<String>,
    pub effects: Vec<String>,
}

/// Move a unit to a destination.
#[tauri::command]
pub fn move_unit(
    unit_id: u64,
    path: Vec<(i32, i32)>,
    state: State<'_, Mutex<AppState>>,
) -> Result<ActionResult, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;
    let current_player = engine.state.current_player;

    // Convert path to HexCoords
    let hex_path: Vec<HexCoord> = path
        .into_iter()
        .map(|(q, r)| HexCoord::new(q, r))
        .collect();

    let result = engine.apply_action(
        current_player,
        &GameAction::MoveUnit { unit_id, path: hex_path },
    ).map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    Ok(ActionResult {
        success: result.success,
        message: result.error,
        effects: result.effects.iter().map(|e| format!("{:?}", e)).collect(),
    })
}

/// Attack an enemy unit.
#[tauri::command]
pub fn attack_unit(
    app_handle: AppHandle,
    attacker_id: u64,
    defender_id: u64,
    random: f32,
    state: State<'_, Mutex<AppState>>,
) -> Result<ActionResult, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;
    let current_player = engine.state.current_player;

    // Capture unit info before combat for event emission
    let attacker_info_before = engine.state.units.get(&attacker_id).map(|u| {
        let owner_name = engine.state.players
            .get(u.owner as usize)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {}", u.owner));
        (
            u.owner,
            owner_name,
            format!("{:?}", u.unit_type),
            u.health,
            u.effective_combat_strength(),
            (u.position.q, u.position.r),
        )
    });

    let defender_info_before = engine.state.units.get(&defender_id).map(|u| {
        let owner_name = engine.state.players
            .get(u.owner as usize)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| format!("Player {}", u.owner));
        (
            u.owner,
            owner_name,
            format!("{:?}", u.unit_type),
            u.health,
            u.effective_combat_strength(),
            (u.position.q, u.position.r),
        )
    });

    let result = engine.apply_action(
        current_player,
        &GameAction::AttackUnit {
            attacker_id,
            defender_id,
            random,
        },
    ).map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    // Emit combat event if we have the unit info
    if let (Some(atk_before), Some(def_before)) = (attacker_info_before, defender_info_before) {
        // Get unit info after combat
        let attacker_health_after = engine.state.units.get(&attacker_id)
            .map(|u| u.health)
            .unwrap_or(0);
        let defender_health_after = engine.state.units.get(&defender_id)
            .map(|u| u.health)
            .unwrap_or(0);

        let attacker_destroyed = !engine.state.units.contains_key(&attacker_id);
        let defender_destroyed = !engine.state.units.contains_key(&defender_id);

        let attacker_damage = atk_before.3.saturating_sub(attacker_health_after);
        let defender_damage = def_before.3.saturating_sub(defender_health_after);

        // Clone unit types for use in multiple places
        let attacker_unit_type = atk_before.2.clone();
        let defender_unit_type = def_before.2.clone();

        let combat_payload = CombatResolvedPayload {
            attacker: CombatantInfo {
                unit_id: attacker_id,
                owner_id: atk_before.0,
                owner_name: atk_before.1.clone(),
                unit_type: attacker_unit_type.clone(),
                health_before: atk_before.3,
                health_after: attacker_health_after,
                strength: atk_before.4,
            },
            defender: CombatantInfo {
                unit_id: defender_id,
                owner_id: def_before.0,
                owner_name: def_before.1.clone(),
                unit_type: defender_unit_type.clone(),
                health_before: def_before.3,
                health_after: defender_health_after,
                strength: def_before.4,
            },
            results: CombatResults {
                defender_damage,
                attacker_damage,
                defender_destroyed,
                attacker_destroyed,
                attacker_xp: 5, // Simplified XP calculation
                defender_xp: if defender_destroyed { 0 } else { 2 },
                was_ranged: false, // Would need to check unit type
            },
            position: def_before.5,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        };

        let _ = emit_combat_resolved(&app_handle, combat_payload);

        // Emit notification based on combat outcome
        let notification = if defender_destroyed {
            NotificationPayload {
                notification_type: NotificationType::Combat,
                title: "Victory!".to_string(),
                message: format!("Your unit destroyed the enemy {}!", defender_unit_type),
                icon: Some("sword".to_string()),
                duration_ms: Some(4000),
                action: None,
            }
        } else if attacker_destroyed {
            NotificationPayload {
                notification_type: NotificationType::Combat,
                title: "Unit Lost".to_string(),
                message: format!("Your {} was destroyed in combat!", attacker_unit_type),
                icon: Some("skull".to_string()),
                duration_ms: Some(4000),
                action: None,
            }
        } else {
            NotificationPayload {
                notification_type: NotificationType::Combat,
                title: "Combat".to_string(),
                message: format!(
                    "Combat: {} dealt {} damage, received {} damage",
                    attacker_unit_type, defender_damage, attacker_damage
                ),
                icon: Some("crossed-swords".to_string()),
                duration_ms: Some(3000),
                action: None,
            }
        };
        let _ = emit_notification(&app_handle, notification);

        // Emit partial game state update with changed units
        let mut changed_units = Vec::new();

        if let Some(unit) = engine.state.units.get(&attacker_id) {
            changed_units.push(UnitUpdate {
                id: attacker_id,
                owner: unit.owner,
                unit_type: format!("{:?}", unit.unit_type),
                position: (unit.position.q, unit.position.r),
                health: unit.health,
                movement_remaining: unit.movement,
                is_destroyed: false,
            });
        } else {
            changed_units.push(UnitUpdate {
                id: attacker_id,
                owner: atk_before.0,
                unit_type: attacker_unit_type,
                position: atk_before.5,
                health: 0,
                movement_remaining: 0,
                is_destroyed: true,
            });
        }

        if let Some(unit) = engine.state.units.get(&defender_id) {
            changed_units.push(UnitUpdate {
                id: defender_id,
                owner: unit.owner,
                unit_type: format!("{:?}", unit.unit_type),
                position: (unit.position.q, unit.position.r),
                health: unit.health,
                movement_remaining: unit.movement,
                is_destroyed: false,
            });
        } else {
            changed_units.push(UnitUpdate {
                id: defender_id,
                owner: def_before.0,
                unit_type: defender_unit_type,
                position: def_before.5,
                health: 0,
                movement_remaining: 0,
                is_destroyed: true,
            });
        }

        let game = &engine.state;
        let _ = emit_game_state_updated(
            &app_handle,
            GameStateUpdatedPayload {
                game_id: game.id.clone(),
                phase: format!("{:?}", game.phase),
                turn: game.turn,
                current_player: game.current_player,
                player_count: game.players.len(),
                map_dimensions: game.settings.map_size.dimensions(),
                is_full_update: false,
                changed_units: Some(changed_units),
                changed_cities: None,
                changed_tiles: None,
            },
        );
    }

    Ok(ActionResult {
        success: result.success,
        message: result.error,
        effects: result.effects.iter().map(|e| format!("{:?}", e)).collect(),
    })
}

/// Found a new city.
#[tauri::command]
pub fn found_city(
    settler_id: u64,
    name: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<ActionResult, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;
    let current_player = engine.state.current_player;

    let result = engine.apply_action(
        current_player,
        &GameAction::FoundCity { settler_id, name },
    ).map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    Ok(ActionResult {
        success: result.success,
        message: result.error,
        effects: result.effects.iter().map(|e| format!("{:?}", e)).collect(),
    })
}

/// Build an improvement on a tile.
#[tauri::command]
pub fn build_improvement(
    unit_id: u64,
    improvement: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<ActionResult, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;
    let current_player = engine.state.current_player;

    let improvement_type = match improvement.as_str() {
        "farm" => Improvement::Farm,
        "mine" => Improvement::Mine,
        "pasture" => Improvement::Pasture,
        "plantation" => Improvement::Plantation,
        "camp" => Improvement::Camp,
        "quarry" => Improvement::Quarry,
        "lumber_mill" => Improvement::LumberMill,
        "trading_post" => Improvement::TradingPost,
        "fort" => Improvement::Fort,
        _ => return Err(AppError::InvalidState(format!("Unknown improvement: {}", improvement))),
    };

    let result = engine.apply_action(
        current_player,
        &GameAction::BuildImprovement {
            unit_id,
            improvement: improvement_type,
        },
    ).map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    Ok(ActionResult {
        success: result.success,
        message: result.error,
        effects: result.effects.iter().map(|e| format!("{:?}", e)).collect(),
    })
}

/// Set the research target for the player.
#[tauri::command]
pub fn set_research(
    tech_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<ActionResult, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;
    let current_player = engine.state.current_player;

    let result = engine.apply_action(
        current_player,
        &GameAction::SetResearch { tech_id },
    ).map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    Ok(ActionResult {
        success: result.success,
        message: result.error,
        effects: result.effects.iter().map(|e| format!("{:?}", e)).collect(),
    })
}

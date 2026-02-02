//! Game management commands.
//!
//! These commands handle game lifecycle: creation, joining, starting, and state queries.

use crate::events::{
    emit_game_state_updated, emit_notification, emit_turn_event,
    GameStateUpdatedPayload, NotificationPayload, TurnEventPayload,
};
use crate::state::{AppError, AppState};
use nostr_nations_core::{GameAction, GamePhase, GameSettings, GameSpeed, Difficulty, MapSize};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, State};

/// Response for game state queries.
#[derive(Clone, Debug, Serialize)]
pub struct GameStateResponse {
    pub game_id: String,
    pub phase: String,
    pub turn: u32,
    pub current_player: u8,
    pub player_count: usize,
    pub map_width: u32,
    pub map_height: u32,
}

/// Options for creating a new game.
#[derive(Clone, Debug, Deserialize)]
pub struct CreateGameOptions {
    pub name: String,
    pub player_name: String,
    pub civilization: String,
    pub map_size: String,
    pub difficulty: String,
    pub game_speed: String,
    pub seed: Option<String>,
}

/// Create a new game.
#[tauri::command]
pub fn create_game(
    options: CreateGameOptions,
    state: State<'_, Mutex<AppState>>,
) -> Result<GameStateResponse, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    if state.has_active_game() {
        return Err(AppError::GameAlreadyActive);
    }

    // Create settings from options
    let mut settings = GameSettings::new(options.name.clone());
    settings.map_size = match options.map_size.as_str() {
        "duel" => MapSize::Duel,
        "small" => MapSize::Small,
        "standard" => MapSize::Standard,
        "large" => MapSize::Large,
        "huge" => MapSize::Huge,
        _ => MapSize::Standard,
    };
    settings.game_speed = match options.game_speed.as_str() {
        "quick" => GameSpeed::Quick,
        "normal" | "standard" => GameSpeed::Normal,
        "epic" => GameSpeed::Epic,
        "marathon" => GameSpeed::Marathon,
        _ => GameSpeed::Normal,
    };
    settings.difficulty = match options.difficulty.as_str() {
        "settler" => Difficulty::Settler,
        "chieftain" => Difficulty::Chieftain,
        "normal" | "prince" => Difficulty::Normal,
        "king" => Difficulty::King,
        "emperor" => Difficulty::Emperor,
        "deity" | "immortal" => Difficulty::Deity,
        _ => Difficulty::Normal,
    };

    // Generate seed
    let seed: [u8; 32] = if let Some(seed_str) = options.seed {
        let mut seed = [0u8; 32];
        let bytes = seed_str.as_bytes();
        for (i, b) in bytes.iter().take(32).enumerate() {
            seed[i] = *b;
        }
        seed
    } else {
        let mut seed = [0u8; 32];
        for (i, b) in std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_le_bytes()
            .iter()
            .enumerate()
        {
            if i < 32 {
                seed[i] = *b;
            }
        }
        seed
    };

    state.create_game(settings, seed)?;

    // Join as first player
    let engine = state.get_engine_mut()?;
    engine.apply_action(
        0,
        &GameAction::JoinGame {
            player_name: options.player_name,
            civilization_id: options.civilization,
        },
    ).map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    // Return game state
    let game = state.get_game_state()?;
    Ok(GameStateResponse {
        game_id: game.id.clone(),
        phase: format!("{:?}", game.phase),
        turn: game.turn,
        current_player: game.current_player,
        player_count: game.players.len(),
        map_width: game.settings.map_size.dimensions().0,
        map_height: game.settings.map_size.dimensions().1,
    })
}

/// Join an existing game.
#[tauri::command]
pub fn join_game(
    player_name: String,
    civilization: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<GameStateResponse, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;
    let player_id = engine.state.players.len() as u8;

    engine.apply_action(
        player_id,
        &GameAction::JoinGame {
            player_name,
            civilization_id: civilization,
        },
    ).map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    let game = &engine.state;
    Ok(GameStateResponse {
        game_id: game.id.clone(),
        phase: format!("{:?}", game.phase),
        turn: game.turn,
        current_player: game.current_player,
        player_count: game.players.len(),
        map_width: game.settings.map_size.dimensions().0,
        map_height: game.settings.map_size.dimensions().1,
    })
}

/// Start the game (transitions from Setup to Playing).
#[tauri::command]
pub fn start_game(
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<GameStateResponse, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;

    if engine.state.phase != GamePhase::Setup {
        return Err(AppError::InvalidState("Game is not in setup phase".to_string()));
    }

    engine.apply_action(0, &GameAction::StartGame)
        .map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    let game = &engine.state;

    // Get first player name
    let first_player_name = game.players
        .first()
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "Player 0".to_string());

    // Emit game state update for game start
    let _ = emit_game_state_updated(
        &app_handle,
        GameStateUpdatedPayload {
            game_id: game.id.clone(),
            phase: format!("{:?}", game.phase),
            turn: game.turn,
            current_player: game.current_player,
            player_count: game.players.len(),
            map_dimensions: game.settings.map_size.dimensions(),
            is_full_update: true,
            changed_units: None,
            changed_cities: None,
            changed_tiles: None,
        },
    );

    // Emit turn started event for turn 1
    let _ = emit_turn_event(
        &app_handle,
        TurnEventPayload::turn_started(
            game.turn,
            game.current_player,
            first_player_name.clone(),
            game.current_player == 0, // Assuming player 0 is local
        ),
    );

    // Emit notification for game start
    let _ = emit_notification(
        &app_handle,
        NotificationPayload::success(
            "Game Started",
            format!("The game has begun! {} players competing.", game.players.len()),
        ),
    );

    Ok(GameStateResponse {
        game_id: game.id.clone(),
        phase: format!("{:?}", game.phase),
        turn: game.turn,
        current_player: game.current_player,
        player_count: game.players.len(),
        map_width: game.settings.map_size.dimensions().0,
        map_height: game.settings.map_size.dimensions().1,
    })
}

/// Get the current game state.
#[tauri::command]
pub fn get_game_state(
    state: State<'_, Mutex<AppState>>,
) -> Result<GameStateResponse, AppError> {
    let state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let game = state.get_game_state()?;
    Ok(GameStateResponse {
        game_id: game.id.clone(),
        phase: format!("{:?}", game.phase),
        turn: game.turn,
        current_player: game.current_player,
        player_count: game.players.len(),
        map_width: game.settings.map_size.dimensions().0,
        map_height: game.settings.map_size.dimensions().1,
    })
}

/// End the current player's turn.
#[tauri::command]
pub fn end_turn(
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<GameStateResponse, AppError> {
    let mut state = state.lock().map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    let engine = state.get_engine_mut()?;
    let previous_player = engine.state.current_player;
    let previous_turn = engine.state.turn;

    // Get player name before applying action
    let previous_player_name = engine.state.players
        .get(previous_player as usize)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Player {}", previous_player));

    engine.apply_action(previous_player, &GameAction::EndTurn)
        .map_err(|e| AppError::InvalidState(format!("{:?}", e)))?;

    let game = &engine.state;
    let new_player = game.current_player;
    let new_turn = game.turn;

    // Get new player name
    let new_player_name = game.players
        .get(new_player as usize)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| format!("Player {}", new_player));

    // Emit turn ended event for the previous player
    let _ = emit_turn_event(
        &app_handle,
        TurnEventPayload::turn_ended(
            previous_turn,
            previous_player,
            previous_player_name,
            true, // Assuming local player ended their turn
        ),
    );

    // Check if a new turn has started (turn number increased)
    if new_turn > previous_turn {
        let _ = emit_turn_event(
            &app_handle,
            TurnEventPayload::turn_started(
                new_turn,
                new_player,
                new_player_name.clone(),
                new_player == 0, // Assuming player 0 is local
            ),
        );
    }

    // Emit player turn event for the new current player
    let _ = emit_turn_event(
        &app_handle,
        TurnEventPayload::player_turn(
            new_turn,
            new_player,
            new_player_name.clone(),
            new_player == 0, // Assuming player 0 is local
        ),
    );

    // Emit game state update
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
            changed_units: None,
            changed_cities: None,
            changed_tiles: None,
        },
    );

    // Emit notification if it's now the local player's turn
    if new_player == 0 {
        let _ = emit_notification(
            &app_handle,
            NotificationPayload::info(
                "Your Turn",
                format!("Turn {} has begun. It's your move!", new_turn),
            ),
        );
    }

    Ok(GameStateResponse {
        game_id: game.id.clone(),
        phase: format!("{:?}", game.phase),
        turn: game.turn,
        current_player: game.current_player,
        player_count: game.players.len(),
        map_width: game.settings.map_size.dimensions().0,
        map_height: game.settings.map_size.dimensions().1,
    })
}

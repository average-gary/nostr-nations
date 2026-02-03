//! Save game management commands.
//!
//! These commands handle saving, loading, and managing saved games.

use crate::state::{AppError, AppState};
use nostr_nations_core::GameEngine;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

/// Information about a saved game (metadata stored separately from full state).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedGame {
    pub id: String,
    pub name: String,
    pub save_date: String,
    pub turn: u32,
    pub civilization: String,
    pub map_size: String,
}

/// Full saved game data (includes complete game state).
#[derive(Clone, Debug, Serialize, Deserialize)]
struct SaveData {
    pub metadata: SavedGame,
    pub game_state: nostr_nations_core::GameState,
    pub seed: [u8; 32],
}

/// Get the saves directory path.
fn get_saves_dir(app_handle: &AppHandle) -> Result<PathBuf, AppError> {
    let app_data_dir = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| AppError::InvalidState(format!("Failed to get app data dir: {}", e)))?;

    let saves_dir = app_data_dir.join("saves");

    // Create saves directory if it doesn't exist
    if !saves_dir.exists() {
        fs::create_dir_all(&saves_dir)
            .map_err(|e| AppError::InvalidState(format!("Failed to create saves dir: {}", e)))?;
    }

    Ok(saves_dir)
}

/// Get the path for a specific save file.
fn get_save_path(app_handle: &AppHandle, save_id: &str) -> Result<PathBuf, AppError> {
    let saves_dir = get_saves_dir(app_handle)?;
    Ok(saves_dir.join(format!("{}.json", save_id)))
}

/// List all saved games.
#[tauri::command]
pub fn list_saved_games(app_handle: AppHandle) -> Result<Vec<SavedGame>, AppError> {
    let saves_dir = get_saves_dir(&app_handle)?;

    let mut saves = Vec::new();

    if let Ok(entries) = fs::read_dir(&saves_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |ext| ext == "json") {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(save_data) = serde_json::from_str::<SaveData>(&content) {
                        saves.push(save_data.metadata);
                    }
                }
            }
        }
    }

    // Sort by save date (newest first)
    saves.sort_by(|a, b| b.save_date.cmp(&a.save_date));

    Ok(saves)
}

/// Response for loading a game.
#[derive(Clone, Debug, Serialize)]
pub struct LoadGameResponse {
    pub game_id: String,
}

/// Load a saved game.
#[tauri::command]
pub fn load_game(
    save_id: String,
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<LoadGameResponse, AppError> {
    let mut app_state = state
        .lock()
        .map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    // Check if there's already an active game
    if app_state.has_active_game() {
        return Err(AppError::GameAlreadyActive);
    }

    // Read save file
    let save_path = get_save_path(&app_handle, &save_id)?;
    let content = fs::read_to_string(&save_path)
        .map_err(|e| AppError::InvalidState(format!("Failed to read save file: {}", e)))?;

    let save_data: SaveData = serde_json::from_str(&content)
        .map_err(|e| AppError::InvalidState(format!("Failed to parse save file: {}", e)))?;

    // Reconstruct game engine from saved state
    let engine = GameEngine::from_state(save_data.game_state, save_data.seed);
    let game_id = engine.state.id.clone();

    app_state.game_engine = Some(engine);

    Ok(LoadGameResponse { game_id })
}

/// Save the current game.
#[tauri::command]
pub fn save_game(
    name: String,
    app_handle: AppHandle,
    state: State<'_, Mutex<AppState>>,
) -> Result<SavedGame, AppError> {
    let app_state = state
        .lock()
        .map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    // Check if there's an active game to save
    let engine = app_state
        .game_engine
        .as_ref()
        .ok_or(AppError::NoActiveGame)?;

    let game = &engine.state;
    let save_id = format!("save-{}", uuid::Uuid::new_v4());

    let metadata = SavedGame {
        id: save_id.clone(),
        name: name.clone(),
        save_date: chrono::Utc::now().to_rfc3339(),
        turn: game.turn,
        civilization: game
            .players
            .first()
            .map(|p| p.civilization.name.clone())
            .unwrap_or_else(|| "Unknown".to_string()),
        map_size: format!("{:?}", game.settings.map_size),
    };

    let save_data = SaveData {
        metadata: metadata.clone(),
        game_state: game.clone(),
        seed: game.seed,
    };

    // Write to file
    let save_path = get_save_path(&app_handle, &save_id)?;
    let content = serde_json::to_string_pretty(&save_data)
        .map_err(|e| AppError::SerializationError(format!("Failed to serialize save: {}", e)))?;

    fs::write(&save_path, content)
        .map_err(|e| AppError::InvalidState(format!("Failed to write save file: {}", e)))?;

    Ok(metadata)
}

/// Delete a saved game.
#[tauri::command]
pub fn delete_saved_game(save_id: String, app_handle: AppHandle) -> Result<(), AppError> {
    let save_path = get_save_path(&app_handle, &save_id)?;

    if !save_path.exists() {
        return Err(AppError::InvalidState(format!(
            "Save '{}' not found",
            save_id
        )));
    }

    fs::remove_file(&save_path)
        .map_err(|e| AppError::InvalidState(format!("Failed to delete save file: {}", e)))?;

    Ok(())
}

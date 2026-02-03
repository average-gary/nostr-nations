//! Nostr Nations - Tauri Application
//!
//! This is the main entry point for the Tauri desktop application.
//! It bridges the Rust game engine with the React frontend.

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
pub mod events;
mod state;

use state::AppState;
use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(AppState::new()))
        .invoke_handler(tauri::generate_handler![
            commands::game::create_game,
            commands::game::join_game,
            commands::game::start_game,
            commands::game::get_game_state,
            commands::game::end_game,
            commands::game::end_turn,
            commands::actions::move_unit,
            commands::actions::attack_unit,
            commands::actions::found_city,
            commands::actions::build_improvement,
            commands::actions::set_research,
            commands::network::connect_peer,
            commands::network::disconnect_peer,
            commands::network::get_connection_ticket,
            commands::network::scan_qr_code,
            commands::saves::list_saved_games,
            commands::saves::load_game,
            commands::saves::save_game,
            commands::saves::delete_saved_game,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

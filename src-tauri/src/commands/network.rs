//! Network commands.
//!
//! These commands handle P2P networking: peer connections, QR codes, and sync.

use crate::events::{
    emit_network_event, emit_notification, NetworkEventPayload, NotificationPayload,
};
use crate::state::{AppError, AppState};
use nostr_nations_network::ConnectionTicket;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, State};

/// Connection status response.
#[derive(Clone, Debug, Serialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub peer_count: usize,
    pub ticket: Option<String>,
}

/// Ticket info for serialization.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TicketInfo {
    pub node_id: String,
    pub addresses: Vec<String>,
    pub game_id: Option<String>,
    pub expires_at: u64,
}

/// Connect to a peer using a connection ticket.
#[tauri::command]
pub fn connect_peer(
    app_handle: AppHandle,
    ticket: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<ConnectionStatus, AppError> {
    let mut state = state
        .lock()
        .map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    // Parse the ticket (using the existing string format)
    let connection_ticket = ConnectionTicket::from_string(&ticket).map_err(|e| {
        // Emit connection error event
        let _ = emit_network_event(
            &app_handle,
            NetworkEventPayload::connection_error(
                format!("Invalid ticket: {}", e),
                state.connected_peers(),
            ),
        );
        AppError::NetworkError(format!("Invalid ticket: {}", e))
    })?;

    // Add peer (simplified for now)
    state.add_peer();
    let peer_count = state.connected_peers();

    // Emit peer connected event
    let _ = emit_network_event(
        &app_handle,
        NetworkEventPayload::peer_connected(
            connection_ticket.node_id.clone(),
            None, // Peer name not known yet
            peer_count,
        ),
    );

    // Emit notification
    let _ = emit_notification(
        &app_handle,
        NotificationPayload::success(
            "Peer Connected",
            format!("Successfully connected. {} peer(s) online.", peer_count),
        ),
    );

    Ok(ConnectionStatus {
        connected: true,
        peer_count,
        ticket: Some(ticket),
    })
}

/// Disconnect from a peer.
#[tauri::command]
pub fn disconnect_peer(
    app_handle: AppHandle,
    peer_id: String,
    state: State<'_, Mutex<AppState>>,
) -> Result<ConnectionStatus, AppError> {
    let mut state = state
        .lock()
        .map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    if state.connected_peers() == 0 {
        return Err(AppError::NetworkError(
            "Not connected to any peers".to_string(),
        ));
    }

    state.remove_peer();
    let peer_count = state.connected_peers();

    // Emit peer disconnected event
    let _ = emit_network_event(
        &app_handle,
        NetworkEventPayload::peer_disconnected(
            peer_id, None, // Peer name
            peer_count,
        ),
    );

    // Emit notification
    let _ = emit_notification(
        &app_handle,
        NotificationPayload::info(
            "Peer Disconnected",
            format!("{} peer(s) remaining.", peer_count),
        ),
    );

    Ok(ConnectionStatus {
        connected: peer_count > 0,
        peer_count,
        ticket: None,
    })
}

/// Get a connection ticket for others to connect to this client.
#[tauri::command]
pub fn get_connection_ticket(state: State<'_, Mutex<AppState>>) -> Result<String, AppError> {
    let state = state
        .lock()
        .map_err(|_| AppError::InvalidState("Lock poisoned".to_string()))?;

    // Generate a connection ticket
    // In a real implementation, this would include the Iroh endpoint info
    let game_id = state
        .game_engine
        .as_ref()
        .map(|e| e.state.id.clone())
        .unwrap_or_else(|| "no_game".to_string());

    let ticket = ConnectionTicket::new(
        "local_node_id".to_string(),
        vec!["127.0.0.1:9000".to_string()],
        game_id,
        3600, // 1 hour TTL
    );

    ticket
        .to_string()
        .map_err(|e| AppError::SerializationError(e.to_string()))
}

/// Process a scanned QR code (extract connection ticket).
#[tauri::command]
pub fn scan_qr_code(qr_data: String) -> Result<TicketInfo, AppError> {
    // QR codes should contain "nn:" prefix followed by base64 ticket
    let data = qr_data.strip_prefix("nn:").unwrap_or(&qr_data);

    let ticket = ConnectionTicket::from_string(data)
        .map_err(|e| AppError::NetworkError(format!("Invalid QR code: {}", e)))?;

    Ok(TicketInfo {
        node_id: ticket.node_id,
        addresses: ticket.addresses,
        game_id: Some(ticket.game_id),
        expires_at: ticket.expires_at,
    })
}

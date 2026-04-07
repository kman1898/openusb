use crate::state::AppState;
use axum::extract::State;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use std::sync::Arc;
use tracing::{info, warn};

/// GET /api/v1/events — upgrades to WebSocket for real-time event streaming.
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

async fn handle_connection(mut socket: WebSocket, state: Arc<AppState>) {
    info!("WebSocket client connected");

    // Send initial snapshot of all devices
    let devices: Vec<_> = state.devices.read().await.values().cloned().collect();
    let snapshot = serde_json::json!({
        "type": "snapshot",
        "devices": devices,
    });
    if let Ok(json) = serde_json::to_string(&snapshot)
        && socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }

    // Subscribe to the event broadcast channel
    let mut rx = state.event_tx.subscribe();

    loop {
        tokio::select! {
            // Forward server events to the WebSocket client
            event = rx.recv() => {
                match event {
                    Ok(event) => {
                        let json = match serde_json::to_string(&event) {
                            Ok(j) => j,
                            Err(_) => continue,
                        };
                        if socket.send(Message::Text(json.into())).await.is_err() {
                            break; // Client disconnected
                        }
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                        warn!(skipped = n, "WebSocket client lagged behind event stream");
                    }
                    Err(_) => break, // Channel closed
                }
            }
            // Handle incoming messages from the client
            msg = socket.recv() => {
                match msg {
                    Some(Ok(Message::Ping(data))) => {
                        let _ = socket.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {} // Ignore other messages for now
                }
            }
        }
    }

    info!("WebSocket client disconnected");
}

use crate::state::AppState;
use anyhow::Result;
use openusb_shared::protocol::ServerEvent;
use std::sync::Arc;
use tracing::info;

/// Background task that logs all server events and records them to history.
pub async fn run_event_logger(state: Arc<AppState>) -> Result<()> {
    let mut rx = state.event_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(event) => {
                info!(event = ?event, "Server event");
                record_to_history(&state, &event).await;
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!(skipped = n, "Event logger fell behind");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                break;
            }
        }
    }

    Ok(())
}

async fn record_to_history(state: &AppState, event: &ServerEvent) {
    let db = state.history_db.lock().await;
    let result = match event {
        ServerEvent::DeviceAttached { device } => db.record(
            "device_attached",
            Some(&device.bus_id),
            Some(device.display_name()),
            None,
            None,
            Some(&device.vid_pid()),
        ),
        ServerEvent::DeviceDetached { bus_id } => {
            db.record("device_detached", Some(bus_id), None, None, None, None)
        }
        ServerEvent::DeviceShared { bus_id } => {
            db.record("device_shared", Some(bus_id), None, None, None, None)
        }
        ServerEvent::DeviceUnshared { bus_id } => {
            db.record("device_unshared", Some(bus_id), None, None, None, None)
        }
        ServerEvent::ClientConnected {
            client_ip,
            client_name,
        } => db.record(
            "client_connected",
            None,
            None,
            Some(client_ip),
            None,
            client_name.as_deref(),
        ),
        ServerEvent::ClientDisconnected { client_ip } => {
            db.record("client_disconnected", None, None, Some(client_ip), None, None)
        }
        ServerEvent::DeviceInUse {
            bus_id, client_ip, ..
        } => db.record(
            "device_in_use",
            Some(bus_id),
            None,
            Some(client_ip),
            None,
            None,
        ),
        ServerEvent::DeviceReleased { bus_id } => {
            db.record("device_released", Some(bus_id), None, None, None, None)
        }
        ServerEvent::AuthFailed {
            client_ip, reason, ..
        } => db.record(
            "auth_failed",
            None,
            None,
            Some(client_ip),
            None,
            Some(reason),
        ),
        ServerEvent::BandwidthAlert {
            bus_id,
            bytes_per_sec,
        } => db.record(
            "bandwidth_alert",
            Some(bus_id),
            None,
            None,
            None,
            Some(&format!("{} bytes/sec", bytes_per_sec)),
        ),
    };

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to record event to history");
    }
}

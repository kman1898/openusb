use crate::config::EventsSection;
use crate::state::AppState;
use anyhow::Result;
use openusb_shared::protocol::ServerEvent;
use std::sync::Arc;
use tracing::{info, warn};

/// Background task that runs event hooks (scripts) in response to server events.
pub async fn run_event_hooks(state: Arc<AppState>) -> Result<()> {
    let mut rx = state.event_tx.subscribe();
    let config = &state.config.events;

    loop {
        match rx.recv().await {
            Ok(event) => {
                if let Err(e) = handle_event(&event, config).await {
                    warn!(error = %e, "Event hook failed");
                }
            }
            Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                warn!(skipped = n, "Event hooks fell behind");
            }
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        }
    }
    Ok(())
}

async fn handle_event(event: &ServerEvent, config: &EventsSection) -> Result<()> {
    match event {
        ServerEvent::DeviceAttached { device } => {
            if !config.on_attach.is_empty() {
                run_script(
                    &config.on_attach,
                    &[
                        ("BUS_ID", &device.bus_id),
                        ("VID_PID", &device.vid_pid()),
                        ("DEVICE_NAME", device.display_name()),
                        ("EVENT", "attach"),
                    ],
                )
                .await?;
            }
        }
        ServerEvent::DeviceDetached { bus_id } => {
            if !config.on_detach.is_empty() {
                run_script(
                    &config.on_detach,
                    &[("BUS_ID", bus_id), ("EVENT", "detach")],
                )
                .await?;
            }
        }
        ServerEvent::ClientConnected {
            client_ip,
            client_name,
        } => {
            if !config.on_client_connect.is_empty() {
                let name = client_name.as_deref().unwrap_or("unknown");
                run_script(
                    &config.on_client_connect,
                    &[
                        ("CLIENT_IP", client_ip),
                        ("CLIENT_NAME", name),
                        ("EVENT", "client_connect"),
                    ],
                )
                .await?;
            }
        }
        ServerEvent::ClientDisconnected { client_ip } => {
            if !config.on_client_disconnect.is_empty() {
                run_script(
                    &config.on_client_disconnect,
                    &[("CLIENT_IP", client_ip), ("EVENT", "client_disconnect")],
                )
                .await?;
            }
        }
        _ => {}
    }
    Ok(())
}

async fn run_script(script: &str, env_vars: &[(&str, &str)]) -> Result<()> {
    info!(script, "Running event hook");

    let mut cmd = tokio::process::Command::new("sh");
    cmd.arg("-c").arg(script);

    for (key, value) in env_vars {
        cmd.env(format!("OPENUSB_{}", key), value);
    }

    let output = cmd.output().await?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        warn!(script, stderr = %stderr.trim(), "Event hook failed");
    }

    Ok(())
}

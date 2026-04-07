use crate::state::AppState;
use anyhow::Result;
use std::sync::Arc;
use tracing::info;

/// Background task that logs all server events.
pub async fn run_event_logger(state: Arc<AppState>) -> Result<()> {
    let mut rx = state.event_tx.subscribe();

    loop {
        match rx.recv().await {
            Ok(event) => {
                info!(event = ?event, "Server event");
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

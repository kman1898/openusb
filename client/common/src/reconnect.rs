use std::time::Duration;
use tracing::{info, warn};

/// Auto-reconnect loop that attempts to re-attach a device with exponential backoff.
pub async fn reconnect_loop(
    server: String,
    bus_id: String,
    base_delay: Duration,
    max_delay: Duration,
) {
    let mut delay = base_delay;

    loop {
        tokio::time::sleep(delay).await;

        info!(server = %server, bus_id = %bus_id, "Attempting reconnect");

        match crate::usbip::attach(&server, &bus_id).await {
            Ok(()) => {
                info!(server = %server, bus_id = %bus_id, "Reconnected successfully");
                return;
            }
            Err(e) => {
                warn!(server = %server, bus_id = %bus_id, delay = ?delay, "Reconnect failed: {}", e);
                delay = (delay * 2).min(max_delay);
            }
        }
    }
}

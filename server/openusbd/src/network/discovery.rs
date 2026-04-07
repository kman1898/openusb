use crate::state::AppState;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Register the OpenUSB service via mDNS for auto-discovery on the local network.
/// This function blocks until shutdown.
pub async fn run_mdns(state: Arc<AppState>) -> Result<()> {
    if !state.config.discovery.enabled {
        info!("mDNS discovery disabled in configuration");
        std::future::pending::<()>().await;
        return Ok(());
    }

    let service_type = format!("{}.", state.config.discovery.mdns_name);
    let hostname = state
        .config
        .server
        .hostname
        .clone()
        .unwrap_or_else(|| gethostname::gethostname().to_string_lossy().to_string());

    let mut properties = HashMap::new();
    properties.insert("version".to_string(), env!("CARGO_PKG_VERSION").to_string());
    properties.insert(
        "api_port".to_string(),
        state.config.server.api_port.to_string(),
    );
    properties.insert("name".to_string(), state.config.server.name.clone());

    let mdns = mdns_sd::ServiceDaemon::new()
        .map_err(|e| anyhow::anyhow!("Failed to create mDNS daemon: {e}"))?;

    let service_info = mdns_sd::ServiceInfo::new(
        &service_type,
        &state.config.server.name,
        &format!("{hostname}.local."),
        "", // auto-detect IP
        state.config.server.api_port,
        properties,
    )
    .map_err(|e| anyhow::anyhow!("Failed to create mDNS service info: {e}"))?;

    mdns.register(service_info)
        .map_err(|e| anyhow::anyhow!("Failed to register mDNS service: {e}"))?;

    info!(
        service_type = %state.config.discovery.mdns_name,
        name = %state.config.server.name,
        hostname,
        "mDNS service registered"
    );

    // Keep the daemon alive — it runs its own background thread
    std::future::pending::<()>().await;
    Ok(())
}

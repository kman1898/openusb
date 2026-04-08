#![allow(dead_code)]

mod api;
mod auth;
mod config;
mod events;
mod metrics;
mod network;
mod plugins;
mod scheduling;
mod state;
mod usb;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing::info;
use usb::manager::DeviceManager;

#[derive(Parser)]
#[command(name = "openusbd", about = "OpenUSB server daemon")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "/etc/openusb/openusb.toml")]
    config: String,

    /// Override log level
    #[arg(short, long)]
    log_level: Option<String>,

    /// Run in simulation mode with fake USB devices (auto-enabled on non-Linux)
    #[arg(long)]
    simulate: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = config::ServerConfig::load(&args.config)?;

    // Initialize logging
    let log_level = args.log_level.as_deref().unwrap_or(&config.logging.level);
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    // Determine simulation mode — auto-enable on non-Linux
    let simulate = args.simulate || cfg!(not(target_os = "linux"));
    if simulate && !args.simulate {
        info!("Non-Linux OS detected, auto-enabling simulation mode");
    }

    info!(
        name = %config.server.name,
        usbip_port = config.server.port,
        api_port = config.server.api_port,
        simulate,
        "Starting OpenUSB server"
    );

    // Create platform backend
    let platform = usb::create_platform(simulate);

    // Initialize databases
    let db_path = if simulate {
        "openusb-dev.db".to_string()
    } else {
        config.security.db_path.clone()
    };
    let user_db = auth::users::UserDb::open(&db_path)?;
    let history_db = metrics::history::HistoryDb::open(&db_path)?;

    // Build application state
    let state = Arc::new(state::AppState::new(config, platform, user_db, history_db));

    // Initial device enumeration
    let manager = DeviceManager::new(state.clone());
    manager.initial_enumerate().await?;

    // Spawn all subsystems
    let mut join_set = tokio::task::JoinSet::new();

    // Hotplug monitor
    let hotplug_manager = manager.clone();
    join_set.spawn(async move { hotplug_manager.run_hotplug_monitor().await });

    // mDNS service advertisement
    let mdns_state = state.clone();
    join_set.spawn(async move { network::discovery::run_mdns(mdns_state).await });

    // REST API + WebSocket server
    let api_state = state.clone();
    join_set.spawn(async move { api::rest::start_api_server(api_state).await });

    // Event logger + history recorder
    let log_state = state.clone();
    join_set.spawn(async move { events::bus::run_event_logger(log_state).await });

    // Event hooks (shell scripts)
    let hooks_state = state.clone();
    join_set.spawn(async move { events::hooks::run_event_hooks(hooks_state).await });

    info!("OpenUSB server running");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    join_set.shutdown().await;

    Ok(())
}

// Scaffolding phase — most fields/modules are defined but not yet wired up.
#![allow(dead_code)]

mod api;
mod auth;
mod config;
mod events;
mod metrics;
mod network;
mod plugins;
mod scheduling;
mod usb;

use anyhow::Result;
use clap::Parser;
use tracing::info;

#[derive(Parser)]
#[command(name = "openusbd", about = "OpenUSB server daemon")]
struct Args {
    /// Path to configuration file
    #[arg(short, long, default_value = "/etc/openusb/openusb.toml")]
    config: String,

    /// Override log level
    #[arg(short, long)]
    log_level: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Load configuration
    let config = config::ServerConfig::load(&args.config)?;

    // Initialize logging
    let log_level = args.log_level.as_deref().unwrap_or(&config.logging.level);

    tracing_subscriber::fmt().with_env_filter(log_level).init();

    info!(
        name = %config.server.name,
        usbip_port = config.server.port,
        api_port = config.server.api_port,
        "Starting OpenUSB server"
    );

    // TODO Phase 1: Initialize subsystems
    // 1. Start USB device monitor (udev hotplug)
    // 2. Start mDNS service advertisement
    // 3. Start REST API + WebSocket server
    // 4. Start USB/IP listener
    // 5. Run until signal

    info!("OpenUSB server scaffolding ready — implementation coming in Phase 1");

    Ok(())
}

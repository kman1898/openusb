use anyhow::Result;
use clap::Parser;
use openusb_client_common::api::{LocalApiState, start_local_api};
use openusb_client_common::config::ClientConfig;
use openusb_client_common::discovery::ServiceBrowser;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

mod menubar;

#[derive(Parser)]
#[command(name = "openusb-client", about = "OpenUSB macOS client service")]
struct Args {
    /// Override log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Run without menu bar icon (headless mode)
    #[arg(long)]
    headless: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    tracing_subscriber::fmt()
        .with_env_filter(&args.log_level)
        .init();

    let config = ClientConfig::load();
    info!("OpenUSB macOS client starting");

    let browser = Arc::new(ServiceBrowser::new());

    let api_state = Arc::new(LocalApiState {
        config: RwLock::new(config),
        browser: browser.clone(),
    });

    let mut join_set = tokio::task::JoinSet::new();

    // mDNS service browser
    let mdns_browser = browser.clone();
    join_set.spawn(async move { mdns_browser.run().await });

    // Local API server on localhost:9245
    let api = api_state.clone();
    join_set.spawn(async move { start_local_api(api).await });

    info!("OpenUSB client running (API on localhost:9245)");

    tokio::signal::ctrl_c().await?;
    info!("Shutting down...");
    join_set.shutdown().await;

    Ok(())
}

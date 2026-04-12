use anyhow::Result;
use clap::Parser;
use openusb_client_common::config::ClientConfig;
use tracing::info;

#[derive(Parser)]
#[command(name = "openusb-client", about = "OpenUSB Linux client")]
struct Args {
    /// Override log level
    #[arg(short, long, default_value = "info")]
    log_level: String,

    /// Run without system tray (headless mode, for systemd service)
    #[arg(long)]
    headless: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let config = ClientConfig::load();

    if args.headless {
        tracing_subscriber::fmt()
            .with_env_filter(&args.log_level)
            .init();
        info!("OpenUSB Linux client starting (headless)");
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { run_headless(config).await })
    } else {
        let dashboard_url = config.servers.first().map(|s| {
            let (host, port) = parse_server_addr(s);
            format!("http://{}:{}", host, port)
        });
        openusb_client_common::tray::run_with_tray(config, dashboard_url)
    }
}

async fn run_headless(config: ClientConfig) -> Result<()> {
    use openusb_client_common::api::{LocalApiState, start_local_api};
    use openusb_client_common::discovery::ServiceBrowser;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let browser = Arc::new(ServiceBrowser::new());
    let api_state = Arc::new(LocalApiState {
        config: RwLock::new(config),
        browser: browser.clone(),
    });

    let mut join_set = tokio::task::JoinSet::new();

    let mdns_browser = browser.clone();
    join_set.spawn(async move { mdns_browser.run().await });

    let api = api_state.clone();
    join_set.spawn(async move { start_local_api(api).await });

    tracing::info!("OpenUSB client running headless (API on localhost:9245)");

    tokio::signal::ctrl_c().await?;
    tracing::info!("Shutting down...");
    join_set.shutdown().await;
    Ok(())
}

fn parse_server_addr(addr: &str) -> (String, u16) {
    if let Some((host, port_str)) = addr.rsplit_once(':')
        && let Ok(port) = port_str.parse()
    {
        return (host.to_string(), port);
    }
    (addr.to_string(), 8443)
}

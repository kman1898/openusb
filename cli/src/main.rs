mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "openusb",
    about = "OpenUSB — share USB devices over the network"
)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all discovered servers and their USB devices
    List,
    /// Attach a remote USB device
    Use {
        /// Device address (e.g., "living-room-pi.1-1.3")
        address: String,
        /// Password for authenticated servers
        #[arg(long)]
        password: Option<String>,
    },
    /// Detach a remote USB device
    Stop {
        /// Device address to detach
        address: String,
    },
    /// Detach all connected devices
    StopAll,
    /// Show connection status summary
    Status,
    /// Show detailed device or server info
    Info {
        /// Device or server address
        address: String,
    },
    /// Configure auto-use rules
    AutoUse {
        #[command(subcommand)]
        rule: AutoUseCommand,
    },
    /// Set a device nickname
    Nickname {
        /// Device address
        address: String,
        /// Display name
        name: String,
    },
    /// Manage known servers
    Servers,
    /// Add a server manually
    AddServer {
        /// Server address (host:port)
        address: String,
    },
    /// Remove a known server
    RemoveServer {
        /// Server address (host:port)
        address: String,
    },
    /// Show or edit client configuration
    Config,
    /// View event log
    Log,
}

#[derive(Subcommand)]
enum AutoUseCommand {
    /// Auto-use a specific device (any port)
    Device { address: String },
    /// Auto-use any device on a specific port
    Port { address: String },
    /// Auto-use all devices on a server
    Hub { server: String },
    /// Auto-use all devices on all servers
    All,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => commands::list::run().await,
        Commands::Use { address, password } => {
            commands::use_device::run(&address, password.as_deref()).await
        }
        Commands::Stop { address } => commands::stop::run(&address).await,
        Commands::StopAll => commands::stop::run_all().await,
        Commands::Status => commands::status::run().await,
        Commands::Info { address } => commands::info::run(&address).await,
        Commands::AutoUse { .. } => {
            println!("Auto-use configuration — coming in Phase 2");
            Ok(())
        }
        Commands::Nickname { address, name } => {
            println!("Setting nickname for {} to '{}'", address, name);
            Ok(())
        }
        Commands::Servers => commands::list::run_servers().await,
        Commands::AddServer { address } => commands::config::add_server(&address).await,
        Commands::RemoveServer { address } => commands::config::remove_server(&address).await,
        Commands::Config => commands::config::run().await,
        Commands::Log => {
            println!("Event log — coming in Phase 2");
            Ok(())
        }
    }
}

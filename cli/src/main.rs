mod commands;

use clap::{Parser, Subcommand};
use openusb_client_common::config::{AutoUseRule, ClientConfig};

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
        /// Device address (e.g., "server:3240/1-1.3")
        address: String,
        /// Password for authenticated servers
        #[arg(long)]
        password: Option<String>,
    },
    /// Detach a remote USB device
    Stop {
        /// Bus ID of the device to detach
        address: String,
    },
    /// Detach all connected devices
    StopAll,
    /// Show connection status summary
    Status,
    /// Show detailed device or server info
    Info {
        /// Server address (host:port)
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
    /// Open web dashboard in browser
    Dashboard,
    /// Show driver status
    Driver,
}

#[derive(Subcommand)]
enum AutoUseCommand {
    /// Auto-use a specific device by vendor:product ID
    Device {
        /// Vendor ID (hex, e.g., "0765")
        vendor_id: String,
        /// Product ID (hex, e.g., "5020")
        product_id: String,
    },
    /// Auto-use all devices from a vendor
    Vendor {
        /// Vendor ID (hex)
        vendor_id: String,
    },
    /// Auto-use all devices on a server
    Hub { server: String },
    /// Auto-use all devices on all servers
    All,
    /// List current auto-use rules
    List,
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
        Commands::AutoUse { rule } => handle_auto_use(rule).await,
        Commands::Nickname { address, name } => {
            let mut config = ClientConfig::load();
            config.nicknames.insert(address.clone(), name.clone());
            config.save()?;
            println!("Set nickname for {} to '{}'", address, name);
            Ok(())
        }
        Commands::Servers => commands::list::run_servers().await,
        Commands::AddServer { address } => commands::config::add_server(&address).await,
        Commands::RemoveServer { address } => commands::config::remove_server(&address).await,
        Commands::Config => commands::config::run().await,
        Commands::Dashboard => {
            let config = ClientConfig::load();
            let url = if let Some(server) = config.servers.first() {
                let (host, port) = parse_server_addr(server);
                format!("http://{}:{}", host, port)
            } else {
                "http://localhost:8443".to_string()
            };
            println!("Opening dashboard: {}", url);
            let _ = open::that(&url);
            Ok(())
        }
        Commands::Driver => {
            let status = openusb_client_common::usbip::check_driver().await?;
            match status {
                openusb_client_common::usbip::DriverStatus::Installed { version } => {
                    println!("USB/IP driver: installed ({})", version);
                }
                openusb_client_common::usbip::DriverStatus::NotInstalled => {
                    println!("USB/IP driver: NOT INSTALLED");
                    println!("Install with: sudo apt install linux-tools-generic (Ubuntu/Debian)");
                }
                openusb_client_common::usbip::DriverStatus::Error { message } => {
                    println!("USB/IP driver: error ({})", message);
                }
            }
            Ok(())
        }
    }
}

async fn handle_auto_use(rule: AutoUseCommand) -> anyhow::Result<()> {
    let mut config = ClientConfig::load();

    match rule {
        AutoUseCommand::Device {
            vendor_id,
            product_id,
        } => {
            config.auto_use_rules.push(AutoUseRule::Device {
                vendor_id: vendor_id.clone(),
                product_id: product_id.clone(),
            });
            config.save()?;
            println!("Added auto-use rule: device {}:{}", vendor_id, product_id);
        }
        AutoUseCommand::Vendor { vendor_id } => {
            config
                .auto_use_rules
                .push(AutoUseRule::VendorId { vendor_id: vendor_id.clone() });
            config.save()?;
            println!("Added auto-use rule: vendor {}", vendor_id);
        }
        AutoUseCommand::Hub { server } => {
            config
                .auto_use_rules
                .push(AutoUseRule::Server { server: server.clone() });
            config.save()?;
            println!("Added auto-use rule: all devices on {}", server);
        }
        AutoUseCommand::All => {
            config.auto_use_rules.push(AutoUseRule::All);
            config.save()?;
            println!("Added auto-use rule: ALL devices on ALL servers");
        }
        AutoUseCommand::List => {
            if config.auto_use_rules.is_empty() {
                println!("No auto-use rules configured.");
            } else {
                println!("Auto-use rules:");
                for (i, rule) in config.auto_use_rules.iter().enumerate() {
                    match rule {
                        AutoUseRule::All => println!("  {}: All devices", i + 1),
                        AutoUseRule::Server { server } => {
                            println!("  {}: All devices on {}", i + 1, server)
                        }
                        AutoUseRule::VendorId { vendor_id } => {
                            println!("  {}: Vendor {}", i + 1, vendor_id)
                        }
                        AutoUseRule::Device {
                            vendor_id,
                            product_id,
                        } => println!("  {}: Device {}:{}", i + 1, vendor_id, product_id),
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_server_addr(addr: &str) -> (String, u16) {
    if let Some((host, port_str)) = addr.rsplit_once(':')
        && let Ok(port) = port_str.parse() {
            return (host.to_string(), port);
        }
    (addr.to_string(), 8443)
}

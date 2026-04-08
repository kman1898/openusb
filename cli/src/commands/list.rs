use anyhow::Result;
use openusb_client_common::config::ClientConfig;
use openusb_client_common::connection::ServerClient;

pub async fn run() -> Result<()> {
    let config = ClientConfig::load();

    if config.servers.is_empty() {
        println!(
            "No servers configured. Use 'openusb add-server <host:port>' or wait for mDNS discovery."
        );
        return Ok(());
    }

    for server_addr in &config.servers {
        let (host, port) = parse_server_addr(server_addr);
        let client = ServerClient::new(&host, port);

        match client.server_info().await {
            Ok(info) => {
                println!("\n{} ({})", info.name, info.hostname);
                println!(
                    "  Version: {} | Uptime: {}s | Clients: {}",
                    info.version, info.uptime_seconds, info.client_count
                );
            }
            Err(e) => {
                println!("\n{} — offline ({})", server_addr, e);
                continue;
            }
        }

        match client.list_devices().await {
            Ok(devices) => {
                if devices.is_empty() {
                    println!("  No USB devices");
                } else {
                    for dev in &devices {
                        let name = dev
                            .nickname
                            .as_deref()
                            .or(dev.product_name.as_deref())
                            .unwrap_or("Unknown Device");
                        let status = match &dev.state {
                            openusb_shared::device::DeviceState::NotShared => "not shared",
                            openusb_shared::device::DeviceState::Available => "available",
                            openusb_shared::device::DeviceState::InUse { client_ip, .. } => {
                                &format!("in use by {}", client_ip)
                            }
                        };
                        println!(
                            "  {} ({}) [{}] — {}",
                            dev.bus_id,
                            name,
                            dev.vid_pid(),
                            status
                        );
                    }
                }
            }
            Err(e) => println!("  Error listing devices: {}", e),
        }
    }
    Ok(())
}

pub async fn run_servers() -> Result<()> {
    let config = ClientConfig::load();
    if config.servers.is_empty() {
        println!("No servers configured.");
        println!("Use 'openusb add-server <host:port>' to add one.");
        return Ok(());
    }

    println!("Known servers:");
    for addr in &config.servers {
        let (host, port) = parse_server_addr(addr);
        let client = ServerClient::new(&host, port);
        match client.server_info().await {
            Ok(info) => println!(
                "  {} — {} (v{}, {} devices)",
                addr, info.name, info.version, info.device_count
            ),
            Err(_) => println!("  {} — offline", addr),
        }
    }
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

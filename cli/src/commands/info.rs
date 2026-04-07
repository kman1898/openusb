use anyhow::Result;
use openusb_client_common::connection::ServerClient;

pub async fn run(address: &str) -> Result<()> {
    // Treat address as a server address (host:port)
    let (host, port) = parse_server_addr(address);
    let client = ServerClient::new(&host, port);

    let info = client.server_info().await?;
    println!("Server: {}", info.name);
    println!("  Hostname: {}", info.hostname);
    println!("  Version: {}", info.version);
    println!("  API Port: {}", info.api_port);
    println!("  USB/IP Port: {}", info.usbip_port);
    println!("  Devices: {}", info.device_count);
    println!("  Clients: {}", info.client_count);
    println!("  Uptime: {}s", info.uptime_seconds);
    println!("  TLS: {}", if info.tls_enabled { "enabled" } else { "disabled" });
    println!("  Auth: {}", if info.auth_required { "required" } else { "open" });

    Ok(())
}

fn parse_server_addr(addr: &str) -> (String, u16) {
    if let Some((host, port_str)) = addr.rsplit_once(':')
        && let Ok(port) = port_str.parse() {
            return (host.to_string(), port);
        }
    (addr.to_string(), 8443)
}

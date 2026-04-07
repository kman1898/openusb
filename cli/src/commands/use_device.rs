use anyhow::Result;
use openusb_client_common::usbip;

pub async fn run(address: &str, _password: Option<&str>) -> Result<()> {
    // Address format: "server.busid" or "host:port/busid"
    let (server, bus_id) = parse_address(address)?;
    println!("Attaching {}:{} ...", server, bus_id);
    usbip::attach(&server, &bus_id).await?;
    println!("Device attached successfully.");
    Ok(())
}

fn parse_address(address: &str) -> Result<(String, String)> {
    // Try "host:port/busid" format first
    if let Some((server, bus_id)) = address.rsplit_once('/') {
        return Ok((server.to_string(), bus_id.to_string()));
    }
    // Try "server.busid" format (dot-separated, last segment is bus_id)
    if let Some((server, bus_id)) = address.rsplit_once('.') {
        return Ok((server.to_string(), bus_id.to_string()));
    }
    anyhow::bail!("Invalid address format. Use 'host:port/busid' or 'server.busid'");
}

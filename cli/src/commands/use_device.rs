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
    // Try "host/busid" or "host:port/busid" format first (unambiguous)
    if let Some((server, bus_id)) = address.split_once('/') {
        return Ok((server.to_string(), bus_id.to_string()));
    }
    anyhow::bail!("Invalid address format. Use 'host/busid' or 'host:port/busid'");
}

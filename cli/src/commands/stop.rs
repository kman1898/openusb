use anyhow::Result;
use openusb_client_common::usbip;

pub async fn run(address: &str) -> Result<()> {
    // address could be a bus_id or a port number
    println!("Detaching {} ...", address);
    usbip::detach(address).await?;
    println!("Device detached.");
    Ok(())
}

pub async fn run_all() -> Result<()> {
    let devices = usbip::list_attached().await?;
    if devices.is_empty() {
        println!("No devices currently attached.");
        return Ok(());
    }
    println!("Detaching {} device(s)...", devices.len());
    usbip::detach_all().await?;
    println!("All devices detached.");
    Ok(())
}

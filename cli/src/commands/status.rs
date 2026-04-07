use anyhow::Result;
use openusb_client_common::config::ClientConfig;
use openusb_client_common::usbip;

pub async fn run() -> Result<()> {
    println!("OpenUSB Status");
    println!("==============");

    // Driver status
    let driver = usbip::check_driver().await?;
    match &driver {
        usbip::DriverStatus::Installed { version } => println!("Driver: installed ({})", version),
        usbip::DriverStatus::NotInstalled => println!("Driver: NOT INSTALLED"),
        usbip::DriverStatus::Error { message } => println!("Driver: error ({})", message),
    }

    // Attached devices
    let attached = usbip::list_attached().await.unwrap_or_default();
    println!("Attached devices: {}", attached.len());
    for dev in &attached {
        println!("  Port {} — {} (bus {})", dev.port, dev.server, dev.bus_id);
    }

    // Config
    let config = ClientConfig::load();
    println!("Known servers: {}", config.servers.len());
    println!("Auto-use rules: {}", config.auto_use_rules.len());

    Ok(())
}

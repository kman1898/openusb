use anyhow::Result;

pub async fn run(address: &str) -> Result<()> {
    println!("Detaching device: {address}");
    // TODO: Use USB/IP detach from openusb-client-common
    println!("(Not yet implemented)");
    Ok(())
}

pub async fn run_all() -> Result<()> {
    println!("Detaching all devices...");
    // TODO: Iterate connected devices and detach
    println!("(Not yet implemented)");
    Ok(())
}

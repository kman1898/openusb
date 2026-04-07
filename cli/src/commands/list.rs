use anyhow::Result;

pub async fn run() -> Result<()> {
    println!("Discovering OpenUSB servers...");
    // TODO: Use mDNS discovery from openusb-client-common
    println!("(No servers found — implementation coming in Phase 1)");
    Ok(())
}

pub async fn run_servers() -> Result<()> {
    println!("Known servers:");
    // TODO: Read from client config
    println!("(No servers configured)");
    Ok(())
}

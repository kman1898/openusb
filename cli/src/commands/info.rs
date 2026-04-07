use anyhow::Result;

pub async fn run(address: &str) -> Result<()> {
    println!("Info for: {address}");
    // TODO: Query server REST API for device/server details
    println!("(Not yet implemented)");
    Ok(())
}

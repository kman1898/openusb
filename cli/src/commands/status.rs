use anyhow::Result;

pub async fn run() -> Result<()> {
    println!("OpenUSB Status");
    println!("==============");
    // TODO: Query connected devices and server states
    println!("Servers: 0 discovered");
    println!("Devices: 0 attached");
    Ok(())
}

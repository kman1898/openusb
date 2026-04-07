use anyhow::Result;

pub async fn run(address: &str, _password: Option<&str>) -> Result<()> {
    println!("Attaching device: {address}");
    // TODO: Use USB/IP attach from openusb-client-common
    println!("(Not yet implemented)");
    Ok(())
}

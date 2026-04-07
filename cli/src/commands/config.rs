use anyhow::Result;

pub async fn run() -> Result<()> {
    println!("Client configuration:");
    // TODO: Read and display client config
    println!("(Not yet implemented)");
    Ok(())
}

pub async fn add_server(address: &str) -> Result<()> {
    println!("Adding server: {address}");
    // TODO: Persist to client config
    println!("(Not yet implemented)");
    Ok(())
}

pub async fn remove_server(address: &str) -> Result<()> {
    println!("Removing server: {address}");
    // TODO: Remove from client config
    println!("(Not yet implemented)");
    Ok(())
}

use anyhow::Result;
use openusb_client_common::config::ClientConfig;

pub async fn run() -> Result<()> {
    let config = ClientConfig::load();
    let toml_str = toml::to_string_pretty(&config)?;
    println!("Client configuration ({}):", ClientConfig::config_path().display());
    println!("{}", toml_str);
    Ok(())
}

pub async fn add_server(address: &str) -> Result<()> {
    let mut config = ClientConfig::load();
    if config.servers.contains(&address.to_string()) {
        println!("Server {} is already configured.", address);
        return Ok(());
    }
    config.servers.push(address.to_string());
    config.save()?;
    println!("Added server: {}", address);
    Ok(())
}

pub async fn remove_server(address: &str) -> Result<()> {
    let mut config = ClientConfig::load();
    let before = config.servers.len();
    config.servers.retain(|s| s != address);
    if config.servers.len() == before {
        println!("Server {} was not found.", address);
        return Ok(());
    }
    config.save()?;
    println!("Removed server: {}", address);
    Ok(())
}

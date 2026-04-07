use mdns_sd::{ServiceDaemon, ServiceEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

const SERVICE_TYPE: &str = "_openusb._tcp.local.";

#[derive(Debug, Clone)]
pub struct DiscoveredServer {
    pub name: String,
    pub host: String,
    pub port: u16,
    pub api_port: u16,
    pub version: String,
}

pub struct ServiceBrowser {
    servers: Arc<RwLock<HashMap<String, DiscoveredServer>>>,
}

impl Default for ServiceBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl ServiceBrowser {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn servers(&self) -> Arc<RwLock<HashMap<String, DiscoveredServer>>> {
        self.servers.clone()
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let mdns = ServiceDaemon::new()?;
        let receiver = mdns.browse(SERVICE_TYPE)?;

        info!("Browsing for OpenUSB servers via mDNS");

        loop {
            match receiver.recv_async().await {
                Ok(event) => match event {
                    ServiceEvent::ServiceResolved(info) => {
                        let name = info
                            .get_property_val_str("name")
                            .unwrap_or_default()
                            .to_string();
                        let api_port: u16 = info
                            .get_property_val_str("api_port")
                            .and_then(|p| p.parse().ok())
                            .unwrap_or(8443);
                        let version = info
                            .get_property_val_str("version")
                            .unwrap_or_default()
                            .to_string();

                        let host = info
                            .get_addresses()
                            .iter()
                            .next()
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| info.get_hostname().to_string());

                        let server = DiscoveredServer {
                            name: name.clone(),
                            host: host.clone(),
                            port: info.get_port(),
                            api_port,
                            version,
                        };

                        info!(name = %name, host = %host, port = api_port, "Discovered server");
                        self.servers
                            .write()
                            .await
                            .insert(info.get_fullname().to_string(), server);
                    }
                    ServiceEvent::ServiceRemoved(_, fullname) => {
                        debug!(fullname = %fullname, "Server removed");
                        self.servers.write().await.remove(&fullname);
                    }
                    _ => {}
                },
                Err(e) => {
                    warn!("mDNS browse error: {}", e);
                    break;
                }
            }
        }
        Ok(())
    }
}

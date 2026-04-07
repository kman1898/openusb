use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub server: ServerSection,
    #[serde(default)]
    pub discovery: DiscoverySection,
    #[serde(default)]
    pub security: SecuritySection,
    #[serde(default)]
    pub devices: DevicesSection,
    #[serde(default)]
    pub events: EventsSection,
    #[serde(default)]
    pub notifications: NotificationsSection,
    #[serde(default)]
    pub scheduling: SchedulingSection,
    #[serde(default)]
    pub metrics: MetricsSection,
    #[serde(default)]
    pub reverse_connections: ReverseConnectionsSection,
    #[serde(default)]
    pub relay: RelaySection,
    #[serde(default)]
    pub logging: LoggingSection,
}

#[derive(Debug, Deserialize)]
pub struct ServerSection {
    pub name: String,
    #[serde(default = "default_usbip_port")]
    pub port: u16,
    #[serde(default = "default_api_port")]
    pub api_port: u16,
    #[serde(default)]
    pub hostname: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct DiscoverySection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_mdns_name")]
    pub mdns_name: String,
}

#[derive(Debug, Deserialize)]
pub struct SecuritySection {
    #[serde(default = "default_security_mode")]
    pub mode: String,
    #[serde(default)]
    pub password_hash: String,
    #[serde(default)]
    pub tls_enabled: bool,
    #[serde(default)]
    pub tls_cert: String,
    #[serde(default)]
    pub tls_key: String,
    #[serde(default)]
    pub tls_ca: String,
    #[serde(default)]
    pub tls_client_certs: bool,
}

impl Default for SecuritySection {
    fn default() -> Self {
        Self {
            mode: "open".to_string(),
            password_hash: String::new(),
            tls_enabled: false,
            tls_cert: String::new(),
            tls_key: String::new(),
            tls_ca: String::new(),
            tls_client_certs: false,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct DevicesSection {
    #[serde(default = "default_true")]
    pub auto_share: bool,
    #[serde(default)]
    pub ignore_vendor_ids: Vec<String>,
    #[serde(default)]
    pub ignore_bus_ids: Vec<String>,
    #[serde(default)]
    pub allow_vendor_ids: Vec<String>,
    #[serde(default)]
    pub nicknames: HashMap<String, String>,
    #[serde(default)]
    pub access: HashMap<String, openusb_shared::config::DeviceAcl>,
}

#[derive(Debug, Default, Deserialize)]
pub struct EventsSection {
    #[serde(default)]
    pub on_attach: String,
    #[serde(default)]
    pub on_detach: String,
    #[serde(default)]
    pub on_client_connect: String,
    #[serde(default)]
    pub on_client_disconnect: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct NotificationsSection {
    #[serde(default)]
    pub webhook_url: String,
    #[serde(default)]
    pub email_smtp: String,
    #[serde(default)]
    pub email_to: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct SchedulingSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_timezone")]
    pub timezone: String,
}

#[derive(Debug, Deserialize)]
pub struct MetricsSection {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_history_days")]
    pub history_days: u32,
    #[serde(default = "default_true")]
    pub bandwidth_tracking: bool,
}

impl Default for MetricsSection {
    fn default() -> Self {
        Self {
            enabled: true,
            history_days: 90,
            bandwidth_tracking: true,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
pub struct ReverseConnectionsSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub clients: Vec<String>,
    #[serde(default = "default_retry_interval")]
    pub retry_interval: u32,
}

#[derive(Debug, Default, Deserialize)]
pub struct RelaySection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub relay_server: String,
    #[serde(default)]
    pub relay_token: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingSection {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default = "default_log_file")]
    pub file: String,
    #[serde(default = "default_log_max_size")]
    pub max_size_mb: u32,
    #[serde(default = "default_rotate_count")]
    pub rotate_count: u32,
}

impl Default for LoggingSection {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: "/var/log/openusb/openusb.log".to_string(),
            max_size_mb: 50,
            rotate_count: 5,
        }
    }
}

impl ServerConfig {
    pub fn load(path: &str) -> Result<Self> {
        let path = Path::new(path);
        if !path.exists() {
            anyhow::bail!(
                "Config file not found: {}. Copy openusb.toml.example to get started.",
                path.display()
            );
        }
        let contents = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;
        let config: Self = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
        Ok(config)
    }
}

fn default_usbip_port() -> u16 {
    3240
}
fn default_api_port() -> u16 {
    8443
}
fn default_true() -> bool {
    true
}
fn default_mdns_name() -> String {
    "_openusb._tcp".to_string()
}
fn default_security_mode() -> String {
    "open".to_string()
}
fn default_timezone() -> String {
    "UTC".to_string()
}
fn default_history_days() -> u32 {
    90
}
fn default_retry_interval() -> u32 {
    15
}
fn default_log_level() -> String {
    "info".to_string()
}
fn default_log_file() -> String {
    "/var/log/openusb/openusb.log".to_string()
}
fn default_log_max_size() -> u32 {
    50
}
fn default_rotate_count() -> u32 {
    5
}

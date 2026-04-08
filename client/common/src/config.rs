use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    #[serde(default)]
    pub auto_reconnect: bool,
    #[serde(default = "default_reconnect_delay")]
    pub reconnect_delay: u64,
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default)]
    pub servers: Vec<String>,
    #[serde(default)]
    pub auto_use_rules: Vec<AutoUseRule>,
    #[serde(default)]
    pub nicknames: HashMap<String, String>,
}

fn default_reconnect_delay() -> u64 {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AutoUseRule {
    Device { vendor_id: String, product_id: String },
    VendorId { vendor_id: String },
    Server { server: String },
    All,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            auto_reconnect: true,
            reconnect_delay: 5,
            auto_start: false,
            servers: Vec::new(),
            auto_use_rules: Vec::new(),
            nicknames: HashMap::new(),
        }
    }
}

impl ClientConfig {
    pub fn config_path() -> PathBuf {
        let dir = dirs_config_dir().join("openusb");
        dir.join("client.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match std::fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents = toml::to_string_pretty(self)?;
        std::fs::write(path, contents)?;
        Ok(())
    }
}

fn dirs_config_dir() -> PathBuf {
    // Windows: use APPDATA (e.g., C:\Users\user\AppData\Roaming)
    if let Some(dir) = std::env::var_os("APPDATA") {
        return PathBuf::from(dir);
    }
    // Linux/macOS: use XDG_CONFIG_HOME
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    // Fallback: ~/.config
    if let Some(home) = std::env::var_os("HOME") {
        return PathBuf::from(home).join(".config");
    }
    PathBuf::from("/etc")
}

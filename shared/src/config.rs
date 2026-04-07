use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Shared configuration types used by both server and clients.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceFilter {
    /// Vendor IDs to ignore (e.g., "1d6b" for Linux Foundation internal hubs)
    #[serde(default)]
    pub ignore_vendor_ids: Vec<String>,
    /// Specific bus IDs to ignore
    #[serde(default)]
    pub ignore_bus_ids: Vec<String>,
    /// If non-empty, ONLY these vendor IDs are shared
    #[serde(default)]
    pub allow_vendor_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceAcl {
    /// Users allowed to access this device
    pub allowed_users: Vec<String>,
    /// Whether a password is required even if the user is in the list
    #[serde(default)]
    pub require_password: bool,
}

/// Nickname mapping: "vid:pid" -> display name
pub type DeviceNicknames = HashMap<String, String>;

/// ACL mapping: "vid:pid" -> access control
pub type DeviceAcls = HashMap<String, DeviceAcl>;

use openusb_shared::config::DeviceAcl;
use std::collections::HashMap;

/// Per-device access control list enforcement.
pub struct AclEngine {
    rules: HashMap<String, DeviceAcl>,
}

impl AclEngine {
    pub fn new(rules: HashMap<String, DeviceAcl>) -> Self {
        Self { rules }
    }

    /// Check if a user can access a device.
    /// `device_key` is typically "vid:pid" (e.g. "0765:5020").
    /// Returns true if no ACL is set for the device (open access).
    pub fn can_access(&self, device_key: &str, username: &str) -> bool {
        match self.rules.get(device_key) {
            None => true, // No ACL = open access
            Some(acl) => {
                if acl.allowed_users.is_empty() {
                    return true; // Empty allow list = open access
                }
                acl.allowed_users.iter().any(|u| u == username || u == "*")
            }
        }
    }

    /// Check if a device requires password auth even for allowed users.
    pub fn requires_password(&self, device_key: &str) -> bool {
        self.rules
            .get(device_key)
            .is_some_and(|acl| acl.require_password)
    }

    /// Get the ACL rules for a specific device.
    pub fn get_acl(&self, device_key: &str) -> Option<&DeviceAcl> {
        self.rules.get(device_key)
    }

    /// Get all ACL rules.
    pub fn all_rules(&self) -> &HashMap<String, DeviceAcl> {
        &self.rules
    }

    /// Update a device's ACL.
    pub fn set_acl(&mut self, device_key: String, acl: DeviceAcl) {
        self.rules.insert(device_key, acl);
    }

    /// Remove a device's ACL (reverting to open access).
    pub fn remove_acl(&mut self, device_key: &str) {
        self.rules.remove(device_key);
    }
}

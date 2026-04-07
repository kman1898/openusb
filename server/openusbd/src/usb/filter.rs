use openusb_shared::device::UsbDevice;
use std::collections::HashSet;
use tracing::warn;

/// Compiled device filter rules — parsed once from config at startup.
pub struct DeviceFilterRules {
    ignore_vendor_ids: HashSet<u16>,
    ignore_bus_ids: HashSet<String>,
    allow_vendor_ids: HashSet<u16>,
}

impl DeviceFilterRules {
    /// Build filter rules from config string lists.
    /// Vendor IDs are hex strings like "1d6b".
    pub fn from_config(
        ignore_vendor_ids: &[String],
        ignore_bus_ids: &[String],
        allow_vendor_ids: &[String],
    ) -> Self {
        let parse_vid = |s: &String| -> Option<u16> {
            match u16::from_str_radix(s.trim(), 16) {
                Ok(v) => Some(v),
                Err(_) => {
                    warn!(value = %s, "Invalid vendor ID in config (expected hex like '1d6b')");
                    None
                }
            }
        };

        Self {
            ignore_vendor_ids: ignore_vendor_ids.iter().filter_map(parse_vid).collect(),
            ignore_bus_ids: ignore_bus_ids.iter().cloned().collect(),
            allow_vendor_ids: allow_vendor_ids.iter().filter_map(parse_vid).collect(),
        }
    }

    /// Returns true if the device passes filter rules and should be visible.
    pub fn should_include(&self, device: &UsbDevice) -> bool {
        // If allow list is set, device must be on it
        if !self.allow_vendor_ids.is_empty() && !self.allow_vendor_ids.contains(&device.vendor_id) {
            return false;
        }
        // Check ignore lists
        if self.ignore_vendor_ids.contains(&device.vendor_id) {
            return false;
        }
        if self.ignore_bus_ids.contains(&device.bus_id) {
            return false;
        }
        true
    }
}

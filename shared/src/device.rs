use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique address for a USB device on a specific server.
/// Format: "server_hostname.busid" (e.g., "living-room-pi.1-1.3")
pub type DeviceAddress = String;

/// Represents a USB device discovered on a server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsbDevice {
    /// Unique ID assigned by OpenUSB
    pub id: Uuid,
    /// USB bus ID (e.g., "1-1.3")
    pub bus_id: String,
    /// USB vendor ID (e.g., 0x0765)
    pub vendor_id: u16,
    /// USB product ID (e.g., 0x5020)
    pub product_id: u16,
    /// Device class
    pub device_class: u8,
    /// Device subclass
    pub device_subclass: u8,
    /// Device protocol
    pub device_protocol: u8,
    /// Vendor name (from usb.ids or override)
    pub vendor_name: Option<String>,
    /// Product name (from usb.ids or override)
    pub product_name: Option<String>,
    /// User-assigned nickname
    pub nickname: Option<String>,
    /// Serial number if available
    pub serial: Option<String>,
    /// Number of USB configurations
    pub num_configurations: u8,
    /// USB speed (e.g., "high", "super")
    pub speed: UsbSpeed,
    /// Current sharing/usage state
    pub state: DeviceState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UsbSpeed {
    Low,
    Full,
    High,
    Super,
    SuperPlus,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceState {
    /// Device is connected but not shared over the network
    NotShared,
    /// Device is shared and available for clients to attach
    Available,
    /// Device is currently in use by a client
    InUse {
        client_ip: String,
        client_name: Option<String>,
        since: chrono::DateTime<chrono::Utc>,
    },
}

impl UsbDevice {
    /// Returns the vendor:product ID string (e.g., "0765:5020")
    pub fn vid_pid(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor_id, self.product_id)
    }

    /// Returns a display name: nickname > product_name > vid:pid
    pub fn display_name(&self) -> &str {
        self.nickname
            .as_deref()
            .or(self.product_name.as_deref())
            .unwrap_or("Unknown Device")
    }
}

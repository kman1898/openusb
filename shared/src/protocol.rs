use serde::{Deserialize, Serialize};

/// OpenUSB control protocol messages.
///
/// These are exchanged over the REST API and WebSocket connections
/// between servers and clients. The actual USB data flows over
/// the standard USB/IP protocol on TCP port 3240.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    DeviceAttached {
        device: super::device::UsbDevice,
    },
    DeviceDetached {
        bus_id: String,
    },
    DeviceShared {
        bus_id: String,
    },
    DeviceUnshared {
        bus_id: String,
    },
    ClientConnected {
        client_ip: String,
        client_name: Option<String>,
    },
    ClientDisconnected {
        client_ip: String,
    },
    DeviceInUse {
        bus_id: String,
        client_ip: String,
    },
    DeviceReleased {
        bus_id: String,
    },
    AuthFailed {
        client_ip: String,
        reason: String,
    },
    BandwidthAlert {
        bus_id: String,
        bytes_per_sec: u64,
    },
}

/// Server info returned by the discovery and REST APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerInfo {
    pub name: String,
    pub hostname: String,
    pub version: String,
    pub api_port: u16,
    pub usbip_port: u16,
    pub device_count: usize,
    pub client_count: usize,
    pub uptime_seconds: u64,
    pub tls_enabled: bool,
    pub auth_required: bool,
}

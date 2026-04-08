use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Per-device bandwidth tracking.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceBandwidth {
    pub bus_id: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct BandwidthTracker {
    devices: Arc<RwLock<HashMap<String, DeviceBandwidth>>>,
}

impl BandwidthTracker {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record bytes transferred for a device.
    pub async fn record(&self, bus_id: &str, sent: u64, received: u64) {
        let mut map = self.devices.write().await;
        let entry = map.entry(bus_id.to_string()).or_insert(DeviceBandwidth {
            bus_id: bus_id.to_string(),
            bytes_sent: 0,
            bytes_received: 0,
            last_updated: chrono::Utc::now(),
        });
        entry.bytes_sent += sent;
        entry.bytes_received += received;
        entry.last_updated = chrono::Utc::now();
    }

    /// Get bandwidth stats for all devices.
    pub async fn all_stats(&self) -> Vec<DeviceBandwidth> {
        let map = self.devices.read().await;
        map.values().cloned().collect()
    }

    /// Get bandwidth stats for a specific device.
    pub async fn device_stats(&self, bus_id: &str) -> Option<DeviceBandwidth> {
        let map = self.devices.read().await;
        map.get(bus_id).cloned()
    }

    /// Reset counters for a device (e.g., when detached).
    pub async fn reset(&self, bus_id: &str) {
        let mut map = self.devices.write().await;
        map.remove(bus_id);
    }
}

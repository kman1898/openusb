use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Per-device latency measurement.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceLatency {
    pub bus_id: String,
    /// Last measured round-trip time in microseconds
    pub rtt_us: u64,
    /// Average round-trip time in microseconds
    pub avg_rtt_us: u64,
    /// Number of samples taken
    pub samples: u64,
    pub last_measured: chrono::DateTime<chrono::Utc>,
}

pub struct LatencyTracker {
    devices: Arc<RwLock<HashMap<String, DeviceLatency>>>,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Record a latency measurement for a device.
    pub async fn record(&self, bus_id: &str, rtt_us: u64) {
        let mut map = self.devices.write().await;
        let entry = map.entry(bus_id.to_string()).or_insert(DeviceLatency {
            bus_id: bus_id.to_string(),
            rtt_us: 0,
            avg_rtt_us: 0,
            samples: 0,
            last_measured: chrono::Utc::now(),
        });
        entry.rtt_us = rtt_us;
        entry.samples += 1;
        // Running average
        entry.avg_rtt_us =
            (entry.avg_rtt_us * (entry.samples - 1) + rtt_us) / entry.samples;
        entry.last_measured = chrono::Utc::now();
    }

    /// Get latency stats for all devices.
    pub async fn all_stats(&self) -> Vec<DeviceLatency> {
        let map = self.devices.read().await;
        map.values().cloned().collect()
    }

    /// Get latency stats for a specific device.
    pub async fn device_stats(&self, bus_id: &str) -> Option<DeviceLatency> {
        let map = self.devices.read().await;
        map.get(bus_id).cloned()
    }

    /// Reset measurements for a device.
    pub async fn reset(&self, bus_id: &str) {
        let mut map = self.devices.write().await;
        map.remove(bus_id);
    }
}

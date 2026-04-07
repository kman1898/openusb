use anyhow::{Result, bail};
use async_trait::async_trait;
use openusb_shared::device::{DeviceState, UsbDevice, UsbSpeed};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::info;
use uuid::Uuid;

use super::platform::{HotplugEvent, UsbPlatform};

/// Simulated USB platform for development on non-Linux systems.
/// Provides fake USB devices and periodic hotplug events.
pub struct SimulatedPlatform {
    devices: Arc<RwLock<HashMap<String, UsbDevice>>>,
}

impl SimulatedPlatform {
    pub fn new() -> Self {
        let mut devices = HashMap::new();

        let sim_devices = vec![
            (
                "1-1",
                0x046d,
                0xc52b,
                0,
                0,
                0,
                "Logitech, Inc.",
                "Unifying Receiver",
                UsbSpeed::Full,
            ),
            (
                "1-2",
                0x0765,
                0x5020,
                0xff,
                0,
                0,
                "X-Rite, Inc.",
                "i1Display Pro / ColorMunki Display",
                UsbSpeed::Full,
            ),
            (
                "1-3",
                0x04b8,
                0x0005,
                7,
                1,
                2,
                "Seiko Epson Corp.",
                "Printer",
                UsbSpeed::High,
            ),
            (
                "2-1",
                0x046d,
                0x0892,
                0xef,
                2,
                1,
                "Logitech, Inc.",
                "C920 HD Pro Webcam",
                UsbSpeed::High,
            ),
            (
                "2-2",
                0x0781,
                0x5583,
                0,
                0,
                0,
                "SanDisk Corp.",
                "Ultra Fit USB 3.1",
                UsbSpeed::Super,
            ),
        ];

        for (bus_id, vid, pid, class, subclass, protocol, vendor, product, speed) in sim_devices {
            let device = UsbDevice {
                id: Uuid::new_v4(),
                bus_id: bus_id.to_string(),
                vendor_id: vid,
                product_id: pid,
                device_class: class,
                device_subclass: subclass,
                device_protocol: protocol,
                vendor_name: Some(vendor.to_string()),
                product_name: Some(product.to_string()),
                nickname: None,
                serial: None,
                num_configurations: 1,
                speed,
                state: DeviceState::NotShared,
            };
            devices.insert(bus_id.to_string(), device);
        }

        Self {
            devices: Arc::new(RwLock::new(devices)),
        }
    }
}

#[async_trait]
impl UsbPlatform for SimulatedPlatform {
    async fn enumerate_devices(&self) -> Result<Vec<UsbDevice>> {
        let devices = self.devices.read().await;
        Ok(devices.values().cloned().collect())
    }

    async fn bind_device(&self, bus_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        match devices.get_mut(bus_id) {
            Some(device) => {
                device.state = DeviceState::Available;
                info!(bus_id, "Simulated: bound device to usbip");
                Ok(())
            }
            None => bail!("Device not found: {bus_id}"),
        }
    }

    async fn unbind_device(&self, bus_id: &str) -> Result<()> {
        let mut devices = self.devices.write().await;
        match devices.get_mut(bus_id) {
            Some(device) => {
                device.state = DeviceState::NotShared;
                info!(bus_id, "Simulated: unbound device from usbip");
                Ok(())
            }
            None => bail!("Device not found: {bus_id}"),
        }
    }

    async fn watch_hotplug(&self, tx: mpsc::Sender<HotplugEvent>) -> Result<()> {
        let devices = self.devices.clone();
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(45));
        let mut cycle = 0u32;

        // Simulate a device appearing and disappearing periodically
        let extra_device = UsbDevice {
            id: Uuid::new_v4(),
            bus_id: "3-1".to_string(),
            vendor_id: 0x1a86,
            product_id: 0x7523,
            device_class: 0xff,
            device_subclass: 0,
            device_protocol: 0,
            vendor_name: Some("QinHeng Electronics".to_string()),
            product_name: Some("CH340 Serial Adapter".to_string()),
            nickname: None,
            serial: None,
            num_configurations: 1,
            speed: UsbSpeed::Full,
            state: DeviceState::NotShared,
        };

        loop {
            interval.tick().await;
            cycle += 1;

            if cycle % 2 == 1 {
                // Attach the extra device
                info!(bus_id = "3-1", "Simulated: hotplug attach");
                devices
                    .write()
                    .await
                    .insert("3-1".to_string(), extra_device.clone());
                let _ = tx.send(HotplugEvent::Attached(extra_device.clone())).await;
            } else {
                // Detach the extra device
                info!(bus_id = "3-1", "Simulated: hotplug detach");
                devices.write().await.remove("3-1");
                let _ = tx
                    .send(HotplugEvent::Detached {
                        bus_id: "3-1".to_string(),
                    })
                    .await;
            }
        }
    }
}

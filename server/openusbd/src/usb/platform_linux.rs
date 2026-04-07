use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use openusb_shared::device::{DeviceState, UsbDevice, UsbSpeed};
use std::path::Path;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use super::platform::{HotplugEvent, UsbPlatform};

/// Real Linux USB platform using sysfs and usbip commands.
pub struct LinuxPlatform;

impl LinuxPlatform {
    pub fn new() -> Self {
        Self
    }

    /// Read a sysfs attribute file, returning None if it doesn't exist.
    fn read_sysfs_attr(device_path: &Path, attr: &str) -> Option<String> {
        let path = device_path.join(attr);
        std::fs::read_to_string(&path)
            .ok()
            .map(|s| s.trim().to_string())
    }

    /// Parse a hex string (e.g., "046d") into a u16.
    fn parse_hex_u16(s: &str) -> Option<u16> {
        u16::from_str_radix(s.trim(), 16).ok()
    }

    /// Parse a decimal string into a u8.
    fn parse_dec_u8(s: &str) -> Option<u8> {
        s.trim().parse().ok()
    }

    /// Parse USB speed string from sysfs (e.g., "480", "5000") into UsbSpeed.
    fn parse_speed(speed_str: &str) -> UsbSpeed {
        match speed_str.trim() {
            "1.5" => UsbSpeed::Low,
            "12" => UsbSpeed::Full,
            "480" => UsbSpeed::High,
            "5000" => UsbSpeed::Super,
            "10000" | "20000" => UsbSpeed::SuperPlus,
            _ => UsbSpeed::Unknown,
        }
    }

    /// Try to parse a single USB device from a sysfs directory.
    fn parse_device_from_sysfs(entry_path: &Path) -> Option<UsbDevice> {
        // Only look at entries that look like USB device bus IDs (e.g., "1-1", "2-1.3")
        let bus_id = entry_path.file_name()?.to_str()?;
        if !bus_id.contains('-') || bus_id.contains(':') {
            return None; // Skip interfaces like "1-1:1.0"
        }

        let vendor_id = Self::parse_hex_u16(&Self::read_sysfs_attr(entry_path, "idVendor")?)?;
        let product_id = Self::parse_hex_u16(&Self::read_sysfs_attr(entry_path, "idProduct")?)?;

        let device_class = Self::read_sysfs_attr(entry_path, "bDeviceClass")
            .and_then(|s| Self::parse_hex_u16(&s))
            .unwrap_or(0) as u8;
        let device_subclass = Self::read_sysfs_attr(entry_path, "bDeviceSubClass")
            .and_then(|s| Self::parse_hex_u16(&s))
            .unwrap_or(0) as u8;
        let device_protocol = Self::read_sysfs_attr(entry_path, "bDeviceProtocol")
            .and_then(|s| Self::parse_hex_u16(&s))
            .unwrap_or(0) as u8;

        let speed = Self::read_sysfs_attr(entry_path, "speed")
            .map(|s| Self::parse_speed(&s))
            .unwrap_or(UsbSpeed::Unknown);

        let num_configurations = Self::read_sysfs_attr(entry_path, "bNumConfigurations")
            .and_then(|s| Self::parse_dec_u8(&s))
            .unwrap_or(1);

        let serial = Self::read_sysfs_attr(entry_path, "serial");
        let product_name = Self::read_sysfs_attr(entry_path, "product");
        let vendor_name = Self::read_sysfs_attr(entry_path, "manufacturer");

        Some(UsbDevice {
            id: Uuid::new_v4(),
            bus_id: bus_id.to_string(),
            vendor_id,
            product_id,
            device_class,
            device_subclass,
            device_protocol,
            vendor_name,
            product_name,
            nickname: None,
            serial,
            num_configurations,
            speed,
            state: DeviceState::NotShared,
        })
    }
}

#[async_trait]
impl UsbPlatform for LinuxPlatform {
    async fn enumerate_devices(&self) -> Result<Vec<UsbDevice>> {
        let sysfs_path = Path::new("/sys/bus/usb/devices");
        if !sysfs_path.exists() {
            bail!("USB sysfs not found at /sys/bus/usb/devices — is this a Linux system?");
        }

        let mut devices = Vec::new();
        let entries =
            std::fs::read_dir(sysfs_path).context("Failed to read /sys/bus/usb/devices")?;

        for entry in entries.flatten() {
            if let Some(device) = Self::parse_device_from_sysfs(&entry.path()) {
                debug!(
                    bus_id = %device.bus_id,
                    vid_pid = %device.vid_pid(),
                    "Enumerated USB device"
                );
                devices.push(device);
            }
        }

        info!(count = devices.len(), "Enumerated USB devices from sysfs");
        Ok(devices)
    }

    async fn bind_device(&self, bus_id: &str) -> Result<()> {
        info!(bus_id, "Binding device to usbip-host");

        let output = tokio::process::Command::new("usbip")
            .args(["bind", "--busid", bus_id])
            .output()
            .await
            .context("Failed to execute usbip bind")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            error!(bus_id, stderr = %stderr, "usbip bind failed");
            bail!("usbip bind failed for {bus_id}: {stderr}");
        }

        info!(bus_id, "Device bound to usbip-host");
        Ok(())
    }

    async fn unbind_device(&self, bus_id: &str) -> Result<()> {
        info!(bus_id, "Unbinding device from usbip-host");

        let output = tokio::process::Command::new("usbip")
            .args(["unbind", "--busid", bus_id])
            .output()
            .await
            .context("Failed to execute usbip unbind")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!(bus_id, stderr = %stderr, "usbip unbind returned error (device may already be unbound)");
        }

        info!(bus_id, "Device unbound from usbip-host");
        Ok(())
    }

    async fn watch_hotplug(&self, tx: mpsc::Sender<HotplugEvent>) -> Result<()> {
        // Poll sysfs periodically for changes.
        // A more sophisticated approach would use inotify or udevadm monitor,
        // but polling is simpler and reliable for Phase 1.
        let mut known_bus_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        // Seed with current devices
        let initial = self.enumerate_devices().await?;
        for device in &initial {
            known_bus_ids.insert(device.bus_id.clone());
        }

        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(2));
        loop {
            interval.tick().await;

            match self.enumerate_devices().await {
                Ok(current_devices) => {
                    let current_ids: std::collections::HashSet<String> =
                        current_devices.iter().map(|d| d.bus_id.clone()).collect();

                    // Detect new devices
                    for device in &current_devices {
                        if !known_bus_ids.contains(&device.bus_id) {
                            info!(bus_id = %device.bus_id, "Hotplug: device attached");
                            let _ = tx.send(HotplugEvent::Attached(device.clone())).await;
                        }
                    }

                    // Detect removed devices
                    for bus_id in &known_bus_ids {
                        if !current_ids.contains(bus_id) {
                            info!(bus_id, "Hotplug: device detached");
                            let _ = tx
                                .send(HotplugEvent::Detached {
                                    bus_id: bus_id.clone(),
                                })
                                .await;
                        }
                    }

                    known_bus_ids = current_ids;
                }
                Err(e) => {
                    warn!(error = %e, "Failed to enumerate devices during hotplug poll");
                }
            }
        }
    }
}

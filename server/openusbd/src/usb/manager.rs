use crate::state::AppState;
use crate::usb::platform::HotplugEvent;
use anyhow::{Result, bail};
use openusb_shared::device::DeviceState;
use openusb_shared::protocol::ServerEvent;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{info, warn};

/// Manages USB device lifecycle: enumeration, hotplug, share/unshare.
#[derive(Clone)]
pub struct DeviceManager {
    state: Arc<AppState>,
}

impl DeviceManager {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    /// Perform initial device enumeration at startup.
    pub async fn initial_enumerate(&self) -> Result<()> {
        let raw_devices = self.state.platform.enumerate_devices().await?;
        let mut devices = self.state.devices.write().await;

        for mut device in raw_devices {
            if !self.state.filter.should_include(&device) {
                continue;
            }

            self.apply_names(&mut device);

            // Auto-share if configured
            if self.state.config.devices.auto_share {
                if let Err(e) = self.state.platform.bind_device(&device.bus_id).await {
                    warn!(bus_id = %device.bus_id, error = %e, "Failed to auto-share device");
                } else {
                    device.state = DeviceState::Available;
                }
            }

            info!(
                bus_id = %device.bus_id,
                name = %device.display_name(),
                vid_pid = %device.vid_pid(),
                "Discovered device"
            );

            devices.insert(device.bus_id.clone(), device);
        }

        info!(count = devices.len(), "Initial device enumeration complete");
        Ok(())
    }

    /// Run the hotplug monitor loop. Blocks until shutdown.
    pub async fn run_hotplug_monitor(self) -> Result<()> {
        let (tx, mut rx) = mpsc::channel::<HotplugEvent>(64);

        // Spawn the platform-specific hotplug watcher
        let platform = self.state.platform.clone();
        tokio::spawn(async move {
            if let Err(e) = platform.watch_hotplug(tx).await {
                tracing::error!(error = %e, "Hotplug monitor failed");
            }
        });

        // Process events
        while let Some(event) = rx.recv().await {
            match event {
                HotplugEvent::Attached(mut device) => {
                    if !self.state.filter.should_include(&device) {
                        continue;
                    }

                    self.apply_names(&mut device);

                    // Auto-share if configured
                    if self.state.config.devices.auto_share {
                        if let Err(e) = self.state.platform.bind_device(&device.bus_id).await {
                            warn!(bus_id = %device.bus_id, error = %e, "Failed to auto-share hotplugged device");
                        } else {
                            device.state = DeviceState::Available;
                        }
                    }

                    info!(
                        bus_id = %device.bus_id,
                        name = %device.display_name(),
                        "Device attached"
                    );

                    self.state.emit(ServerEvent::DeviceAttached {
                        device: device.clone(),
                    });
                    self.state
                        .devices
                        .write()
                        .await
                        .insert(device.bus_id.clone(), device);
                }
                HotplugEvent::Detached { bus_id } => {
                    if self.state.devices.write().await.remove(&bus_id).is_some() {
                        info!(bus_id, "Device detached");
                        self.state.emit(ServerEvent::DeviceDetached {
                            bus_id: bus_id.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Share (bind) a device to make it available over the network.
    pub async fn share_device(&self, bus_id: &str) -> Result<()> {
        // Check state first, then drop the lock before platform I/O
        {
            let devices = self.state.devices.read().await;
            match devices.get(bus_id) {
                None => bail!("Device not found: {bus_id}"),
                Some(device) => match &device.state {
                    DeviceState::Available | DeviceState::InUse { .. } => {
                        bail!("Device {bus_id} is already shared")
                    }
                    DeviceState::NotShared => {} // proceed
                },
            }
        }

        // Platform I/O without holding the lock
        self.state.platform.bind_device(bus_id).await?;

        // Re-acquire lock and update state
        let mut devices = self.state.devices.write().await;
        if let Some(device) = devices.get_mut(bus_id) {
            device.state = DeviceState::Available;
        }
        self.state.emit(ServerEvent::DeviceShared {
            bus_id: bus_id.to_string(),
        });
        Ok(())
    }

    /// Unshare (unbind) a device from the network.
    pub async fn unshare_device(&self, bus_id: &str) -> Result<()> {
        // Check state first, then drop the lock before platform I/O
        {
            let devices = self.state.devices.read().await;
            match devices.get(bus_id) {
                None => bail!("Device not found: {bus_id}"),
                Some(device) => match &device.state {
                    DeviceState::NotShared => bail!("Device {bus_id} is not shared"),
                    DeviceState::Available | DeviceState::InUse { .. } => {} // proceed
                },
            }
        }

        // Platform I/O without holding the lock
        self.state.platform.unbind_device(bus_id).await?;

        // Re-acquire lock and update state
        let mut devices = self.state.devices.write().await;
        if let Some(device) = devices.get_mut(bus_id) {
            device.state = DeviceState::NotShared;
        }
        self.state.emit(ServerEvent::DeviceUnshared {
            bus_id: bus_id.to_string(),
        });
        Ok(())
    }

    /// Set a nickname for a device.
    pub async fn set_nickname(&self, bus_id: &str, nickname: String) -> Result<()> {
        let mut devices = self.state.devices.write().await;
        match devices.get_mut(bus_id) {
            None => bail!("Device not found: {bus_id}"),
            Some(device) => {
                device.nickname = Some(nickname);
                Ok(())
            }
        }
    }

    /// Apply nicknames and USB ID database names to a device.
    fn apply_names(&self, device: &mut openusb_shared::device::UsbDevice) {
        // Apply nickname from config
        let vid_pid = device.vid_pid();
        if let Some(nickname) = self.state.nicknames.get(&vid_pid) {
            device.nickname = Some(nickname.clone());
        }

        // Fill in vendor/product names from USB ID database if not already set
        if device.vendor_name.is_none() {
            device.vendor_name = self.state.vendor_name(device.vendor_id).map(str::to_string);
        }
        if device.product_name.is_none() {
            device.product_name = self
                .state
                .product_name(device.vendor_id, device.product_id)
                .map(str::to_string);
        }
    }
}

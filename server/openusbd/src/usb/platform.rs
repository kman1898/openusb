use anyhow::Result;
use async_trait::async_trait;
use openusb_shared::device::UsbDevice;
use tokio::sync::mpsc;

/// Events emitted by the USB hotplug monitor.
#[derive(Debug, Clone)]
pub enum HotplugEvent {
    Attached(UsbDevice),
    Detached { bus_id: String },
}

/// Platform abstraction for USB operations.
///
/// The real Linux implementation reads sysfs and calls usbip commands.
/// The simulated implementation fakes devices for development on non-Linux systems.
#[async_trait]
pub trait UsbPlatform: Send + Sync + 'static {
    /// Enumerate all currently connected USB devices.
    async fn enumerate_devices(&self) -> Result<Vec<UsbDevice>>;

    /// Bind a device to usbip-host (make it available for remote attachment).
    async fn bind_device(&self, bus_id: &str) -> Result<()>;

    /// Unbind a device from usbip-host.
    async fn unbind_device(&self, bus_id: &str) -> Result<()>;

    /// Start watching for hotplug events. Sends events through the provided channel.
    /// This method runs indefinitely until the channel is closed or an error occurs.
    async fn watch_hotplug(&self, tx: mpsc::Sender<HotplugEvent>) -> Result<()>;
}

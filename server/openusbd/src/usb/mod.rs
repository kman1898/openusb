pub mod bind;
pub mod database;
pub mod device;
pub mod filter;
pub mod manager;
pub mod monitor;
pub mod platform;
pub mod platform_linux;
pub mod platform_sim;

use platform::UsbPlatform;
use std::sync::Arc;

/// Create the appropriate USB platform backend.
pub fn create_platform(simulate: bool) -> Arc<dyn UsbPlatform> {
    if simulate {
        tracing::info!("Using simulated USB platform");
        Arc::new(platform_sim::SimulatedPlatform::new())
    } else {
        #[cfg(target_os = "linux")]
        {
            tracing::info!("Using Linux USB platform (sysfs + usbip)");
            Arc::new(platform_linux::LinuxPlatform::new())
        }
        #[cfg(not(target_os = "linux"))]
        {
            panic!("Real USB platform is only available on Linux. Use --simulate.");
        }
    }
}

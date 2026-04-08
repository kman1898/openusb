use crate::auth::acl::AclEngine;
use crate::auth::users::UserDb;
use crate::config::ServerConfig;
use crate::metrics::bandwidth::BandwidthTracker;
use crate::metrics::history::HistoryDb;
use crate::metrics::latency::LatencyTracker;
use crate::usb::filter::DeviceFilterRules;
use crate::usb::platform::UsbPlatform;
use openusb_shared::device::UsbDevice;
use openusb_shared::protocol::ServerEvent;
use openusb_shared::usb_ids::UsbIdDatabase;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, broadcast};

/// Central application state shared across all subsystems.
pub struct AppState {
    pub config: ServerConfig,
    pub devices: RwLock<HashMap<String, UsbDevice>>,
    pub platform: Arc<dyn UsbPlatform>,
    pub event_tx: broadcast::Sender<ServerEvent>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub usb_ids: Option<UsbIdDatabase>,
    pub nicknames: HashMap<String, String>,
    pub filter: DeviceFilterRules,
    pub user_db: Mutex<UserDb>,
    pub acl: RwLock<AclEngine>,
    pub history_db: Mutex<HistoryDb>,
    pub bandwidth: BandwidthTracker,
    pub latency: LatencyTracker,
}

impl AppState {
    pub fn new(
        config: ServerConfig,
        platform: Arc<dyn UsbPlatform>,
        user_db: UserDb,
        history_db: HistoryDb,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(256);

        let filter = DeviceFilterRules::from_config(
            &config.devices.ignore_vendor_ids,
            &config.devices.ignore_bus_ids,
            &config.devices.allow_vendor_ids,
        );

        let nicknames = config.devices.nicknames.clone();
        let acl = AclEngine::new(config.devices.access.clone());
        let usb_ids = load_usb_ids();

        Self {
            config,
            devices: RwLock::new(HashMap::new()),
            platform,
            event_tx,
            started_at: chrono::Utc::now(),
            usb_ids,
            nicknames,
            filter,
            user_db: Mutex::new(user_db),
            acl: RwLock::new(acl),
            history_db: Mutex::new(history_db),
            bandwidth: BandwidthTracker::new(),
            latency: LatencyTracker::new(),
        }
    }

    pub fn vendor_name(&self, vendor_id: u16) -> Option<&str> {
        self.usb_ids.as_ref()?.vendor_name(vendor_id)
    }

    pub fn product_name(&self, vendor_id: u16, product_id: u16) -> Option<&str> {
        self.usb_ids.as_ref()?.product_name(vendor_id, product_id)
    }

    pub fn emit(&self, event: ServerEvent) {
        let _ = self.event_tx.send(event);
    }
}

fn load_usb_ids() -> Option<UsbIdDatabase> {
    let paths = [
        "/usr/share/hwdata/usb.ids",
        "/usr/share/misc/usb.ids",
        "/var/lib/usbutils/usb.ids",
    ];
    for path in &paths {
        if let Ok(contents) = std::fs::read_to_string(path) {
            tracing::info!(path, "Loaded USB ID database");
            return Some(UsbIdDatabase::parse(&contents));
        }
    }
    tracing::debug!("USB ID database not found (device names may be unavailable)");
    None
}

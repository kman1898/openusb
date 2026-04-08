use crate::api::{LocalApiState, start_local_api};
use crate::config::ClientConfig;
use crate::discovery::ServiceBrowser;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::RwLock;
use tracing::info;
use tray_icon::{Icon, TrayIconBuilder};

/// Shared server count updated from the background thread.
static SERVER_COUNT: AtomicUsize = AtomicUsize::new(0);

/// Run the OpenUSB client with a system tray icon.
/// This blocks the main thread with the event loop.
/// The API server and mDNS browser run on a background tokio runtime.
pub fn run_with_tray(config: ClientConfig, dashboard_url: Option<String>) -> anyhow::Result<()> {
    // Build the tray menu
    let menu = Menu::new();
    let open_dashboard = MenuItem::new("Open Dashboard", true, None);
    let status_item = MenuItem::new("No servers found", false, None);
    let quit = MenuItem::new("Quit OpenUSB", true, None);

    menu.append(&open_dashboard)?;
    menu.append(&status_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit)?;

    let open_id = open_dashboard.id().clone();
    let quit_id = quit.id().clone();

    // Create tray icon
    let icon = create_default_icon();
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("OpenUSB Client")
        .with_icon(icon)
        .build()?;

    // Spawn tokio runtime on a background thread for async work
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let browser = Arc::new(ServiceBrowser::new());
            let api_state = Arc::new(LocalApiState {
                config: RwLock::new(config),
                browser: browser.clone(),
            });

            let mut join_set = tokio::task::JoinSet::new();

            // mDNS browser
            let mdns_browser = browser.clone();
            join_set.spawn(async move { mdns_browser.run().await });

            // Local API server
            let api = api_state.clone();
            join_set.spawn(async move { start_local_api(api).await });

            // Status updater — update the shared counter for the main thread to read
            let status_browser = browser.clone();
            join_set.spawn(async move {
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    let servers = status_browser.servers();
                    let count = servers.read().await.len();
                    SERVER_COUNT.store(count, Ordering::Relaxed);
                }
                #[allow(unreachable_code)]
                Ok::<(), anyhow::Error>(())
            });

            info!("OpenUSB client running (API on localhost:9245)");
            join_set.join_all().await;
        });
    });

    // Main thread: handle menu events with polling
    let menu_channel = MenuEvent::receiver();
    let url = dashboard_url.unwrap_or_else(|| "http://localhost:8443".to_string());
    let mut last_count = 0usize;

    loop {
        // Check for menu events (non-blocking with timeout)
        if let Ok(event) = menu_channel.recv_timeout(std::time::Duration::from_secs(2)) {
            if event.id() == &open_id {
                info!("Opening dashboard: {}", url);
                let _ = open::that(&url);
            } else if event.id() == &quit_id {
                info!("Quit requested");
                std::process::exit(0);
            }
        }

        // Update status text from the shared counter
        let count = SERVER_COUNT.load(Ordering::Relaxed);
        if count != last_count {
            let text = if count == 0 {
                "No servers found".to_string()
            } else {
                format!(
                    "{} server{} found",
                    count,
                    if count == 1 { "" } else { "s" }
                )
            };
            status_item.set_text(&text);
            last_count = count;
        }
    }
}

/// Create a simple 16x16 RGBA icon (green circle on transparent background).
fn create_default_icon() -> Icon {
    let size = 16u32;
    let mut rgba = vec![0u8; (size * size * 4) as usize];

    let center = size as f32 / 2.0;
    let radius = 6.0f32;

    for y in 0..size {
        for x in 0..size {
            let dx = x as f32 - center;
            let dy = y as f32 - center;
            let dist = (dx * dx + dy * dy).sqrt();
            let idx = ((y * size + x) * 4) as usize;

            if dist <= radius {
                rgba[idx] = 0x22; // R
                rgba[idx + 1] = 0xc5; // G
                rgba[idx + 2] = 0x5e; // B
                rgba[idx + 3] = 0xFF; // A
            }
        }
    }

    Icon::from_rgba(rgba, size, size).expect("Failed to create icon")
}

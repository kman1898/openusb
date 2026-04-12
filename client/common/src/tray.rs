use crate::api::{LocalApiState, start_local_api};
use crate::config::ClientConfig;
use crate::discovery::ServiceBrowser;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tokio::sync::RwLock;
use tray_icon::{Icon, TrayIconBuilder};

/// Shared state updated from the background thread.
static SERVER_COUNT: AtomicUsize = AtomicUsize::new(0);
static API_READY: AtomicBool = AtomicBool::new(false);

const DEFAULT_API_PORT: u16 = 9245;

/// Run the OpenUSB client with a system tray icon.
/// This blocks the main thread with the platform event loop.
pub fn run_with_tray(config: ClientConfig, dashboard_url: Option<String>) -> anyhow::Result<()> {
    // Set up logging to file for debugging
    setup_file_logging();

    tracing::info!("Starting OpenUSB tray app");

    let api_port = DEFAULT_API_PORT;
    let url = dashboard_url.unwrap_or_else(|| "http://localhost:8443".to_string());

    // Build the tray menu
    let menu = Menu::new();

    let open_dashboard = MenuItem::new("Open Dashboard", true, None);
    let separator1 = PredefinedMenuItem::separator();

    let status_item = MenuItem::new("Starting...", false, None);
    let servers_item = MenuItem::new("No servers found", false, None);
    let api_item = MenuItem::new(format!("Client API: localhost:{}", api_port), false, None);

    let separator2 = PredefinedMenuItem::separator();

    let settings_menu = Submenu::new("Settings", true);
    let port_item = MenuItem::new(format!("API Port: {}", api_port), false, None);
    settings_menu.append(&port_item)?;

    let separator3 = PredefinedMenuItem::separator();
    let quit = MenuItem::new("Quit OpenUSB", true, None);

    menu.append(&open_dashboard)?;
    menu.append(&separator1)?;
    menu.append(&status_item)?;
    menu.append(&servers_item)?;
    menu.append(&api_item)?;
    menu.append(&separator2)?;
    menu.append(&settings_menu)?;
    menu.append(&separator3)?;
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

    tracing::info!("Tray icon created");

    // Spawn background services on a separate thread with its own tokio runtime
    let bg_config = config.clone();
    let bg_url = url.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        rt.block_on(async {
            let browser = Arc::new(ServiceBrowser::new());
            let api_state = Arc::new(LocalApiState {
                config: RwLock::new(bg_config),
                browser: browser.clone(),
            });

            let mut join_set = tokio::task::JoinSet::new();

            // mDNS browser
            let mdns_browser = browser.clone();
            join_set.spawn(async move {
                tracing::info!("Starting mDNS browser");
                mdns_browser.run().await
            });

            // Local API server
            let api = api_state.clone();
            join_set.spawn(async move {
                tracing::info!("Starting local API on port {}", DEFAULT_API_PORT);
                let result = start_local_api(api).await;
                API_READY.store(true, Ordering::Relaxed);
                result
            });

            // Mark API as ready after a short delay
            join_set.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                API_READY.store(true, Ordering::Relaxed);
                tracing::info!("API server ready");
                Ok(())
            });

            // Server count updater
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

            // Auto-open browser after startup
            join_set.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
                tracing::info!("Auto-opening browser: {}", bg_url);
                let _ = open::that(&bg_url);
                Ok(())
            });

            tracing::info!("OpenUSB client services running");
            join_set.join_all().await;
        });
    });

    // Main thread: platform event loop
    tracing::info!("Entering main event loop");
    run_event_loop(open_id, quit_id, url, status_item, servers_item);

    Ok(())
}

/// Platform-specific event loop.
/// On macOS, this needs to run the Cocoa run loop.
/// On other platforms, a simple polling loop works.
fn run_event_loop(
    open_id: muda::MenuId,
    quit_id: muda::MenuId,
    url: String,
    status_item: MenuItem,
    servers_item: MenuItem,
) {
    #[cfg(target_os = "macos")]
    {
        // macOS requires running on the main thread with a proper run loop.
        // We use a timer-based approach to poll for events while the run loop runs.
        use std::ffi::c_void;

        unsafe extern "C" {
            fn CFRunLoopGetCurrent() -> *mut c_void;
            fn CFRunLoopRun();
            fn CFRunLoopTimerCreate(
                allocator: *mut c_void,
                fire_date: f64,
                interval: f64,
                flags: u64,
                order: i64,
                callback: extern "C" fn(*mut c_void, *mut c_void),
                context: *mut c_void,
            ) -> *mut c_void;
            fn CFRunLoopAddTimer(rl: *mut c_void, timer: *mut c_void, mode: *mut c_void);
            fn CFAbsoluteTimeGetCurrent() -> f64;
        }

        unsafe extern "C" {
            static kCFRunLoopDefaultMode: *mut c_void;
        }

        // Store state in a Box that we leak for the callback
        struct CallbackState {
            open_id: muda::MenuId,
            quit_id: muda::MenuId,
            url: String,
            status_item: MenuItem,
            servers_item: MenuItem,
            last_count: usize,
            was_ready: bool,
        }

        let state = Box::new(CallbackState {
            open_id,
            quit_id,
            url,
            status_item,
            servers_item,
            last_count: 0,
            was_ready: false,
        });
        let state_ptr = Box::into_raw(state) as *mut c_void;

        extern "C" fn timer_callback(_timer: *mut c_void, info: *mut c_void) {
            let state = unsafe { &mut *(info as *mut CallbackState) };
            let menu_channel = MenuEvent::receiver();

            while let Ok(event) = menu_channel.try_recv() {
                if event.id() == &state.open_id {
                    tracing::info!("Opening dashboard: {}", state.url);
                    let _ = open::that(&state.url);
                } else if event.id() == &state.quit_id {
                    tracing::info!("Quit requested");
                    std::process::exit(0);
                }
            }

            // Update status
            let ready = API_READY.load(Ordering::Relaxed);
            if ready && !state.was_ready {
                state.status_item.set_text("Running");
                state.was_ready = true;
            }

            let count = SERVER_COUNT.load(Ordering::Relaxed);
            if count != state.last_count {
                let text = if count == 0 {
                    "No servers found".to_string()
                } else {
                    format!(
                        "{} server{} found",
                        count,
                        if count == 1 { "" } else { "s" }
                    )
                };
                state.servers_item.set_text(&text);
                state.last_count = count;
            }
        }

        unsafe {
            let rl = CFRunLoopGetCurrent();
            let now = CFAbsoluteTimeGetCurrent();
            let timer = CFRunLoopTimerCreate(
                std::ptr::null_mut(),
                now,
                0.5, // check every 500ms
                0,
                0,
                timer_callback,
                state_ptr,
            );
            CFRunLoopAddTimer(rl, timer, kCFRunLoopDefaultMode);
            tracing::info!("Starting CFRunLoop");
            CFRunLoopRun();
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        let menu_channel = MenuEvent::receiver();
        let mut last_count = 0usize;
        let mut was_ready = false;

        // Non-macOS: simple polling loop
        loop {
            if let Ok(event) = menu_channel.recv_timeout(std::time::Duration::from_millis(500)) {
                if event.id() == &open_id {
                    tracing::info!("Opening dashboard: {}", url);
                    let _ = open::that(&url);
                } else if event.id() == &quit_id {
                    tracing::info!("Quit requested");
                    std::process::exit(0);
                }
            }

            let ready = API_READY.load(Ordering::Relaxed);
            if ready && !was_ready {
                status_item.set_text("Running");
                was_ready = true;
            }

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
                servers_item.set_text(&text);
                last_count = count;
            }
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

/// Set up logging to a file in the user's config directory.
fn setup_file_logging() {
    let log_dir = if let Some(home) = std::env::var_os("HOME") {
        std::path::PathBuf::from(home)
            .join("Library")
            .join("Logs")
            .join("OpenUSB")
    } else if let Some(appdata) = std::env::var_os("APPDATA") {
        std::path::PathBuf::from(appdata)
            .join("OpenUSB")
            .join("logs")
    } else {
        std::path::PathBuf::from("/tmp/openusb")
    };

    let _ = std::fs::create_dir_all(&log_dir);
    let log_file = log_dir.join("client.log");

    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)
    {
        let _ = tracing_subscriber::fmt()
            .with_writer(std::sync::Mutex::new(file))
            .with_ansi(false)
            .try_init();
        eprintln!("OpenUSB: logging to {}", log_file.display());
    }
}

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
        // macOS requires the main thread to run a Cocoa event loop for the
        // tray icon to render. We use NSApplication's run loop.
        use std::ffi::c_void;

        #[link(name = "AppKit", kind = "framework")]
        unsafe extern "C" {}

        unsafe extern "C" {
            fn NSApplicationLoad() -> bool;
        }

        #[repr(C)]
        struct CFRunLoopTimerContext {
            version: isize,
            info: *mut c_void,
            retain: *const c_void,
            release: *const c_void,
            copy_description: *const c_void,
        }

        unsafe extern "C" {
            fn CFRunLoopGetCurrent() -> *mut c_void;
            fn CFRunLoopRun();
            fn CFRunLoopTimerCreate(
                allocator: *mut c_void,
                fire_date: f64,
                interval: f64,
                flags: u64,
                order: isize,
                callback: unsafe extern "C" fn(timer: *mut c_void, info: *mut c_void),
                context: *const CFRunLoopTimerContext,
            ) -> *mut c_void;
            fn CFRunLoopAddTimer(rl: *mut c_void, timer: *mut c_void, mode: *mut c_void);
            fn CFAbsoluteTimeGetCurrent() -> f64;
            static kCFRunLoopDefaultMode: *mut c_void;
        }

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
        let state_ptr = Box::into_raw(state);

        unsafe extern "C" fn timer_callback(_timer: *mut c_void, info: *mut c_void) {
            let state = &mut *(info as *mut CallbackState);
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
            NSApplicationLoad();

            let context = CFRunLoopTimerContext {
                version: 0,
                info: state_ptr as *mut c_void,
                retain: std::ptr::null(),
                release: std::ptr::null(),
                copy_description: std::ptr::null(),
            };

            let rl = CFRunLoopGetCurrent();
            let now = CFAbsoluteTimeGetCurrent();
            let timer = CFRunLoopTimerCreate(
                std::ptr::null_mut(),
                now,
                0.5,
                0,
                0,
                timer_callback,
                &context,
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

/// Load the OpenUSB logo embedded at compile time.
/// Falls back to a simple green circle if decoding fails.
fn create_default_icon() -> Icon {
    // Embed the 32x32 PNG at compile time
    static ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-32.png");

    if let Ok(icon) = decode_png_icon(ICON_PNG) {
        return icon;
    }

    // Fallback: simple 16x16 green circle
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
                rgba[idx] = 0x22;
                rgba[idx + 1] = 0xc5;
                rgba[idx + 2] = 0x5e;
                rgba[idx + 3] = 0xFF;
            }
        }
    }
    Icon::from_rgba(rgba, size, size).expect("Failed to create fallback icon")
}

/// Decode a PNG file into an RGBA Icon.
fn decode_png_icon(png_data: &[u8]) -> Result<Icon, Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(std::io::Cursor::new(png_data));
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let rgba = &buf[..info.buffer_size()];

    // Convert to RGBA if needed (PNG might be RGB without alpha)
    let rgba_data = if info.color_type == png::ColorType::Rgba {
        rgba.to_vec()
    } else if info.color_type == png::ColorType::Rgb {
        let mut out = Vec::with_capacity(info.width as usize * info.height as usize * 4);
        for chunk in rgba.chunks(3) {
            out.extend_from_slice(chunk);
            out.push(0xFF);
        }
        out
    } else {
        return Err("Unsupported PNG color type".into());
    };

    Ok(Icon::from_rgba(rgba_data, info.width, info.height)?)
}

/// Set up logging to a `logs/` folder next to the executable.
fn setup_file_logging() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });

    // On macOS .app bundles, the exe is inside Contents/MacOS/ — put logs next to the .app
    let log_dir = if exe_dir.ends_with("Contents/MacOS") {
        exe_dir
            .parent() // Contents
            .and_then(|p| p.parent()) // .app
            .and_then(|p| p.parent()) // folder containing .app
            .unwrap_or(&exe_dir)
            .join("logs")
    } else {
        exe_dir.join("logs")
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

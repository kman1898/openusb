use crate::api::{LocalApiState, start_local_api};
use crate::config::ClientConfig;
use crate::discovery::ServiceBrowser;
use muda::{Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tokio::sync::RwLock;
use tray_icon::{Icon, TrayIconBuilder};

static SERVER_COUNT: AtomicUsize = AtomicUsize::new(0);
static API_READY: AtomicBool = AtomicBool::new(false);

const DEFAULT_API_PORT: u16 = 9245;

/// Run the OpenUSB client with a system tray icon.
/// Uses tao event loop for proper macOS/Windows/Linux integration.
pub fn run_with_tray(config: ClientConfig, dashboard_url: Option<String>) -> anyhow::Result<()> {
    setup_file_logging();
    tracing::info!("Starting OpenUSB tray app");

    let api_port = DEFAULT_API_PORT;
    let url = dashboard_url.unwrap_or_else(|| "http://localhost:8443".to_string());

    // Build tao event loop
    let mut event_loop = EventLoopBuilder::new().build();

    // On macOS, set Accessory activation policy to hide dock icon
    #[cfg(target_os = "macos")]
    {
        use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
        event_loop.set_activation_policy(ActivationPolicy::Accessory);
    }

    // Build the tray menu
    let menu = Menu::new();
    let open_dashboard = MenuItem::new("Open Dashboard", true, None);
    let status_item = MenuItem::new("Starting...", false, None);
    let servers_item = MenuItem::new("No servers found", false, None);
    let api_item = MenuItem::new(format!("Client API: localhost:{}", api_port), false, None);
    let settings_menu = Submenu::new("Settings", true);
    let port_item = MenuItem::new(format!("API Port: {}", api_port), false, None);
    settings_menu.append(&port_item)?;
    let quit = MenuItem::new("Quit OpenUSB", true, None);

    menu.append(&open_dashboard)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&status_item)?;
    menu.append(&servers_item)?;
    menu.append(&api_item)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&settings_menu)?;
    menu.append(&PredefinedMenuItem::separator())?;
    menu.append(&quit)?;

    let open_id = open_dashboard.id().clone();
    let quit_id = quit.id().clone();

    // Spawn background services
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

            let mdns_browser = browser.clone();
            join_set.spawn(async move { mdns_browser.run().await });

            let api = api_state.clone();
            join_set.spawn(async move {
                let result = start_local_api(api).await;
                API_READY.store(true, Ordering::Relaxed);
                result
            });

            join_set.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                API_READY.store(true, Ordering::Relaxed);
                Ok(())
            });

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

    // Create tray icon — must happen after event loop is created
    let icon = create_default_icon();
    let _tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip("OpenUSB Client")
        .with_icon(icon)
        .build()?;

    tracing::info!("Tray icon created, entering event loop");

    let menu_channel = MenuEvent::receiver();
    let mut last_count = 0usize;
    let mut was_ready = false;

    // Run the event loop — this properly integrates with macOS AppKit
    event_loop.run(move |event, _, control_flow| {
        // Poll with a timeout so we can update status periodically
        *control_flow =
            ControlFlow::WaitUntil(std::time::Instant::now() + std::time::Duration::from_secs(2));

        // Handle menu events
        if let Ok(event) = menu_channel.try_recv() {
            if event.id() == &open_id {
                tracing::info!("Opening dashboard: {}", url);
                let _ = open::that(&url);
            } else if event.id() == &quit_id {
                tracing::info!("Quit requested");
                *control_flow = ControlFlow::Exit;
            }
        }

        // Update status items
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

        if let Event::NewEvents(StartCause::Init) = event {
            tracing::info!("Event loop initialized");
        }
    });
}

/// Load the OpenUSB logo embedded at compile time.
fn create_default_icon() -> Icon {
    static ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-32.png");

    if let Ok(icon) = decode_png_icon(ICON_PNG) {
        return icon;
    }

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

fn decode_png_icon(png_data: &[u8]) -> Result<Icon, Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(std::io::Cursor::new(png_data));
    let mut reader = decoder.read_info()?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    let rgba = &buf[..info.buffer_size()];

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

fn setup_file_logging() {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| {
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
        });

    let log_dir = if exe_dir.ends_with("Contents/MacOS") {
        exe_dir
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
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

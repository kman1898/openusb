use crate::state::AppState;
use axum::Router;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

use super::rest;
use super::websocket;

pub fn api_router(state: Arc<AppState>) -> Router {
    // Determine the web dashboard directory
    // In production: /usr/share/openusb/web or next to the binary
    // In development: ../web-dashboard/dist (relative to server crate)
    let web_dir = find_web_dir();

    let mut app = Router::new()
        .route(
            "/api/v1/server/info",
            axum::routing::get(rest::get_server_info),
        )
        .route("/api/v1/devices", axum::routing::get(rest::list_devices))
        .route(
            "/api/v1/devices/{bus_id}/share",
            axum::routing::post(rest::share_device),
        )
        .route(
            "/api/v1/devices/{bus_id}/unshare",
            axum::routing::post(rest::unshare_device),
        )
        .route(
            "/api/v1/devices/{bus_id}/nickname",
            axum::routing::put(rest::set_nickname),
        )
        .route(
            "/api/v1/events",
            axum::routing::get(websocket::websocket_handler),
        )
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .with_state(state);

    // Serve web dashboard static files if the directory exists
    if let Some(dir) = web_dir {
        let index = format!("{}/index.html", dir);
        let serve_dir = ServeDir::new(&dir).not_found_service(ServeFile::new(&index));
        app = app.fallback_service(serve_dir);
        tracing::info!(path = %dir, "Serving web dashboard");
    } else {
        tracing::warn!("Web dashboard directory not found — API-only mode");
    }

    app
}

fn find_web_dir() -> Option<String> {
    let candidates = [
        // Development: relative to where cargo runs from
        "web-dashboard/dist",
        "../web-dashboard/dist",
        // Installed locations
        "/usr/share/openusb/web",
        "/opt/openusb/web",
    ];

    for path in &candidates {
        let p = std::path::Path::new(path);
        if p.is_dir() && p.join("index.html").exists() {
            return Some(path.to_string());
        }
    }

    None
}

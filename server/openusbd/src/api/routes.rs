use crate::auth::middleware::{admin_middleware, auth_middleware};
use crate::state::AppState;
use axum::Router;
use axum::middleware;
use std::sync::Arc;
use tower_http::services::{ServeDir, ServeFile};

use super::rest;
use super::websocket;

pub fn api_router(state: Arc<AppState>) -> Router {
    let web_dir = find_web_dir();

    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/api/v1/auth/login", axum::routing::post(rest::login))
        .route(
            "/api/v1/events",
            axum::routing::get(websocket::websocket_handler),
        );

    // Protected routes (auth required)
    let protected_routes = Router::new()
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
            "/api/v1/devices/{bus_id}/kick",
            axum::routing::post(rest::kick_device),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    // Admin-only routes
    let admin_routes = Router::new()
        .route("/api/v1/users", axum::routing::get(rest::list_users))
        .route("/api/v1/users", axum::routing::post(rest::create_user))
        .route(
            "/api/v1/users/{username}",
            axum::routing::put(rest::update_user),
        )
        .route(
            "/api/v1/users/{username}",
            axum::routing::delete(rest::delete_user),
        )
        .route("/api/v1/acl", axum::routing::get(rest::get_acls))
        .route(
            "/api/v1/acl/{device_key}",
            axum::routing::put(rest::set_device_acl),
        )
        .route(
            "/api/v1/acl/{device_key}",
            axum::routing::delete(rest::delete_device_acl),
        )
        .layer(middleware::from_fn(admin_middleware))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let mut app = public_routes
        .merge(protected_routes)
        .merge(admin_routes)
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
        "web-dashboard/dist",
        "../web-dashboard/dist",
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

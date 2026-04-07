use crate::state::AppState;
use axum::Router;
use std::sync::Arc;

use super::rest;
use super::websocket;

pub fn api_router(state: Arc<AppState>) -> Router {
    Router::new()
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
        .with_state(state)
}

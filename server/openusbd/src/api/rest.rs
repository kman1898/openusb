use crate::state::AppState;
use crate::usb::manager::DeviceManager;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use openusb_shared::protocol::ServerInfo;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

use super::routes;

/// Start the REST API + WebSocket server.
pub async fn start_api_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let app = routes::api_router(state.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], state.config.server.api_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "REST API server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

/// GET /api/v1/server/info
pub async fn get_server_info(State(state): State<Arc<AppState>>) -> Json<ServerInfo> {
    let devices = state.devices.read().await;
    let hostname = state
        .config
        .server
        .hostname
        .clone()
        .unwrap_or_else(|| gethostname::gethostname().to_string_lossy().to_string());

    let uptime = chrono::Utc::now()
        .signed_duration_since(state.started_at)
        .num_seconds()
        .unsigned_abs();

    Json(ServerInfo {
        name: state.config.server.name.clone(),
        hostname,
        version: env!("CARGO_PKG_VERSION").to_string(),
        api_port: state.config.server.api_port,
        usbip_port: state.config.server.port,
        device_count: devices.len(),
        client_count: 0, // TODO: track connected USB/IP clients
        uptime_seconds: uptime,
        tls_enabled: state.config.security.tls_enabled,
        auth_required: state.config.security.mode != "open",
    })
}

/// GET /api/v1/devices
pub async fn list_devices(
    State(state): State<Arc<AppState>>,
) -> Json<Vec<openusb_shared::device::UsbDevice>> {
    let devices = state.devices.read().await;
    Json(devices.values().cloned().collect())
}

/// POST /api/v1/devices/:bus_id/share
pub async fn share_device(
    State(state): State<Arc<AppState>>,
    Path(bus_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let manager = DeviceManager::new(state);
    manager.share_device(&bus_id).await.map_err(|e| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: e.to_string(),
    })?;
    Ok(StatusCode::OK)
}

/// POST /api/v1/devices/:bus_id/unshare
pub async fn unshare_device(
    State(state): State<Arc<AppState>>,
    Path(bus_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let manager = DeviceManager::new(state);
    manager
        .unshare_device(&bus_id)
        .await
        .map_err(|e| ApiError {
            status: StatusCode::BAD_REQUEST,
            message: e.to_string(),
        })?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
pub struct NicknameBody {
    pub nickname: String,
}

/// PUT /api/v1/devices/:bus_id/nickname
pub async fn set_nickname(
    State(state): State<Arc<AppState>>,
    Path(bus_id): Path<String>,
    Json(body): Json<NicknameBody>,
) -> Result<StatusCode, ApiError> {
    let manager = DeviceManager::new(state);
    manager
        .set_nickname(&bus_id, body.nickname)
        .await
        .map_err(|e| ApiError {
            status: StatusCode::NOT_FOUND,
            message: e.to_string(),
        })?;
    Ok(StatusCode::OK)
}

/// Consistent error response for the API.
pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({
            "error": self.message,
        });
        (self.status, Json(body)).into_response()
    }
}

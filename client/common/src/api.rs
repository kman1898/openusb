use crate::config::ClientConfig;
use crate::discovery::ServiceBrowser;
use crate::usbip;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Json, Router};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// Shared state for the local client API.
pub struct LocalApiState {
    pub config: RwLock<ClientConfig>,
    pub browser: Arc<ServiceBrowser>,
}

/// Start the local client API server on localhost:9245.
pub async fn start_local_api(state: Arc<LocalApiState>) -> anyhow::Result<()> {
    let app = local_api_router(state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 9245));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "Local client API listening");
    axum::serve(listener, app).await?;
    Ok(())
}

fn local_api_router(state: Arc<LocalApiState>) -> Router {
    Router::new()
        .route("/api/status", axum::routing::get(get_status))
        .route("/api/devices", axum::routing::get(get_devices))
        .route("/api/attach", axum::routing::post(attach_device))
        .route("/api/detach", axum::routing::post(detach_device))
        .route("/api/detach-all", axum::routing::post(detach_all))
        .route("/api/servers", axum::routing::get(get_servers))
        .route("/api/driver", axum::routing::get(get_driver))
        .route("/api/config", axum::routing::get(get_config))
        .route("/api/config", axum::routing::put(update_config))
        .route("/api/auto-use", axum::routing::get(get_auto_use))
        .route("/api/auto-use", axum::routing::post(add_auto_use))
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        )
        .with_state(state)
}

/// GET /api/status
async fn get_status(State(_state): State<Arc<LocalApiState>>) -> Json<serde_json::Value> {
    let driver = usbip::check_driver()
        .await
        .unwrap_or(usbip::DriverStatus::NotInstalled);
    Json(serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "driver_status": driver,
    }))
}

/// GET /api/devices — currently attached USB/IP devices on this PC.
async fn get_devices() -> Result<Json<Vec<usbip::AttachedDevice>>, ApiError> {
    let devices = usbip::list_attached()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(devices))
}

#[derive(Deserialize)]
struct AttachBody {
    server: String,
    busid: String,
}

/// POST /api/attach
async fn attach_device(Json(body): Json<AttachBody>) -> Result<StatusCode, ApiError> {
    usbip::attach(&body.server, &body.busid)
        .await
        .map_err(|e| ApiError(StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(StatusCode::OK)
}

#[derive(Deserialize)]
struct DetachBody {
    busid: String,
}

/// POST /api/detach
async fn detach_device(Json(body): Json<DetachBody>) -> Result<StatusCode, ApiError> {
    usbip::detach(&body.busid)
        .await
        .map_err(|e| ApiError(StatusCode::BAD_REQUEST, e.to_string()))?;
    Ok(StatusCode::OK)
}

/// POST /api/detach-all
async fn detach_all() -> Result<StatusCode, ApiError> {
    usbip::detach_all()
        .await
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

/// GET /api/servers — discovered servers via mDNS.
async fn get_servers(State(state): State<Arc<LocalApiState>>) -> Json<serde_json::Value> {
    let servers = state.browser.servers();
    let map = servers.read().await;
    let list: Vec<_> = map
        .values()
        .map(|s| {
            serde_json::json!({
                "name": s.name,
                "host": s.host,
                "port": s.port,
                "api_port": s.api_port,
                "version": s.version,
            })
        })
        .collect();
    Json(serde_json::json!(list))
}

/// GET /api/driver — driver install status.
async fn get_driver() -> Json<serde_json::Value> {
    let status = usbip::check_driver()
        .await
        .unwrap_or(usbip::DriverStatus::NotInstalled);
    Json(serde_json::json!(status))
}

/// GET /api/config
async fn get_config(State(state): State<Arc<LocalApiState>>) -> Json<ClientConfig> {
    let config = state.config.read().await.clone();
    Json(config)
}

/// PUT /api/config
async fn update_config(
    State(state): State<Arc<LocalApiState>>,
    Json(new_config): Json<ClientConfig>,
) -> Result<StatusCode, ApiError> {
    let mut config = state.config.write().await;
    *config = new_config;
    config
        .save()
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::OK)
}

/// GET /api/auto-use
async fn get_auto_use(
    State(state): State<Arc<LocalApiState>>,
) -> Json<Vec<crate::config::AutoUseRule>> {
    let config = state.config.read().await;
    Json(config.auto_use_rules.clone())
}

/// POST /api/auto-use
async fn add_auto_use(
    State(state): State<Arc<LocalApiState>>,
    Json(rule): Json<crate::config::AutoUseRule>,
) -> Result<StatusCode, ApiError> {
    let mut config = state.config.write().await;
    config.auto_use_rules.push(rule);
    config
        .save()
        .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(StatusCode::CREATED)
}

struct ApiError(StatusCode, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = serde_json::json!({ "error": self.1 });
        (self.0, Json(body)).into_response()
    }
}

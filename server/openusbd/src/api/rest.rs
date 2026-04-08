use crate::auth::tokens;
use crate::auth::users::{CreateUser, UpdateUser};
use crate::state::AppState;
use crate::usb::manager::DeviceManager;
use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use openusb_shared::config::DeviceAcl;
use openusb_shared::protocol::ServerInfo;
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::info;

/// Start the REST API + WebSocket server.
pub async fn start_api_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let app = super::routes::api_router(state.clone());
    let addr = SocketAddr::from(([0, 0, 0, 0], state.config.server.api_port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    info!(%addr, "REST API server listening");
    axum::serve(listener, app).await?;
    Ok(())
}

// ──── Auth ────

#[derive(Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

/// POST /api/v1/auth/login
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(body): Json<LoginBody>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // In open mode, generate a token for anyone
    if state.config.security.mode == "open" {
        let token = tokens::create_token(
            &body.username,
            "admin",
            &state.config.security.jwt_secret,
            state.config.security.token_expire_hours,
        )
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        })?;
        return Ok(Json(serde_json::json!({
            "token": token,
            "username": body.username,
            "role": "admin",
        })));
    }

    let db = state.user_db.lock().await;
    let user = db
        .authenticate(&body.username, &body.password)
        .map_err(|e| ApiError {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: e.to_string(),
        })?;

    match user {
        Some(u) => {
            let token = tokens::create_token(
                &u.username,
                &u.role,
                &state.config.security.jwt_secret,
                state.config.security.token_expire_hours,
            )
            .map_err(|e| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: e.to_string(),
            })?;
            Ok(Json(serde_json::json!({
                "token": token,
                "username": u.username,
                "role": u.role,
            })))
        }
        None => {
            state.emit(openusb_shared::protocol::ServerEvent::AuthFailed {
                client_ip: "unknown".to_string(),
                reason: format!("Invalid credentials for {}", body.username),
            });
            Err(ApiError {
                status: StatusCode::UNAUTHORIZED,
                message: "Invalid username or password".to_string(),
            })
        }
    }
}

// ──── Server Info ────

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

// ──── Devices ────

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

/// POST /api/v1/devices/:bus_id/kick — force-disconnect a client from a device.
pub async fn kick_device(
    State(state): State<Arc<AppState>>,
    Path(bus_id): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut devices = state.devices.write().await;
    if let Some(device) = devices.get_mut(&bus_id) {
        if matches!(
            device.state,
            openusb_shared::device::DeviceState::InUse { .. }
        ) {
            device.state = openusb_shared::device::DeviceState::Available;
            state.emit(openusb_shared::protocol::ServerEvent::DeviceReleased {
                bus_id: bus_id.clone(),
            });
            info!(bus_id = %bus_id, "Client kicked from device");
            Ok(StatusCode::OK)
        } else {
            Err(ApiError {
                status: StatusCode::BAD_REQUEST,
                message: "Device is not in use".to_string(),
            })
        }
    } else {
        Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: "Device not found".to_string(),
        })
    }
}

// ──── User Management ────

/// GET /api/v1/users
pub async fn list_users(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state.user_db.lock().await;
    let users = db.list_users().map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: e.to_string(),
    })?;
    Ok(Json(serde_json::json!(users)))
}

/// POST /api/v1/users
pub async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateUser>,
) -> Result<(StatusCode, Json<serde_json::Value>), ApiError> {
    let db = state.user_db.lock().await;
    let user = db.create_user(&body).map_err(|e| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: e.to_string(),
    })?;
    info!(username = %user.username, role = %user.role, "Created user");
    Ok((StatusCode::CREATED, Json(serde_json::json!(user))))
}

/// PUT /api/v1/users/:username
pub async fn update_user(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
    Json(body): Json<UpdateUser>,
) -> Result<StatusCode, ApiError> {
    let db = state.user_db.lock().await;
    let updated = db.update_user(&username, &body).map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: e.to_string(),
    })?;
    if updated {
        info!(username = %username, "Updated user");
        Ok(StatusCode::OK)
    } else {
        Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: "User not found".to_string(),
        })
    }
}

/// DELETE /api/v1/users/:username
pub async fn delete_user(
    State(state): State<Arc<AppState>>,
    Path(username): Path<String>,
) -> Result<StatusCode, ApiError> {
    let db = state.user_db.lock().await;
    let deleted = db.delete_user(&username).map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: e.to_string(),
    })?;
    if deleted {
        info!(username = %username, "Deleted user");
        Ok(StatusCode::OK)
    } else {
        Err(ApiError {
            status: StatusCode::NOT_FOUND,
            message: "User not found".to_string(),
        })
    }
}

// ──── ACL Management ────

/// GET /api/v1/acl
pub async fn get_acls(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let acl = state.acl.read().await;
    Json(serde_json::json!(acl.all_rules()))
}

/// PUT /api/v1/acl/:device_key
pub async fn set_device_acl(
    State(state): State<Arc<AppState>>,
    Path(device_key): Path<String>,
    Json(body): Json<DeviceAcl>,
) -> StatusCode {
    let mut acl = state.acl.write().await;
    acl.set_acl(device_key, body);
    StatusCode::OK
}

/// DELETE /api/v1/acl/:device_key
pub async fn delete_device_acl(
    State(state): State<Arc<AppState>>,
    Path(device_key): Path<String>,
) -> StatusCode {
    let mut acl = state.acl.write().await;
    acl.remove_acl(&device_key);
    StatusCode::OK
}

// ──── Metrics & History ────

/// GET /api/v1/metrics/bandwidth
pub async fn get_bandwidth(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let stats = state.bandwidth.all_stats().await;
    Json(serde_json::json!(stats))
}

/// GET /api/v1/metrics/latency
pub async fn get_latency(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let stats = state.latency.all_stats().await;
    Json(serde_json::json!(stats))
}

#[derive(Deserialize)]
pub struct HistoryQuery {
    #[serde(default = "default_history_limit")]
    pub limit: usize,
    pub event_type: Option<String>,
}

fn default_history_limit() -> usize {
    100
}

/// GET /api/v1/history?limit=100&event_type=device_attached
pub async fn get_history(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<HistoryQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let db = state.history_db.lock().await;
    let entries = if let Some(ref event_type) = query.event_type {
        db.by_type(event_type, query.limit)
    } else {
        db.recent(query.limit)
    }
    .map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: e.to_string(),
    })?;
    Ok(Json(serde_json::json!(entries)))
}

// ──── Error Type ────

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

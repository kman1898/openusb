use crate::state::AppState;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use std::sync::Arc;

use super::tokens;

/// Auth middleware that checks JWT tokens on protected routes.
/// Extracts the Bearer token from Authorization header and validates it.
/// Sets the username as a request extension for downstream handlers.
pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Response {
    // If security mode is "open", skip auth
    if state.config.security.mode == "open" {
        req.extensions_mut().insert(AuthUser {
            username: "anonymous".to_string(),
            role: "admin".to_string(),
        });
        return next.run(req).await;
    }

    // Extract Bearer token
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "));

    let token = match token {
        Some(t) => t,
        None => {
            return (
                StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "error": "Missing authentication token" })),
            )
                .into_response();
        }
    };

    // Validate token
    let secret = &state.config.security.jwt_secret;
    match tokens::validate_token(token, secret) {
        Ok(claims) => {
            req.extensions_mut().insert(AuthUser {
                username: claims.sub,
                role: claims.role,
            });
            next.run(req).await
        }
        Err(_) => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Invalid or expired token" })),
        )
            .into_response(),
    }
}

/// Middleware that requires admin role.
pub async fn admin_middleware(req: Request, next: Next) -> Response {
    let auth_user = req.extensions().get::<AuthUser>().cloned();
    match auth_user {
        Some(user) if user.role == "admin" => next.run(req).await,
        Some(_) => (
            StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({ "error": "Admin access required" })),
        )
            .into_response(),
        None => (
            StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Not authenticated" })),
        )
            .into_response(),
    }
}

/// Authenticated user identity, set as a request extension by auth_middleware.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub username: String,
    pub role: String,
}

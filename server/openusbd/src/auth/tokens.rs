use anyhow::Result;
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    /// Username
    pub sub: String,
    /// Role (admin, user, viewer)
    pub role: String,
    /// Expiry (Unix timestamp)
    pub exp: i64,
    /// Issued at (Unix timestamp)
    pub iat: i64,
}

/// Generate a JWT token for an authenticated user.
pub fn create_token(username: &str, role: &str, secret: &str, expire_hours: i64) -> Result<String> {
    let now = Utc::now();
    let claims = Claims {
        sub: username.to_string(),
        role: role.to_string(),
        exp: (now + Duration::hours(expire_hours)).timestamp(),
        iat: now.timestamp(),
    };
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )?;
    Ok(token)
}

/// Validate a JWT token and return the claims if valid.
pub fn validate_token(token: &str, secret: &str) -> Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(data.claims)
}

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create app request
#[derive(Debug, Deserialize)]
pub struct CreateAppRequest {
    pub code: String,
    pub name: String,
}

/// App response
#[derive(Debug, Serialize)]
pub struct AppResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
}

/// App authentication request (app_id + secret)
/// Requirements: 3.1
#[derive(Debug, Deserialize)]
pub struct AppAuthRequest {
    pub app_id: Uuid,
    pub secret: String,
}

/// App authentication response with access token
/// Requirements: 3.2
#[derive(Debug, Serialize)]
pub struct AppAuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// Response when creating an app (includes secret for one-time return)
/// Requirements: 1.2
#[derive(Debug, Serialize)]
pub struct CreateAppWithSecretResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    /// Plain-text secret, returned only once during creation
    pub secret: String,
}

/// Response when regenerating app secret
/// Requirements: 2.3
#[derive(Debug, Serialize)]
pub struct RegenerateSecretResponse {
    /// Plain-text secret, returned only once
    pub secret: String,
}

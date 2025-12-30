use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Not system admin")]
    NotSystemAdmin,
    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("User not found")]
    UserNotFound,

    #[error("User is inactive")]
    UserInactive,

    #[error("User is banned from this app")]
    UserBanned { reason: Option<String> },

    #[error("Email already exists")]
    EmailAlreadyExists,

    #[error("Invalid email format")]
    InvalidEmailFormat,

    #[error("Password does not meet requirements")]
    WeakPassword,

    #[error("Invalid token")]
    InvalidToken,

    #[error("Token expired")]
    TokenExpired,

    #[error("Insufficient scope")]
    InsufficientScope,

    #[error("Account is locked")]
    AccountLocked {
        locked_until: chrono::DateTime<chrono::Utc>,
        remaining_seconds: i64,
    },

    #[error("Rate limit exceeded")]
    RateLimitExceeded {
        retry_after_seconds: i64,
        limit: i32,
        remaining: i32,
    },

    #[error("MFA required")]
    MfaRequired {
        mfa_token: String,
        available_methods: Vec<String>,
    },

    #[error("Invalid MFA code")]
    InvalidMfaCode,

    #[error("MFA not enabled")]
    MfaNotEnabled,

    #[error("Session not found")]
    SessionNotFound,

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("App code already exists")]
    CodeAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Not app owner")]
    NotAppOwner,

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Authentication error")]
    Auth(#[from] AuthError),

    #[error("Database error")]
    Database(#[from] sqlx::Error),

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum RoleError {
    #[error("Role not found")]
    NotFound,

    #[error("Role name already exists in this app")]
    NameAlreadyExists,

    #[error("App not found")]
    AppNotFound,

    #[error("User not found")]
    UserNotFound,

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum PermissionError {
    #[error("Permission not found")]
    NotFound,

    #[error("Permission code already exists in this app")]
    CodeAlreadyExists,

    #[error("App not found")]
    AppNotFound,

    #[error("Cross-app permission assignment not allowed")]
    CrossAppAssignment,

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub status_code: u16,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            AuthError::NotSystemAdmin => (StatusCode::FORBIDDEN, "not_system_admin"),
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "user_not_found"),
            AuthError::UserInactive => (StatusCode::FORBIDDEN, "user_inactive"),
            AuthError::UserBanned { .. } => (StatusCode::FORBIDDEN, "user_banned"),
            AuthError::EmailAlreadyExists => (StatusCode::CONFLICT, "email_exists"),
            AuthError::InvalidEmailFormat => (StatusCode::BAD_REQUEST, "invalid_email"),
            AuthError::WeakPassword => (StatusCode::BAD_REQUEST, "weak_password"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "token_expired"),
            AuthError::InsufficientScope => (StatusCode::FORBIDDEN, "insufficient_scope"),
            AuthError::AccountLocked { .. } => (StatusCode::FORBIDDEN, "account_locked"),
            AuthError::RateLimitExceeded { .. } => (StatusCode::TOO_MANY_REQUESTS, "rate_limit_exceeded"),
            AuthError::MfaRequired { .. } => (StatusCode::FORBIDDEN, "mfa_required"),
            AuthError::InvalidMfaCode => (StatusCode::UNAUTHORIZED, "invalid_mfa_code"),
            AuthError::MfaNotEnabled => (StatusCode::BAD_REQUEST, "mfa_not_enabled"),
            AuthError::SessionNotFound => (StatusCode::NOT_FOUND, "session_not_found"),
            AuthError::InternalError(ref e) => {
                tracing::error!("Internal error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "internal_error")
            }
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "not_found"),
            AppError::CodeAlreadyExists => (StatusCode::CONFLICT, "app_code_exists"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AppError::NotAppOwner => (StatusCode::FORBIDDEN, "not_app_owner"),
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "validation_error"),
            AppError::Auth(_) => (StatusCode::FORBIDDEN, "auth_error"),
            AppError::Database(ref e) => {
                tracing::error!("Database error: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "database_error")
            }
            AppError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

impl IntoResponse for RoleError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            RoleError::NotFound => (StatusCode::NOT_FOUND, "role_not_found"),
            RoleError::NameAlreadyExists => (StatusCode::CONFLICT, "role_name_exists"),
            RoleError::AppNotFound => (StatusCode::NOT_FOUND, "app_not_found"),
            RoleError::UserNotFound => (StatusCode::NOT_FOUND, "user_not_found"),
            RoleError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

impl IntoResponse for PermissionError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            PermissionError::NotFound => (StatusCode::NOT_FOUND, "permission_not_found"),
            PermissionError::CodeAlreadyExists => (StatusCode::CONFLICT, "permission_code_exists"),
            PermissionError::AppNotFound => (StatusCode::NOT_FOUND, "app_not_found"),
            PermissionError::CrossAppAssignment => (StatusCode::BAD_REQUEST, "cross_app_assignment"),
            PermissionError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum UserManagementError {
    #[error("Not app owner")]
    NotAppOwner,

    #[error("Not system admin")]
    NotSystemAdmin,

    #[error("User is banned")]
    UserBanned { reason: Option<String> },

    #[error("User already registered")]
    UserAlreadyRegistered,

    #[error("User not registered")]
    UserNotRegistered,

    #[error("User not found")]
    UserNotFound,

    #[error("App not found")]
    AppNotFound,

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for UserManagementError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            UserManagementError::NotAppOwner => (StatusCode::FORBIDDEN, "not_app_owner"),
            UserManagementError::NotSystemAdmin => (StatusCode::FORBIDDEN, "not_system_admin"),
            UserManagementError::UserBanned { .. } => (StatusCode::FORBIDDEN, "user_banned"),
            UserManagementError::UserAlreadyRegistered => (StatusCode::CONFLICT, "user_already_registered"),
            UserManagementError::UserNotRegistered => (StatusCode::NOT_FOUND, "user_not_registered"),
            UserManagementError::UserNotFound => (StatusCode::NOT_FOUND, "user_not_found"),
            UserManagementError::AppNotFound => (StatusCode::NOT_FOUND, "app_not_found"),
            UserManagementError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

/// Error types for App Authentication operations
/// Used for machine-to-machine authentication via App ID and Secret
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppAuthError {
    /// Generic error for invalid app_id or secret (Requirements 3.3, 3.4, 9.3)
    /// Does not reveal whether app_id or secret was incorrect
    #[error("Invalid credentials")]
    InvalidCredentials,

    /// User is not the owner of the app (Requirement 2.4)
    #[error("Not app owner")]
    NotAppOwner,

    /// Attempt to access resources of another app (Requirements 4.5, 5.5, 6.3)
    #[error("Cross-app access denied")]
    CrossAppAccess,

    /// User account has been deactivated (Requirement 8.6)
    #[error("User inactive")]
    UserInactive,

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

impl IntoResponse for AppAuthError {
    fn into_response(self) -> Response {
        let (status, error_type) = match &self {
            AppAuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AppAuthError::NotAppOwner => (StatusCode::FORBIDDEN, "not_app_owner"),
            AppAuthError::CrossAppAccess => (StatusCode::FORBIDDEN, "cross_app_access"),
            AppAuthError::UserInactive => (StatusCode::FORBIDDEN, "user_inactive"),
            AppAuthError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };

        let body = Json(ErrorResponse {
            error: error_type.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

/// Error types for OAuth2 operations
/// RFC 6749 compliant error codes
#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum OAuthError {
    /// Missing or invalid parameter (RFC 6749 Section 4.1.2.1)
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Client authentication failed (RFC 6749 Section 5.2)
    #[error("Invalid client")]
    InvalidClient,

    /// Invalid authorization code or refresh token (RFC 6749 Section 5.2)
    #[error("Invalid grant: {0}")]
    InvalidGrant(String),

    /// Client not authorized for this grant type (RFC 6749 Section 5.2)
    #[error("Unauthorized client")]
    UnauthorizedClient,

    /// Grant type not supported (RFC 6749 Section 5.2)
    #[error("Unsupported grant type")]
    UnsupportedGrantType,

    /// Invalid or unknown scope (RFC 6749 Section 5.2)
    #[error("Invalid scope: {0}")]
    InvalidScope(String),

    /// User denied consent (RFC 6749 Section 4.1.2.1)
    #[error("Access denied")]
    AccessDenied,

    /// Internal server error
    #[error("Server error: {0}")]
    ServerError(String),
}

impl IntoResponse for OAuthError {
    fn into_response(self) -> Response {
        let (status, error_code) = match &self {
            OAuthError::InvalidRequest(_) => (StatusCode::BAD_REQUEST, "invalid_request"),
            OAuthError::InvalidClient => (StatusCode::UNAUTHORIZED, "invalid_client"),
            OAuthError::InvalidGrant(_) => (StatusCode::BAD_REQUEST, "invalid_grant"),
            OAuthError::UnauthorizedClient => (StatusCode::UNAUTHORIZED, "unauthorized_client"),
            OAuthError::UnsupportedGrantType => (StatusCode::BAD_REQUEST, "unsupported_grant_type"),
            OAuthError::InvalidScope(_) => (StatusCode::BAD_REQUEST, "invalid_scope"),
            OAuthError::AccessDenied => (StatusCode::FORBIDDEN, "access_denied"),
            OAuthError::ServerError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "server_error"),
        };

        let body = Json(ErrorResponse {
            error: error_code.to_string(),
            message: self.to_string(),
            status_code: status.as_u16(),
        });

        (status, body).into_response()
    }
}

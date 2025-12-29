use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
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

    #[error("Internal server error")]
    InternalError(#[from] anyhow::Error),
}

#[allow(dead_code)]
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("App not found")]
    NotFound,

    #[error("App code already exists")]
    CodeAlreadyExists,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Not app owner")]
    NotAppOwner,

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
            AuthError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AuthError::UserNotFound => (StatusCode::NOT_FOUND, "user_not_found"),
            AuthError::UserInactive => (StatusCode::FORBIDDEN, "user_inactive"),
            AuthError::UserBanned { .. } => (StatusCode::FORBIDDEN, "user_banned"),
            AuthError::EmailAlreadyExists => (StatusCode::CONFLICT, "email_exists"),
            AuthError::InvalidEmailFormat => (StatusCode::BAD_REQUEST, "invalid_email"),
            AuthError::WeakPassword => (StatusCode::BAD_REQUEST, "weak_password"),
            AuthError::InvalidToken => (StatusCode::UNAUTHORIZED, "invalid_token"),
            AuthError::TokenExpired => (StatusCode::UNAUTHORIZED, "token_expired"),
            AuthError::InternalError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
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
            AppError::NotFound => (StatusCode::NOT_FOUND, "app_not_found"),
            AppError::CodeAlreadyExists => (StatusCode::CONFLICT, "app_code_exists"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "invalid_credentials"),
            AppError::NotAppOwner => (StatusCode::FORBIDDEN, "not_app_owner"),
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

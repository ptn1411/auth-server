use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Serialize;

use crate::config::AppState;
use crate::dto::{
    CompleteMfaLoginRequest, ForgotPasswordRequest, LoginRequest, MessageResponse, RefreshRequest,
    RegisterRequest, RegisterResponse, ResetPasswordRequest, TokenResponse,
};
use crate::error::AuthError;
use crate::services::{AuthService, LoginContext, LoginResult};
use crate::utils::jwt::JwtManager;

/// Login response - can be either tokens or MFA required
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum LoginResponse {
    /// Direct login success - tokens returned
    Success(TokenResponse),
    /// MFA verification required
    MfaRequired(MfaRequiredResponse),
}

/// Response when MFA is required
#[derive(Debug, Serialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub mfa_token: String,
    pub available_methods: Vec<String>,
}

/// Extract client IP address from headers
/// Checks X-Forwarded-For, X-Real-IP, then falls back to direct connection
fn extract_ip_address(headers: &HeaderMap) -> Option<String> {
    // Check X-Forwarded-For first (for proxied requests)
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            // X-Forwarded-For can contain multiple IPs, take the first one
            return Some(value.split(',').next()?.trim().to_string());
        }
    }

    // Check X-Real-IP
    if let Some(real_ip) = headers.get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return Some(value.to_string());
        }
    }

    None
}

/// Extract User-Agent from headers
fn extract_user_agent(headers: &HeaderMap) -> Option<String> {
    headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

/// POST /auth/register - Register a new user
/// 
/// # Requirements
/// - 14.1: Expose POST /auth/register for user registration
/// - 1.1-1.5: User registration requirements
pub async fn register_handler(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<RegisterResponse>), AuthError> {
    let jwt_manager = create_jwt_manager(&state)?;
    let auth_service = AuthService::new(state.pool.clone(), jwt_manager);
    
    let user = auth_service.register(&req.email, &req.password).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(RegisterResponse {
            id: user.id,
            email: user.email,
        }),
    ))
}

/// POST /auth/login - Authenticate user and return tokens
/// 
/// # Requirements
/// - 14.2: Expose POST /auth/login for user authentication
/// - 2.1-2.5: User login requirements
/// - 3.4: Check if user is banned from app before allowing login
/// 
/// # Security Features
/// - Rate limiting: 5 attempts per minute per IP+email
/// - Account lockout: 5 failed attempts locks account for 15 minutes
/// - Audit logging: All login attempts are logged
/// - MFA support: Returns mfa_required if user has MFA enabled
pub async fn login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, AuthError> {
    let jwt_manager = create_jwt_manager(&state)?;
    let auth_service = AuthService::new(state.pool.clone(), jwt_manager);

    // Extract request context for rate limiting and audit logging
    let context = LoginContext {
        ip_address: extract_ip_address(&headers),
        user_agent: extract_user_agent(&headers),
    };

    let result = auth_service
        .login(&req.email, &req.password, req.app_id, context)
        .await?;

    match result {
        LoginResult::Success { tokens, .. } => Ok(Json(LoginResponse::Success(TokenResponse {
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            token_type: tokens.token_type,
            expires_in: tokens.expires_in,
        }))),
        LoginResult::MfaRequired {
            mfa_token,
            available_methods,
            ..
        } => Ok(Json(LoginResponse::MfaRequired(MfaRequiredResponse {
            mfa_required: true,
            mfa_token,
            available_methods,
        }))),
    }
}

/// POST /auth/mfa/verify - Complete MFA login
/// 
/// # Description
/// Verify MFA code and complete the login process.
/// Called after login returns mfa_required.
/// 
/// # Security Features
/// - Rate limiting: 5 attempts per 5 minutes
/// - Supports TOTP and backup codes
pub async fn complete_mfa_login_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CompleteMfaLoginRequest>,
) -> Result<Json<TokenResponse>, AuthError> {
    let jwt_manager = create_jwt_manager(&state)?;
    let auth_service = AuthService::new(state.pool.clone(), jwt_manager);

    let context = LoginContext {
        ip_address: extract_ip_address(&headers),
        user_agent: extract_user_agent(&headers),
    };

    let token_pair = auth_service
        .complete_mfa_login(&req.mfa_token, &req.code, req.is_backup_code, context)
        .await?;

    Ok(Json(TokenResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
    }))
}


/// POST /auth/refresh - Refresh access token using refresh token
/// 
/// # Requirements
/// - 14.3: Expose POST /auth/refresh for token refresh
/// - 3.1-3.3: Token refresh requirements
pub async fn refresh_handler(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<TokenResponse>, AuthError> {
    let jwt_manager = create_jwt_manager(&state)?;
    let auth_service = AuthService::new(state.pool.clone(), jwt_manager);
    
    let token_pair = auth_service.refresh(&req.refresh_token).await?;
    
    Ok(Json(TokenResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in,
    }))
}

/// POST /auth/forgot-password - Initiate password reset
/// 
/// # Requirements
/// - 14.4: Expose POST /auth/forgot-password for initiating password reset
/// - 4.1-4.2: Password reset initiation requirements
pub async fn forgot_password_handler(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    let jwt_manager = create_jwt_manager(&state)?;
    let auth_service = AuthService::new(state.pool.clone(), jwt_manager);
    
    // Always return success to prevent email enumeration (Requirement 4.2)
    let _ = auth_service.forgot_password(&req.email).await?;
    
    Ok(Json(MessageResponse {
        message: "If the email exists, a password reset link has been sent.".to_string(),
    }))
}

/// POST /auth/reset-password - Complete password reset with token
/// 
/// # Requirements
/// - 14.5: Expose POST /auth/reset-password for completing password reset
/// - 4.3-4.4: Password reset completion requirements
pub async fn reset_password_handler(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    let jwt_manager = create_jwt_manager(&state)?;
    let auth_service = AuthService::new(state.pool.clone(), jwt_manager);
    
    auth_service.reset_password(&req.token, &req.new_password).await?;
    
    Ok(Json(MessageResponse {
        message: "Password has been reset successfully.".to_string(),
    }))
}

/// Helper function to create JwtManager from AppState
fn create_jwt_manager(state: &AppState) -> Result<JwtManager, AuthError> {
    JwtManager::new(
        &state.config.jwt_private_key,
        &state.config.jwt_public_key,
        state.config.access_token_expiry_secs,
        state.config.refresh_token_expiry_secs,
    )
}

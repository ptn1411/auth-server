use axum::{
    extract::State,
    http::StatusCode,
    Json,
};

use crate::config::AppState;
use crate::dto::{
    ForgotPasswordRequest, LoginRequest, MessageResponse, RefreshRequest,
    RegisterRequest, RegisterResponse, ResetPasswordRequest, TokenResponse,
};
use crate::error::AuthError;
use crate::services::AuthService;
use crate::utils::jwt::JwtManager;

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
pub async fn login_handler(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<TokenResponse>, AuthError> {
    let jwt_manager = create_jwt_manager(&state)?;
    let auth_service = AuthService::new(state.pool.clone(), jwt_manager);
    
    let token_pair = auth_service.login(&req.email, &req.password, req.app_id).await?;
    
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

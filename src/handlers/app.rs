use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{
    AppAuthRequest, AppAuthResponse, CreateAppRequest, CreateAppWithSecretResponse,
    RegenerateSecretResponse,
};
use crate::error::{AppError, AuthError};
use crate::repositories::UserRepository;
use crate::services::AppService;
use crate::utils::jwt::Claims;

/// POST /apps - Create a new app with generated secret
/// 
/// # Requirements
/// - 1.1: Generate a cryptographically secure random App_Secret
/// - 1.2: Return the plain-text secret only once during creation
/// - 5.1-5.2: App management requirements
pub async fn create_app_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateAppRequest>,
) -> Result<(StatusCode, Json<CreateAppWithSecretResponse>), AppError> {
    let owner_id = claims.user_id()
        .map_err(|_| AppError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let app_service = AppService::new(state.pool.clone(), state.jwt_manager.clone());
    
    // Create app with secret (Requirements: 1.1, 1.2)
    let (app, secret) = app_service.create_app_with_secret(&req.code, &req.name, owner_id).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(CreateAppWithSecretResponse {
            id: app.id,
            code: app.code,
            name: app.name,
            secret, // Plain-text secret, returned only once (Requirement 1.2)
        }),
    ))
}

/// POST /apps/auth - Authenticate app using App ID and Secret
/// 
/// # Requirements
/// - 3.1: Authenticate the request when valid App_ID and App_Secret are provided
/// - 3.2: Return an access token with app context
/// - 7.1: Expose POST /apps/auth endpoint for App credential authentication
pub async fn app_auth_handler(
    State(state): State<AppState>,
    Json(req): Json<AppAuthRequest>,
) -> Result<Json<AppAuthResponse>, AppError> {
    let app_service = AppService::new(state.pool.clone(), state.jwt_manager.clone());
    
    // Authenticate app and get access token (Requirements: 3.1, 3.2, 3.3, 3.4, 9.3)
    let access_token = app_service.authenticate_app(req.app_id, &req.secret).await?;
    
    Ok(Json(AppAuthResponse {
        access_token,
        token_type: "Bearer".to_string(),
        expires_in: state.jwt_manager.access_token_expiry_secs(),
    }))
}

/// POST /apps/{id}/secret/regenerate - Regenerate app secret (owner only)
/// 
/// # Requirements
/// - 2.1: Generate a new App_Secret when owner requests regeneration
/// - 2.3: Return the new plain-text secret only once
/// - 7.2: Expose POST /apps/{id}/secret/regenerate endpoint for secret regeneration
pub async fn regenerate_secret_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<RegenerateSecretResponse>, AppError> {
    let requester_id = claims.user_id()
        .map_err(|_| AppError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let app_service = AppService::new(state.pool.clone(), state.jwt_manager.clone());
    
    // Regenerate secret (Requirements: 2.1, 2.2, 2.4)
    let new_secret = app_service.regenerate_secret(app_id, requester_id).await?;
    
    Ok(Json(RegenerateSecretResponse {
        secret: new_secret, // Plain-text secret, returned only once (Requirement 2.3)
    }))
}

/// GET /users/me - Get current user profile from token
/// 
/// # Requirements
/// - 8.1: Expose GET /users/me endpoint for retrieving current user information
/// - 8.2: Return the user's profile information when valid access token is provided
/// - 8.3: Include id, email, is_active, email_verified, and created_at
/// - 8.4: NOT return sensitive information like password_hash
/// - 8.5: Reject with 401 Unauthorized if access token is invalid or expired
/// - 8.6: Reject with 403 Forbidden if user account has been deactivated
pub async fn get_user_profile_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserProfileResponse>, AuthError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    
    // Get user from database
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    // Check if user is active (Requirement 8.6)
    if !user.is_active {
        return Err(AuthError::UserInactive);
    }
    
    // Return profile without sensitive data (Requirements: 8.3, 8.4)
    Ok(Json(UserProfileResponse {
        id: user.id,
        email: user.email,
        is_active: user.is_active,
        email_verified: user.email_verified,
        created_at: user.created_at,
    }))
}

/// User profile response DTO
/// Requirements: 8.3, 8.4
#[derive(Debug, serde::Serialize)]
pub struct UserProfileResponse {
    pub id: Uuid,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

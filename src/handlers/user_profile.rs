use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};

use crate::config::AppState;
use crate::dto::auth::{
    ChangePasswordRequest, MessageResponse, ResendVerificationRequest, UpdateProfileRequest,
    UserProfileResponse, VerifyEmailRequest,
};
use crate::dto::user_management::{
    BulkImportResponse, BulkOperationResponse, BulkRoleAssignmentRequest, PaginatedResponse,
    UserExportData, UserImportRequest, UserSearchQuery, UserSearchResult,
};
use crate::error::AuthError;
use crate::repositories::UserRepository;
use crate::services::UserProfileService;
use crate::utils::jwt::Claims;

/// GET /users/me - Get current user's profile
pub async fn get_profile_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<UserProfileResponse>, AuthError> {
    let user_id = claims
        .user_id()
        .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;

    let service = UserProfileService::new(state.pool.clone());
    let profile = service.get_profile(user_id).await?;

    Ok(Json(profile))
}

/// PUT /users/me - Update current user's profile
pub async fn update_profile_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<UserProfileResponse>, AuthError> {
    let user_id = claims
        .user_id()
        .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;

    let service = UserProfileService::new(state.pool.clone());
    let profile = service.update_profile(user_id, req).await?;

    Ok(Json(profile))
}

/// POST /users/me/change-password - Change password when logged in
pub async fn change_password_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    let user_id = claims
        .user_id()
        .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;

    let service = UserProfileService::new(state.pool.clone());
    service.change_password(user_id, req).await?;

    Ok(Json(MessageResponse {
        message: "Password changed successfully".to_string(),
    }))
}

/// POST /auth/verify-email - Verify email with token
pub async fn verify_email_handler(
    State(state): State<AppState>,
    Json(req): Json<VerifyEmailRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    let service = UserProfileService::new(state.pool.clone());
    service.verify_email(&req.token).await?;

    Ok(Json(MessageResponse {
        message: "Email verified successfully".to_string(),
    }))
}

/// POST /auth/resend-verification - Resend verification email
pub async fn resend_verification_handler(
    State(state): State<AppState>,
    Json(req): Json<ResendVerificationRequest>,
) -> Result<Json<MessageResponse>, AuthError> {
    let service = UserProfileService::new(state.pool.clone());
    // Always return success to prevent email enumeration
    let _ = service.resend_verification(&req.email).await?;

    Ok(Json(MessageResponse {
        message: "If the email exists and is not verified, a verification link has been sent"
            .to_string(),
    }))
}

/// GET /admin/users/search - Search users with filters (admin only)
pub async fn search_users_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<PaginatedResponse<UserSearchResult>>, AuthError> {
    let user_id = claims
        .user_id()
        .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;

    // Check if user is system admin
    let user_repo = UserRepository::new(state.pool.clone());
    let is_admin = user_repo.is_system_admin(user_id).await?;
    if !is_admin {
        return Err(AuthError::InsufficientScope);
    }

    let service = UserProfileService::new(state.pool.clone());
    let results = service.search_users(query).await?;

    Ok(Json(results))
}

/// GET /admin/users/export - Export all users (admin only)
pub async fn export_users_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<UserExportData>>, AuthError> {
    let user_id = claims
        .user_id()
        .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;

    // Check if user is system admin
    let user_repo = UserRepository::new(state.pool.clone());
    let is_admin = user_repo.is_system_admin(user_id).await?;
    if !is_admin {
        return Err(AuthError::InsufficientScope);
    }

    let service = UserProfileService::new(state.pool.clone());
    let users = service.export_users().await?;

    Ok(Json(users))
}

/// POST /admin/users/import - Import users (admin only)
pub async fn import_users_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(users): Json<Vec<UserImportRequest>>,
) -> Result<(StatusCode, Json<BulkImportResponse>), AuthError> {
    let user_id = claims
        .user_id()
        .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;

    // Check if user is system admin
    let user_repo = UserRepository::new(state.pool.clone());
    let is_admin = user_repo.is_system_admin(user_id).await?;
    if !is_admin {
        return Err(AuthError::InsufficientScope);
    }

    let service = UserProfileService::new(state.pool.clone());
    let result = service.import_users(users).await?;

    Ok((StatusCode::OK, Json(result)))
}

/// POST /admin/users/bulk-assign-role - Bulk assign role to users (admin only)
pub async fn bulk_assign_role_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<BulkRoleAssignmentRequest>,
) -> Result<Json<BulkOperationResponse>, AuthError> {
    let user_id = claims
        .user_id()
        .map_err(|_| AuthError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;

    // Check if user is system admin
    let user_repo = UserRepository::new(state.pool.clone());
    let is_admin = user_repo.is_system_admin(user_id).await?;
    if !is_admin {
        return Err(AuthError::InsufficientScope);
    }

    let service = UserProfileService::new(state.pool.clone());
    let result = service.bulk_assign_role(req).await?;

    Ok(Json(result))
}

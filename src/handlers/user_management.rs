use axum::{
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::user_management::{AppUserInfo, BanUserRequest, PaginatedResponse, PaginationQuery};
use crate::error::UserManagementError;
use crate::models::UserApp;
use crate::services::{UserManagementService, IpRuleService, IpAccessResult};
use crate::utils::jwt::Claims;

/// Extract client IP from headers
fn extract_client_ip(headers: &HeaderMap) -> Option<String> {
    // Check X-Forwarded-For first
    if let Some(forwarded) = headers.get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
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

/// POST /apps/{app_id}/register - Register current user to an app
/// 
/// # Requirements
/// - 8.5: Expose POST /apps/{app_id}/register for user app registration
/// - 2.1-2.4: User app registration requirements
/// - Also checks IP rules for the app
pub async fn register_to_app_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    headers: HeaderMap,
    Path(app_id): Path<Uuid>,
) -> Result<(StatusCode, Json<UserApp>), UserManagementError> {
    let user_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    // Check IP rules for this app
    if let Some(ip) = extract_client_ip(&headers) {
        let ip_service = IpRuleService::new(state.pool.clone());
        let ip_result = ip_service.check_ip_access(&ip, Some(app_id)).await
            .map_err(|e| UserManagementError::InternalError(anyhow::anyhow!("{}", e)))?;
        
        if ip_result == IpAccessResult::Blocked {
            return Err(UserManagementError::UserBanned {
                reason: Some(format!("IP address {} is blocked for this app", ip)),
            });
        }
    }
    
    let service = UserManagementService::new(state.pool.clone());
    let user_app = service.register_to_app(user_id, app_id).await?;
    
    Ok((StatusCode::CREATED, Json(user_app)))
}

/// POST /apps/{app_id}/users/{user_id}/ban - Ban a user from an app
/// 
/// # Requirements
/// - 8.1: Expose POST /apps/{app_id}/users/{user_id}/ban for banning a user
/// - 3.1-3.5: Ban user requirements
pub async fn ban_user_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((app_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<BanUserRequest>,
) -> Result<Json<UserApp>, UserManagementError> {
    let actor_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let service = UserManagementService::new(state.pool.clone());
    let user_app = service.ban_user(actor_id, user_id, app_id, req.reason).await?;
    
    Ok(Json(user_app))
}


/// POST /apps/{app_id}/users/{user_id}/unban - Unban a user from an app
/// 
/// # Requirements
/// - 8.2: Expose POST /apps/{app_id}/users/{user_id}/unban for unbanning a user
/// - 4.1-4.3: Unban user requirements
pub async fn unban_user_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((app_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<UserApp>, UserManagementError> {
    let actor_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let service = UserManagementService::new(state.pool.clone());
    let user_app = service.unban_user(actor_id, user_id, app_id).await?;
    
    Ok(Json(user_app))
}

/// DELETE /apps/{app_id}/users/{user_id} - Remove a user from an app
/// 
/// # Requirements
/// - 8.3: Expose DELETE /apps/{app_id}/users/{user_id} for removing a user
/// - 5.1-5.4: Remove user requirements
pub async fn remove_user_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path((app_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, UserManagementError> {
    let actor_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let service = UserManagementService::new(state.pool.clone());
    service.remove_user(actor_id, user_id, app_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// GET /apps/{app_id}/users - List all users in an app
/// 
/// # Requirements
/// - 8.4: Expose GET /apps/{app_id}/users for listing app users
/// - 6.1-6.4: List app users requirements
pub async fn list_app_users_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<AppUserInfo>>, UserManagementError> {
    let actor_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let service = UserManagementService::new(state.pool.clone());
    let response = service.list_app_users(actor_id, app_id, pagination.page, pagination.limit).await?;
    
    Ok(Json(response))
}

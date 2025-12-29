use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::user_management::{PaginatedResponse, PaginationQuery};
use crate::error::UserManagementError;
use crate::models::{App, User};
use crate::services::AdminService;
use crate::utils::jwt::Claims;

/// Response DTO for user info (excludes password_hash)
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub is_system_admin: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            is_active: user.is_active,
            email_verified: user.email_verified,
            is_system_admin: user.is_system_admin,
            created_at: user.created_at,
        }
    }
}

/// Response DTO for app info
#[derive(Debug, Serialize)]
pub struct AppResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub owner_id: Option<Uuid>,
}

impl From<App> for AppResponse {
    fn from(app: App) -> Self {
        Self {
            id: app.id,
            code: app.code,
            name: app.name,
            owner_id: app.owner_id,
        }
    }
}


/// GET /admin/users - List all users (admin only)
/// 
/// # Requirements
/// - 8.6: Expose GET /admin/users for System_Admin to list all users
/// - 7.4: System admin can list all users
pub async fn list_all_users_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<UserResponse>>, UserManagementError> {
    let actor_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let service = AdminService::new(state.pool.clone());
    let response = service.list_all_users(actor_id, pagination.page, pagination.limit).await?;
    
    // Convert User to UserResponse (excludes password_hash)
    let user_responses: Vec<UserResponse> = response.data.into_iter().map(UserResponse::from).collect();
    
    Ok(Json(PaginatedResponse::new(
        user_responses,
        response.page,
        response.limit,
        response.total,
    )))
}

/// GET /admin/apps - List all apps (admin only)
/// 
/// # Requirements
/// - 8.7: Expose GET /admin/apps for System_Admin to list all apps
/// - 7.4: System admin can list all apps
pub async fn list_all_apps_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<AppResponse>>, UserManagementError> {
    let actor_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let service = AdminService::new(state.pool.clone());
    let response = service.list_all_apps(actor_id, pagination.page, pagination.limit).await?;
    
    // Convert App to AppResponse
    let app_responses: Vec<AppResponse> = response.data.into_iter().map(AppResponse::from).collect();
    
    Ok(Json(PaginatedResponse::new(
        app_responses,
        response.page,
        response.limit,
        response.total,
    )))
}

/// POST /admin/users/{user_id}/deactivate - Deactivate a user globally (admin only)
/// 
/// # Requirements
/// - 8.8: Expose POST /admin/users/{user_id}/deactivate for System_Admin to deactivate a user
/// - 7.5: System admin can deactivate any user globally
pub async fn deactivate_user_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, UserManagementError> {
    let actor_id = claims.user_id()
        .map_err(|_| UserManagementError::InternalError(anyhow::anyhow!("Invalid user ID in token")))?;
    
    let service = AdminService::new(state.pool.clone());
    service.deactivate_user(actor_id, user_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

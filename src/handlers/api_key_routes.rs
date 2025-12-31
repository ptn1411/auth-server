//! API Key authenticated routes
//! 
//! These endpoints can be accessed using API Key authentication via X-API-Key header.
//! Each endpoint requires specific scopes.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use serde::{Deserialize, Serialize};

use crate::config::AppState;
use crate::dto::{PaginationQuery, UserAppResponse};
use crate::error::AppError;
use crate::middleware::ApiKeyContext;
use crate::services::{UserManagementService, RoleService, api_key_scopes};

/// GET /api/v1/users - List users in app (requires read:users scope)
pub async fn list_users_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
    Query(pagination): Query<PaginationQuery>,
) -> Result<Json<ListUsersResponse>, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::READ_USERS) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = UserManagementService::new(state.pool.clone());
    let page = pagination.page;
    let limit = pagination.limit.min(100);

    let (users, total) = service.list_app_users_by_api_key(api_key.app_id, page, limit).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(ListUsersResponse {
        data: users,
        page,
        limit,
        total,
    }))
}

/// GET /api/v1/users/:user_id - Get user details (requires read:users scope)
pub async fn get_user_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
    Path(user_id): Path<Uuid>,
) -> Result<Json<UserAppResponse>, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::READ_USERS) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = UserManagementService::new(state.pool.clone());
    let user = service.get_user_in_app(api_key.app_id, user_id).await
        .map_err(|e| AppError::NotFound(e.to_string()))?;

    Ok(Json(user))
}

/// POST /api/v1/users/:user_id/ban - Ban user (requires write:users scope)
pub async fn ban_user_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
    Path(user_id): Path<Uuid>,
    Json(req): Json<BanUserRequest>,
) -> Result<StatusCode, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::WRITE_USERS) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = UserManagementService::new(state.pool.clone());
    service.ban_user_by_api_key(api_key.app_id, user_id, req.reason).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// POST /api/v1/users/:user_id/unban - Unban user (requires write:users scope)
pub async fn unban_user_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::WRITE_USERS) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = UserManagementService::new(state.pool.clone());
    service.unban_user_by_api_key(api_key.app_id, user_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

/// GET /api/v1/roles - List roles in app (requires read:roles scope)
pub async fn list_roles_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::READ_ROLES) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = RoleService::new(state.pool.clone());
    let roles = service.get_roles_by_app(api_key.app_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    let response: Vec<RoleResponse> = roles.into_iter().map(|r| RoleResponse {
        id: r.id,
        name: r.name,
        app_id: r.app_id,
    }).collect();

    Ok(Json(response))
}

/// GET /api/v1/users/:user_id/roles - Get user roles (requires read:roles scope)
pub async fn get_user_roles_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<RoleResponse>>, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::READ_ROLES) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = RoleService::new(state.pool.clone());
    let roles = service.get_user_roles_in_app(user_id, api_key.app_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    let response: Vec<RoleResponse> = roles.into_iter().map(|r| RoleResponse {
        id: r.id,
        name: r.name,
        app_id: r.app_id,
    }).collect();

    Ok(Json(response))
}

/// POST /api/v1/users/:user_id/roles - Assign role to user (requires write:roles scope)
pub async fn assign_role_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
    Path(user_id): Path<Uuid>,
    Json(req): Json<AssignRoleRequest>,
) -> Result<StatusCode, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::WRITE_ROLES) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = RoleService::new(state.pool.clone());
    service.assign_role_to_user(req.role_id, user_id, api_key.app_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(StatusCode::CREATED)
}

/// DELETE /api/v1/users/:user_id/roles/:role_id - Remove role from user (requires write:roles scope)
pub async fn remove_role_api_key_handler(
    State(state): State<AppState>,
    api_key: ApiKeyContext,
    Path((user_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    // Check scope
    if !api_key.has_scope(api_key_scopes::WRITE_ROLES) {
        return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
    }

    let service = RoleService::new(state.pool.clone());
    service.remove_role_from_user(role_id, user_id, api_key.app_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(StatusCode::NO_CONTENT)
}

// ============ DTOs ============

#[derive(Debug, Serialize)]
pub struct ListUsersResponse {
    pub data: Vec<UserAppResponse>,
    pub page: u32,
    pub limit: u32,
    pub total: i64,
}

#[derive(Debug, Deserialize)]
pub struct BanUserRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub name: String,
    pub app_id: Uuid,
}

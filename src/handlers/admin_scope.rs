use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::config::AppState;
use crate::error::{AppError, AuthError};
use crate::repositories::{OAuthScopeRepository, UserRepository};
use crate::utils::jwt::Claims;

#[derive(Debug, Deserialize)]
pub struct CreateScopeRequest {
    pub code: String,
    pub description: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScopeRequest {
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct ScopeResponse {
    pub id: String,
    pub code: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct ListScopesAdminResponse {
    pub scopes: Vec<ScopeResponse>,
    pub total: u64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

/// GET /admin/scopes - List all OAuth scopes (admin only)
pub async fn list_all_scopes_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(params): Query<PaginationParams>,
) -> Result<Json<ListScopesAdminResponse>, AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(20).min(100);

    let scope_repo = OAuthScopeRepository::new(state.pool.clone());
    let scopes = scope_repo.list_all(page, limit).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;
    let total = scope_repo.count_all().await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    let scope_responses: Vec<ScopeResponse> = scopes
        .into_iter()
        .map(|s| ScopeResponse {
            id: s.id.to_string(),
            code: s.code,
            description: s.description,
            is_active: s.is_active,
            created_at: s.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ListScopesAdminResponse {
        scopes: scope_responses,
        total,
        page,
        limit,
    }))
}

/// POST /admin/scopes - Create a new OAuth scope (admin only)
pub async fn create_scope_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateScopeRequest>,
) -> Result<(StatusCode, Json<ScopeResponse>), AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    if req.code.is_empty() || req.code.len() > 100 {
        return Err(AppError::ValidationError("Scope code must be 1-100 characters".into()));
    }

    if req.description.is_empty() {
        return Err(AppError::ValidationError("Description is required".into()));
    }

    let scope_repo = OAuthScopeRepository::new(state.pool.clone());
    let scope = scope_repo.create(&req.code, &req.description).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok((
        StatusCode::CREATED,
        Json(ScopeResponse {
            id: scope.id.to_string(),
            code: scope.code,
            description: scope.description,
            is_active: scope.is_active,
            created_at: scope.created_at.to_rfc3339(),
        }),
    ))
}

/// GET /admin/scopes/:id - Get a specific OAuth scope (admin only)
pub async fn get_scope_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<ScopeResponse>, AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let scope_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::ValidationError("Invalid scope ID".into()))?;

    let scope_repo = OAuthScopeRepository::new(state.pool.clone());
    let scope = scope_repo
        .find_by_id(scope_id)
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?
        .ok_or_else(|| AppError::NotFound("Scope not found".into()))?;

    Ok(Json(ScopeResponse {
        id: scope.id.to_string(),
        code: scope.code,
        description: scope.description,
        is_active: scope.is_active,
        created_at: scope.created_at.to_rfc3339(),
    }))
}

/// PUT /admin/scopes/:id - Update an OAuth scope (admin only)
pub async fn update_scope_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
    Json(req): Json<UpdateScopeRequest>,
) -> Result<Json<ScopeResponse>, AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    if req.description.is_empty() {
        return Err(AppError::ValidationError("Description is required".into()));
    }

    let scope_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::ValidationError("Invalid scope ID".into()))?;

    let scope_repo = OAuthScopeRepository::new(state.pool.clone());
    let scope = scope_repo.update(scope_id, &req.description).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(ScopeResponse {
        id: scope.id.to_string(),
        code: scope.code,
        description: scope.description,
        is_active: scope.is_active,
        created_at: scope.created_at.to_rfc3339(),
    }))
}

/// POST /admin/scopes/:id/activate - Activate an OAuth scope (admin only)
pub async fn activate_scope_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let scope_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::ValidationError("Invalid scope ID".into()))?;

    let scope_repo = OAuthScopeRepository::new(state.pool.clone());
    scope_repo.activate(scope_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Scope activated" })))
}

/// POST /admin/scopes/:id/deactivate - Deactivate an OAuth scope (admin only)
pub async fn deactivate_scope_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let scope_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::ValidationError("Invalid scope ID".into()))?;

    let scope_repo = OAuthScopeRepository::new(state.pool.clone());
    scope_repo.deactivate(scope_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Scope deactivated" })))
}

/// DELETE /admin/scopes/:id - Delete an OAuth scope (admin only)
pub async fn delete_scope_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let scope_id = Uuid::parse_str(&id)
        .map_err(|_| AppError::ValidationError("Invalid scope ID".into()))?;

    let scope_repo = OAuthScopeRepository::new(state.pool.clone());
    scope_repo.delete(scope_id).await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;

    Ok(Json(serde_json::json!({ "message": "Scope deleted" })))
}

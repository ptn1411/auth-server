use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{CreatePermissionRequest, PermissionResponse};
use crate::error::{AppAuthError, PermissionError};
use crate::middleware::AppContext;
use crate::services::PermissionService;

/// POST /apps/{app_id}/permissions - Create a new permission for an app
/// 
/// # Requirements
/// - 14.8: Expose POST /apps/{app_id}/permissions for creating permissions
/// - 7.1-7.2: Permission management requirements
pub async fn create_permission_handler(
    State(state): State<AppState>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<CreatePermissionRequest>,
) -> Result<(StatusCode, Json<PermissionResponse>), PermissionError> {
    let permission_service = PermissionService::new(state.pool.clone());
    
    let permission = permission_service.create_permission(app_id, &req.code).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(PermissionResponse {
            id: permission.id,
            app_id: permission.app_id,
            code: permission.code,
        }),
    ))
}

/// POST /apps/{id}/permissions - Create a new permission for an app (App Auth)
/// 
/// This endpoint is protected by app authentication middleware.
/// The app_id from the token must match the path parameter.
/// 
/// # Requirements
/// - 5.1: WHEN an authenticated App creates a permission, THE Auth_Server SHALL create the permission scoped to that App
/// - 5.5: IF an App attempts to access permissions of another App, THEN THE Auth_Server SHALL reject with 403 Forbidden
pub async fn create_permission_app_auth_handler(
    State(state): State<AppState>,
    AppContext(token_app_id): AppContext,
    Path(path_app_id): Path<Uuid>,
    Json(req): Json<CreatePermissionRequest>,
) -> Result<(StatusCode, Json<PermissionResponse>), AppAuthError> {
    // Verify app_id from token matches path parameter (Requirement 5.5)
    if token_app_id != path_app_id {
        return Err(AppAuthError::CrossAppAccess);
    }
    
    let permission_service = PermissionService::new(state.pool.clone());
    
    let permission = permission_service.create_permission(path_app_id, &req.code).await
        .map_err(|e| AppAuthError::InternalError(e.into()))?;
    
    Ok((
        StatusCode::CREATED,
        Json(PermissionResponse {
            id: permission.id,
            app_id: permission.app_id,
            code: permission.code,
        }),
    ))
}

/// GET /apps/{id}/permissions - List all permissions for an app (App Auth)
/// 
/// This endpoint is protected by app authentication middleware.
/// The app_id from the token must match the path parameter.
/// 
/// # Requirements
/// - 5.2: WHEN an authenticated App lists permissions, THE Auth_Server SHALL return only permissions belonging to that App
/// - 5.5: IF an App attempts to access permissions of another App, THEN THE Auth_Server SHALL reject with 403 Forbidden
pub async fn list_permissions_app_auth_handler(
    State(state): State<AppState>,
    AppContext(token_app_id): AppContext,
    Path(path_app_id): Path<Uuid>,
) -> Result<Json<Vec<PermissionResponse>>, AppAuthError> {
    // Verify app_id from token matches path parameter (Requirement 5.5)
    if token_app_id != path_app_id {
        return Err(AppAuthError::CrossAppAccess);
    }
    
    let permission_service = PermissionService::new(state.pool.clone());
    
    let permissions = permission_service.get_permissions_by_app(path_app_id).await
        .map_err(|e| AppAuthError::InternalError(e.into()))?;
    
    let response: Vec<PermissionResponse> = permissions
        .into_iter()
        .map(|permission| PermissionResponse {
            id: permission.id,
            app_id: permission.app_id,
            code: permission.code,
        })
        .collect();
    
    Ok(Json(response))
}


/// POST /apps/{id}/roles/{role_id}/permissions - Assign a permission to a role (App Auth)
/// 
/// This endpoint is protected by app authentication middleware.
/// The app_id from the token must match the path parameter.
/// Both the role and permission must belong to the same app.
/// 
/// # Requirements
/// - 6.1: WHEN an authenticated App assigns a permission to a role, THE Auth_Server SHALL verify both belong to the same App
/// - 6.2: WHEN assignment is valid, THE Auth_Server SHALL create the role-permission association
/// - 6.3: IF the role or permission belongs to a different App, THEN THE Auth_Server SHALL reject with 403 Forbidden
pub async fn assign_permission_to_role_handler(
    State(state): State<AppState>,
    AppContext(token_app_id): AppContext,
    Path((path_app_id, role_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<crate::dto::AssignPermissionRequest>,
) -> Result<StatusCode, AppAuthError> {
    // Verify app_id from token matches path parameter (Requirement 6.3)
    if token_app_id != path_app_id {
        return Err(AppAuthError::CrossAppAccess);
    }
    
    let permission_service = PermissionService::new(state.pool.clone());
    
    // The service layer will verify that both role and permission belong to the same app
    // (Requirement 6.1, 6.3)
    permission_service.assign_permission_to_role(role_id, req.permission_id).await
        .map_err(|e| match e {
            PermissionError::CrossAppAssignment => AppAuthError::CrossAppAccess,
            PermissionError::NotFound => AppAuthError::InternalError(anyhow::anyhow!("Role or permission not found")),
            _ => AppAuthError::InternalError(e.into()),
        })?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// POST /apps/{app_id}/roles/{role_id}/permissions - Assign a permission to a role (User Auth)
/// 
/// This endpoint is protected by JWT authentication.
/// The user must be the owner of the app.
pub async fn assign_permission_to_role_user_handler(
    State(state): State<AppState>,
    Path((app_id, role_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<crate::dto::AssignPermissionRequest>,
) -> Result<StatusCode, PermissionError> {
    let permission_service = PermissionService::new(state.pool.clone());
    
    permission_service.assign_permission_to_role(role_id, req.permission_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /apps/{app_id}/roles/{role_id}/permissions/{permission_id} - Remove a permission from a role (User Auth)
pub async fn remove_permission_from_role_handler(
    State(state): State<AppState>,
    Path((app_id, role_id, permission_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<StatusCode, PermissionError> {
    let permission_service = PermissionService::new(state.pool.clone());
    
    permission_service.remove_permission_from_role(role_id, permission_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

/// GET /apps/{app_id}/roles/{role_id}/permissions - Get all permissions for a role (User Auth)
pub async fn get_role_permissions_handler(
    State(state): State<AppState>,
    Path((app_id, role_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Vec<PermissionResponse>>, PermissionError> {
    let permission_service = PermissionService::new(state.pool.clone());
    
    let permissions = permission_service.get_role_permissions(role_id).await?;
    
    let response: Vec<PermissionResponse> = permissions
        .into_iter()
        .map(|p| PermissionResponse {
            id: p.id,
            app_id: p.app_id,
            code: p.code,
        })
        .collect();
    
    Ok(Json(response))
}

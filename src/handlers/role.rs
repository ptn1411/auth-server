use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{AssignRoleRequest, CreateRoleRequest, RoleResponse};
use crate::error::{AppAuthError, RoleError};
use crate::middleware::AppContext;
use crate::services::RoleService;

/// POST /apps/{app_id}/roles - Create a new role for an app
/// 
/// # Requirements
/// - 14.7: Expose POST /apps/{app_id}/roles for creating roles
/// - 6.1-6.2: Role management requirements
pub async fn create_role_handler(
    State(state): State<AppState>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), RoleError> {
    let role_service = RoleService::new(state.pool.clone());
    
    let role = role_service.create_role(app_id, &req.name).await?;
    
    Ok((
        StatusCode::CREATED,
        Json(RoleResponse {
            id: role.id,
            app_id: role.app_id,
            name: role.name,
        }),
    ))
}

/// POST /apps/{id}/roles - Create a new role for an app (App Auth)
/// 
/// This endpoint is protected by app authentication middleware.
/// The app_id from the token must match the path parameter.
/// 
/// # Requirements
/// - 4.1: WHEN an authenticated App creates a role, THE Auth_Server SHALL create the role scoped to that App
/// - 4.5: IF an App attempts to access roles of another App, THEN THE Auth_Server SHALL reject with 403 Forbidden
pub async fn create_role_app_auth_handler(
    State(state): State<AppState>,
    AppContext(token_app_id): AppContext,
    Path(path_app_id): Path<Uuid>,
    Json(req): Json<CreateRoleRequest>,
) -> Result<(StatusCode, Json<RoleResponse>), AppAuthError> {
    // Verify app_id from token matches path parameter (Requirement 4.5)
    if token_app_id != path_app_id {
        return Err(AppAuthError::CrossAppAccess);
    }
    
    let role_service = RoleService::new(state.pool.clone());
    
    let role = role_service.create_role(path_app_id, &req.name).await
        .map_err(|e| AppAuthError::InternalError(e.into()))?;
    
    Ok((
        StatusCode::CREATED,
        Json(RoleResponse {
            id: role.id,
            app_id: role.app_id,
            name: role.name,
        }),
    ))
}

/// GET /apps/{id}/roles - List all roles for an app (App Auth)
/// 
/// This endpoint is protected by app authentication middleware.
/// The app_id from the token must match the path parameter.
/// 
/// # Requirements
/// - 4.2: WHEN an authenticated App lists roles, THE Auth_Server SHALL return only roles belonging to that App
/// - 4.5: IF an App attempts to access roles of another App, THEN THE Auth_Server SHALL reject with 403 Forbidden
pub async fn list_roles_app_auth_handler(
    State(state): State<AppState>,
    AppContext(token_app_id): AppContext,
    Path(path_app_id): Path<Uuid>,
) -> Result<Json<Vec<RoleResponse>>, AppAuthError> {
    // Verify app_id from token matches path parameter (Requirement 4.5)
    if token_app_id != path_app_id {
        return Err(AppAuthError::CrossAppAccess);
    }
    
    let role_service = RoleService::new(state.pool.clone());
    
    let roles = role_service.get_roles_by_app(path_app_id).await
        .map_err(|e| AppAuthError::InternalError(e.into()))?;
    
    let response: Vec<RoleResponse> = roles
        .into_iter()
        .map(|role| RoleResponse {
            id: role.id,
            app_id: role.app_id,
            name: role.name,
        })
        .collect();
    
    Ok(Json(response))
}

/// POST /apps/{app_id}/users/{user_id}/roles - Assign a role to a user
/// 
/// # Requirements
/// - 14.9: Expose POST /apps/{app_id}/users/{user_id}/roles for assigning roles to users
/// - 8.1-8.2: User role assignment requirements
pub async fn assign_role_handler(
    State(state): State<AppState>,
    Path((app_id, user_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AssignRoleRequest>,
) -> Result<StatusCode, RoleError> {
    let role_service = RoleService::new(state.pool.clone());
    
    role_service.assign_role_to_user(user_id, app_id, req.role_id).await?;
    
    Ok(StatusCode::NO_CONTENT)
}

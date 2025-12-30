use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{CreateApiKeyRequest, UpdateApiKeyRequest, ApiKeyResponse, ApiKeyWithSecretResponse};
use crate::error::AppError;
use crate::services::ApiKeyService;
use crate::utils::jwt::Claims;

/// POST /apps/:app_id/api-keys - Create API key
pub async fn create_api_key_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<CreateApiKeyRequest>,
) -> Result<(StatusCode, Json<ApiKeyWithSecretResponse>), AppError> {
    let service = ApiKeyService::new(state.pool.clone());
    let (api_key, key) = service.create_api_key(
        app_id,
        &req.name,
        req.scopes,
        req.expires_at,
    ).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiKeyWithSecretResponse {
            id: api_key.id,
            app_id: api_key.app_id,
            name: api_key.name,
            key,
            key_prefix: api_key.key_prefix,
            scopes: api_key.scopes.0,
            expires_at: api_key.expires_at,
            is_active: api_key.is_active,
            created_at: api_key.created_at,
        }),
    ))
}

/// GET /apps/:app_id/api-keys - List API keys
pub async fn list_api_keys_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<Vec<ApiKeyResponse>>, AppError> {
    let service = ApiKeyService::new(state.pool.clone());
    let keys = service.list_api_keys(app_id).await?;

    let response: Vec<ApiKeyResponse> = keys
        .into_iter()
        .map(|k| ApiKeyResponse {
            id: k.id,
            app_id: k.app_id,
            name: k.name,
            key_prefix: k.key_prefix,
            scopes: k.scopes.0,
            expires_at: k.expires_at,
            last_used_at: k.last_used_at,
            is_active: k.is_active,
            created_at: k.created_at,
        })
        .collect();

    Ok(Json(response))
}

/// GET /apps/:app_id/api-keys/:key_id - Get API key
pub async fn get_api_key_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path((_app_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiKeyResponse>, AppError> {
    let service = ApiKeyService::new(state.pool.clone());
    let key = service.get_api_key(key_id).await?
        .ok_or_else(|| AppError::NotFound("API key not found".into()))?;

    Ok(Json(ApiKeyResponse {
        id: key.id,
        app_id: key.app_id,
        name: key.name,
        key_prefix: key.key_prefix,
        scopes: key.scopes.0,
        expires_at: key.expires_at,
        last_used_at: key.last_used_at,
        is_active: key.is_active,
        created_at: key.created_at,
    }))
}

/// PUT /apps/:app_id/api-keys/:key_id - Update API key
pub async fn update_api_key_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path((_app_id, key_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateApiKeyRequest>,
) -> Result<Json<ApiKeyResponse>, AppError> {
    let service = ApiKeyService::new(state.pool.clone());
    let key = service.update_api_key(
        key_id,
        req.name.as_deref(),
        req.scopes,
        req.is_active,
    ).await?;

    Ok(Json(ApiKeyResponse {
        id: key.id,
        app_id: key.app_id,
        name: key.name,
        key_prefix: key.key_prefix,
        scopes: key.scopes.0,
        expires_at: key.expires_at,
        last_used_at: key.last_used_at,
        is_active: key.is_active,
        created_at: key.created_at,
    }))
}

/// DELETE /apps/:app_id/api-keys/:key_id - Delete API key
pub async fn delete_api_key_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path((_app_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let service = ApiKeyService::new(state.pool.clone());
    service.delete_api_key(key_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// POST /apps/:app_id/api-keys/:key_id/revoke - Revoke API key
pub async fn revoke_api_key_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path((_app_id, key_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let service = ApiKeyService::new(state.pool.clone());
    service.revoke_api_key(key_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

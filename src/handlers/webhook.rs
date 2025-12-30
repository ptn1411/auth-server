use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{CreateWebhookRequest, UpdateWebhookRequest, WebhookResponse, WebhookWithSecretResponse};
use crate::error::AppError;
use crate::services::WebhookService;
use crate::utils::jwt::Claims;

/// POST /apps/:app_id/webhooks - Create webhook
pub async fn create_webhook_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<WebhookWithSecretResponse>), AppError> {
    // Verify user owns the app (simplified - should check ownership)
    let _ = claims.user_id()?;

    let service = WebhookService::new(state.pool.clone());
    let (webhook, secret) = service.create_webhook(app_id, &req.url, req.events).await?;

    Ok((
        StatusCode::CREATED,
        Json(WebhookWithSecretResponse {
            id: webhook.id,
            app_id: webhook.app_id,
            url: webhook.url,
            secret,
            events: webhook.events.0,
            is_active: webhook.is_active,
            created_at: webhook.created_at,
        }),
    ))
}

/// GET /apps/:app_id/webhooks - List webhooks
pub async fn list_webhooks_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<Vec<WebhookResponse>>, AppError> {
    let service = WebhookService::new(state.pool.clone());
    let webhooks = service.list_webhooks(app_id).await?;

    let response: Vec<WebhookResponse> = webhooks
        .into_iter()
        .map(|w| WebhookResponse {
            id: w.id,
            app_id: w.app_id,
            url: w.url,
            events: w.events.0,
            is_active: w.is_active,
            created_at: w.created_at,
        })
        .collect();

    Ok(Json(response))
}

/// GET /apps/:app_id/webhooks/:webhook_id - Get webhook
pub async fn get_webhook_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path((app_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<WebhookResponse>, AppError> {
    let service = WebhookService::new(state.pool.clone());
    let webhook = service.get_webhook(webhook_id).await?
        .ok_or_else(|| AppError::NotFound("Webhook not found".into()))?;

    if webhook.app_id != app_id {
        return Err(AppError::NotFound("Webhook not found".into()));
    }

    Ok(Json(WebhookResponse {
        id: webhook.id,
        app_id: webhook.app_id,
        url: webhook.url,
        events: webhook.events.0,
        is_active: webhook.is_active,
        created_at: webhook.created_at,
    }))
}

/// PUT /apps/:app_id/webhooks/:webhook_id - Update webhook
pub async fn update_webhook_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path((_app_id, webhook_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateWebhookRequest>,
) -> Result<Json<WebhookResponse>, AppError> {
    let service = WebhookService::new(state.pool.clone());
    let webhook = service.update_webhook(
        webhook_id,
        req.url.as_deref(),
        req.events,
        req.is_active,
    ).await?;

    Ok(Json(WebhookResponse {
        id: webhook.id,
        app_id: webhook.app_id,
        url: webhook.url,
        events: webhook.events.0,
        is_active: webhook.is_active,
        created_at: webhook.created_at,
    }))
}

/// DELETE /apps/:app_id/webhooks/:webhook_id - Delete webhook
pub async fn delete_webhook_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path((_app_id, webhook_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    let service = WebhookService::new(state.pool.clone());
    service.delete_webhook(webhook_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{CreateIpRuleRequest, IpRuleResponse, IpCheckResponse};
use crate::error::{AppError, AuthError};
use crate::models::IpRuleType;
use crate::services::{IpRuleService, IpAccessResult};
use crate::utils::jwt::Claims;
use crate::repositories::UserRepository;

#[derive(Debug, Deserialize)]
pub struct IpCheckQuery {
    pub ip: String,
    pub app_id: Option<Uuid>,
}

/// POST /admin/ip-rules - Create IP rule (admin only)
pub async fn create_ip_rule_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<CreateIpRuleRequest>,
) -> Result<(StatusCode, Json<IpRuleResponse>), AppError> {
    let user_id = claims.user_id()?;
    
    // Check admin
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let rule_type = match req.rule_type.as_str() {
        "whitelist" => IpRuleType::Whitelist,
        "blacklist" => IpRuleType::Blacklist,
        _ => return Err(AppError::ValidationError("Invalid rule type".into())),
    };

    let service = IpRuleService::new(state.pool.clone());
    let rule = service.create_rule(
        None, // Global rule
        &req.ip_address,
        req.ip_range.as_deref(),
        rule_type,
        req.reason.as_deref(),
        req.expires_at,
        Some(user_id),
    ).await?;

    Ok((
        StatusCode::CREATED,
        Json(IpRuleResponse {
            id: rule.id_uuid(),
            app_id: rule.app_id,
            ip_address: rule.ip_address,
            ip_range: rule.ip_range,
            rule_type: rule.rule_type,
            reason: rule.reason,
            expires_at: rule.expires_at,
            created_by: rule.created_by,
            created_at: rule.created_at,
        }),
    ))
}

/// POST /apps/:app_id/ip-rules - Create IP rule for app
pub async fn create_app_ip_rule_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
    Json(req): Json<CreateIpRuleRequest>,
) -> Result<(StatusCode, Json<IpRuleResponse>), AppError> {
    let user_id = claims.user_id()?;

    let rule_type = match req.rule_type.as_str() {
        "whitelist" => IpRuleType::Whitelist,
        "blacklist" => IpRuleType::Blacklist,
        _ => return Err(AppError::ValidationError("Invalid rule type".into())),
    };

    let service = IpRuleService::new(state.pool.clone());
    let rule = service.create_rule(
        Some(app_id),
        &req.ip_address,
        req.ip_range.as_deref(),
        rule_type,
        req.reason.as_deref(),
        req.expires_at,
        Some(user_id),
    ).await?;

    Ok((
        StatusCode::CREATED,
        Json(IpRuleResponse {
            id: rule.id_uuid(),
            app_id: rule.app_id,
            ip_address: rule.ip_address,
            ip_range: rule.ip_range,
            rule_type: rule.rule_type,
            reason: rule.reason,
            expires_at: rule.expires_at,
            created_by: rule.created_by,
            created_at: rule.created_at,
        }),
    ))
}

/// GET /admin/ip-rules - List global IP rules
pub async fn list_ip_rules_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<IpRuleResponse>>, AppError> {
    let user_id = claims.user_id()?;
    
    // Check admin
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let service = IpRuleService::new(state.pool.clone());
    let rules = service.list_rules(None).await?;

    let response: Vec<IpRuleResponse> = rules
        .into_iter()
        .map(|r| IpRuleResponse {
            id: r.id_uuid(),
            app_id: r.app_id,
            ip_address: r.ip_address,
            ip_range: r.ip_range,
            rule_type: r.rule_type,
            reason: r.reason,
            expires_at: r.expires_at,
            created_by: r.created_by,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(response))
}

/// GET /apps/:app_id/ip-rules - List app IP rules
pub async fn list_app_ip_rules_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(app_id): Path<Uuid>,
) -> Result<Json<Vec<IpRuleResponse>>, AppError> {
    let service = IpRuleService::new(state.pool.clone());
    let rules = service.list_rules(Some(app_id)).await?;

    let response: Vec<IpRuleResponse> = rules
        .into_iter()
        .map(|r| IpRuleResponse {
            id: r.id_uuid(),
            app_id: r.app_id,
            ip_address: r.ip_address,
            ip_range: r.ip_range,
            rule_type: r.rule_type,
            reason: r.reason,
            expires_at: r.expires_at,
            created_by: r.created_by,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(response))
}

/// GET /admin/ip-rules/check - Check IP access
pub async fn check_ip_handler(
    State(state): State<AppState>,
    Query(query): Query<IpCheckQuery>,
) -> Result<Json<IpCheckResponse>, AppError> {
    let service = IpRuleService::new(state.pool.clone());
    let result = service.check_ip_access(&query.ip, query.app_id).await?;

    let (allowed, rule_type) = match result {
        IpAccessResult::Allowed => (true, Some("whitelist".to_string())),
        IpAccessResult::Blocked => (false, Some("blacklist".to_string())),
        IpAccessResult::NoRule => (true, None),
    };

    Ok(Json(IpCheckResponse {
        ip: query.ip,
        allowed,
        rule_type,
    }))
}

/// DELETE /admin/ip-rules/:rule_id - Delete IP rule
pub async fn delete_ip_rule_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(rule_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let user_id = claims.user_id()?;
    
    // Check admin
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AuthError::UserNotFound)?;
    
    if !user.is_system_admin {
        return Err(AppError::Auth(AuthError::NotSystemAdmin));
    }

    let service = IpRuleService::new(state.pool.clone());
    service.delete_rule(rule_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

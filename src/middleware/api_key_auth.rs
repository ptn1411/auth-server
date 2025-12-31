use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{request::Parts, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::error::AppError;
use crate::services::{ApiKeyService, IpRuleService, IpAccessResult};

/// Header name for API Key authentication
pub const API_KEY_HEADER: &str = "X-API-Key";

/// API Key Authentication Middleware
/// 
/// This middleware extracts and verifies API keys from the X-API-Key header.
/// On successful verification, it injects the ApiKeyContext into request extensions.
/// Also checks IP rules for the associated app.
/// 
/// # Usage
/// ```rust,ignore
/// use axum::{Router, middleware};
/// use auth_server::middleware::api_key_auth_middleware;
/// 
/// let api_routes = Router::new()
///     .route("/api/users", get(handler))
///     .layer(middleware::from_fn_with_state(state.clone(), api_key_auth_middleware));
/// ```
pub async fn api_key_auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    // 1. Extract API key from X-API-Key header
    let api_key_value = request
        .headers()
        .get(API_KEY_HEADER)
        .and_then(|value| value.to_str().ok());

    let key = match api_key_value {
        Some(k) if !k.trim().is_empty() => k.trim(),
        _ => {
            tracing::warn!("Missing or empty X-API-Key header for path: {}", request.uri().path());
            return Err(AppError::Auth(crate::error::AuthError::InvalidToken));
        }
    };

    // 2. Verify API key
    let service = ApiKeyService::new(state.pool.clone());
    let api_key = service.verify_api_key(key).await?
        .ok_or_else(|| {
            tracing::warn!("Invalid API key attempted");
            AppError::Auth(crate::error::AuthError::InvalidToken)
        })?;

    // 3. Check if key is active
    if !api_key.is_active {
        tracing::warn!("Revoked API key attempted: {}", api_key.key_prefix);
        return Err(AppError::Auth(crate::error::AuthError::InvalidToken));
    }

    // 4. Check if key is expired
    if api_key.is_expired() {
        tracing::warn!("Expired API key attempted: {}", api_key.key_prefix);
        return Err(AppError::Auth(crate::error::AuthError::TokenExpired));
    }

    // 5. Check IP rules for this app
    let client_ip = extract_client_ip(&request);
    if let Some(ref ip) = client_ip {
        let ip_service = IpRuleService::new(state.pool.clone());
        let ip_result = ip_service.check_ip_access(ip, Some(api_key.app_id)).await
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("{}", e)))?;
        
        if ip_result == IpAccessResult::Blocked {
            tracing::warn!("IP {} blocked for app {} via API key", ip, api_key.app_id);
            return Err(AppError::Auth(crate::error::AuthError::UserBanned {
                reason: Some(format!("IP address {} is blocked for this app", ip)),
            }));
        }
    }

    // 6. Update last_used_at (fire and forget)
    let pool = state.pool.clone();
    let key_id = api_key.id;
    tokio::spawn(async move {
        let repo = crate::repositories::ApiKeyRepository::new(pool);
        let _ = repo.update_last_used(key_id).await;
    });

    // 7. Create context and inject into request extensions
    let context = ApiKeyContext {
        api_key_id: api_key.id,
        app_id: api_key.app_id,
        scopes: api_key.scopes.0.clone(),
    };
    request.extensions_mut().insert(context);

    // 8. Call next handler
    Ok(next.run(request).await)
}

/// Extract client IP from request headers
fn extract_client_ip(request: &Request<Body>) -> Option<String> {
    // Check X-Forwarded-For first (for proxied requests)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            return Some(value.split(',').next()?.trim().to_string());
        }
    }

    // Check X-Real-IP
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return Some(value.to_string());
        }
    }

    None
}

/// Context extracted from API Key authentication
#[derive(Debug, Clone)]
pub struct ApiKeyContext {
    pub api_key_id: Uuid,
    pub app_id: Uuid,
    pub scopes: Vec<String>,
}

impl ApiKeyContext {
    /// Check if the API key has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.contains(&scope.to_string()) 
            || self.scopes.contains(&"*".to_string())
            || self.scopes.contains(&"admin".to_string())
    }

    /// Check if the API key has any of the specified scopes
    pub fn has_any_scope(&self, scopes: &[&str]) -> bool {
        scopes.iter().any(|s| self.has_scope(s))
    }

    /// Check if the API key has all of the specified scopes
    pub fn has_all_scopes(&self, scopes: &[&str]) -> bool {
        scopes.iter().all(|s| self.has_scope(s))
    }
}

impl<S> FromRequestParts<S> for ApiKeyContext
where
    S: Send + Sync,
{
    type Rejection = AppError;

    fn from_request_parts<'life0, 'life1, 'async_trait>(
        parts: &'life0 mut Parts,
        _state: &'life1 S,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<Self, Self::Rejection>> + Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            parts
                .extensions
                .get::<ApiKeyContext>()
                .cloned()
                .ok_or(AppError::Auth(crate::error::AuthError::InvalidToken))
        })
    }
}

/// Scope guard middleware - checks if API key has required scope
/// 
/// # Usage
/// ```rust,ignore
/// use axum::{Router, middleware};
/// use auth_server::middleware::{api_key_auth_middleware, require_scope};
/// 
/// let routes = Router::new()
///     .route("/users", get(list_users))
///     .layer(middleware::from_fn(require_scope("read:users")))
///     .layer(middleware::from_fn_with_state(state.clone(), api_key_auth_middleware));
/// ```
pub fn require_scope(
    scope: &'static str,
) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> + Clone {
    move |request: Request<Body>, next: Next| {
        Box::pin(async move {
            let context = request
                .extensions()
                .get::<ApiKeyContext>()
                .ok_or(AppError::Auth(crate::error::AuthError::InvalidToken))?;

            if !context.has_scope(scope) {
                tracing::warn!(
                    "API key {} missing required scope: {}",
                    context.api_key_id,
                    scope
                );
                return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
            }

            Ok(next.run(request).await)
        })
    }
}

/// Scope guard that requires any of the specified scopes
pub fn require_any_scope(
    scopes: &'static [&'static str],
) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, AppError>> + Send>> + Clone {
    move |request: Request<Body>, next: Next| {
        Box::pin(async move {
            let context = request
                .extensions()
                .get::<ApiKeyContext>()
                .ok_or(AppError::Auth(crate::error::AuthError::InvalidToken))?;

            if !context.has_any_scope(scopes) {
                tracing::warn!(
                    "API key {} missing any of required scopes: {:?}",
                    context.api_key_id,
                    scopes
                );
                return Err(AppError::Auth(crate::error::AuthError::InsufficientScope));
            }

            Ok(next.run(request).await)
        })
    }
}

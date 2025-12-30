use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use uuid::Uuid;

use crate::config::AppState;
use crate::error::AuthError;
use crate::utils::jwt::OAuth2Claims;

/// OAuth2 Authentication Middleware
/// 
/// This middleware extracts and verifies OAuth2 access tokens from the Authorization header.
/// On successful verification, it injects the OAuth2Claims into request extensions.
/// 
/// # Requirements
/// - 8.1: WHEN a request includes an access_token, THE Resource_Server SHALL verify 
///        the token signature and expiration
/// - 8.2: WHEN token is expired or invalid, THE Resource_Server SHALL return 401 Unauthorized
/// - 8.4: THE Resource_Server SHALL extract user_id and scopes from validated token 
///        for authorization decisions
/// 
/// # Usage
/// ```rust,ignore
/// use axum::{Router, middleware};
/// use auth_server::middleware::oauth_auth_middleware;
/// 
/// let protected_routes = Router::new()
///     .route("/api/resource", get(handler))
///     .layer(middleware::from_fn_with_state(state.clone(), oauth_auth_middleware));
/// ```
pub async fn oauth_auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AuthError> {
    // 1. Extract token from Authorization header
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(header) => {
            // Expect "Bearer <token>" format
            if let Some(token) = header.strip_prefix("Bearer ") {
                token.trim()
            } else {
                return Err(AuthError::InvalidToken);
            }
        }
        None => {
            return Err(AuthError::InvalidToken);
        }
    };

    // Check for empty token
    if token.is_empty() {
        return Err(AuthError::InvalidToken);
    }

    // 2. Verify OAuth2 token (Requirements 8.1, 8.2)
    let claims = state.jwt_manager.verify_oauth2_token(token)?;

    // 3. Inject claims into request extensions (Requirement 8.4)
    request.extensions_mut().insert(claims);

    // 4. Call next handler
    Ok(next.run(request).await)
}

/// OAuth2Context extractor for handlers
/// 
/// Extracts the user_id and scopes from request extensions that were injected 
/// by oauth_auth_middleware.
/// 
/// # Requirements
/// - 8.4: THE Resource_Server SHALL extract user_id and scopes from validated token 
///        for authorization decisions
/// 
/// # Usage
/// ```rust,ignore
/// use auth_server::middleware::OAuth2Context;
/// 
/// async fn protected_handler(
///     OAuth2Context { user_id, scopes, client_id }: OAuth2Context,
/// ) -> impl IntoResponse {
///     format!("Hello, user {:?}", user_id)
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OAuth2Context {
    /// User ID (None for client credentials tokens)
    pub user_id: Option<Uuid>,
    /// Client ID (audience)
    pub client_id: String,
    /// Granted scopes
    pub scopes: Vec<String>,
}

impl OAuth2Context {
    /// Check if the context has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }

    /// Check if the context has all the required scopes
    pub fn has_all_scopes(&self, required_scopes: &[String]) -> bool {
        required_scopes.iter().all(|s| self.scopes.contains(s))
    }
}

impl<S> FromRequestParts<S> for OAuth2Context
where
    S: Send + Sync,
{
    type Rejection = AuthError;

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
            // Extract OAuth2Claims from request extensions
            let claims = parts
                .extensions
                .get::<OAuth2Claims>()
                .ok_or(AuthError::InvalidToken)?;

            Ok(OAuth2Context {
                user_id: claims.user_id(),
                client_id: claims.client_id().to_string(),
                scopes: claims.scope.clone(),
            })
        })
    }
}

/// Extension trait to easily extract OAuth2 claims from request extensions
pub trait OAuth2ClaimsExt {
    fn oauth2_claims(&self) -> Option<&OAuth2Claims>;
}

impl<B> OAuth2ClaimsExt for Request<B> {
    fn oauth2_claims(&self) -> Option<&OAuth2Claims> {
        self.extensions().get::<OAuth2Claims>()
    }
}

/// Scope Guard Middleware Factory
/// 
/// Creates a middleware that checks if the OAuth2 token has the required scopes.
/// Returns 403 Forbidden if any required scope is missing.
/// 
/// # Requirements
/// - 8.3: WHEN token lacks required scope for the endpoint, THE Resource_Server 
///        SHALL return 403 Forbidden
/// 
/// # Usage
/// ```rust,ignore
/// use axum::{Router, middleware};
/// use auth_server::middleware::{oauth_auth_middleware, scope_guard};
/// 
/// let protected_routes = Router::new()
///     .route("/api/profile", get(handler))
///     .layer(middleware::from_fn(scope_guard(vec!["profile.read".to_string()])))
///     .layer(middleware::from_fn_with_state(state.clone(), oauth_auth_middleware));
/// ```
pub fn scope_guard(
    required_scopes: Vec<String>,
) -> impl Fn(Request<Body>, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, ScopeError>> + Send>> + Clone + Send + 'static {
    move |request: Request<Body>, next: Next| {
        let required_scopes = required_scopes.clone();
        Box::pin(async move {
            // Extract OAuth2Claims from request extensions
            let claims = request
                .extensions()
                .get::<OAuth2Claims>()
                .ok_or(ScopeError::MissingClaims)?;

            // Check if token has all required scopes
            if !claims.has_all_scopes(&required_scopes) {
                return Err(ScopeError::InsufficientScope {
                    required: required_scopes,
                    provided: claims.scope.clone(),
                });
            }

            // All scopes present, proceed
            Ok(next.run(request).await)
        })
    }
}

/// Error type for scope guard middleware
#[derive(Debug)]
pub enum ScopeError {
    /// OAuth2 claims not found in request extensions
    MissingClaims,
    /// Token lacks required scopes
    InsufficientScope {
        required: Vec<String>,
        provided: Vec<String>,
    },
}

#[derive(Serialize)]
struct ScopeErrorResponse {
    error: String,
    message: String,
    required_scopes: Option<Vec<String>>,
}

impl IntoResponse for ScopeError {
    fn into_response(self) -> Response {
        match self {
            ScopeError::MissingClaims => {
                let body = Json(ScopeErrorResponse {
                    error: "invalid_token".to_string(),
                    message: "OAuth2 token required".to_string(),
                    required_scopes: None,
                });
                (StatusCode::UNAUTHORIZED, body).into_response()
            }
            ScopeError::InsufficientScope { required, .. } => {
                let body = Json(ScopeErrorResponse {
                    error: "insufficient_scope".to_string(),
                    message: format!("Token lacks required scope(s): {}", required.join(", ")),
                    required_scopes: Some(required),
                });
                (StatusCode::FORBIDDEN, body).into_response()
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Router,
    };
    use sqlx::mysql::MySqlPoolOptions;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::config::Config;
    use crate::utils::jwt::JwtManager;

    fn get_test_keys() -> (String, String) {
        let private_key = std::fs::read_to_string("keys/private.pem")
            .expect("Failed to read private key");
        let public_key = std::fs::read_to_string("keys/public.pem")
            .expect("Failed to read public key");
        (private_key, public_key)
    }

    fn create_test_jwt_manager() -> JwtManager {
        let (private_key, public_key) = get_test_keys();
        JwtManager::new(&private_key, &public_key, 900, 604800).unwrap()
    }

    async fn create_test_app_state() -> AppState {
        let (private_key, public_key) = get_test_keys();
        
        let config = Config {
            database_url: "mysql://test:test@localhost/test".to_string(),
            jwt_private_key: private_key,
            jwt_public_key: public_key,
            access_token_expiry_secs: 900,
            refresh_token_expiry_secs: 604800,
            server_host: "127.0.0.1".to_string(),
            server_port: 3000,
        };

        let pool = MySqlPoolOptions::new()
            .max_connections(1)
            .connect_lazy(&config.database_url)
            .expect("Failed to create lazy pool");

        AppState::new(pool, config)
    }

    async fn protected_handler(ctx: OAuth2Context) -> String {
        format!(
            "User: {:?}, Client: {}, Scopes: {}",
            ctx.user_id,
            ctx.client_id,
            ctx.scopes.join(", ")
        )
    }

    async fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/protected", get(protected_handler))
            .layer(middleware::from_fn_with_state(state.clone(), oauth_auth_middleware))
            .with_state(state)
    }

    // Feature: oauth2-external-access, Property 26: Token Signature and Expiration Validation
    // For any access token, validation SHALL verify the RS256 signature and check expiration.
    #[tokio::test]
    async fn test_valid_oauth2_token_passes_middleware() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let client_id = "test-client";
        let scopes = vec!["profile.read".to_string(), "email.read".to_string()];
        
        let token = jwt_manager.create_oauth2_token(user_id, client_id, scopes.clone()).unwrap();
        
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        assert!(body_str.contains(&user_id.to_string()));
        assert!(body_str.contains(client_id));
    }

    #[tokio::test]
    async fn test_missing_authorization_header_rejected() {
        let state = create_test_app_state().await;
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_invalid_token_format_rejected() {
        let state = create_test_app_state().await;
        let app = create_test_router(state).await;
        
        // Missing "Bearer " prefix
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, "invalid-token")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_malformed_jwt_rejected() {
        let state = create_test_app_state().await;
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, "Bearer not.a.valid.jwt")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_oauth2_token_rejected() {
        let state = create_test_app_state().await;
        
        // Create a JWT manager with expiry in the past
        let (private_key, public_key) = get_test_keys();
        let jwt_manager = JwtManager::new(&private_key, &public_key, -3600, 604800).unwrap();
        
        let user_id = Uuid::new_v4();
        let token = jwt_manager.create_oauth2_token(user_id, "client", vec![]).unwrap();
        
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // Feature: oauth2-external-access, Property 28: Token Claims Extraction
    // For any validated token, extracting user_id and scopes SHALL return the values 
    // that were encoded during token creation.
    #[tokio::test]
    async fn test_oauth2_context_extracts_correct_claims() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let client_id = "my-client-app";
        let scopes = vec!["profile.read".to_string(), "email.read".to_string()];
        
        let token = jwt_manager.create_oauth2_token(user_id, client_id, scopes.clone()).unwrap();
        
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        
        // Verify the handler received the correct claims
        assert!(body_str.contains(&user_id.to_string()));
        assert!(body_str.contains(client_id));
        assert!(body_str.contains("profile.read"));
        assert!(body_str.contains("email.read"));
    }

    #[tokio::test]
    async fn test_user_token_rejected_by_oauth2_middleware() {
        // Regular user tokens should NOT be accepted by OAuth2 auth middleware
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        // Create a regular user token (not an OAuth2 token)
        let user_token = jwt_manager.create_access_token(user_id, std::collections::HashMap::new()).unwrap();
        
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, format!("Bearer {}", user_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be rejected because it's a user token, not an OAuth2 token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_app_token_rejected_by_oauth2_middleware() {
        // App tokens should NOT be accepted by OAuth2 auth middleware
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        // Create an app token (not an OAuth2 token)
        let app_token = jwt_manager.create_app_token(app_id).unwrap();
        
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, format!("Bearer {}", app_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Should be rejected because it's an app token, not an OAuth2 token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_bearer_prefix_case_sensitive() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let token = jwt_manager.create_oauth2_token(user_id, "client", vec![]).unwrap();
        
        let app = create_test_router(state).await;
        
        // "bearer" lowercase should be rejected
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, format!("bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_empty_bearer_token_rejected() {
        let state = create_test_app_state().await;
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, "Bearer ")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_client_credentials_token_passes_middleware() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let client_id = "internal-service";
        let scopes = vec!["service.read".to_string()];
        
        let token = jwt_manager.create_oauth2_client_credentials_token(client_id, scopes.clone()).unwrap();
        
        let app = create_test_router(state).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/protected")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body_str = String::from_utf8(body.to_vec()).unwrap();
        // For client credentials, user_id is None
        assert!(body_str.contains("User: None"));
        assert!(body_str.contains(client_id));
    }

    // ============================================
    // Scope Guard Tests
    // ============================================

    async fn scoped_handler() -> &'static str {
        "Access granted"
    }

    async fn create_scoped_router(state: AppState, required_scopes: Vec<String>) -> Router {
        Router::new()
            .route("/scoped", get(scoped_handler))
            .layer(middleware::from_fn(scope_guard(required_scopes)))
            .layer(middleware::from_fn_with_state(state.clone(), oauth_auth_middleware))
            .with_state(state)
    }

    // Feature: oauth2-external-access, Property 27: Scope Enforcement
    // For any API request with a valid token, if the token lacks the required scope 
    // for the endpoint, the request SHALL return 403 Forbidden.
    #[tokio::test]
    async fn test_scope_guard_allows_matching_scopes() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let scopes = vec!["profile.read".to_string(), "email.read".to_string()];
        
        let token = jwt_manager.create_oauth2_token(user_id, "client", scopes).unwrap();
        
        let app = create_scoped_router(state, vec!["profile.read".to_string()]).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/scoped")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_scope_guard_rejects_missing_scope() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let scopes = vec!["profile.read".to_string()];
        
        let token = jwt_manager.create_oauth2_token(user_id, "client", scopes).unwrap();
        
        // Require a scope that the token doesn't have
        let app = create_scoped_router(state, vec!["drive.read".to_string()]).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/scoped")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_scope_guard_requires_all_scopes() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let scopes = vec!["profile.read".to_string()];
        
        let token = jwt_manager.create_oauth2_token(user_id, "client", scopes).unwrap();
        
        // Require multiple scopes, but token only has one
        let app = create_scoped_router(
            state, 
            vec!["profile.read".to_string(), "email.read".to_string()]
        ).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/scoped")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_scope_guard_allows_superset_scopes() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        // Token has more scopes than required
        let scopes = vec![
            "profile.read".to_string(), 
            "email.read".to_string(),
            "drive.read".to_string(),
        ];
        
        let token = jwt_manager.create_oauth2_token(user_id, "client", scopes).unwrap();
        
        // Only require one scope
        let app = create_scoped_router(state, vec!["profile.read".to_string()]).await;
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/scoped")
                    .header(AUTHORIZATION, format!("Bearer {}", token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_scope_guard_without_oauth_middleware_returns_401() {
        // If scope_guard is used without oauth_auth_middleware, it should return 401
        let app = Router::new()
            .route("/scoped", get(scoped_handler))
            .layer(middleware::from_fn(scope_guard(vec!["profile.read".to_string()])));
        
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/scoped")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // ============================================
    // OAuth2Context Tests
    // ============================================

    #[test]
    fn test_oauth2_context_has_scope() {
        let ctx = OAuth2Context {
            user_id: Some(Uuid::new_v4()),
            client_id: "client".to_string(),
            scopes: vec!["profile.read".to_string(), "email.read".to_string()],
        };

        assert!(ctx.has_scope("profile.read"));
        assert!(ctx.has_scope("email.read"));
        assert!(!ctx.has_scope("drive.read"));
    }

    #[test]
    fn test_oauth2_context_has_all_scopes() {
        let ctx = OAuth2Context {
            user_id: Some(Uuid::new_v4()),
            client_id: "client".to_string(),
            scopes: vec!["profile.read".to_string(), "email.read".to_string()],
        };

        assert!(ctx.has_all_scopes(&["profile.read".to_string()]));
        assert!(ctx.has_all_scopes(&["profile.read".to_string(), "email.read".to_string()]));
        assert!(!ctx.has_all_scopes(&["profile.read".to_string(), "drive.read".to_string()]));
    }
}

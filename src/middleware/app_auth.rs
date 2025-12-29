use axum::{
    body::Body,
    extract::{FromRequestParts, State},
    http::{header::AUTHORIZATION, request::Parts, Request},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::error::AuthError;
use crate::utils::jwt::AppTokenClaims;

/// App Authentication Middleware
/// 
/// This middleware extracts and verifies App JWT tokens from the Authorization header.
/// On successful verification, it injects the AppTokenClaims into request extensions.
/// 
/// # Requirements
/// - 7.3: WHEN using app-authenticated endpoints, THE Auth_Server SHALL accept Bearer token 
///        in Authorization header
/// 
/// # Usage
/// ```rust,ignore
/// use axum::{Router, middleware};
/// use auth_server::middleware::app_auth_middleware;
/// 
/// let app_routes = Router::new()
///     .route("/apps/:id/roles", post(handler))
///     .layer(middleware::from_fn_with_state(state.clone(), app_auth_middleware));
/// ```
pub async fn app_auth_middleware(
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

    // 2. Verify token as app token (not user token)
    let claims = state.jwt_manager.verify_app_token(token)?;

    // 3. Inject claims into request extensions
    request.extensions_mut().insert(claims);

    // 4. Call next handler
    Ok(next.run(request).await)
}

/// AppContext extractor for handlers
/// 
/// Extracts the app_id from request extensions that were injected by app_auth_middleware.
/// 
/// # Requirements
/// - 4.1: WHEN an authenticated App creates a role, THE Auth_Server SHALL create the role 
///        scoped to that App
/// - 5.1: WHEN an authenticated App creates a permission, THE Auth_Server SHALL create the 
///        permission scoped to that App
/// 
/// # Usage
/// ```rust,ignore
/// use auth_server::middleware::AppContext;
/// 
/// async fn create_role(
///     AppContext(app_id): AppContext,
///     // ... other extractors
/// ) -> impl IntoResponse {
///     // app_id is the authenticated app's UUID
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AppContext(pub Uuid);

impl AppContext {
    /// Get the app_id
    pub fn app_id(&self) -> Uuid {
        self.0
    }
}

impl<S> FromRequestParts<S> for AppContext
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
            // Extract AppTokenClaims from request extensions
            let claims = parts
                .extensions
                .get::<AppTokenClaims>()
                .ok_or(AuthError::InvalidToken)?;

            Ok(AppContext(claims.app_id))
        })
    }
}

/// Extension trait to easily extract app claims from request extensions
pub trait AppClaimsExt {
    fn app_claims(&self) -> Option<&AppTokenClaims>;
}

impl<B> AppClaimsExt for Request<B> {
    fn app_claims(&self) -> Option<&AppTokenClaims> {
        self.extensions().get::<AppTokenClaims>()
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

    async fn protected_handler(AppContext(app_id): AppContext) -> String {
        format!("App ID: {}", app_id)
    }

    async fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/protected", get(protected_handler))
            .layer(middleware::from_fn_with_state(state.clone(), app_auth_middleware))
            .with_state(state)
    }

    #[tokio::test]
    async fn test_valid_app_token_passes_middleware() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        let token = jwt_manager.create_app_token(app_id).unwrap();
        
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
        assert!(body_str.contains(&app_id.to_string()));
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
    async fn test_user_token_rejected_by_app_middleware() {
        // User tokens should NOT be accepted by app auth middleware
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        // Create a user token (not an app token)
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

        // Should be rejected because it's a user token, not an app token
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_expired_app_token_rejected() {
        let state = create_test_app_state().await;
        
        // Create a JWT manager with expiry in the past
        let (private_key, public_key) = get_test_keys();
        let jwt_manager = JwtManager::new(&private_key, &public_key, -3600, 604800).unwrap();
        
        let app_id = Uuid::new_v4();
        let token = jwt_manager.create_app_token(app_id).unwrap();
        
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

    #[tokio::test]
    async fn test_app_context_extracts_correct_app_id() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        let token = jwt_manager.create_app_token(app_id).unwrap();
        
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
        
        // Verify the handler received the correct app_id
        assert_eq!(body_str, format!("App ID: {}", app_id));
    }

    #[tokio::test]
    async fn test_bearer_prefix_case_sensitive() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        let token = jwt_manager.create_app_token(app_id).unwrap();
        
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
}

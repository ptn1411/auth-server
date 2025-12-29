use axum::{
    body::Body,
    extract::State,
    http::{header::AUTHORIZATION, Request},
    middleware::Next,
    response::Response,
};

use crate::config::AppState;
use crate::error::AuthError;
use crate::utils::jwt::{Claims, JwtManager};

/// JWT Authentication Middleware
/// 
/// This middleware extracts and verifies JWT tokens from the Authorization header.
/// On successful verification, it injects the claims into request extensions.
/// 
/// # Requirements
/// - 11.1: WHEN a request contains a valid JWT in Authorization header, THE Auth_Server SHALL 
///         verify the token and inject claims into request context
/// - 11.2: WHEN a request contains an expired JWT, THE Auth_Server SHALL reject the request 
///         with appropriate error
/// - 11.3: WHEN a request contains an invalid or malformed JWT, THE Auth_Server SHALL reject 
///         the request with appropriate error
/// - 11.4: THE Auth_Server SHALL check token expiry on every protected request
/// 
/// # Usage
/// ```rust,ignore
/// use axum::{Router, middleware};
/// use auth_server::middleware::jwt_auth_middleware;
/// 
/// let protected_routes = Router::new()
///     .route("/protected", get(handler))
///     .layer(middleware::from_fn_with_state(state.clone(), jwt_auth_middleware));
/// ```
pub async fn jwt_auth_middleware(
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

    // 2. Create JWT manager and verify token
    let jwt_manager = JwtManager::new(
        &state.config.jwt_private_key,
        &state.config.jwt_public_key,
        state.config.access_token_expiry_secs,
        state.config.refresh_token_expiry_secs,
    )?;

    // 3. Verify signature and expiry (Requirements 11.2, 11.3, 11.4)
    let claims = jwt_manager.verify_token(token)?;

    // 4. Inject claims into request extensions
    request.extensions_mut().insert(claims);

    // 5. Call next handler
    Ok(next.run(request).await)
}

/// Extension trait to easily extract claims from request extensions
pub trait ClaimsExt {
    fn claims(&self) -> Option<&Claims>;
}

impl<B> ClaimsExt for Request<B> {
    fn claims(&self) -> Option<&Claims> {
        self.extensions().get::<Claims>()
    }
}

/// Helper function to extract claims from request extensions in handlers
/// 
/// # Example
/// ```rust,ignore
/// use axum::Extension;
/// use auth_server::utils::jwt::Claims;
/// 
/// async fn protected_handler(Extension(claims): Extension<Claims>) -> impl IntoResponse {
///     format!("Hello, user {}", claims.sub)
/// }
/// ```
#[allow(dead_code)]
pub fn extract_claims<B>(request: &Request<B>) -> Option<&Claims> {
    request.extensions().get::<Claims>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::get,
        Extension, Router,
    };
    use sqlx::mysql::MySqlPoolOptions;
    use std::collections::HashMap;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::config::Config;
    use crate::utils::jwt::AppClaims;

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

        // Create a mock pool - we won't actually use it in these tests
        // For real integration tests, you'd use a test database
        let pool = MySqlPoolOptions::new()
            .max_connections(1)
            .connect_lazy(&config.database_url)
            .expect("Failed to create lazy pool");

        AppState::new(pool, config)
    }

    async fn protected_handler(Extension(claims): Extension<Claims>) -> String {
        format!("Hello, user {}", claims.sub)
    }

    async fn create_test_router(state: AppState) -> Router {
        Router::new()
            .route("/protected", get(protected_handler))
            .layer(middleware::from_fn_with_state(state.clone(), jwt_auth_middleware))
            .with_state(state)
    }

    // Feature: auth-server, Property 26: Valid JWT Passes Middleware
    // For any request with valid, non-expired JWT in Authorization header, 
    // the middleware SHALL pass the request and inject claims.
    #[tokio::test]
    async fn test_valid_jwt_passes_middleware() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let mut apps = HashMap::new();
        apps.insert(
            "test_app".to_string(),
            AppClaims {
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
            },
        );
        
        let token = jwt_manager.create_access_token(user_id, apps).unwrap();
        
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
    }

    // Feature: auth-server, Property 27: Invalid JWT Rejected by Middleware
    // For any request with expired, invalid, or malformed JWT, 
    // the middleware SHALL reject the request.
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
    async fn test_expired_jwt_rejected() {
        let state = create_test_app_state().await;
        
        // Create a JWT manager with expiry in the past (token will be expired immediately)
        let (private_key, public_key) = get_test_keys();
        // Use -3600 (1 hour in the past) to ensure the token is definitely expired
        let jwt_manager = JwtManager::new(&private_key, &public_key, -3600, 604800).unwrap();
        
        let user_id = Uuid::new_v4();
        let token = jwt_manager.create_access_token(user_id, HashMap::new()).unwrap();
        
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
    async fn test_claims_injected_into_request() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let mut apps = HashMap::new();
        apps.insert(
            "my_app".to_string(),
            AppClaims {
                roles: vec!["admin".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
            },
        );
        
        let token = jwt_manager.create_access_token(user_id, apps.clone()).unwrap();
        
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
    }

    #[tokio::test]
    async fn test_bearer_prefix_case_sensitive() {
        let state = create_test_app_state().await;
        let jwt_manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let token = jwt_manager.create_access_token(user_id, HashMap::new()).unwrap();
        
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

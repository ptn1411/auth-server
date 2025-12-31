mod config;
mod dto;
mod error;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod services;
mod utils;

use axum::{
    http::{header, Method},
    middleware as axum_middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Serialize;
use sqlx::mysql::MySqlPoolOptions;
use std::time::Duration;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{AppState, Config};
use crate::handlers::{
    admin::{
        activate_user_handler, deactivate_user_handler, delete_app_handler, delete_user_handler,
        get_app_handler, get_user_handler, get_user_roles_handler, list_all_apps_handler,
        list_all_users_handler, update_app_handler, update_user_handler,
    },
    app::{app_auth_handler, create_app_handler, list_my_apps_handler, regenerate_secret_handler},
    auth::{
        complete_mfa_login_handler, forgot_password_handler, login_handler, refresh_handler,
        register_handler, reset_password_handler,
    },
    oauth::{
        authorize_callback_handler, authorize_handler, connected_apps_handler,
        openid_configuration_handler, register_client_handler, revoke_consent_handler,
        revoke_handler, token_handler, userinfo_handler,
    },
    permission::{
        assign_permission_to_role_handler, create_permission_app_auth_handler,
        create_permission_handler, list_permissions_app_auth_handler,
    },
    role::{
        assign_role_handler, create_role_app_auth_handler, create_role_handler,
        get_user_roles_in_app_handler, list_roles_app_auth_handler, remove_role_handler,
    },
    user_management::{
        ban_user_handler, list_app_users_handler, register_to_app_handler, remove_user_handler,
        unban_user_handler,
    },
    user_profile::{
        bulk_assign_role_handler, change_password_handler, export_users_handler,
        get_profile_handler, import_users_handler, resend_verification_handler,
        search_users_handler, update_profile_handler, verify_email_handler,
    },
    security::{
        disable_mfa_handler, get_all_audit_logs_handler, get_audit_logs_handler,
        list_mfa_methods_handler, list_sessions_handler, logout_handler,
        regenerate_backup_codes_handler, revoke_other_sessions_handler, revoke_session_handler,
        setup_totp_handler, unlock_account_handler, verify_totp_setup_handler,
    },
    webhook::{
        create_webhook_handler, list_webhooks_handler, get_webhook_handler,
        update_webhook_handler, delete_webhook_handler,
    },
    api_key::{
        create_api_key_handler, list_api_keys_handler, get_api_key_handler,
        update_api_key_handler, delete_api_key_handler, revoke_api_key_handler,
    },
    ip_rule::{
        create_ip_rule_handler, create_app_ip_rule_handler, list_ip_rules_handler,
        list_app_ip_rules_handler, check_ip_handler, delete_ip_rule_handler,
    },
    webauthn::{
        start_registration_handler, finish_registration_handler,
        start_authentication_handler, finish_authentication_handler,
        list_credentials_handler, rename_credential_handler, delete_credential_handler,
    },
};
use crate::middleware::{app_auth_middleware, jwt_auth_middleware, oauth_auth_middleware};

/// Health check response
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

/// Health check endpoint
async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy",
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Readiness check - verifies database connection
async fn ready_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<HealthResponse>, axum::http::StatusCode> {
    // Check database connection
    sqlx::query("SELECT 1")
        .execute(&state.pool)
        .await
        .map_err(|_| axum::http::StatusCode::SERVICE_UNAVAILABLE)?;

    Ok(Json(HealthResponse {
        status: "ready",
        version: env!("CARGO_PKG_VERSION"),
    }))
}

/// Create the application router with all routes configured
/// 
/// # Routes
/// 
/// ## Public Routes (no authentication required)
/// - POST /auth/register - User registration (Requirement 14.1)
/// - POST /auth/login - User authentication (Requirement 14.2)
/// - POST /auth/refresh - Token refresh (Requirement 14.3)
/// - POST /auth/forgot-password - Initiate password reset (Requirement 14.4)
/// - POST /auth/reset-password - Complete password reset (Requirement 14.5)
/// - POST /auth/verify-email - Verify email with token
/// - POST /auth/resend-verification - Resend verification email
/// - POST /apps/auth - App authentication with ID and Secret (Requirement 7.1)
/// 
/// ## OAuth2 Public Routes (no authentication required)
/// - GET /oauth/authorize - Authorization endpoint (Requirement 11.1)
/// - POST /oauth/authorize/callback - Consent callback endpoint
/// - POST /oauth/token - Token endpoint (Requirement 11.2)
/// - POST /oauth/revoke - Token revocation endpoint (Requirement 11.3)
/// - POST /oauth/clients - Client registration endpoint (Requirement 1.1, 1.4)
/// - GET /.well-known/openid-configuration - Discovery endpoint (Requirement 11.5)
/// 
/// ## OAuth2 Protected Routes (OAuth2 token required)
/// - GET /oauth/userinfo - UserInfo endpoint (Requirement 11.4)
/// 
/// ## Protected Routes (JWT authentication required)
/// - POST /apps - Create new app (Requirement 14.6)
/// - POST /apps/{app_id}/roles - Create role for app (Requirement 14.7)
/// - POST /apps/{app_id}/permissions - Create permission for app (Requirement 14.8)
/// - POST /apps/{app_id}/users/{user_id}/roles - Assign role to user (Requirement 14.9)
/// - POST /apps/{id}/secret/regenerate - Regenerate app secret (Requirement 7.2)
/// - GET /users/me - Get current user profile (Requirement 8.1)
/// - PUT /users/me - Update current user profile
/// - POST /users/me/change-password - Change password when logged in
/// 
/// ## App User Management Routes (JWT authentication required)
/// - POST /apps/{app_id}/register - Register current user to app (Requirement 8.5)
/// - POST /apps/{app_id}/users/{user_id}/ban - Ban user from app (Requirement 8.1)
/// - POST /apps/{app_id}/users/{user_id}/unban - Unban user from app (Requirement 8.2)
/// - DELETE /apps/{app_id}/users/{user_id} - Remove user from app (Requirement 8.3)
/// - GET /apps/{app_id}/users - List app users (Requirement 8.4)
/// 
/// ## App-Authenticated Routes (App JWT token required)
/// - POST /app-api/apps/{id}/roles - Create role (App auth, Requirement 4.1)
/// - GET /app-api/apps/{id}/roles - List roles (App auth, Requirement 4.2)
/// - POST /app-api/apps/{id}/permissions - Create permission (App auth, Requirement 5.1)
/// - GET /app-api/apps/{id}/permissions - List permissions (App auth, Requirement 5.2)
/// - POST /app-api/apps/{id}/roles/{role_id}/permissions - Assign permission to role (App auth, Requirement 6.1)
/// 
/// ## Account Management Routes (JWT authentication required)
/// - GET /account/connected-apps - List connected OAuth apps (Requirement 9.1)
/// - DELETE /account/connected-apps/{client_id} - Revoke consent (Requirement 9.2, 9.3)
/// 
/// ## Admin Routes (JWT authentication required, system admin only)
/// - GET /admin/users - List all users (Requirement 8.6)
/// - GET /admin/apps - List all apps (Requirement 8.7)
/// - POST /admin/users/{user_id}/deactivate - Deactivate user globally (Requirement 8.8)
/// - GET /admin/users/search - Search users with filters
/// - GET /admin/users/export - Export all users
/// - POST /admin/users/import - Import users
/// - POST /admin/users/bulk-assign-role - Bulk assign role to users
pub fn create_router(state: AppState) -> Router {
    // Public auth routes - no authentication required
    let auth_routes = Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_handler))
        .route("/forgot-password", post(forgot_password_handler))
        .route("/reset-password", post(reset_password_handler))
        .route("/verify-email", post(verify_email_handler))
        .route("/resend-verification", post(resend_verification_handler))
        // MFA login completion - public (uses mfa_token for auth)
        .route("/mfa/verify", post(complete_mfa_login_handler))
        // WebAuthn public routes
        .route("/webauthn/authenticate/start", post(start_authentication_handler))
        .route("/webauthn/authenticate/finish", post(finish_authentication_handler));

    // Protected auth routes - JWT authentication required
    let protected_auth_routes = Router::new()
        .route("/logout", post(logout_handler))
        .route("/sessions", get(list_sessions_handler))
        .route("/sessions", delete(revoke_other_sessions_handler))
        .route("/sessions/revoke", post(revoke_session_handler))
        .route("/mfa/totp/setup", post(setup_totp_handler))
        .route("/mfa/totp/verify", post(verify_totp_setup_handler))
        .route("/mfa/methods", get(list_mfa_methods_handler))
        .route("/mfa", delete(disable_mfa_handler))
        .route("/mfa/backup-codes/regenerate", post(regenerate_backup_codes_handler))
        .route("/audit-logs", get(get_audit_logs_handler))
        // WebAuthn protected routes
        .route("/webauthn/register/start", post(start_registration_handler))
        .route("/webauthn/register/finish", post(finish_registration_handler))
        .route("/webauthn/credentials", get(list_credentials_handler))
        .route("/webauthn/credentials/:credential_id", put(rename_credential_handler))
        .route("/webauthn/credentials/:credential_id", delete(delete_credential_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // OAuth2 public routes - no authentication required
    // Requirements: 11.1, 11.2, 11.3, 1.1, 1.4
    let oauth_public_routes = Router::new()
        .route("/authorize", get(authorize_handler))
        .route("/authorize/callback", post(authorize_callback_handler))
        .route("/token", post(token_handler))
        .route("/revoke", post(revoke_handler))
        .route("/clients", post(register_client_handler));

    // OAuth2 protected routes - OAuth2 token required
    // Requirement: 11.4
    let oauth_protected_routes = Router::new()
        .route("/userinfo", get(userinfo_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            oauth_auth_middleware,
        ));

    // OpenID Connect discovery endpoint - public
    // Requirement: 11.5
    let wellknown_routes = Router::new()
        .route("/openid-configuration", get(openid_configuration_handler));

    // Protected user routes - JWT authentication required (Requirement 8.1)
    let protected_user_routes = Router::new()
        .route("/me", get(get_profile_handler))
        .route("/me", put(update_profile_handler))
        .route("/me/change-password", post(change_password_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // Account management routes - JWT authentication required
    // Requirements: 9.1, 9.2, 9.3
    let account_routes = Router::new()
        .route("/connected-apps", get(connected_apps_handler))
        .route("/connected-apps/:client_id", delete(revoke_consent_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // Protected app management routes - JWT authentication required
    let protected_app_routes = Router::new()
        .route("/apps", get(list_my_apps_handler).post(create_app_handler))
        .route("/apps/:app_id/roles", post(create_role_handler))
        .route("/apps/:app_id/permissions", post(create_permission_handler))
        // User role management
        .route("/apps/:app_id/users/:user_id/roles", post(assign_role_handler))
        .route("/apps/:app_id/users/:user_id/roles", get(get_user_roles_in_app_handler))
        .route("/apps/:app_id/users/:user_id/roles/:role_id", delete(remove_role_handler))
        // Secret regeneration (Requirement 7.2)
        .route("/apps/:id/secret/regenerate", post(regenerate_secret_handler))
        // App user management routes (Requirements 8.1-8.5)
        .route("/apps/:app_id/register", post(register_to_app_handler))
        .route("/apps/:app_id/users/:user_id/ban", post(ban_user_handler))
        .route("/apps/:app_id/users/:user_id/unban", post(unban_user_handler))
        .route("/apps/:app_id/users/:user_id", delete(remove_user_handler))
        .route("/apps/:app_id/users", get(list_app_users_handler))
        // Webhook routes
        .route("/apps/:app_id/webhooks", post(create_webhook_handler))
        .route("/apps/:app_id/webhooks", get(list_webhooks_handler))
        .route("/apps/:app_id/webhooks/:webhook_id", get(get_webhook_handler))
        .route("/apps/:app_id/webhooks/:webhook_id", put(update_webhook_handler))
        .route("/apps/:app_id/webhooks/:webhook_id", delete(delete_webhook_handler))
        // API Key routes
        .route("/apps/:app_id/api-keys", post(create_api_key_handler))
        .route("/apps/:app_id/api-keys", get(list_api_keys_handler))
        .route("/apps/:app_id/api-keys/:key_id", get(get_api_key_handler))
        .route("/apps/:app_id/api-keys/:key_id", put(update_api_key_handler))
        .route("/apps/:app_id/api-keys/:key_id", delete(delete_api_key_handler))
        .route("/apps/:app_id/api-keys/:key_id/revoke", post(revoke_api_key_handler))
        // App IP rules
        .route("/apps/:app_id/ip-rules", post(create_app_ip_rule_handler))
        .route("/apps/:app_id/ip-rules", get(list_app_ip_rules_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // App-authenticated routes - App JWT token required (Requirements 4.1, 4.2, 5.1, 5.2, 6.1)
    let app_auth_routes = Router::new()
        .route("/:id/roles", post(create_role_app_auth_handler))
        .route("/:id/roles", get(list_roles_app_auth_handler))
        .route("/:id/permissions", post(create_permission_app_auth_handler))
        .route("/:id/permissions", get(list_permissions_app_auth_handler))
        .route("/:id/roles/:role_id/permissions", post(assign_permission_to_role_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            app_auth_middleware,
        ));

    // Admin routes - JWT authentication required (admin check in handlers)
    // Requirements 8.6-8.8
    let admin_routes = Router::new()
        // User management
        .route("/users", get(list_all_users_handler))
        .route("/users/search", get(search_users_handler))
        .route("/users/export", get(export_users_handler))
        .route("/users/import", post(import_users_handler))
        .route("/users/bulk-assign-role", post(bulk_assign_role_handler))
        .route("/users/:user_id", get(get_user_handler))
        .route("/users/:user_id", put(update_user_handler))
        .route("/users/:user_id", delete(delete_user_handler))
        .route("/users/:user_id/deactivate", post(deactivate_user_handler))
        .route("/users/:user_id/activate", post(activate_user_handler))
        .route("/users/:user_id/unlock", post(unlock_account_handler))
        .route("/users/:user_id/roles", get(get_user_roles_handler))
        // App management
        .route("/apps", get(list_all_apps_handler))
        .route("/apps/:app_id", get(get_app_handler))
        .route("/apps/:app_id", put(update_app_handler))
        .route("/apps/:app_id", delete(delete_app_handler))
        // Audit logs
        .route("/audit-logs", get(get_all_audit_logs_handler))
        // Global IP rules (admin only)
        .route("/ip-rules", post(create_ip_rule_handler))
        .route("/ip-rules", get(list_ip_rules_handler))
        .route("/ip-rules/check", get(check_ip_handler))
        .route("/ip-rules/:rule_id", delete(delete_ip_rule_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // Combine all routes
    Router::new()
        // Health check endpoints
        .route("/health", get(health_handler))
        .route("/ready", get(ready_handler))
        .nest("/auth", auth_routes)
        .nest("/auth", protected_auth_routes)
        .nest("/users", protected_user_routes)
        .merge(protected_app_routes)
        // Public app auth route - no authentication required (Requirement 7.1)
        .route("/apps/auth", post(app_auth_handler))
        .nest("/app-api/apps", app_auth_routes)
        .nest("/admin", admin_routes)
        // OAuth2 routes (Requirements 11.1-11.5)
        .nest("/oauth", oauth_public_routes)
        .nest("/oauth", oauth_protected_routes)
        .nest("/.well-known", wellknown_routes)
        // Account management routes (Requirements 9.1-9.3)
        .nest("/account", account_routes)
        // Middleware layers
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                ])
                .allow_headers([
                    header::AUTHORIZATION,
                    header::CONTENT_TYPE,
                    header::ACCEPT,
                ])
                .max_age(Duration::from_secs(3600)),
        )
        .with_state(state)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "auth_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = Config::from_env()?;

    // Create database pool with production settings
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .min_connections(2)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .max_lifetime(Duration::from_secs(1800))
        .connect(&config.database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    // Get socket address before moving config
    let addr = config.socket_addr();

    // Create app state
    let state = AppState::new(pool, config);

    // Build router
    let app = create_router(state);

    // Start server with graceful shutdown
    tracing::info!("Auth Server v{} listening on {}", env!("CARGO_PKG_VERSION"), addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    tracing::info!("Server shutdown complete");
    Ok(())
}

/// Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown...");
        },
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown...");
        },
    }
}

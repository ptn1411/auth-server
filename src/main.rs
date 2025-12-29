mod config;
mod dto;
mod error;
mod handlers;
mod middleware;
mod models;
mod repositories;
mod services;
mod utils;

use axum::{middleware as axum_middleware, routing::{delete, get, post}, Router};
use sqlx::mysql::MySqlPoolOptions;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::{AppState, Config};
use crate::handlers::{
    admin::{deactivate_user_handler, list_all_apps_handler, list_all_users_handler},
    app::{app_auth_handler, create_app_handler, get_user_profile_handler, regenerate_secret_handler},
    auth::{
        forgot_password_handler, login_handler, refresh_handler, register_handler,
        reset_password_handler,
    },
    permission::{
        assign_permission_to_role_handler, create_permission_app_auth_handler,
        create_permission_handler, list_permissions_app_auth_handler,
    },
    role::{assign_role_handler, create_role_app_auth_handler, create_role_handler, list_roles_app_auth_handler},
    user_management::{
        ban_user_handler, list_app_users_handler, register_to_app_handler, remove_user_handler,
        unban_user_handler,
    },
};
use crate::middleware::{app_auth_middleware, jwt_auth_middleware};

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
/// - POST /apps/auth - App authentication with ID and Secret (Requirement 7.1)
/// 
/// ## Protected Routes (JWT authentication required)
/// - POST /apps - Create new app (Requirement 14.6)
/// - POST /apps/{app_id}/roles - Create role for app (Requirement 14.7)
/// - POST /apps/{app_id}/permissions - Create permission for app (Requirement 14.8)
/// - POST /apps/{app_id}/users/{user_id}/roles - Assign role to user (Requirement 14.9)
/// - POST /apps/{id}/secret/regenerate - Regenerate app secret (Requirement 7.2)
/// - GET /users/me - Get current user profile (Requirement 8.1)
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
/// ## Admin Routes (JWT authentication required, system admin only)
/// - GET /admin/users - List all users (Requirement 8.6)
/// - GET /admin/apps - List all apps (Requirement 8.7)
/// - POST /admin/users/{user_id}/deactivate - Deactivate user globally (Requirement 8.8)
pub fn create_router(state: AppState) -> Router {
    // Public auth routes - no authentication required
    let auth_routes = Router::new()
        .route("/register", post(register_handler))
        .route("/login", post(login_handler))
        .route("/refresh", post(refresh_handler))
        .route("/forgot-password", post(forgot_password_handler))
        .route("/reset-password", post(reset_password_handler));

    // Public app auth route - no authentication required (Requirement 7.1)
    let public_app_routes = Router::new()
        .route("/auth", post(app_auth_handler));

    // Protected user routes - JWT authentication required (Requirement 8.1)
    let protected_user_routes = Router::new()
        .route("/me", get(get_user_profile_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // Protected app management routes - JWT authentication required
    let protected_app_routes = Router::new()
        .route("/", post(create_app_handler))
        .route("/{app_id}/roles", post(create_role_handler))
        .route("/{app_id}/permissions", post(create_permission_handler))
        .route("/{app_id}/users/{user_id}/roles", post(assign_role_handler))
        // Secret regeneration (Requirement 7.2)
        .route("/{id}/secret/regenerate", post(regenerate_secret_handler))
        // App user management routes (Requirements 8.1-8.5)
        .route("/{app_id}/register", post(register_to_app_handler))
        .route("/{app_id}/users/{user_id}/ban", post(ban_user_handler))
        .route("/{app_id}/users/{user_id}/unban", post(unban_user_handler))
        .route("/{app_id}/users/{user_id}", delete(remove_user_handler))
        .route("/{app_id}/users", get(list_app_users_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // App-authenticated routes - App JWT token required (Requirements 4.1, 4.2, 5.1, 5.2, 6.1)
    let app_auth_routes = Router::new()
        .route("/{id}/roles", post(create_role_app_auth_handler))
        .route("/{id}/roles", get(list_roles_app_auth_handler))
        .route("/{id}/permissions", post(create_permission_app_auth_handler))
        .route("/{id}/permissions", get(list_permissions_app_auth_handler))
        .route("/{id}/roles/{role_id}/permissions", post(assign_permission_to_role_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            app_auth_middleware,
        ));

    // Admin routes - JWT authentication required (admin check in handlers)
    // Requirements 8.6-8.8
    let admin_routes = Router::new()
        .route("/users", get(list_all_users_handler))
        .route("/apps", get(list_all_apps_handler))
        .route("/users/{user_id}/deactivate", post(deactivate_user_handler))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            jwt_auth_middleware,
        ));

    // Combine all routes
    Router::new()
        .nest("/auth", auth_routes)
        .nest("/users", protected_user_routes)
        .merge(Router::new().nest("/apps", public_app_routes))
        .nest("/apps", protected_app_routes)
        .nest("/app-api/apps", app_auth_routes)
        .nest("/admin", admin_routes)
        .layer(TraceLayer::new_for_http())
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

    // Create database pool
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
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

    // Start server
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

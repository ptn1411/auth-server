//! OAuth2 Handlers - HTTP endpoints for OAuth2/OpenID Connect
//!
//! This module implements the OAuth2 endpoints:
//! - GET /oauth/authorize - Authorization endpoint (Requirement 11.1)
//! - POST /oauth/token - Token endpoint (Requirement 11.2)
//! - POST /oauth/revoke - Revocation endpoint (Requirement 11.3)
//! - GET /oauth/userinfo - UserInfo endpoint (Requirement 11.4)
//! - GET /.well-known/openid-configuration - Discovery endpoint (Requirement 11.5)
//! - POST /oauth/clients - Client registration endpoint (Requirement 1.1, 1.4)
//! - GET /account/connected-apps - List connected apps (Requirement 9.1)
//! - DELETE /account/connected-apps/{client_id} - Revoke consent (Requirement 9.2, 9.3)

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::oauth::{
    AuthorizationRequest, ClientRegistrationRequest, ClientRegistrationResponse,
    ConnectedAppInfo, ConnectedAppsResponse, OAuthTokenResponseDto, OpenIdConfiguration,
    RevokeRequest, TokenRequest, UserInfoResponse,
};
use crate::error::OAuthError;
use crate::models::OAuthEventType;
use crate::repositories::{OAuthAuditLogRepository, OAuthClientRepository, UserRepository};
use crate::services::{ConsentService, OAuthService};
use crate::utils::jwt::{Claims, OAuth2Claims};
use crate::utils::secret::{generate_secret, hash_secret};

// ============================================================================
// Authorization Endpoint (Task 11.1)
// Requirements: 3.1, 4.1, 11.1
// ============================================================================

/// Query parameters for consent callback
#[derive(Debug, Deserialize)]
pub struct ConsentCallbackParams {
    /// Whether user approved the consent
    pub approved: bool,
    /// Client ID
    pub client_id: String,
    /// User ID (from session/auth)
    pub user_id: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Scopes (comma-separated)
    pub scopes: String,
    /// State parameter
    pub state: Option<String>,
    /// Code challenge for PKCE
    pub code_challenge: Option<String>,
    /// Code challenge method
    pub code_challenge_method: Option<String>,
}

/// GET /oauth/authorize - Authorization endpoint
///
/// Initiates the OAuth2 Authorization Code Flow.
/// For external apps, validates PKCE parameters and checks user consent.
///
/// # Requirements
/// - 3.1: Require response_type=code, client_id, redirect_uri, scope, code_challenge
/// - 4.1: Display consent screen showing requested scopes
/// - 11.1: Expose GET /oauth/authorize for authorization requests
/// - 10.6: Log all authorization events for audit
///
/// # Flow
/// 1. Validate request parameters
/// 2. Check if user is authenticated (via session/cookie - simplified here)
/// 3. Check if user has already consented
/// 4. If consent needed, redirect to consent page
/// 5. If consent exists, generate authorization code and redirect
///
/// # Note
/// This is a simplified implementation. In production, you would:
/// - Use session management for user authentication
/// - Render an actual consent page
/// - Handle the consent form submission
pub async fn authorize_handler(
    State(state): State<AppState>,
    Query(req): Query<AuthorizationRequest>,
) -> Response {
    let oauth_service = OAuthService::new(state.pool.clone(), state.jwt_manager.clone());
    let audit_repo = OAuthAuditLogRepository::new(state.pool.clone());

    // Validate response_type
    if req.response_type != "code" {
        return build_error_redirect(
            &req.redirect_uri,
            "unsupported_response_type",
            "Only response_type=code is supported",
            req.state.as_deref(),
        );
    }

    // Validate authorization request
    let client = match oauth_service
        .validate_authorization_request(
            &req.client_id,
            &req.redirect_uri,
            &req.scopes(),
            req.code_challenge.as_deref(),
            req.code_challenge_method.as_deref(),
        )
        .await
    {
        Ok(client) => client,
        Err(e) => {
            return build_error_redirect(
                &req.redirect_uri,
                &error_code(&e),
                &e.to_string(),
                req.state.as_deref(),
            );
        }
    };

    // Log authorization request event
    // Requirement 10.6
    audit_repo
        .create(
            OAuthEventType::AuthorizationRequested,
            Some(client.id),
            None, // User not yet authenticated
            None,
            Some(serde_json::json!({
                "scopes": req.scopes(),
                "redirect_uri": req.redirect_uri,
            })),
        )
        .await
        .ok(); // Don't fail if audit logging fails

    // In a real implementation, we would:
    // 1. Check if user is authenticated via session
    // 2. If not, redirect to login page with return URL
    // 3. After login, check consent
    // 4. If consent needed, show consent page
    // 5. After consent, generate code and redirect
    //
    // For this implementation, we return a response indicating
    // that the client needs to handle user authentication and consent
    // through a separate flow, then call the consent callback endpoint.

    // Return information about what's needed for the authorization
    // In production, this would be a redirect to login/consent page
    let response = serde_json::json!({
        "status": "consent_required",
        "client_id": client.client_id,
        "client_name": client.name,
        "redirect_uri": req.redirect_uri,
        "scopes": req.scopes(),
        "state": req.state,
        "code_challenge": req.code_challenge,
        "code_challenge_method": req.code_challenge_method,
        "message": "User authentication and consent required. Submit consent decision to POST /oauth/authorize/callback"
    });

    (StatusCode::OK, Json(response)).into_response()
}

/// POST /oauth/authorize/callback - Handle consent decision
///
/// Called after user authenticates and makes consent decision.
/// Generates authorization code and redirects to client.
///
/// # Requirements
/// - 3.4: Generate short-lived authorization code (max 10 minutes)
/// - 4.3: Store consent record with user_id, client_id, scopes, timestamp
/// - 4.4: Redirect with access_denied error if user denies consent
/// - 9.5, 10.6: Log consent events for audit
pub async fn authorize_callback_handler(
    State(state): State<AppState>,
    Json(params): Json<ConsentCallbackParams>,
) -> Response {
    let oauth_service = OAuthService::new(state.pool.clone(), state.jwt_manager.clone());
    let consent_service = ConsentService::new(state.pool.clone());

    // Parse user_id
    let user_id = match uuid::Uuid::parse_str(&params.user_id) {
        Ok(id) => id,
        Err(_) => {
            return build_error_redirect(
                &params.redirect_uri,
                "invalid_request",
                "Invalid user_id",
                params.state.as_deref(),
            );
        }
    };

    // Get client first for logging
    let client = match oauth_service
        .client_repo()
        .find_active_by_client_id(&params.client_id)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return build_error_redirect(
                &params.redirect_uri,
                "invalid_client",
                "Client not found",
                params.state.as_deref(),
            );
        }
        Err(e) => {
            return build_error_redirect(
                &params.redirect_uri,
                "server_error",
                &e.to_string(),
                params.state.as_deref(),
            );
        }
    };

    let scopes: Vec<String> = params.scopes.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();

    // Validate that all requested scopes exist
    // Requirement 2.4
    if let Err(e) = oauth_service.validate_scopes(&scopes).await {
        return build_error_redirect(
            &params.redirect_uri,
            "invalid_scope",
            &e.to_string(),
            params.state.as_deref(),
        );
    }

    // If user denied consent
    if !params.approved {
        // Log consent denied event
        // Requirements: 9.5, 10.6
        consent_service
            .log_consent_denied(user_id, client.id, &scopes)
            .await
            .ok();

        return build_error_redirect(
            &params.redirect_uri,
            "access_denied",
            "User denied consent",
            params.state.as_deref(),
        );
    }

    // Store consent if this is an external app
    if client.is_external() {
        if let Err(e) = consent_service
            .grant_consent(user_id, client.id, &scopes)
            .await
        {
            return build_error_redirect(
                &params.redirect_uri,
                "server_error",
                &e.to_string(),
                params.state.as_deref(),
            );
        }
    }

    // Generate authorization code
    let code_challenge = params.code_challenge.as_deref().unwrap_or("");
    let code = match oauth_service
        .create_authorization_code(
            client.id,
            user_id,
            &params.redirect_uri,
            &scopes,
            code_challenge,
            params.code_challenge_method.as_deref(),
        )
        .await
    {
        Ok(code) => code,
        Err(e) => {
            return build_error_redirect(
                &params.redirect_uri,
                "server_error",
                &e.to_string(),
                params.state.as_deref(),
            );
        }
    };

    // Build redirect URL with authorization code
    let mut redirect_url = params.redirect_uri.clone();
    redirect_url.push_str(if redirect_url.contains('?') { "&" } else { "?" });
    redirect_url.push_str(&format!("code={}", urlencoding::encode(&code)));
    if let Some(state) = &params.state {
        redirect_url.push_str(&format!("&state={}", urlencoding::encode(state)));
    }

    Redirect::temporary(&redirect_url).into_response()
}

// ============================================================================
// Token Endpoint (Task 11.2)
// Requirements: 5.1, 6.1, 7.1, 11.2
// ============================================================================

/// POST /oauth/token - Token endpoint
///
/// Exchanges authorization code for tokens, handles client credentials,
/// and refreshes tokens.
///
/// # Supported Grant Types
/// - authorization_code: Exchange code for tokens (Requirement 5.1)
/// - client_credentials: Machine-to-machine auth (Requirement 6.1)
/// - refresh_token: Refresh access token (Requirement 7.1)
///
/// # Requirements
/// - 11.2: Expose POST /oauth/token for token requests
pub async fn token_handler(
    State(state): State<AppState>,
    axum::Form(req): axum::Form<TokenRequest>,
) -> Result<Json<OAuthTokenResponseDto>, OAuthError> {
    let oauth_service = OAuthService::new(state.pool.clone(), state.jwt_manager.clone());

    let response = match req.grant_type.as_str() {
        "authorization_code" => {
            handle_authorization_code_grant(&oauth_service, &req).await?
        }
        "client_credentials" => {
            handle_client_credentials_grant(&oauth_service, &req).await?
        }
        "refresh_token" => {
            handle_refresh_token_grant(&oauth_service, &req).await?
        }
        _ => {
            return Err(OAuthError::UnsupportedGrantType);
        }
    };

    Ok(Json(response))
}

/// Handle authorization_code grant type
async fn handle_authorization_code_grant(
    oauth_service: &OAuthService,
    req: &TokenRequest,
) -> Result<OAuthTokenResponseDto, OAuthError> {
    let code = req.code.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("code is required".to_string())
    })?;

    let client_id = req.client_id.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("client_id is required".to_string())
    })?;

    let redirect_uri = req.redirect_uri.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("redirect_uri is required".to_string())
    })?;

    let code_verifier = req.code_verifier.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("code_verifier is required".to_string())
    })?;

    let response = oauth_service
        .exchange_code_for_tokens(
            code,
            client_id,
            req.client_secret.as_deref(),
            redirect_uri,
            code_verifier,
        )
        .await?;

    Ok(response.into())
}

/// Handle client_credentials grant type
async fn handle_client_credentials_grant(
    oauth_service: &OAuthService,
    req: &TokenRequest,
) -> Result<OAuthTokenResponseDto, OAuthError> {
    let client_id = req.client_id.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("client_id is required".to_string())
    })?;

    let client_secret = req.client_secret.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("client_secret is required".to_string())
    })?;

    let response = oauth_service
        .client_credentials_grant(client_id, client_secret, &req.scopes())
        .await?;

    Ok(response.into())
}

/// Handle refresh_token grant type
async fn handle_refresh_token_grant(
    oauth_service: &OAuthService,
    req: &TokenRequest,
) -> Result<OAuthTokenResponseDto, OAuthError> {
    let refresh_token = req.refresh_token.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("refresh_token is required".to_string())
    })?;

    let client_id = req.client_id.as_ref().ok_or_else(|| {
        OAuthError::InvalidRequest("client_id is required".to_string())
    })?;

    let response = oauth_service.refresh_token(refresh_token, client_id).await?;

    Ok(response.into())
}

// ============================================================================
// Revoke Endpoint (Task 11.3)
// Requirements: 9.4, 11.3
// ============================================================================

/// POST /oauth/revoke - Token revocation endpoint
///
/// Revokes an access token or refresh token.
///
/// # Requirements
/// - 9.4: Invalidate specific token when app calls revoke endpoint
/// - 11.3: Expose POST /oauth/revoke for token revocation
///
/// # Note
/// Per RFC 7009, this endpoint always returns 200 OK even if the token
/// is invalid or already revoked.
pub async fn revoke_handler(
    State(state): State<AppState>,
    axum::Form(req): axum::Form<RevokeRequest>,
) -> StatusCode {
    let oauth_service = OAuthService::new(state.pool.clone(), state.jwt_manager.clone());

    let client_id = match &req.client_id {
        Some(id) => id.as_str(),
        None => return StatusCode::OK, // Per RFC 7009, return OK even without client_id
    };

    // Attempt to revoke - ignore errors per RFC 7009
    let _ = oauth_service.revoke_token(&req.token, client_id).await;

    StatusCode::OK
}

// ============================================================================
// UserInfo Endpoint (Task 11.4)
// Requirements: 11.4
// ============================================================================

/// Request with Authorization header for userinfo
#[derive(Debug)]
pub struct AuthorizationHeader(pub String);

/// GET /oauth/userinfo - UserInfo endpoint
///
/// Returns user profile information based on the granted scopes.
///
/// # Requirements
/// - 11.4: Expose GET /oauth/userinfo for retrieving user profile
/// - 9.5, 10.6: Log invalid token attempts for audit
///
/// # Scopes
/// - profile: Returns name
/// - email: Returns email and email_verified
pub async fn userinfo_handler(
    State(state): State<AppState>,
    headers: axum::http::HeaderMap,
) -> Result<Json<UserInfoResponse>, OAuthError> {
    let audit_repo = OAuthAuditLogRepository::new(state.pool.clone());

    // Extract Bearer token from Authorization header
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| OAuthError::InvalidRequest("Authorization header required".to_string()))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| OAuthError::InvalidRequest("Bearer token required".to_string()))?;

    // Verify the token
    let claims: OAuth2Claims = match state.jwt_manager.verify_oauth2_token(token) {
        Ok(claims) => claims,
        Err(_) => {
            // Log invalid token attempt
            // Requirements: 9.5, 10.6
            audit_repo
                .create(
                    OAuthEventType::InvalidTokenAttempt,
                    None,
                    None,
                    None,
                    Some(serde_json::json!({
                        "endpoint": "/oauth/userinfo",
                        "reason": "invalid_or_expired_token",
                    })),
                )
                .await
                .ok();

            return Err(OAuthError::InvalidGrant("Invalid or expired token".to_string()));
        }
    };

    // Get user_id from claims
    let user_id = claims.user_id().ok_or_else(|| {
        OAuthError::InvalidGrant("Token does not contain user information".to_string())
    })?;

    // Fetch user from database
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo
        .find_by_id(user_id)
        .await
        .map_err(|e| OAuthError::ServerError(e.to_string()))?
        .ok_or_else(|| OAuthError::InvalidGrant("User not found".to_string()))?;

    // Build response based on scopes
    let mut response = UserInfoResponse {
        sub: user_id.to_string(),
        email: None,
        email_verified: None,
        name: None,
    };

    // Check scopes and include appropriate fields
    if claims.has_scope("email") || claims.has_scope("openid") {
        response.email = Some(user.email.clone());
        response.email_verified = Some(user.email_verified);
    }

    if claims.has_scope("profile") {
        // For now, use email as name since we don't have a separate name field
        response.name = Some(user.email.clone());
    }

    Ok(Json(response))
}

// ============================================================================
// OpenID Configuration Endpoint (Task 11.5)
// Requirements: 11.5
// ============================================================================

/// GET /.well-known/openid-configuration - Discovery endpoint
///
/// Returns OpenID Connect discovery metadata.
///
/// # Requirements
/// - 11.5: Expose GET /.well-known/openid-configuration for discovery metadata
pub async fn openid_configuration_handler(
    State(state): State<AppState>,
) -> Json<OpenIdConfiguration> {
    // Get base URL from config or use default
    let base_url = format!(
        "http://{}:{}",
        state.config.server_host, state.config.server_port
    );

    // Get available scopes from database (simplified - using static list)
    let scopes = vec![
        "openid".to_string(),
        "profile".to_string(),
        "email".to_string(),
    ];

    Json(OpenIdConfiguration::new(&base_url, scopes))
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Build an error redirect response
fn build_error_redirect(
    redirect_uri: &str,
    error: &str,
    description: &str,
    state: Option<&str>,
) -> Response {
    let mut url = redirect_uri.to_string();
    url.push_str(if url.contains('?') { "&" } else { "?" });
    url.push_str(&format!(
        "error={}&error_description={}",
        urlencoding::encode(error),
        urlencoding::encode(description)
    ));
    if let Some(s) = state {
        url.push_str(&format!("&state={}", urlencoding::encode(s)));
    }

    Redirect::temporary(&url).into_response()
}

/// Get error code from OAuthError
fn error_code(error: &OAuthError) -> String {
    match error {
        OAuthError::InvalidRequest(_) => "invalid_request".to_string(),
        OAuthError::InvalidClient => "invalid_client".to_string(),
        OAuthError::InvalidGrant(_) => "invalid_grant".to_string(),
        OAuthError::UnauthorizedClient => "unauthorized_client".to_string(),
        OAuthError::UnsupportedGrantType => "unsupported_grant_type".to_string(),
        OAuthError::InvalidScope(_) => "invalid_scope".to_string(),
        OAuthError::AccessDenied => "access_denied".to_string(),
        OAuthError::ServerError(_) => "server_error".to_string(),
    }
}

// ============================================================================
// Client Registration Endpoint (Task 13.1)
// Requirements: 1.1, 1.4
// ============================================================================

/// POST /oauth/clients - Client registration endpoint
///
/// Registers a new OAuth client application.
///
/// # Requirements
/// - 1.1: Store client_id, client_secret, redirect_uris, and is_internal flag
/// - 1.2: Generate a unique client_id for each registered OAuth_Client
/// - 1.3: Securely hash the client_secret before storing
/// - 1.4: Validate that each URI uses HTTPS protocol for External_App
/// - 1.5: Distinguish between Internal_App and External_App
/// - 9.5, 10.6: Log client registration events for audit
///
/// # Returns
/// - 201 Created with client_id and client_secret (secret only returned once)
pub async fn register_client_handler(
    State(state): State<AppState>,
    Json(req): Json<ClientRegistrationRequest>,
) -> Result<(StatusCode, Json<ClientRegistrationResponse>), OAuthError> {
    let oauth_service = OAuthService::new(state.pool.clone(), state.jwt_manager.clone());
    let audit_repo = OAuthAuditLogRepository::new(state.pool.clone());

    // Validate redirect URIs (HTTPS required for external apps)
    // Requirement 1.4
    oauth_service.validate_redirect_uris_for_registration(&req.redirect_uris, req.is_internal)?;

    // Generate unique client_id
    // Requirement 1.2
    let client_id = generate_client_id();

    // Generate client_secret
    let client_secret = generate_secret();

    // Hash the client_secret before storing
    // Requirement 1.3
    let client_secret_hash = hash_secret(&client_secret)
        .map_err(|e| OAuthError::ServerError(format!("Failed to hash secret: {}", e)))?;

    // Store the client
    // Requirement 1.1
    let client_repo = OAuthClientRepository::new(state.pool.clone());
    let client = client_repo
        .create(
            &client_id,
            &client_secret_hash,
            &req.name,
            &req.redirect_uris,
            req.is_internal,
        )
        .await?;

    // Log client registration event
    // Requirements: 9.5, 10.6
    audit_repo
        .create(
            OAuthEventType::ClientRegistered,
            Some(client.id),
            None,
            None,
            Some(serde_json::json!({
                "name": client.name,
                "is_internal": client.is_internal,
                "redirect_uris_count": client.redirect_uris.len(),
            })),
        )
        .await
        .ok(); // Don't fail if audit logging fails

    // Return the response with the plain text secret (only returned once)
    Ok((
        StatusCode::CREATED,
        Json(ClientRegistrationResponse {
            client_id: client.client_id,
            client_secret, // Plain text, only returned once
            name: client.name,
            redirect_uris: client.redirect_uris,
            is_internal: client.is_internal,
        }),
    ))
}

/// Generate a unique client_id
fn generate_client_id() -> String {
    // Use UUID v4 for uniqueness
    Uuid::new_v4().to_string()
}

// ============================================================================
// Connected Apps Endpoint (Task 13.3)
// Requirements: 9.1
// ============================================================================

/// GET /account/connected-apps - List connected apps
///
/// Returns a list of apps the authenticated user has authorized.
///
/// # Requirements
/// - 9.1: Return list of apps with granted scopes and consent timestamps
///
/// # Authentication
/// Requires JWT authentication (user must be logged in)
pub async fn connected_apps_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<ConnectedAppsResponse>, OAuthError> {
    let user_id = claims.user_id()
        .map_err(|_| OAuthError::InvalidGrant("Invalid user ID in token".to_string()))?;

    let consent_service = ConsentService::new(state.pool.clone());
    let client_repo = OAuthClientRepository::new(state.pool.clone());

    // Get all consents for the user
    let consents = consent_service.list_user_consents(user_id).await?;

    // Build the response with client details
    let mut apps = Vec::with_capacity(consents.len());
    for consent in consents {
        // Get the client to retrieve the client_id string
        if let Ok(Some(client)) = client_repo.find_by_id(consent.client_id).await {
            apps.push(ConnectedAppInfo {
                client_id: client.client_id,
                name: consent.client_name,
                scopes: consent.scopes,
                granted_at: consent.granted_at,
            });
        }
    }

    Ok(Json(ConnectedAppsResponse { apps }))
}

// ============================================================================
// Revoke Consent Endpoint (Task 13.5)
// Requirements: 9.2, 9.3
// ============================================================================

/// DELETE /account/connected-apps/{client_id} - Revoke consent
///
/// Revokes the user's consent for a specific app and invalidates all tokens.
///
/// # Requirements
/// - 9.2: Invalidate all tokens for that client-user pair when user revokes access
/// - 9.3: Delete the consent record when user revokes access
///
/// # Authentication
/// Requires JWT authentication (user must be logged in)
pub async fn revoke_consent_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(client_id): Path<String>,
) -> Result<StatusCode, OAuthError> {
    let user_id = claims.user_id()
        .map_err(|_| OAuthError::InvalidGrant("Invalid user ID in token".to_string()))?;

    let oauth_service = OAuthService::new(state.pool.clone(), state.jwt_manager.clone());
    let consent_service = ConsentService::new(state.pool.clone());
    let client_repo = OAuthClientRepository::new(state.pool.clone());

    // Find the client by client_id string
    let client = client_repo
        .find_by_client_id(&client_id)
        .await?
        .ok_or(OAuthError::InvalidClient)?;

    // Revoke all tokens for this user-client pair
    // Requirement 9.2
    oauth_service
        .revoke_all_tokens_for_user_client(user_id, client.id)
        .await?;

    // Delete the consent record
    // Requirement 9.3
    consent_service.revoke_consent(user_id, client.id).await?;

    Ok(StatusCode::NO_CONTENT)
}

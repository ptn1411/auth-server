//! OAuth2 Request/Response DTOs
//!
//! Data Transfer Objects for OAuth2/OpenID Connect endpoints.
//! Follows RFC 6749 (OAuth 2.0) and RFC 7636 (PKCE) specifications.
//!
//! # Requirements
//! - 11.1: GET /oauth/authorize for authorization requests
//! - 11.2: POST /oauth/token for token requests
//! - 11.3: POST /oauth/revoke for token revocation
//! - 3.6, 6.4, 7.3: Error responses per RFC 6749

use serde::{Deserialize, Serialize};

// ============================================================================
// Authorization Request DTOs (Requirement 11.1)
// ============================================================================

/// Authorization Request - GET /oauth/authorize
///
/// Query parameters for the authorization endpoint.
/// Supports Authorization Code Flow with PKCE.
///
/// # Requirements
/// - 3.1: Require response_type=code, client_id, redirect_uri, scope, code_challenge
/// - 3.2: code_challenge required for external apps
#[derive(Debug, Clone, Deserialize)]
pub struct AuthorizationRequest {
    /// Must be "code" for Authorization Code Flow
    pub response_type: String,
    /// The client's public identifier
    pub client_id: String,
    /// The redirect URI for the callback
    pub redirect_uri: String,
    /// Space-separated list of requested scopes
    #[serde(default)]
    pub scope: Option<String>,
    /// PKCE code challenge (required for external apps)
    pub code_challenge: Option<String>,
    /// PKCE code challenge method (must be "S256")
    #[serde(default = "default_code_challenge_method")]
    pub code_challenge_method: Option<String>,
    /// Opaque value to maintain state between request and callback
    pub state: Option<String>,
}

fn default_code_challenge_method() -> Option<String> {
    Some("S256".to_string())
}

impl AuthorizationRequest {
    /// Parse scopes from space-separated string to Vec
    pub fn scopes(&self) -> Vec<String> {
        self.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }
}

/// Authorization Response - redirect with code
///
/// Returned as query parameters in the redirect URI.
#[derive(Debug, Clone, Serialize)]
pub struct AuthorizationResponse {
    /// The authorization code
    pub code: String,
    /// The state value from the request (if provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

// ============================================================================
// Token Request DTOs (Requirement 11.2)
// ============================================================================

/// Token Request - POST /oauth/token
///
/// Supports multiple grant types:
/// - authorization_code: Exchange code for tokens
/// - client_credentials: Machine-to-machine authentication
/// - refresh_token: Refresh access token
///
/// # Requirements
/// - 5.1: Exchange authorization code for tokens
/// - 6.1: Client credentials grant
/// - 7.1: Refresh token grant
#[derive(Debug, Clone, Deserialize)]
pub struct TokenRequest {
    /// The grant type (authorization_code, client_credentials, refresh_token)
    pub grant_type: String,
    /// Authorization code (for authorization_code grant)
    pub code: Option<String>,
    /// Redirect URI (for authorization_code grant, must match original)
    pub redirect_uri: Option<String>,
    /// Client ID
    pub client_id: Option<String>,
    /// Client secret (for confidential clients)
    pub client_secret: Option<String>,
    /// PKCE code verifier (for authorization_code grant)
    pub code_verifier: Option<String>,
    /// Refresh token (for refresh_token grant)
    pub refresh_token: Option<String>,
    /// Requested scopes (for client_credentials grant)
    pub scope: Option<String>,
}

impl TokenRequest {
    /// Parse scopes from space-separated string to Vec
    pub fn scopes(&self) -> Vec<String> {
        self.scope
            .as_ref()
            .map(|s| s.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }
}

/// OAuth Token Response - POST /oauth/token success response
///
/// This is the DTO version for handlers. The service layer uses
/// `crate::services::OAuthTokenResponse` which has the same structure.
///
/// # Requirements
/// - 5.1: Return access_token and refresh_token
/// - 5.3: Include granted scopes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponseDto {
    /// The access token
    pub access_token: String,
    /// The refresh token (not included for client_credentials grant)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Token type (always "Bearer")
    pub token_type: String,
    /// Token expiration time in seconds
    pub expires_in: i64,
    /// Space-separated list of granted scopes
    pub scope: String,
}

impl OAuthTokenResponseDto {
    /// Create a new token response
    pub fn new(
        access_token: String,
        refresh_token: Option<String>,
        expires_in: i64,
        scopes: &[String],
    ) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
            scope: scopes.join(" "),
        }
    }
}

/// Convert from service layer OAuthTokenResponse to DTO
impl From<crate::services::OAuthTokenResponse> for OAuthTokenResponseDto {
    fn from(response: crate::services::OAuthTokenResponse) -> Self {
        Self {
            access_token: response.access_token,
            refresh_token: response.refresh_token,
            token_type: response.token_type,
            expires_in: response.expires_in,
            scope: response.scope,
        }
    }
}

// ============================================================================
// Revoke Request DTOs (Requirement 11.3)
// ============================================================================

/// Revoke Request - POST /oauth/revoke
///
/// # Requirements
/// - 9.4: Revoke specific token
#[derive(Debug, Clone, Deserialize)]
pub struct RevokeRequest {
    /// The token to revoke (access_token or refresh_token)
    pub token: String,
    /// Optional hint about the token type
    #[serde(default)]
    pub token_type_hint: Option<String>,
    /// Client ID (required for client authentication)
    pub client_id: Option<String>,
    /// Client secret (for confidential clients)
    pub client_secret: Option<String>,
}

// ============================================================================
// UserInfo Response DTOs (Requirement 11.4)
// ============================================================================

/// UserInfo Response - GET /oauth/userinfo
///
/// Returns user profile information based on granted scopes.
#[derive(Debug, Clone, Serialize)]
pub struct UserInfoResponse {
    /// Subject identifier (user ID)
    pub sub: String,
    /// User's email (requires email scope)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Whether email is verified (requires email scope)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    /// User's name (requires profile scope)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ============================================================================
// OpenID Configuration Response (Requirement 11.5)
// ============================================================================

/// OpenID Configuration - GET /.well-known/openid-configuration
///
/// Discovery metadata for OpenID Connect.
#[derive(Debug, Clone, Serialize)]
pub struct OpenIdConfiguration {
    /// URL of the authorization server's authorization endpoint
    pub authorization_endpoint: String,
    /// URL of the authorization server's token endpoint
    pub token_endpoint: String,
    /// URL of the authorization server's userinfo endpoint
    pub userinfo_endpoint: String,
    /// URL of the authorization server's revocation endpoint
    pub revocation_endpoint: String,
    /// URL of the authorization server's issuer identifier
    pub issuer: String,
    /// JSON array of supported response types
    pub response_types_supported: Vec<String>,
    /// JSON array of supported grant types
    pub grant_types_supported: Vec<String>,
    /// JSON array of supported scopes
    pub scopes_supported: Vec<String>,
    /// JSON array of supported token endpoint authentication methods
    pub token_endpoint_auth_methods_supported: Vec<String>,
    /// JSON array of supported code challenge methods
    pub code_challenge_methods_supported: Vec<String>,
}

impl OpenIdConfiguration {
    /// Create a new OpenID configuration with the given base URL
    pub fn new(base_url: &str, scopes: Vec<String>) -> Self {
        Self {
            issuer: base_url.to_string(),
            authorization_endpoint: format!("{}/oauth/authorize", base_url),
            token_endpoint: format!("{}/oauth/token", base_url),
            userinfo_endpoint: format!("{}/oauth/userinfo", base_url),
            revocation_endpoint: format!("{}/oauth/revoke", base_url),
            response_types_supported: vec!["code".to_string()],
            grant_types_supported: vec![
                "authorization_code".to_string(),
                "client_credentials".to_string(),
                "refresh_token".to_string(),
            ],
            scopes_supported: scopes,
            token_endpoint_auth_methods_supported: vec![
                "client_secret_post".to_string(),
                "client_secret_basic".to_string(),
            ],
            code_challenge_methods_supported: vec!["S256".to_string()],
        }
    }
}

// ============================================================================
// OAuth Error Response (Requirements 3.6, 6.4, 7.3)
// ============================================================================

/// OAuth Error Response - RFC 6749 compliant error response
///
/// # Requirements
/// - 3.6: Error response for code_verifier validation failure
/// - 6.4: Error response for invalid client credentials
/// - 7.3: Error response for invalid/expired refresh token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthErrorResponse {
    /// Error code (e.g., "invalid_request", "invalid_client")
    pub error: String,
    /// Human-readable error description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_description: Option<String>,
    /// URI for more information about the error
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_uri: Option<String>,
}

impl OAuthErrorResponse {
    /// Create a new error response
    pub fn new(error: &str, description: Option<&str>) -> Self {
        Self {
            error: error.to_string(),
            error_description: description.map(String::from),
            error_uri: None,
        }
    }

    /// Create an invalid_request error
    pub fn invalid_request(description: &str) -> Self {
        Self::new("invalid_request", Some(description))
    }

    /// Create an invalid_client error
    pub fn invalid_client() -> Self {
        Self::new("invalid_client", Some("Client authentication failed"))
    }

    /// Create an invalid_grant error
    pub fn invalid_grant(description: &str) -> Self {
        Self::new("invalid_grant", Some(description))
    }

    /// Create an unauthorized_client error
    pub fn unauthorized_client() -> Self {
        Self::new("unauthorized_client", Some("Client not authorized for this grant type"))
    }

    /// Create an unsupported_grant_type error
    pub fn unsupported_grant_type() -> Self {
        Self::new("unsupported_grant_type", Some("Grant type not supported"))
    }

    /// Create an invalid_scope error
    pub fn invalid_scope(description: &str) -> Self {
        Self::new("invalid_scope", Some(description))
    }

    /// Create an access_denied error
    pub fn access_denied() -> Self {
        Self::new("access_denied", Some("User denied consent"))
    }

    /// Create a server_error
    pub fn server_error() -> Self {
        Self::new("server_error", Some("Internal server error"))
    }
}

// ============================================================================
// Client Registration DTOs
// ============================================================================

/// Client Registration Request
///
/// Used to register a new OAuth client.
#[derive(Debug, Clone, Deserialize)]
pub struct ClientRegistrationRequest {
    /// Client name
    pub name: String,
    /// Redirect URIs
    pub redirect_uris: Vec<String>,
    /// Whether this is an internal app
    #[serde(default)]
    pub is_internal: bool,
}

/// Client Registration Response
///
/// Returned after successful client registration.
#[derive(Debug, Clone, Serialize)]
pub struct ClientRegistrationResponse {
    /// The client's public identifier
    pub client_id: String,
    /// The client's secret (only returned once)
    pub client_secret: String,
    /// Client name
    pub name: String,
    /// Redirect URIs
    pub redirect_uris: Vec<String>,
    /// Whether this is an internal app
    pub is_internal: bool,
}

// ============================================================================
// Connected Apps DTOs (Requirement 9.1)
// ============================================================================

/// Connected App Info
///
/// Information about an app the user has authorized.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectedAppInfo {
    /// Client ID
    pub client_id: String,
    /// Client name
    pub name: String,
    /// Granted scopes
    pub scopes: Vec<String>,
    /// When consent was granted
    pub granted_at: chrono::DateTime<chrono::Utc>,
}

/// Connected Apps Response
///
/// List of apps the user has authorized.
#[derive(Debug, Clone, Serialize)]
pub struct ConnectedAppsResponse {
    /// List of connected apps
    pub apps: Vec<ConnectedAppInfo>,
}

// ============================================================================
// Consent DTOs
// ============================================================================

/// Consent Request
///
/// Information displayed on the consent screen.
#[derive(Debug, Clone, Serialize)]
pub struct ConsentScreenInfo {
    /// Client name
    pub client_name: String,
    /// Requested scopes with descriptions
    pub scopes: Vec<ScopeInfo>,
    /// Redirect URI
    pub redirect_uri: String,
    /// State parameter
    pub state: Option<String>,
}

/// Scope Info
///
/// Scope code and description for consent screen.
#[derive(Debug, Clone, Serialize)]
pub struct ScopeInfo {
    /// Scope code
    pub code: String,
    /// Human-readable description
    pub description: String,
}

/// Consent Decision
///
/// User's decision on the consent screen.
#[derive(Debug, Clone, Deserialize)]
pub struct ConsentDecision {
    /// Whether user approved
    pub approved: bool,
    /// Client ID
    pub client_id: String,
    /// Redirect URI
    pub redirect_uri: String,
    /// Approved scopes
    pub scopes: Vec<String>,
    /// State parameter
    pub state: Option<String>,
    /// Code challenge for PKCE
    pub code_challenge: Option<String>,
    /// Code challenge method
    pub code_challenge_method: Option<String>,
}


// ============================================================================
// Conversion from OAuthError to OAuthErrorResponse
// ============================================================================

impl From<&crate::error::OAuthError> for OAuthErrorResponse {
    fn from(error: &crate::error::OAuthError) -> Self {
        match error {
            crate::error::OAuthError::InvalidRequest(desc) => {
                OAuthErrorResponse::invalid_request(desc)
            }
            crate::error::OAuthError::InvalidClient => {
                OAuthErrorResponse::invalid_client()
            }
            crate::error::OAuthError::InvalidGrant(desc) => {
                OAuthErrorResponse::invalid_grant(desc)
            }
            crate::error::OAuthError::UnauthorizedClient => {
                OAuthErrorResponse::unauthorized_client()
            }
            crate::error::OAuthError::UnsupportedGrantType => {
                OAuthErrorResponse::unsupported_grant_type()
            }
            crate::error::OAuthError::InvalidScope(desc) => {
                OAuthErrorResponse::invalid_scope(desc)
            }
            crate::error::OAuthError::AccessDenied => {
                OAuthErrorResponse::access_denied()
            }
            crate::error::OAuthError::ServerError(_) => {
                OAuthErrorResponse::server_error()
            }
        }
    }
}

impl From<crate::error::OAuthError> for OAuthErrorResponse {
    fn from(error: crate::error::OAuthError) -> Self {
        OAuthErrorResponse::from(&error)
    }
}

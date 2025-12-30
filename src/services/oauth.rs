//! OAuth2 Service - Core logic for OAuth2/OpenID Connect authorization
//!
//! This service implements the OAuth2 Authorization Code Flow with PKCE for external apps
//! and Client Credentials Flow for internal apps.
//!
//! # Requirements
//! - 3.1, 3.3, 10.5: Authorization request validation
//! - 3.4: Authorization code generation
//! - 3.5, 5.1, 5.3: Token exchange
//! - 6.1, 6.2, 6.5: Client credentials flow
//! - 7.1, 7.2, 7.4: Token refresh with rotation
//! - 9.2, 9.4: Token revocation

use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::{OAuthClient, OAuthEventType};
use crate::repositories::{
    AuthorizationCodeRepository, OAuthAuditLogRepository, OAuthClientRepository,
    OAuthScopeRepository, OAuthTokenRepository, UserConsentRepository,
};
use crate::services::ConsentService;
use crate::utils::jwt::JwtManager;
use crate::utils::pkce::{validate_code_challenge, validate_code_verifier, verify_pkce, PKCE_METHOD_S256};
use crate::utils::secret::{generate_oauth_token, hash_oauth_token, verify_secret};

/// OAuth2 Token Response
/// Requirements: 5.1, 5.3
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokenResponse {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
    pub scope: String,
}

impl OAuthTokenResponse {
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

/// OAuth2 Service - handles OAuth2 authorization flows
#[derive(Clone)]
pub struct OAuthService {
    client_repo: OAuthClientRepository,
    scope_repo: OAuthScopeRepository,
    code_repo: AuthorizationCodeRepository,
    token_repo: OAuthTokenRepository,
    consent_repo: UserConsentRepository,
    audit_repo: OAuthAuditLogRepository,
    consent_service: ConsentService,
    jwt_manager: JwtManager,
    pool: MySqlPool,
}


impl OAuthService {
    /// Create a new OAuthService with the given database pool and JWT manager
    pub fn new(pool: MySqlPool, jwt_manager: JwtManager) -> Self {
        Self {
            client_repo: OAuthClientRepository::new(pool.clone()),
            scope_repo: OAuthScopeRepository::new(pool.clone()),
            code_repo: AuthorizationCodeRepository::new(pool.clone()),
            token_repo: OAuthTokenRepository::new(pool.clone()),
            consent_repo: UserConsentRepository::new(pool.clone()),
            audit_repo: OAuthAuditLogRepository::new(pool.clone()),
            consent_service: ConsentService::new(pool.clone()),
            jwt_manager,
            pool,
        }
    }

    // ========================================================================
    // Authorization Request Validation (Task 8.1)
    // Requirements: 3.1, 3.3, 10.5
    // ========================================================================

    /// Validate an authorization request
    ///
    /// # Arguments
    /// * `client_id` - The client's public identifier
    /// * `redirect_uri` - The redirect URI for the callback
    /// * `scopes` - The requested scopes
    /// * `code_challenge` - The PKCE code challenge (required for external apps)
    /// * `code_challenge_method` - The PKCE method (must be "S256" for external apps)
    ///
    /// # Returns
    /// * `Ok(OAuthClient)` - The validated client
    /// * `Err(OAuthError)` - If validation fails
    ///
    /// # Requirements
    /// - 3.1: Require response_type=code, client_id, redirect_uri, scope, and code_challenge
    /// - 3.2: Reject request if code_challenge is missing for External_App
    /// - 3.3: Reject request if redirect_uri does not match registered URIs
    /// - 10.2: Require PKCE for all External_App authorization requests
    /// - 10.5: Validate redirect_uri exactly matches registered URIs
    pub async fn validate_authorization_request(
        &self,
        client_id: &str,
        redirect_uri: &str,
        scopes: &[String],
        code_challenge: Option<&str>,
        code_challenge_method: Option<&str>,
    ) -> Result<OAuthClient, OAuthError> {
        // Find the client
        let client = self.client_repo
            .find_active_by_client_id(client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;

        // Validate redirect_uri exactly matches one of the registered URIs
        // Requirements: 3.3, 10.5
        if !self.validate_redirect_uri(&client, redirect_uri) {
            return Err(OAuthError::InvalidRequest(
                "redirect_uri does not match any registered URI".to_string(),
            ));
        }

        // For external apps, PKCE is required
        // Requirements: 3.2, 10.2
        if client.is_external() {
            let challenge = code_challenge.ok_or_else(|| {
                OAuthError::InvalidRequest("code_challenge is required for external apps".to_string())
            })?;

            // Validate code_challenge format
            if !validate_code_challenge(challenge) {
                return Err(OAuthError::InvalidRequest(
                    "Invalid code_challenge format".to_string(),
                ));
            }

            // Validate code_challenge_method is S256
            let method = code_challenge_method.unwrap_or(PKCE_METHOD_S256);
            if method != PKCE_METHOD_S256 {
                return Err(OAuthError::InvalidRequest(
                    "code_challenge_method must be S256".to_string(),
                ));
            }
        }

        // Validate scopes exist
        // Requirement: 2.4
        if !scopes.is_empty() {
            let valid = self.scope_repo.validate_scopes(scopes).await?;
            if !valid {
                return Err(OAuthError::InvalidScope(
                    "One or more requested scopes are invalid".to_string(),
                ));
            }
        }

        Ok(client)
    }

    /// Validate that a redirect_uri exactly matches one of the registered URIs
    ///
    /// # Requirements
    /// - 3.3: Reject request if redirect_uri does not match registered URIs
    /// - 10.5: Validate redirect_uri exactly matches registered URIs (no partial matching)
    pub fn validate_redirect_uri(&self, client: &OAuthClient, redirect_uri: &str) -> bool {
        client.has_redirect_uri(redirect_uri)
    }

    /// Validate redirect URIs for client registration
    ///
    /// # Requirements
    /// - 1.4: Validate that each URI uses HTTPS protocol for External_App
    pub fn validate_redirect_uris_for_registration(
        &self,
        redirect_uris: &[String],
        is_internal: bool,
    ) -> Result<(), OAuthError> {
        for uri in redirect_uris {
            // For external apps, require HTTPS
            if !is_internal && !uri.starts_with("https://") {
                // Allow localhost for development
                if !uri.starts_with("http://localhost") && !uri.starts_with("http://127.0.0.1") {
                    return Err(OAuthError::InvalidRequest(format!(
                        "External apps must use HTTPS for redirect URIs: {}",
                        uri
                    )));
                }
            }
        }
        Ok(())
    }


    // ========================================================================
    // Authorization Code Generation (Task 8.3)
    // Requirements: 3.4
    // ========================================================================

    /// Generate an authorization code after user consent
    ///
    /// # Arguments
    /// * `client_id` - The client's UUID
    /// * `user_id` - The user's UUID
    /// * `redirect_uri` - The redirect URI for the callback
    /// * `scopes` - The granted scopes
    /// * `code_challenge` - The PKCE code challenge
    /// * `code_challenge_method` - The PKCE method (default: "S256")
    ///
    /// # Returns
    /// * `Ok(String)` - The authorization code (plain text, to be sent to client)
    /// * `Err(OAuthError)` - If code generation fails
    ///
    /// # Requirements
    /// - 3.4: Generate short-lived authorization code (max 10 minutes)
    pub async fn create_authorization_code(
        &self,
        client_id: Uuid,
        user_id: Uuid,
        redirect_uri: &str,
        scopes: &[String],
        code_challenge: &str,
        code_challenge_method: Option<&str>,
    ) -> Result<String, OAuthError> {
        // Generate a random authorization code
        let code = generate_oauth_token();
        let code_hash = hash_oauth_token(&code);

        let method = code_challenge_method.unwrap_or(PKCE_METHOD_S256);

        // Store the authorization code (max 10 minutes = 600 seconds)
        self.code_repo
            .create(
                &code_hash,
                client_id,
                user_id,
                redirect_uri,
                scopes,
                code_challenge,
                method,
                600, // 10 minutes max
            )
            .await?;

        // Log the event
        self.audit_repo
            .create(
                OAuthEventType::AuthorizationCodeIssued,
                Some(client_id),
                Some(user_id),
                None,
                Some(serde_json::json!({
                    "scopes": scopes,
                    "redirect_uri": redirect_uri,
                })),
            )
            .await
            .ok(); // Don't fail if audit logging fails

        Ok(code)
    }

    // ========================================================================
    // Token Exchange (Task 8.5)
    // Requirements: 3.5, 5.1, 5.3
    // ========================================================================

    /// Exchange an authorization code for tokens
    ///
    /// # Arguments
    /// * `code` - The authorization code
    /// * `client_id` - The client's public identifier
    /// * `client_secret` - The client's secret (optional for public clients)
    /// * `redirect_uri` - The redirect URI (must match the one used in authorization)
    /// * `code_verifier` - The PKCE code verifier
    ///
    /// # Returns
    /// * `Ok(OAuthTokenResponse)` - The access and refresh tokens
    /// * `Err(OAuthError)` - If exchange fails
    ///
    /// # Requirements
    /// - 3.5: Verify code_verifier matches the original code_challenge
    /// - 3.6: Reject if code_verifier validation fails
    /// - 5.1: Issue access_token and refresh_token
    /// - 5.3: Include granted scopes in the token response
    pub async fn exchange_code_for_tokens(
        &self,
        code: &str,
        client_id: &str,
        client_secret: Option<&str>,
        redirect_uri: &str,
        code_verifier: &str,
    ) -> Result<OAuthTokenResponse, OAuthError> {
        // Find the client
        let client = self.client_repo
            .find_active_by_client_id(client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;

        // Verify client secret if provided (confidential clients)
        if let Some(secret) = client_secret {
            let valid = verify_secret(secret, &client.client_secret_hash)
                .map_err(|_| OAuthError::InvalidClient)?;
            if !valid {
                return Err(OAuthError::InvalidClient);
            }
        }

        // Find the authorization code
        let code_hash = hash_oauth_token(code);
        let auth_code = self.code_repo
            .find_valid_by_code_hash(&code_hash)
            .await?
            .ok_or_else(|| OAuthError::InvalidGrant("Invalid or expired authorization code".to_string()))?;

        // Verify the code belongs to this client
        if auth_code.client_id != client.id {
            return Err(OAuthError::InvalidGrant("Authorization code was not issued to this client".to_string()));
        }

        // Verify redirect_uri matches
        if auth_code.redirect_uri != redirect_uri {
            return Err(OAuthError::InvalidGrant("redirect_uri does not match".to_string()));
        }

        // Verify PKCE code_verifier
        // Requirements: 3.5, 3.6
        if !validate_code_verifier(code_verifier) {
            return Err(OAuthError::InvalidGrant("Invalid code_verifier format".to_string()));
        }

        if !verify_pkce(code_verifier, &auth_code.code_challenge, &auth_code.code_challenge_method) {
            return Err(OAuthError::InvalidGrant("code_verifier does not match code_challenge".to_string()));
        }

        // Mark the code as used
        self.code_repo.mark_as_used(auth_code.id).await?;

        // Issue tokens
        let token_response = self.issue_tokens(
            Some(auth_code.user_id),
            client.id,
            &client.client_id,
            &auth_code.scopes,
        ).await?;

        // Log the event
        self.audit_repo
            .create(
                OAuthEventType::TokenIssued,
                Some(client.id),
                Some(auth_code.user_id),
                None,
                Some(serde_json::json!({
                    "scopes": auth_code.scopes,
                    "grant_type": "authorization_code",
                })),
            )
            .await
            .ok();

        Ok(token_response)
    }


    // ========================================================================
    // Client Credentials Flow (Task 8.7)
    // Requirements: 6.1, 6.2, 6.5
    // ========================================================================

    /// Client credentials grant for internal apps
    ///
    /// # Arguments
    /// * `client_id` - The client's public identifier
    /// * `client_secret` - The client's secret
    /// * `scopes` - The requested scopes
    ///
    /// # Returns
    /// * `Ok(OAuthTokenResponse)` - The access token (no refresh token for client credentials)
    /// * `Err(OAuthError)` - If authentication fails
    ///
    /// # Requirements
    /// - 6.1: Authenticate using client_id and client_secret
    /// - 6.2: Do not require PKCE for Internal_App using client credentials flow
    /// - 6.3: Do not require user consent for Internal_App
    /// - 6.4: Reject with invalid_client error if credentials are invalid
    /// - 6.5: Issue service-scoped tokens for Internal_App
    pub async fn client_credentials_grant(
        &self,
        client_id: &str,
        client_secret: &str,
        scopes: &[String],
    ) -> Result<OAuthTokenResponse, OAuthError> {
        // Find the client
        let client = self.client_repo
            .find_active_by_client_id(client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;

        // Verify client secret
        let valid = verify_secret(client_secret, &client.client_secret_hash)
            .map_err(|_| OAuthError::InvalidClient)?;
        if !valid {
            // Log failed attempt
            self.audit_repo
                .create(
                    OAuthEventType::InvalidClientCredentials,
                    Some(client.id),
                    None,
                    None,
                    None,
                )
                .await
                .ok();
            return Err(OAuthError::InvalidClient);
        }

        // Validate scopes if provided
        if !scopes.is_empty() {
            let valid = self.scope_repo.validate_scopes(scopes).await?;
            if !valid {
                return Err(OAuthError::InvalidScope(
                    "One or more requested scopes are invalid".to_string(),
                ));
            }
        }

        // Issue access token only (no refresh token for client credentials)
        // Requirements: 6.5
        let access_token = self.jwt_manager
            .create_oauth2_client_credentials_token(&client.client_id, scopes.to_vec())
            .map_err(|e| OAuthError::ServerError(format!("Failed to create token: {}", e)))?;

        let access_token_hash = hash_oauth_token(&access_token);

        // Store the token
        self.token_repo
            .create(
                None, // No user for client credentials
                client.id,
                &access_token_hash,
                None, // No refresh token
                scopes,
                self.jwt_manager.access_token_expiry_secs(),
            )
            .await?;

        // Log the event
        self.audit_repo
            .create(
                OAuthEventType::TokenIssued,
                Some(client.id),
                None,
                None,
                Some(serde_json::json!({
                    "scopes": scopes,
                    "grant_type": "client_credentials",
                })),
            )
            .await
            .ok();

        Ok(OAuthTokenResponse::new(
            access_token,
            None, // No refresh token for client credentials
            self.jwt_manager.access_token_expiry_secs(),
            scopes,
        ))
    }

    // ========================================================================
    // Token Refresh (Task 8.9)
    // Requirements: 7.1, 7.2, 7.4
    // ========================================================================

    /// Refresh an access token using a refresh token
    ///
    /// # Arguments
    /// * `refresh_token` - The refresh token
    /// * `client_id` - The client's public identifier
    ///
    /// # Returns
    /// * `Ok(OAuthTokenResponse)` - New access and refresh tokens
    /// * `Err(OAuthError)` - If refresh fails
    ///
    /// # Requirements
    /// - 7.1: Issue a new access_token when valid refresh_token is provided
    /// - 7.2: Implement refresh token rotation (issue new refresh_token on each refresh)
    /// - 7.3: Reject with invalid_grant error if refresh_token is expired or invalid
    /// - 7.4: Invalidate the old refresh_token after rotation
    /// - 7.5: If a revoked refresh_token is used, revoke all tokens for that client-user pair
    pub async fn refresh_token(
        &self,
        refresh_token: &str,
        client_id: &str,
    ) -> Result<OAuthTokenResponse, OAuthError> {
        // Find the client
        let client = self.client_repo
            .find_active_by_client_id(client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;

        // Find the token by refresh token hash
        let refresh_token_hash = hash_oauth_token(refresh_token);
        let token = self.token_repo
            .find_by_refresh_token_hash(&refresh_token_hash)
            .await?
            .ok_or_else(|| OAuthError::InvalidGrant("Invalid refresh token".to_string()))?;

        // Check if token belongs to this client
        if token.client_id != client.id {
            return Err(OAuthError::InvalidGrant("Refresh token was not issued to this client".to_string()));
        }

        // Check if token is revoked
        // Requirement 7.5: If revoked token is used, revoke all tokens for client-user pair
        if token.revoked {
            if let Some(user_id) = token.user_id {
                self.token_repo.revoke_all_for_user_client(user_id, client.id).await?;
                
                // Log the cascade revocation
                self.audit_repo
                    .create(
                        OAuthEventType::TokenRevoked,
                        Some(client.id),
                        Some(user_id),
                        None,
                        Some(serde_json::json!({
                            "reason": "revoked_refresh_token_reuse",
                            "cascade": true,
                        })),
                    )
                    .await
                    .ok();
            }
            return Err(OAuthError::InvalidGrant("Refresh token has been revoked".to_string()));
        }

        // Revoke the old token (rotation)
        // Requirement 7.4
        self.token_repo.revoke(token.id).await?;

        // Issue new tokens
        let token_response = self.issue_tokens(
            token.user_id,
            client.id,
            &client.client_id,
            &token.scopes,
        ).await?;

        // Log the event
        self.audit_repo
            .create(
                OAuthEventType::TokenRefreshed,
                Some(client.id),
                token.user_id,
                None,
                Some(serde_json::json!({
                    "scopes": token.scopes,
                })),
            )
            .await
            .ok();

        Ok(token_response)
    }


    // ========================================================================
    // Token Revocation (Task 8.11)
    // Requirements: 9.2, 9.4
    // ========================================================================

    /// Revoke a specific token
    ///
    /// # Arguments
    /// * `token` - The token to revoke (access or refresh token)
    /// * `client_id` - The client's public identifier
    ///
    /// # Returns
    /// * `Ok(())` - Token revoked successfully
    /// * `Err(OAuthError)` - If revocation fails
    ///
    /// # Requirements
    /// - 9.4: Invalidate the specific token when app calls revoke endpoint
    pub async fn revoke_token(
        &self,
        token: &str,
        client_id: &str,
    ) -> Result<(), OAuthError> {
        // Find the client
        let client = self.client_repo
            .find_active_by_client_id(client_id)
            .await?
            .ok_or(OAuthError::InvalidClient)?;

        let token_hash = hash_oauth_token(token);

        // Try to find by access token hash first
        if let Some(oauth_token) = self.token_repo.find_by_access_token_hash(&token_hash).await? {
            // Verify token belongs to this client
            if oauth_token.client_id != client.id {
                return Err(OAuthError::InvalidClient);
            }

            self.token_repo.revoke(oauth_token.id).await?;

            // Log the event
            self.audit_repo
                .create(
                    OAuthEventType::TokenRevoked,
                    Some(client.id),
                    oauth_token.user_id,
                    None,
                    Some(serde_json::json!({
                        "token_type": "access_token",
                    })),
                )
                .await
                .ok();

            return Ok(());
        }

        // Try to find by refresh token hash
        if let Some(oauth_token) = self.token_repo.find_by_refresh_token_hash(&token_hash).await? {
            // Verify token belongs to this client
            if oauth_token.client_id != client.id {
                return Err(OAuthError::InvalidClient);
            }

            self.token_repo.revoke(oauth_token.id).await?;

            // Log the event
            self.audit_repo
                .create(
                    OAuthEventType::TokenRevoked,
                    Some(client.id),
                    oauth_token.user_id,
                    None,
                    Some(serde_json::json!({
                        "token_type": "refresh_token",
                    })),
                )
                .await
                .ok();

            return Ok(());
        }

        // Token not found - per RFC 7009, this is not an error
        // The revocation endpoint should return success even if token is invalid
        Ok(())
    }

    /// Revoke all tokens for a user-client pair
    ///
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `client_id` - The client's UUID
    ///
    /// # Returns
    /// * `Ok(u64)` - Number of tokens revoked
    /// * `Err(OAuthError)` - If revocation fails
    ///
    /// # Requirements
    /// - 9.2: Invalidate all tokens for that client-user pair when user revokes access
    pub async fn revoke_all_tokens_for_user_client(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<u64, OAuthError> {
        let count = self.token_repo.revoke_all_for_user_client(user_id, client_id).await?;

        // Log the event
        self.audit_repo
            .create(
                OAuthEventType::TokenRevoked,
                Some(client_id),
                Some(user_id),
                None,
                Some(serde_json::json!({
                    "revoked_count": count,
                    "reason": "user_revocation",
                })),
            )
            .await
            .ok();

        Ok(count)
    }

    // ========================================================================
    // Helper Methods
    // ========================================================================

    /// Issue access and refresh tokens
    ///
    /// # Requirements
    /// - 5.1: Issue access_token and refresh_token
    /// - 5.2: Set access_token expiration to max 15 minutes
    /// - 5.3: Include granted scopes in the token response
    /// - 5.4: Include sub, aud, scope, and exp claims in JWT
    /// - 5.5: Sign JWT tokens using RS256 algorithm
    /// - 5.6: Hash tokens before storing in the database
    async fn issue_tokens(
        &self,
        user_id: Option<Uuid>,
        client_uuid: Uuid,
        client_id: &str,
        scopes: &[String],
    ) -> Result<OAuthTokenResponse, OAuthError> {
        // Generate access token
        let access_token = if let Some(uid) = user_id {
            self.jwt_manager
                .create_oauth2_token(uid, client_id, scopes.to_vec())
                .map_err(|e| OAuthError::ServerError(format!("Failed to create access token: {}", e)))?
        } else {
            self.jwt_manager
                .create_oauth2_client_credentials_token(client_id, scopes.to_vec())
                .map_err(|e| OAuthError::ServerError(format!("Failed to create access token: {}", e)))?
        };

        // Generate refresh token (opaque token, not JWT)
        let refresh_token = generate_oauth_token();

        // Hash tokens for storage
        let access_token_hash = hash_oauth_token(&access_token);
        let refresh_token_hash = hash_oauth_token(&refresh_token);

        // Store the token
        self.token_repo
            .create(
                user_id,
                client_uuid,
                &access_token_hash,
                Some(&refresh_token_hash),
                scopes,
                self.jwt_manager.access_token_expiry_secs(),
            )
            .await?;

        Ok(OAuthTokenResponse::new(
            access_token,
            Some(refresh_token),
            self.jwt_manager.access_token_expiry_secs(),
            scopes,
        ))
    }

    /// Get the consent service for checking/granting consent
    pub fn consent_service(&self) -> &ConsentService {
        &self.consent_service
    }

    /// Get the client repository for client operations
    pub fn client_repo(&self) -> &OAuthClientRepository {
        &self.client_repo
    }

    /// Get the scope repository for scope operations
    pub fn scope_repo(&self) -> &OAuthScopeRepository {
        &self.scope_repo
    }

    /// Validate that all requested scopes exist and are active
    ///
    /// # Arguments
    /// * `scopes` - The scopes to validate
    ///
    /// # Returns
    /// * `Ok(())` - All scopes are valid
    /// * `Err(OAuthError::InvalidScope)` - One or more scopes are invalid
    ///
    /// # Requirements
    /// - 2.4: Verify all requested scopes exist and are valid
    pub async fn validate_scopes(&self, scopes: &[String]) -> Result<(), OAuthError> {
        if scopes.is_empty() {
            return Ok(());
        }

        let valid = self.scope_repo.validate_scopes(scopes).await?;
        if !valid {
            return Err(OAuthError::InvalidScope(
                "One or more requested scopes are invalid".to_string(),
            ));
        }

        Ok(())
    }
}

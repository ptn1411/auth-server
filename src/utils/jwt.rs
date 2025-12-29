use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::error::AuthError;

/// Claims for each app in the user JWT token (roles/permissions per app)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AppClaims {
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

/// JWT Claims for App authentication tokens (machine-to-machine)
/// 
/// # Requirements
/// - 3.2: Return access token with app context
/// - 7.4: Include app_id in the token claims for app-authenticated requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppTokenClaims {
    /// Subject - App ID as string
    pub sub: String,
    /// App ID (for clarity)
    pub app_id: Uuid,
    /// Token type - always "app" to distinguish from user tokens
    pub token_type: String,
    /// Expiration timestamp (Unix timestamp)
    pub exp: i64,
    /// Issued at timestamp (Unix timestamp)
    pub iat: i64,
}

impl AppTokenClaims {
    /// Create new claims for an app token
    pub fn new(app_id: Uuid, expiry_secs: i64) -> Self {
        let now = Utc::now();
        Self {
            sub: app_id.to_string(),
            app_id,
            token_type: "app".to_string(),
            exp: (now + Duration::seconds(expiry_secs)).timestamp(),
            iat: now.timestamp(),
        }
    }

    /// Get the app_id from claims
    pub fn get_app_id(&self) -> Uuid {
        self.app_id
    }

    /// Check if this is an app token
    pub fn is_app_token(&self) -> bool {
        self.token_type == "app"
    }
}

/// JWT Claims structure
/// 
/// # Requirements
/// - 10.1: JWT tokens with payload containing: sub (user_id), apps (object with app codes as keys), and exp
/// - 10.2: Include roles array and permissions array for each app in the token payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject - user_id
    pub sub: String,
    /// Apps with their roles and permissions (app_code -> AppClaims)
    pub apps: HashMap<String, AppClaims>,
    /// Expiration timestamp (Unix timestamp)
    pub exp: i64,
    /// Issued at timestamp (Unix timestamp)
    pub iat: i64,
}

impl Claims {
    /// Create new claims for a user
    pub fn new(user_id: Uuid, apps: HashMap<String, AppClaims>, expiry_secs: i64) -> Self {
        let now = Utc::now();
        Self {
            sub: user_id.to_string(),
            apps,
            exp: (now + Duration::seconds(expiry_secs)).timestamp(),
            iat: now.timestamp(),
        }
    }

    /// Get the user_id from claims
    pub fn user_id(&self) -> Result<Uuid, AuthError> {
        Uuid::parse_str(&self.sub)
            .map_err(|_| AuthError::InvalidToken)
    }
}

/// Token pair returned on login/refresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

impl TokenPair {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in,
        }
    }
}

/// JWT token manager for creating and verifying tokens
/// 
/// # Requirements
/// - 10.3: Sign all tokens using RS256 algorithm
/// - 10.4: Use public/private key pairs, not shared secrets
#[derive(Clone)]
pub struct JwtManager {
    encoding_key: Arc<EncodingKey>,
    decoding_key: Arc<DecodingKey>,
    access_token_expiry_secs: i64,
    refresh_token_expiry_secs: i64,
}

impl JwtManager {
    /// Create a new JWT manager with RSA keys
    /// 
    /// # Arguments
    /// * `private_key_pem` - RSA private key in PEM format (supports both PKCS#1 and PKCS#8)
    /// * `public_key_pem` - RSA public key in PEM format
    /// * `access_token_expiry_secs` - Access token expiry in seconds (default: 900 = 15 minutes)
    /// * `refresh_token_expiry_secs` - Refresh token expiry in seconds (default: 604800 = 7 days)
    pub fn new(
        private_key_pem: &str,
        public_key_pem: &str,
        access_token_expiry_secs: i64,
        refresh_token_expiry_secs: i64,
    ) -> Result<Self, AuthError> {
        // Try PKCS#8 format first (BEGIN PRIVATE KEY), then fall back to PKCS#1 (BEGIN RSA PRIVATE KEY)
        let encoding_key = if private_key_pem.contains("BEGIN PRIVATE KEY") {
            EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
                .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Invalid private key: {}", e)))?
        } else {
            EncodingKey::from_rsa_pem(private_key_pem.as_bytes())
                .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Invalid private key: {}", e)))?
        };
        
        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem.as_bytes())
            .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Invalid public key: {}", e)))?;
        
        Ok(Self {
            encoding_key: Arc::new(encoding_key),
            decoding_key: Arc::new(decoding_key),
            access_token_expiry_secs,
            refresh_token_expiry_secs,
        })
    }

    /// Create an access token for a user
    /// 
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `apps` - Map of app codes to their roles and permissions
    /// 
    /// # Returns
    /// * `Ok(String)` - The JWT access token
    /// * `Err(AuthError)` - If token creation fails
    /// 
    /// # Requirements
    /// - 2.4: Generate JWT tokens using RS256 algorithm
    /// - 2.5: Set access token expiry to 15 minutes
    pub fn create_access_token(
        &self,
        user_id: Uuid,
        apps: HashMap<String, AppClaims>,
    ) -> Result<String, AuthError> {
        let claims = Claims::new(user_id, apps, self.access_token_expiry_secs);
        
        let header = Header::new(Algorithm::RS256);
        
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Token encoding failed: {}", e)))
    }

    /// Create a refresh token for a user
    /// 
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// 
    /// # Returns
    /// * `Ok(String)` - The JWT refresh token
    /// * `Err(AuthError)` - If token creation fails
    pub fn create_refresh_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        // Refresh tokens have minimal claims - just user_id
        let claims = Claims::new(user_id, HashMap::new(), self.refresh_token_expiry_secs);
        
        let header = Header::new(Algorithm::RS256);
        
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Token encoding failed: {}", e)))
    }

    /// Create a token pair (access + refresh tokens)
    /// 
    /// # Arguments
    /// * `user_id` - The user's UUID
    /// * `apps` - Map of app codes to their roles and permissions
    /// 
    /// # Returns
    /// * `Ok(TokenPair)` - The access and refresh tokens
    /// * `Err(AuthError)` - If token creation fails
    pub fn create_token_pair(
        &self,
        user_id: Uuid,
        apps: HashMap<String, AppClaims>,
    ) -> Result<TokenPair, AuthError> {
        let access_token = self.create_access_token(user_id, apps)?;
        let refresh_token = self.create_refresh_token(user_id)?;
        
        Ok(TokenPair::new(
            access_token,
            refresh_token,
            self.access_token_expiry_secs,
        ))
    }

    /// Verify and decode a JWT token
    /// 
    /// # Arguments
    /// * `token` - The JWT token to verify
    /// 
    /// # Returns
    /// * `Ok(Claims)` - The decoded claims if valid
    /// * `Err(AuthError::TokenExpired)` - If the token has expired
    /// * `Err(AuthError::InvalidToken)` - If the token is invalid
    /// 
    /// # Requirements
    /// - 11.1: Verify token signature (RS256)
    /// - 11.2: Check expiration
    pub fn verify_token(&self, token: &str) -> Result<Claims, AuthError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        
        decode::<Claims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidToken,
                }
            })
    }

    /// Create an access token for an App (machine-to-machine authentication)
    /// 
    /// # Arguments
    /// * `app_id` - The App's UUID
    /// 
    /// # Returns
    /// * `Ok(String)` - The JWT access token for the app
    /// * `Err(AuthError)` - If token creation fails
    /// 
    /// # Requirements
    /// - 3.1: Authenticate app and return access token
    /// - 3.2: Return access token with app context
    pub fn create_app_token(&self, app_id: Uuid) -> Result<String, AuthError> {
        let claims = AppTokenClaims::new(app_id, self.access_token_expiry_secs);
        
        let header = Header::new(Algorithm::RS256);
        
        encode(&header, &claims, &self.encoding_key)
            .map_err(|e| AuthError::InternalError(anyhow::anyhow!("App token encoding failed: {}", e)))
    }

    /// Verify and decode an App JWT token
    /// 
    /// # Arguments
    /// * `token` - The JWT token to verify
    /// 
    /// # Returns
    /// * `Ok(AppTokenClaims)` - The decoded app claims if valid
    /// * `Err(AuthError::TokenExpired)` - If the token has expired
    /// * `Err(AuthError::InvalidToken)` - If the token is invalid or not an app token
    /// 
    /// # Requirements
    /// - 3.1: Verify app authentication
    /// - 7.4: Extract app_id from token claims
    pub fn verify_app_token(&self, token: &str) -> Result<AppTokenClaims, AuthError> {
        let mut validation = Validation::new(Algorithm::RS256);
        validation.validate_exp = true;
        
        let claims = decode::<AppTokenClaims>(token, &self.decoding_key, &validation)
            .map(|data| data.claims)
            .map_err(|e| {
                match e.kind() {
                    jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    _ => AuthError::InvalidToken,
                }
            })?;
        
        // Verify this is actually an app token
        if !claims.is_app_token() {
            return Err(AuthError::InvalidToken);
        }
        
        Ok(claims)
    }

    /// Get the access token expiry duration in seconds
    pub fn access_token_expiry_secs(&self) -> i64 {
        self.access_token_expiry_secs
    }

    /// Get the refresh token expiry duration in seconds
    pub fn refresh_token_expiry_secs(&self) -> i64 {
        self.refresh_token_expiry_secs
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_keys() -> (String, String) {
        // Use the keys from the keys directory
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

    #[test]
    fn test_create_access_token() {
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let apps = HashMap::new();
        
        let token = manager.create_access_token(user_id, apps).unwrap();
        
        assert!(!token.is_empty());
        // JWT has 3 parts separated by dots
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_create_refresh_token() {
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let token = manager.create_refresh_token(user_id).unwrap();
        
        assert!(!token.is_empty());
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_create_token_pair() {
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let apps = HashMap::new();
        
        let pair = manager.create_token_pair(user_id, apps).unwrap();
        
        assert!(!pair.access_token.is_empty());
        assert!(!pair.refresh_token.is_empty());
        assert_eq!(pair.token_type, "Bearer");
        assert_eq!(pair.expires_in, 900);
    }

    #[test]
    fn test_verify_valid_token() {
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let mut apps = HashMap::new();
        apps.insert(
            "test_app".to_string(),
            AppClaims {
                roles: vec!["admin".to_string()],
                permissions: vec!["read".to_string(), "write".to_string()],
            },
        );
        
        let token = manager.create_access_token(user_id, apps.clone()).unwrap();
        let claims = manager.verify_token(&token).unwrap();
        
        assert_eq!(claims.sub, user_id.to_string());
        assert_eq!(claims.apps.get("test_app").unwrap().roles, vec!["admin"]);
        assert_eq!(
            claims.apps.get("test_app").unwrap().permissions,
            vec!["read", "write"]
        );
    }

    #[test]
    fn test_verify_invalid_token() {
        let manager = create_test_jwt_manager();
        
        let result = manager.verify_token("invalid.token.here");
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_verify_malformed_token() {
        let manager = create_test_jwt_manager();
        
        let result = manager.verify_token("not-a-jwt");
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_token_contains_correct_structure() {
        // Property 8: JWT Token Structure Correctness
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let mut apps = HashMap::new();
        apps.insert(
            "app1".to_string(),
            AppClaims {
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
            },
        );
        
        let token = manager.create_access_token(user_id, apps).unwrap();
        let claims = manager.verify_token(&token).unwrap();
        
        // Verify structure
        assert!(!claims.sub.is_empty()); // sub field with user_id
        assert!(claims.apps.contains_key("app1")); // apps object with app codes as keys
        assert!(claims.exp > 0); // exp field with expiration timestamp
        assert!(claims.iat > 0); // iat field
        
        // Each app entry has roles and permissions arrays
        let app_claims = claims.apps.get("app1").unwrap();
        assert!(!app_claims.roles.is_empty());
        assert!(!app_claims.permissions.is_empty());
    }

    #[test]
    fn test_token_expiry_duration() {
        // Property 9: Token Expiry Duration
        // Access token exp should be approximately 15 minutes (900 seconds) from iat
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let apps = HashMap::new();
        
        let token = manager.create_access_token(user_id, apps).unwrap();
        let claims = manager.verify_token(&token).unwrap();
        
        let duration = claims.exp - claims.iat;
        
        // Should be exactly 900 seconds (15 minutes)
        assert_eq!(duration, 900);
    }

    #[test]
    fn test_claims_user_id_extraction() {
        let user_id = Uuid::new_v4();
        let claims = Claims::new(user_id, HashMap::new(), 900);
        
        let extracted_id = claims.user_id().unwrap();
        
        assert_eq!(extracted_id, user_id);
    }

    #[test]
    fn test_multiple_apps_in_token() {
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let mut apps = HashMap::new();
        apps.insert(
            "app1".to_string(),
            AppClaims {
                roles: vec!["admin".to_string()],
                permissions: vec!["all".to_string()],
            },
        );
        apps.insert(
            "app2".to_string(),
            AppClaims {
                roles: vec!["user".to_string()],
                permissions: vec!["read".to_string()],
            },
        );
        
        let token = manager.create_access_token(user_id, apps).unwrap();
        let claims = manager.verify_token(&token).unwrap();
        
        assert_eq!(claims.apps.len(), 2);
        assert!(claims.apps.contains_key("app1"));
        assert!(claims.apps.contains_key("app2"));
    }

    #[test]
    fn test_refresh_token_has_empty_apps() {
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        
        let token = manager.create_refresh_token(user_id).unwrap();
        let claims = manager.verify_token(&token).unwrap();
        
        // Refresh tokens should have empty apps
        assert!(claims.apps.is_empty());
    }

    // ============================================
    // App Token Tests (Machine-to-Machine Auth)
    // ============================================

    #[test]
    fn test_create_app_token() {
        let manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        let token = manager.create_app_token(app_id).unwrap();
        
        assert!(!token.is_empty());
        // JWT has 3 parts separated by dots
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_verify_app_token() {
        let manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        let token = manager.create_app_token(app_id).unwrap();
        let claims = manager.verify_app_token(&token).unwrap();
        
        assert_eq!(claims.app_id, app_id);
        assert_eq!(claims.sub, app_id.to_string());
        assert_eq!(claims.token_type, "app");
    }

    #[test]
    fn test_app_token_contains_app_context() {
        // Property 7: App Token Contains App Context
        // For any token issued via POST /apps/auth, decoding the token SHALL reveal 
        // app_id in the claims and token_type as "app"
        let manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        let token = manager.create_app_token(app_id).unwrap();
        let claims = manager.verify_app_token(&token).unwrap();
        
        // Verify app_id is in claims
        assert_eq!(claims.app_id, app_id);
        assert_eq!(claims.get_app_id(), app_id);
        
        // Verify token_type is "app"
        assert_eq!(claims.token_type, "app");
        assert!(claims.is_app_token());
        
        // Verify sub matches app_id
        assert_eq!(claims.sub, app_id.to_string());
    }

    #[test]
    fn test_app_token_expiry_duration() {
        let manager = create_test_jwt_manager();
        let app_id = Uuid::new_v4();
        
        let token = manager.create_app_token(app_id).unwrap();
        let claims = manager.verify_app_token(&token).unwrap();
        
        let duration = claims.exp - claims.iat;
        
        // Should be exactly 900 seconds (15 minutes) - same as access token
        assert_eq!(duration, 900);
    }

    #[test]
    fn test_verify_invalid_app_token() {
        let manager = create_test_jwt_manager();
        
        let result = manager.verify_app_token("invalid.token.here");
        
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_verify_user_token_as_app_token_fails() {
        // User tokens should not be accepted as app tokens
        let manager = create_test_jwt_manager();
        let user_id = Uuid::new_v4();
        let apps = HashMap::new();
        
        let user_token = manager.create_access_token(user_id, apps).unwrap();
        let result = manager.verify_app_token(&user_token);
        
        // Should fail because user token doesn't have token_type: "app"
        assert!(matches!(result, Err(AuthError::InvalidToken)));
    }

    #[test]
    fn test_app_token_claims_new() {
        let app_id = Uuid::new_v4();
        let expiry_secs = 3600;
        
        let claims = AppTokenClaims::new(app_id, expiry_secs);
        
        assert_eq!(claims.app_id, app_id);
        assert_eq!(claims.sub, app_id.to_string());
        assert_eq!(claims.token_type, "app");
        assert!(claims.exp > claims.iat);
        assert_eq!(claims.exp - claims.iat, expiry_secs);
    }

    #[test]
    fn test_app_token_claims_is_app_token() {
        let app_id = Uuid::new_v4();
        let claims = AppTokenClaims::new(app_id, 900);
        
        assert!(claims.is_app_token());
    }
}

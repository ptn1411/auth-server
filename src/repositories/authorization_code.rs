use chrono::{DateTime, Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::AuthorizationCode;

/// Repository for authorization code database operations
/// Requirements: 3.4
#[derive(Clone)]
pub struct AuthorizationCodeRepository {
    pool: MySqlPool,
}

impl AuthorizationCodeRepository {
    /// Create a new AuthorizationCodeRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new authorization code
    /// Requirements: 3.4 - Generate short-lived authorization code (max 10 minutes)
    pub async fn create(
        &self,
        code_hash: &str,
        client_id: Uuid,
        user_id: Uuid,
        redirect_uri: &str,
        scopes: &[String],
        code_challenge: &str,
        code_challenge_method: &str,
        expires_in_seconds: i64,
    ) -> Result<AuthorizationCode, OAuthError> {
        // Enforce max 10 minutes expiration
        let max_expiration = 600; // 10 minutes in seconds
        let actual_expiration = expires_in_seconds.min(max_expiration);
        
        let id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::seconds(actual_expiration);
        let scopes_json = serde_json::to_value(scopes)
            .map_err(|e| OAuthError::ServerError(format!("Failed to serialize scopes: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO oauth_authorization_codes 
            (id, code_hash, client_id, user_id, redirect_uri, scopes, code_challenge, code_challenge_method, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(code_hash)
        .bind(client_id.to_string())
        .bind(user_id.to_string())
        .bind(redirect_uri)
        .bind(&scopes_json)
        .bind(code_challenge)
        .bind(code_challenge_method)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| OAuthError::ServerError("Failed to fetch created authorization code".to_string()))
    }

    /// Find an authorization code by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AuthorizationCode>, OAuthError> {
        let code = sqlx::query_as::<_, AuthorizationCode>(
            r#"
            SELECT id, code_hash, client_id, user_id, redirect_uri, scopes, 
                   code_challenge, code_challenge_method, expires_at, used, created_at
            FROM oauth_authorization_codes
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(code)
    }

    /// Find an authorization code by its hash
    pub async fn find_by_code_hash(&self, code_hash: &str) -> Result<Option<AuthorizationCode>, OAuthError> {
        let code = sqlx::query_as::<_, AuthorizationCode>(
            r#"
            SELECT id, code_hash, client_id, user_id, redirect_uri, scopes, 
                   code_challenge, code_challenge_method, expires_at, used, created_at
            FROM oauth_authorization_codes
            WHERE code_hash = ?
            "#,
        )
        .bind(code_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(code)
    }

    /// Find a valid (not used, not expired) authorization code by its hash
    pub async fn find_valid_by_code_hash(&self, code_hash: &str) -> Result<Option<AuthorizationCode>, OAuthError> {
        let code = sqlx::query_as::<_, AuthorizationCode>(
            r#"
            SELECT id, code_hash, client_id, user_id, redirect_uri, scopes, 
                   code_challenge, code_challenge_method, expires_at, used, created_at
            FROM oauth_authorization_codes
            WHERE code_hash = ? AND used = false AND expires_at > NOW()
            "#,
        )
        .bind(code_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(code)
    }

    /// Mark an authorization code as used
    /// Requirements: 3.4 - Authorization codes are single-use
    pub async fn mark_as_used(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_authorization_codes
            SET used = true
            WHERE id = ? AND used = false
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidGrant("Authorization code already used or not found".to_string()));
        }

        Ok(())
    }

    /// Delete an authorization code
    pub async fn delete(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_authorization_codes
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidGrant("Authorization code not found".to_string()));
        }

        Ok(())
    }

    /// Delete expired authorization codes (cleanup)
    pub async fn delete_expired(&self) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_authorization_codes
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Delete used authorization codes (cleanup)
    pub async fn delete_used(&self) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_authorization_codes
            WHERE used = true
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Delete all authorization codes for a user-client pair
    pub async fn delete_for_user_client(&self, user_id: Uuid, client_id: Uuid) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_authorization_codes
            WHERE user_id = ? AND client_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(client_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Get expiration time for a code
    pub async fn get_expiration(&self, id: Uuid) -> Result<Option<DateTime<Utc>>, OAuthError> {
        let expires_at = sqlx::query_scalar::<_, DateTime<Utc>>(
            r#"
            SELECT expires_at
            FROM oauth_authorization_codes
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(expires_at)
    }
}

use chrono::{DateTime, Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::OAuthToken;

/// Repository for OAuth token database operations
/// Requirements: 5.1, 5.6, 7.4, 9.2
#[derive(Clone)]
pub struct OAuthTokenRepository {
    pool: MySqlPool,
}

impl OAuthTokenRepository {
    /// Create a new OAuthTokenRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new OAuth token
    /// Requirements: 5.1, 5.6
    pub async fn create(
        &self,
        user_id: Option<Uuid>,
        client_id: Uuid,
        access_token_hash: &str,
        refresh_token_hash: Option<&str>,
        scopes: &[String],
        expires_in_seconds: i64,
    ) -> Result<OAuthToken, OAuthError> {
        let id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::seconds(expires_in_seconds);
        let scopes_json = serde_json::to_value(scopes)
            .map_err(|e| OAuthError::ServerError(format!("Failed to serialize scopes: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO oauth_tokens 
            (id, user_id, client_id, access_token_hash, refresh_token_hash, scopes, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.map(|u| u.to_string()))
        .bind(client_id.to_string())
        .bind(access_token_hash)
        .bind(refresh_token_hash)
        .bind(&scopes_json)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| OAuthError::ServerError("Failed to fetch created token".to_string()))
    }

    /// Find a token by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OAuthToken>, OAuthError> {
        let token = sqlx::query_as::<_, OAuthToken>(
            r#"
            SELECT id, user_id, client_id, access_token_hash, refresh_token_hash, 
                   scopes, expires_at, revoked, created_at
            FROM oauth_tokens
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(token)
    }

    /// Find a token by access token hash
    pub async fn find_by_access_token_hash(&self, access_token_hash: &str) -> Result<Option<OAuthToken>, OAuthError> {
        let token = sqlx::query_as::<_, OAuthToken>(
            r#"
            SELECT id, user_id, client_id, access_token_hash, refresh_token_hash, 
                   scopes, expires_at, revoked, created_at
            FROM oauth_tokens
            WHERE access_token_hash = ?
            "#,
        )
        .bind(access_token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(token)
    }

    /// Find a valid (not revoked, not expired) token by access token hash
    pub async fn find_valid_by_access_token_hash(&self, access_token_hash: &str) -> Result<Option<OAuthToken>, OAuthError> {
        let token = sqlx::query_as::<_, OAuthToken>(
            r#"
            SELECT id, user_id, client_id, access_token_hash, refresh_token_hash, 
                   scopes, expires_at, revoked, created_at
            FROM oauth_tokens
            WHERE access_token_hash = ? AND revoked = false AND expires_at > NOW()
            "#,
        )
        .bind(access_token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(token)
    }

    /// Find a token by refresh token hash
    pub async fn find_by_refresh_token_hash(&self, refresh_token_hash: &str) -> Result<Option<OAuthToken>, OAuthError> {
        let token = sqlx::query_as::<_, OAuthToken>(
            r#"
            SELECT id, user_id, client_id, access_token_hash, refresh_token_hash, 
                   scopes, expires_at, revoked, created_at
            FROM oauth_tokens
            WHERE refresh_token_hash = ?
            "#,
        )
        .bind(refresh_token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(token)
    }

    /// Find a valid (not revoked) token by refresh token hash
    /// Note: Refresh tokens don't have their own expiration, they're valid until revoked
    pub async fn find_valid_by_refresh_token_hash(&self, refresh_token_hash: &str) -> Result<Option<OAuthToken>, OAuthError> {
        let token = sqlx::query_as::<_, OAuthToken>(
            r#"
            SELECT id, user_id, client_id, access_token_hash, refresh_token_hash, 
                   scopes, expires_at, revoked, created_at
            FROM oauth_tokens
            WHERE refresh_token_hash = ? AND revoked = false
            "#,
        )
        .bind(refresh_token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(token)
    }

    /// Revoke a specific token
    /// Requirements: 9.4
    pub async fn revoke(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_tokens
            SET revoked = true
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidGrant("Token not found".to_string()));
        }

        Ok(())
    }

    /// Revoke a token by access token hash
    pub async fn revoke_by_access_token_hash(&self, access_token_hash: &str) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_tokens
            SET revoked = true
            WHERE access_token_hash = ?
            "#,
        )
        .bind(access_token_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidGrant("Token not found".to_string()));
        }

        Ok(())
    }

    /// Revoke a token by refresh token hash
    /// Requirements: 7.4
    pub async fn revoke_by_refresh_token_hash(&self, refresh_token_hash: &str) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_tokens
            SET revoked = true
            WHERE refresh_token_hash = ?
            "#,
        )
        .bind(refresh_token_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidGrant("Token not found".to_string()));
        }

        Ok(())
    }

    /// Revoke all tokens for a user-client pair
    /// Requirements: 9.2
    pub async fn revoke_all_for_user_client(&self, user_id: Uuid, client_id: Uuid) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_tokens
            SET revoked = true
            WHERE user_id = ? AND client_id = ? AND revoked = false
            "#,
        )
        .bind(user_id.to_string())
        .bind(client_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Revoke all tokens for a client (service tokens)
    pub async fn revoke_all_for_client(&self, client_id: Uuid) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_tokens
            SET revoked = true
            WHERE client_id = ? AND user_id IS NULL AND revoked = false
            "#,
        )
        .bind(client_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Revoke all tokens for a user
    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_tokens
            SET revoked = true
            WHERE user_id = ? AND revoked = false
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Check if a token is revoked
    pub async fn is_revoked(&self, id: Uuid) -> Result<bool, OAuthError> {
        let revoked = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT revoked
            FROM oauth_tokens
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(revoked.unwrap_or(true))
    }

    /// List all tokens for a user
    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<OAuthToken>, OAuthError> {
        let tokens = sqlx::query_as::<_, OAuthToken>(
            r#"
            SELECT id, user_id, client_id, access_token_hash, refresh_token_hash, 
                   scopes, expires_at, revoked, created_at
            FROM oauth_tokens
            WHERE user_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(tokens)
    }

    /// List active (not revoked) tokens for a user-client pair
    pub async fn list_active_for_user_client(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Vec<OAuthToken>, OAuthError> {
        let tokens = sqlx::query_as::<_, OAuthToken>(
            r#"
            SELECT id, user_id, client_id, access_token_hash, refresh_token_hash, 
                   scopes, expires_at, revoked, created_at
            FROM oauth_tokens
            WHERE user_id = ? AND client_id = ? AND revoked = false
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id.to_string())
        .bind(client_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(tokens)
    }

    /// Delete expired tokens (cleanup)
    pub async fn delete_expired(&self) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_tokens
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Delete revoked tokens (cleanup)
    pub async fn delete_revoked(&self) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_tokens
            WHERE revoked = true
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Get expiration time for a token
    pub async fn get_expiration(&self, id: Uuid) -> Result<Option<DateTime<Utc>>, OAuthError> {
        let expires_at = sqlx::query_scalar::<_, DateTime<Utc>>(
            r#"
            SELECT expires_at
            FROM oauth_tokens
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(expires_at)
    }

    /// Count total tokens
    pub async fn count_all(&self) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_tokens
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }

    /// Count active (not revoked, not expired) tokens
    pub async fn count_active(&self) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_tokens
            WHERE revoked = false AND expires_at > NOW()
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }
}

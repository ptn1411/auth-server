use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;

/// Repository for revoked token database operations
#[derive(Clone)]
pub struct RevokedTokenRepository {
    pool: MySqlPool,
}

impl RevokedTokenRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Add a token to the revocation list
    pub async fn revoke(
        &self,
        token_hash: &str,
        token_type: &str,
        user_id: Option<Uuid>,
        expires_at: DateTime<Utc>,
        reason: Option<&str>,
    ) -> Result<(), AuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO revoked_tokens (id, token_hash, token_type, user_id, expires_at, reason)
            VALUES (?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE revoked_at = NOW(), reason = VALUES(reason)
            "#,
        )
        .bind(id.to_string())
        .bind(token_hash)
        .bind(token_type)
        .bind(user_id.map(|u| u.to_string()))
        .bind(expires_at)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Check if a token is revoked
    pub async fn is_revoked(&self, token_hash: &str) -> Result<bool, AuthError> {
        let exists = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM revoked_tokens
            WHERE token_hash = ?
            "#,
        )
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(exists > 0)
    }

    /// Revoke all tokens for a user
    pub async fn revoke_all_for_user(
        &self,
        user_id: Uuid,
        token_type: &str,
        expires_at: DateTime<Utc>,
        reason: &str,
    ) -> Result<(), AuthError> {
        let id = Uuid::new_v4();
        let token_hash = format!("user_all_{}_{}", user_id, chrono::Utc::now().timestamp());

        sqlx::query(
            r#"
            INSERT INTO revoked_tokens (id, token_hash, token_type, user_id, expires_at, reason)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(&token_hash)
        .bind(token_type)
        .bind(user_id.to_string())
        .bind(expires_at)
        .bind(reason)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Delete expired revoked tokens (cleanup)
    pub async fn delete_expired(&self) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM revoked_tokens
            WHERE expires_at < NOW()
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }
}

use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::UserSession;

/// Repository for user session database operations
#[derive(Clone)]
pub struct SessionRepository {
    pool: MySqlPool,
}

impl SessionRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new session
    pub async fn create(
        &self,
        user_id: Uuid,
        refresh_token_hash: &str,
        device_name: Option<&str>,
        device_type: Option<&str>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<UserSession, AuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO user_sessions (id, user_id, refresh_token_hash, device_name, device_type, ip_address, user_agent, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(refresh_token_hash)
        .bind(device_name)
        .bind(device_type)
        .bind(ip_address)
        .bind(user_agent)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        self.find_by_id(id).await?.ok_or(AuthError::InternalError(anyhow::anyhow!("Failed to fetch created session")))
    }

    /// Find session by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<UserSession>, AuthError> {
        let session = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, refresh_token_hash, device_name, device_type, ip_address, user_agent, 
                   last_active_at, expires_at, is_revoked, revoked_at, created_at
            FROM user_sessions
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(session)
    }

    /// Find session by refresh token hash
    pub async fn find_by_token_hash(&self, token_hash: &str) -> Result<Option<UserSession>, AuthError> {
        let session = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, refresh_token_hash, device_name, device_type, ip_address, user_agent, 
                   last_active_at, expires_at, is_revoked, revoked_at, created_at
            FROM user_sessions
            WHERE refresh_token_hash = ? AND is_revoked = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(session)
    }

    /// List active sessions for a user
    pub async fn list_active_by_user(&self, user_id: Uuid) -> Result<Vec<UserSession>, AuthError> {
        let sessions = sqlx::query_as::<_, UserSession>(
            r#"
            SELECT id, user_id, refresh_token_hash, device_name, device_type, ip_address, user_agent, 
                   last_active_at, expires_at, is_revoked, revoked_at, created_at
            FROM user_sessions
            WHERE user_id = ? AND is_revoked = FALSE AND expires_at > NOW()
            ORDER BY last_active_at DESC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(sessions)
    }

    /// Update last active timestamp
    pub async fn update_last_active(&self, id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE user_sessions
            SET last_active_at = NOW()
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Revoke a specific session
    pub async fn revoke(&self, id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_revoked = TRUE, revoked_at = NOW()
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::InvalidToken);
        }

        Ok(())
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all_for_user(&self, user_id: Uuid) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_revoked = TRUE, revoked_at = NOW()
            WHERE user_id = ? AND is_revoked = FALSE
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }

    /// Revoke all sessions except current
    pub async fn revoke_all_except(&self, user_id: Uuid, current_session_id: Uuid) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE user_sessions
            SET is_revoked = TRUE, revoked_at = NOW()
            WHERE user_id = ? AND id != ? AND is_revoked = FALSE
            "#,
        )
        .bind(user_id.to_string())
        .bind(current_session_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }

    /// Delete expired sessions (cleanup)
    pub async fn delete_expired(&self) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_sessions
            WHERE expires_at < NOW() OR (is_revoked = TRUE AND revoked_at < DATE_SUB(NOW(), INTERVAL 7 DAY))
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }

    /// Count active sessions for a user
    pub async fn count_active_by_user(&self, user_id: Uuid) -> Result<i64, AuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM user_sessions
            WHERE user_id = ? AND is_revoked = FALSE AND expires_at > NOW()
            "#,
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(count)
    }
}

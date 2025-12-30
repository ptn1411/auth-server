use chrono::{DateTime, Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;

/// Repository for rate limiting database operations
#[derive(Clone)]
pub struct RateLimitRepository {
    pool: MySqlPool,
}

impl RateLimitRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Increment request count for an identifier/endpoint combination
    /// Returns the current count after increment
    pub async fn increment(
        &self,
        identifier: &str,
        endpoint: &str,
        window_seconds: i64,
    ) -> Result<i32, AuthError> {
        let id = Uuid::new_v4();
        let window_start = Utc::now() - Duration::seconds(window_seconds);

        // First, try to update existing entry within the window
        let result = sqlx::query(
            r#"
            UPDATE rate_limit_entries
            SET request_count = request_count + 1
            WHERE identifier = ? AND endpoint = ? AND window_start > ?
            "#,
        )
        .bind(identifier)
        .bind(endpoint)
        .bind(window_start)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        if result.rows_affected() > 0 {
            // Entry was updated, get the new count
            let count = sqlx::query_scalar::<_, i32>(
                r#"
                SELECT request_count
                FROM rate_limit_entries
                WHERE identifier = ? AND endpoint = ? AND window_start > ?
                "#,
            )
            .bind(identifier)
            .bind(endpoint)
            .bind(window_start)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| AuthError::InternalError(e.into()))?;

            return Ok(count);
        }

        // No existing entry or entry is outside window, create new one
        // First delete old entries for this identifier/endpoint
        sqlx::query(
            r#"
            DELETE FROM rate_limit_entries
            WHERE identifier = ? AND endpoint = ?
            "#,
        )
        .bind(identifier)
        .bind(endpoint)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Insert new entry
        sqlx::query(
            r#"
            INSERT INTO rate_limit_entries (id, identifier, endpoint, request_count, window_start)
            VALUES (?, ?, ?, 1, NOW())
            "#,
        )
        .bind(id.to_string())
        .bind(identifier)
        .bind(endpoint)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(1)
    }

    /// Get current request count for an identifier/endpoint
    pub async fn get_count(
        &self,
        identifier: &str,
        endpoint: &str,
        window_seconds: i64,
    ) -> Result<i32, AuthError> {
        let window_start = Utc::now() - Duration::seconds(window_seconds);

        let count = sqlx::query_scalar::<_, i32>(
            r#"
            SELECT COALESCE(request_count, 0)
            FROM rate_limit_entries
            WHERE identifier = ? AND endpoint = ? AND window_start > ?
            "#,
        )
        .bind(identifier)
        .bind(endpoint)
        .bind(window_start)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(count.unwrap_or(0))
    }

    /// Reset rate limit for an identifier/endpoint
    pub async fn reset(&self, identifier: &str, endpoint: &str) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            DELETE FROM rate_limit_entries
            WHERE identifier = ? AND endpoint = ?
            "#,
        )
        .bind(identifier)
        .bind(endpoint)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Get window start time for an identifier/endpoint
    pub async fn get_window_start(
        &self,
        identifier: &str,
        endpoint: &str,
    ) -> Result<Option<DateTime<Utc>>, AuthError> {
        let window_start = sqlx::query_scalar::<_, DateTime<Utc>>(
            r#"
            SELECT window_start
            FROM rate_limit_entries
            WHERE identifier = ? AND endpoint = ?
            "#,
        )
        .bind(identifier)
        .bind(endpoint)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(window_start)
    }

    /// Delete old rate limit entries (cleanup)
    pub async fn delete_expired(&self, window_seconds: i64) -> Result<u64, AuthError> {
        let window_start = Utc::now() - Duration::seconds(window_seconds);

        let result = sqlx::query(
            r#"
            DELETE FROM rate_limit_entries
            WHERE window_start < ?
            "#,
        )
        .bind(window_start)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }
}

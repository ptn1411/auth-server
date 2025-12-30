use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::error::AppError;

#[derive(Clone)]
pub struct RateLimitRepository {
    pool: MySqlPool,
}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: i32,
    pub window_seconds: i64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

impl RateLimitRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Check if request is allowed and increment counter
    /// Returns (allowed, remaining, reset_at)
    pub async fn check_and_increment(
        &self,
        identifier: &str,
        endpoint: &str,
        config: &RateLimitConfig,
    ) -> Result<(bool, i32, i64), AppError> {
        let window_start = Utc::now() - Duration::seconds(config.window_seconds);
        
        // Clean old entries
        sqlx::query("DELETE FROM rate_limits WHERE window_start < ?")
            .bind(window_start)
            .execute(&self.pool)
            .await?;

        // Get current count
        let count: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT CAST(COALESCE(SUM(request_count), 0) AS SIGNED) as count
            FROM rate_limits 
            WHERE identifier = ? AND endpoint = ? AND window_start >= ?
            "#,
        )
        .bind(identifier)
        .bind(endpoint)
        .bind(window_start)
        .fetch_optional(&self.pool)
        .await?;

        let current_count = count.map(|c| c.0 as i32).unwrap_or(0);
        let remaining = config.max_requests - current_count - 1;
        let reset_at = (Utc::now() + Duration::seconds(config.window_seconds)).timestamp();

        if current_count >= config.max_requests {
            return Ok((false, 0, reset_at));
        }

        // Increment counter - use a fixed window start time (truncated to the second)
        let now = Utc::now();
        let window_key = now.format("%Y-%m-%d %H:%M:%S").to_string();
        let id = Uuid::new_v4();
        
        // Try to insert or update existing record for this window
        sqlx::query(
            r#"
            INSERT INTO rate_limits (id, identifier, endpoint, request_count, window_start)
            VALUES (?, ?, ?, 1, ?)
            ON DUPLICATE KEY UPDATE request_count = request_count + 1
            "#,
        )
        .bind(id.to_string())
        .bind(identifier)
        .bind(endpoint)
        .bind(&window_key)
        .execute(&self.pool)
        .await?;

        Ok((true, remaining.max(0), reset_at))
    }

    /// Get current rate limit status without incrementing
    pub async fn get_status(
        &self,
        identifier: &str,
        endpoint: &str,
        config: &RateLimitConfig,
    ) -> Result<(i32, i32, i64), AppError> {
        let window_start = Utc::now() - Duration::seconds(config.window_seconds);

        let count: Option<(i64,)> = sqlx::query_as(
            r#"
            SELECT CAST(COALESCE(SUM(request_count), 0) AS SIGNED) as count
            FROM rate_limits 
            WHERE identifier = ? AND endpoint = ? AND window_start >= ?
            "#,
        )
        .bind(identifier)
        .bind(endpoint)
        .bind(window_start)
        .fetch_optional(&self.pool)
        .await?;

        let current_count = count.map(|c| c.0 as i32).unwrap_or(0);
        let remaining = (config.max_requests - current_count).max(0);
        let reset_at = (Utc::now() + Duration::seconds(config.window_seconds)).timestamp();

        Ok((current_count, remaining, reset_at))
    }

    /// Reset rate limit for identifier
    pub async fn reset(&self, identifier: &str, endpoint: Option<&str>) -> Result<(), AppError> {
        if let Some(endpoint) = endpoint {
            sqlx::query("DELETE FROM rate_limits WHERE identifier = ? AND endpoint = ?")
                .bind(identifier)
                .bind(endpoint)
                .execute(&self.pool)
                .await?;
        } else {
            sqlx::query("DELETE FROM rate_limits WHERE identifier = ?")
                .bind(identifier)
                .execute(&self.pool)
                .await?;
        }
        Ok(())
    }
}

// Predefined rate limit configs
impl RateLimitConfig {
    pub fn login() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 60,
        }
    }

    pub fn register() -> Self {
        Self {
            max_requests: 3,
            window_seconds: 60,
        }
    }

    pub fn password_reset() -> Self {
        Self {
            max_requests: 3,
            window_seconds: 300,
        }
    }

    pub fn api() -> Self {
        Self {
            max_requests: 1000,
            window_seconds: 60,
        }
    }

    pub fn strict() -> Self {
        Self {
            max_requests: 10,
            window_seconds: 60,
        }
    }
}

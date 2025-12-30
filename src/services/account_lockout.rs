use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;

/// Configuration for account lockout
#[derive(Debug, Clone)]
pub struct LockoutConfig {
    pub max_failed_attempts: i32,
    pub lockout_duration_minutes: i64,
    pub reset_after_minutes: i64,
}

impl Default for LockoutConfig {
    fn default() -> Self {
        Self {
            max_failed_attempts: 5,
            lockout_duration_minutes: 15,
            reset_after_minutes: 30,
        }
    }
}

/// Service for account lockout management
#[derive(Clone)]
pub struct AccountLockoutService {
    pool: MySqlPool,
    config: LockoutConfig,
}

impl AccountLockoutService {
    pub fn new(pool: MySqlPool, config: LockoutConfig) -> Self {
        Self { pool, config }
    }

    /// Check if an account is currently locked
    pub async fn is_locked(&self, user_id: Uuid) -> Result<bool, AuthError> {
        let result = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
            r#"
            SELECT locked_until
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        match result {
            Some(Some(locked_until)) => Ok(locked_until > Utc::now()),
            _ => Ok(false),
        }
    }

    /// Get lockout info for a user
    pub async fn get_lockout_info(&self, user_id: Uuid) -> Result<LockoutInfo, AuthError> {
        let row = sqlx::query_as::<_, LockoutRow>(
            r#"
            SELECT failed_login_attempts, locked_until, last_failed_login
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        match row {
            Some(r) => {
                let is_locked = r.locked_until.map(|t| t > Utc::now()).unwrap_or(false);
                let remaining_attempts = (self.config.max_failed_attempts - r.failed_login_attempts).max(0);
                
                Ok(LockoutInfo {
                    is_locked,
                    failed_attempts: r.failed_login_attempts,
                    remaining_attempts,
                    locked_until: r.locked_until,
                    last_failed_login: r.last_failed_login,
                })
            }
            None => Err(AuthError::UserNotFound),
        }
    }

    /// Record a failed login attempt
    pub async fn record_failed_attempt(&self, user_id: Uuid) -> Result<LockoutInfo, AuthError> {
        // First, check if we should reset the counter (if last failure was too long ago)
        let should_reset = self.should_reset_counter(user_id).await?;
        
        if should_reset {
            self.reset_failed_attempts(user_id).await?;
        }

        // Increment failed attempts
        sqlx::query(
            r#"
            UPDATE users
            SET failed_login_attempts = failed_login_attempts + 1,
                last_failed_login = NOW()
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Check if we need to lock the account
        let info = self.get_lockout_info(user_id).await?;
        
        if info.failed_attempts >= self.config.max_failed_attempts {
            self.lock_account(user_id).await?;
            return self.get_lockout_info(user_id).await;
        }

        Ok(info)
    }

    /// Record a successful login (resets failed attempts)
    pub async fn record_successful_login(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE users
            SET failed_login_attempts = 0,
                locked_until = NULL,
                last_failed_login = NULL
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Lock an account
    pub async fn lock_account(&self, user_id: Uuid) -> Result<(), AuthError> {
        let locked_until = Utc::now() + Duration::minutes(self.config.lockout_duration_minutes);

        sqlx::query(
            r#"
            UPDATE users
            SET locked_until = ?
            WHERE id = ?
            "#,
        )
        .bind(locked_until)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Unlock an account (admin action)
    pub async fn unlock_account(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE users
            SET locked_until = NULL,
                failed_login_attempts = 0,
                last_failed_login = NULL
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Reset failed login attempts
    async fn reset_failed_attempts(&self, user_id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE users
            SET failed_login_attempts = 0,
                last_failed_login = NULL
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Check if the failed attempt counter should be reset
    async fn should_reset_counter(&self, user_id: Uuid) -> Result<bool, AuthError> {
        let last_failed = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
            r#"
            SELECT last_failed_login
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        match last_failed {
            Some(Some(last)) => {
                let reset_threshold = Utc::now() - Duration::minutes(self.config.reset_after_minutes);
                Ok(last < reset_threshold)
            }
            _ => Ok(false),
        }
    }
}

/// Lockout information
#[derive(Debug, Clone)]
pub struct LockoutInfo {
    pub is_locked: bool,
    pub failed_attempts: i32,
    pub remaining_attempts: i32,
    pub locked_until: Option<chrono::DateTime<Utc>>,
    pub last_failed_login: Option<chrono::DateTime<Utc>>,
}

/// Database row for lockout query
#[derive(Debug, sqlx::FromRow)]
struct LockoutRow {
    failed_login_attempts: i32,
    locked_until: Option<chrono::DateTime<Utc>>,
    last_failed_login: Option<chrono::DateTime<Utc>>,
}

use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::{UserMfaBackupCode, UserMfaMethod};

/// Repository for MFA database operations
#[derive(Clone)]
pub struct MfaRepository {
    pool: MySqlPool,
}

impl MfaRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // MFA Methods
    // ========================================================================

    /// Create a new MFA method for a user
    pub async fn create_method(
        &self,
        user_id: Uuid,
        method_type: &str,
        secret_encrypted: Option<&str>,
        phone_number: Option<&str>,
        email: Option<&str>,
        is_primary: bool,
    ) -> Result<UserMfaMethod, AuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO user_mfa_methods (id, user_id, method_type, secret_encrypted, phone_number, email, is_primary)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(method_type)
        .bind(secret_encrypted)
        .bind(phone_number)
        .bind(email)
        .bind(is_primary)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        self.find_method_by_id(id).await?.ok_or(AuthError::InternalError(anyhow::anyhow!("Failed to fetch created MFA method")))
    }

    /// Find MFA method by ID
    pub async fn find_method_by_id(&self, id: Uuid) -> Result<Option<UserMfaMethod>, AuthError> {
        let method = sqlx::query_as::<_, UserMfaMethod>(
            r#"
            SELECT id, user_id, method_type, secret_encrypted, phone_number, email, is_primary, is_verified, last_used_at, created_at
            FROM user_mfa_methods
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(method)
    }

    /// List all MFA methods for a user
    pub async fn list_methods_by_user(&self, user_id: Uuid) -> Result<Vec<UserMfaMethod>, AuthError> {
        let methods = sqlx::query_as::<_, UserMfaMethod>(
            r#"
            SELECT id, user_id, method_type, secret_encrypted, phone_number, email, is_primary, is_verified, last_used_at, created_at
            FROM user_mfa_methods
            WHERE user_id = ?
            ORDER BY is_primary DESC, created_at ASC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(methods)
    }

    /// Get primary MFA method for a user
    pub async fn get_primary_method(&self, user_id: Uuid) -> Result<Option<UserMfaMethod>, AuthError> {
        let method = sqlx::query_as::<_, UserMfaMethod>(
            r#"
            SELECT id, user_id, method_type, secret_encrypted, phone_number, email, is_primary, is_verified, last_used_at, created_at
            FROM user_mfa_methods
            WHERE user_id = ? AND is_primary = TRUE AND is_verified = TRUE
            "#,
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(method)
    }

    /// Verify an MFA method
    pub async fn verify_method(&self, id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE user_mfa_methods
            SET is_verified = TRUE
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::InternalError(anyhow::anyhow!("MFA method not found")));
        }

        Ok(())
    }

    /// Update last used timestamp
    pub async fn update_last_used(&self, id: Uuid) -> Result<(), AuthError> {
        sqlx::query(
            r#"
            UPDATE user_mfa_methods
            SET last_used_at = NOW()
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Set primary MFA method (unsets others)
    pub async fn set_primary(&self, user_id: Uuid, method_id: Uuid) -> Result<(), AuthError> {
        // First, unset all primary flags for this user
        sqlx::query(
            r#"
            UPDATE user_mfa_methods
            SET is_primary = FALSE
            WHERE user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Then set the specified method as primary
        sqlx::query(
            r#"
            UPDATE user_mfa_methods
            SET is_primary = TRUE
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(method_id.to_string())
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Delete an MFA method
    pub async fn delete_method(&self, id: Uuid, user_id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_mfa_methods
            WHERE id = ? AND user_id = ?
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::InternalError(anyhow::anyhow!("MFA method not found")));
        }

        Ok(())
    }

    /// Delete all MFA methods for a user
    pub async fn delete_all_methods(&self, user_id: Uuid) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_mfa_methods
            WHERE user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }

    // ========================================================================
    // Backup Codes
    // ========================================================================

    /// Create backup codes for a user
    pub async fn create_backup_codes(
        &self,
        user_id: Uuid,
        code_hashes: Vec<String>,
    ) -> Result<Vec<UserMfaBackupCode>, AuthError> {
        // First delete existing backup codes
        sqlx::query(
            r#"
            DELETE FROM user_mfa_backup_codes
            WHERE user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Insert new backup codes
        for code_hash in &code_hashes {
            let id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO user_mfa_backup_codes (id, user_id, code_hash)
                VALUES (?, ?, ?)
                "#,
            )
            .bind(id.to_string())
            .bind(user_id.to_string())
            .bind(code_hash)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::InternalError(e.into()))?;
        }

        self.list_backup_codes(user_id).await
    }

    /// List backup codes for a user
    pub async fn list_backup_codes(&self, user_id: Uuid) -> Result<Vec<UserMfaBackupCode>, AuthError> {
        let codes = sqlx::query_as::<_, UserMfaBackupCode>(
            r#"
            SELECT id, user_id, code_hash, is_used, used_at, created_at
            FROM user_mfa_backup_codes
            WHERE user_id = ?
            ORDER BY created_at ASC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(codes)
    }

    /// Find unused backup code by hash
    pub async fn find_unused_backup_code(
        &self,
        user_id: Uuid,
        code_hash: &str,
    ) -> Result<Option<UserMfaBackupCode>, AuthError> {
        let code = sqlx::query_as::<_, UserMfaBackupCode>(
            r#"
            SELECT id, user_id, code_hash, is_used, used_at, created_at
            FROM user_mfa_backup_codes
            WHERE user_id = ? AND code_hash = ? AND is_used = FALSE
            "#,
        )
        .bind(user_id.to_string())
        .bind(code_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(code)
    }

    /// Mark backup code as used
    pub async fn use_backup_code(&self, id: Uuid) -> Result<(), AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE user_mfa_backup_codes
            SET is_used = TRUE, used_at = NOW()
            WHERE id = ? AND is_used = FALSE
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

    /// Count remaining unused backup codes
    pub async fn count_unused_backup_codes(&self, user_id: Uuid) -> Result<i64, AuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM user_mfa_backup_codes
            WHERE user_id = ? AND is_used = FALSE
            "#,
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(count)
    }

    // ========================================================================
    // MFA Verification Attempts
    // ========================================================================

    /// Record an MFA verification attempt
    pub async fn record_attempt(
        &self,
        user_id: Uuid,
        attempt_type: &str,
        is_successful: bool,
        ip_address: Option<&str>,
    ) -> Result<(), AuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO mfa_verification_attempts (id, user_id, attempt_type, is_successful, ip_address)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(attempt_type)
        .bind(is_successful)
        .bind(ip_address)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Count recent failed MFA attempts
    pub async fn count_recent_failed_attempts(
        &self,
        user_id: Uuid,
        minutes: i64,
    ) -> Result<i64, AuthError> {
        let since = Utc::now() - Duration::minutes(minutes);

        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM mfa_verification_attempts
            WHERE user_id = ? AND is_successful = FALSE AND created_at > ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(since)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(count)
    }

    /// Delete old MFA attempts (cleanup)
    pub async fn delete_old_attempts(&self, days: i64) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM mfa_verification_attempts
            WHERE created_at < DATE_SUB(NOW(), INTERVAL ? DAY)
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }
}

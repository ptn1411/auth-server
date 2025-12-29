use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::UserManagementError;
use crate::models::user_app::{UserApp, UserAppStatus};

/// Repository for user-app association database operations
/// Requirements: 2.1, 2.4, 3.1, 4.1, 5.1
#[derive(Clone)]
pub struct UserAppRepository {
    pool: MySqlPool,
}

impl UserAppRepository {
    /// Create a new UserAppRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new user-app association with status "active"
    /// Requirements: 2.1
    pub async fn create(&self, user_id: Uuid, app_id: Uuid) -> Result<UserApp, UserManagementError> {
        sqlx::query(
            r#"
            INSERT INTO user_apps (user_id, app_id, status)
            VALUES (?, ?, 'active')
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry")
                {
                    return UserManagementError::UserAlreadyRegistered;
                }
            }
            UserManagementError::InternalError(e.into())
        })?;

        self.find(user_id, app_id)
            .await?
            .ok_or(UserManagementError::InternalError(anyhow::anyhow!(
                "Failed to fetch created user_app"
            )))
    }

    /// Find a user-app association
    pub async fn find(&self, user_id: Uuid, app_id: Uuid) -> Result<Option<UserApp>, UserManagementError> {
        let user_app = sqlx::query_as::<_, UserApp>(
            r#"
            SELECT user_id, app_id, status, banned_at, banned_reason, created_at
            FROM user_apps
            WHERE user_id = ? AND app_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(user_app)
    }


    /// Update user-app status (for ban/unban operations)
    /// Requirements: 3.1, 4.1
    pub async fn update_status(
        &self,
        user_id: Uuid,
        app_id: Uuid,
        status: UserAppStatus,
        banned_reason: Option<String>,
    ) -> Result<UserApp, UserManagementError> {
        let banned_at = if status == UserAppStatus::Banned {
            Some(Utc::now())
        } else {
            None
        };

        let result = sqlx::query(
            r#"
            UPDATE user_apps
            SET status = ?, banned_at = ?, banned_reason = ?
            WHERE user_id = ? AND app_id = ?
            "#,
        )
        .bind(status.as_str())
        .bind(banned_at)
        .bind(&banned_reason)
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| UserManagementError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(UserManagementError::UserNotRegistered);
        }

        self.find(user_id, app_id)
            .await?
            .ok_or(UserManagementError::InternalError(anyhow::anyhow!(
                "Failed to fetch updated user_app"
            )))
    }

    /// Create a banned user-app record (for users not yet registered)
    /// Requirements: 3.5
    pub async fn create_banned(
        &self,
        user_id: Uuid,
        app_id: Uuid,
        banned_reason: Option<String>,
    ) -> Result<UserApp, UserManagementError> {
        let banned_at = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO user_apps (user_id, app_id, status, banned_at, banned_reason)
            VALUES (?, ?, 'banned', ?, ?)
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .bind(banned_at)
        .bind(&banned_reason)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry")
                {
                    return UserManagementError::UserAlreadyRegistered;
                }
            }
            UserManagementError::InternalError(e.into())
        })?;

        self.find(user_id, app_id)
            .await?
            .ok_or(UserManagementError::InternalError(anyhow::anyhow!(
                "Failed to fetch created banned user_app"
            )))
    }

    /// Delete a user-app association
    /// Requirements: 5.1
    pub async fn delete(&self, user_id: Uuid, app_id: Uuid) -> Result<(), UserManagementError> {
        sqlx::query(
            r#"
            DELETE FROM user_apps
            WHERE user_id = ? AND app_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(())
    }

    /// List users in an app with pagination
    /// Requirements: 6.1, 6.2
    pub async fn list_by_app(
        &self,
        app_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Vec<UserApp>, UserManagementError> {
        let offset = (page.saturating_sub(1)) * limit;

        let user_apps = sqlx::query_as::<_, UserApp>(
            r#"
            SELECT user_id, app_id, status, banned_at, banned_reason, created_at
            FROM user_apps
            WHERE app_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(app_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(user_apps)
    }

    /// Count total users in an app (for pagination)
    pub async fn count_by_app(&self, app_id: Uuid) -> Result<u64, UserManagementError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM user_apps
            WHERE app_id = ?
            "#,
        )
        .bind(app_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(count as u64)
    }

    /// Check if a user is banned from an app
    /// Requirements: 2.2, 3.4
    pub async fn is_banned(&self, user_id: Uuid, app_id: Uuid) -> Result<bool, UserManagementError> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM user_apps
            WHERE user_id = ? AND app_id = ? AND status = 'banned'
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(result > 0)
    }
}

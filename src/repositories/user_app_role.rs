use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::RoleError;
use crate::models::UserAppRole;

/// Repository for user-app-role association database operations
#[derive(Clone)]
pub struct UserAppRoleRepository {
    pool: MySqlPool,
}

impl UserAppRoleRepository {
    /// Create a new UserAppRoleRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Assign a role to a user for a specific app
    /// Returns RoleError if user, app, or role doesn't exist
    /// Requirements: 8.1
    pub async fn assign_role(
        &self,
        user_id: Uuid,
        app_id: Uuid,
        role_id: Uuid,
    ) -> Result<UserAppRole, RoleError> {
        sqlx::query(
            r#"
            INSERT INTO user_app_roles (user_id, app_id, role_id)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE user_id = user_id
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .bind(role_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.message().contains("foreign key constraint")
                    || db_err.message().contains("Cannot add or update") {
                    let msg = db_err.message().to_lowercase();
                    if msg.contains("user") {
                        return RoleError::UserNotFound;
                    } else if msg.contains("app") {
                        return RoleError::AppNotFound;
                    } else if msg.contains("role") {
                        return RoleError::NotFound;
                    }
                    return RoleError::NotFound;
                }
            }
            RoleError::InternalError(e.into())
        })?;

        Ok(UserAppRole {
            user_id,
            app_id,
            role_id,
        })
    }

    /// Remove a role from a user for a specific app
    /// Returns Ok(true) if the role was removed, Ok(false) if it didn't exist
    pub async fn remove_role(
        &self,
        user_id: Uuid,
        app_id: Uuid,
        role_id: Uuid,
    ) -> Result<bool, RoleError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_app_roles
            WHERE user_id = ? AND app_id = ? AND role_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .bind(role_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Find all role assignments for a user across all apps
    /// Requirements: 8.3
    pub async fn find_by_user(&self, user_id: Uuid) -> Result<Vec<UserAppRole>, RoleError> {
        let user_app_roles = sqlx::query_as::<_, UserAppRole>(
            r#"
            SELECT user_id, app_id, role_id
            FROM user_app_roles
            WHERE user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(user_app_roles)
    }

    /// Find all role assignments for a user within a specific app
    /// Requirements: 8.1, 8.3
    pub async fn find_by_user_and_app(
        &self,
        user_id: Uuid,
        app_id: Uuid,
    ) -> Result<Vec<UserAppRole>, RoleError> {
        let user_app_roles = sqlx::query_as::<_, UserAppRole>(
            r#"
            SELECT user_id, app_id, role_id
            FROM user_app_roles
            WHERE user_id = ? AND app_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(user_app_roles)
    }

    /// Delete all role assignments for a user within a specific app
    /// Requirements: 5.1 - Remove user from app deletes user_app_roles
    pub async fn delete_by_user_and_app(
        &self,
        user_id: Uuid,
        app_id: Uuid,
    ) -> Result<(), RoleError> {
        sqlx::query(
            r#"
            DELETE FROM user_app_roles
            WHERE user_id = ? AND app_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(())
    }
}

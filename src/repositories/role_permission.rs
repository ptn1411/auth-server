use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::PermissionError;
use crate::models::RolePermission;

/// Repository for role-permission association database operations
#[derive(Clone)]
pub struct RolePermissionRepository {
    pool: MySqlPool,
}

impl RolePermissionRepository {
    /// Create a new RolePermissionRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Assign a permission to a role
    /// Returns PermissionError if role or permission doesn't exist
    /// Requirements: 9.1
    pub async fn assign_permission(
        &self,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<RolePermission, PermissionError> {
        sqlx::query(
            r#"
            INSERT INTO role_permissions (role_id, permission_id)
            VALUES (?, ?)
            ON DUPLICATE KEY UPDATE role_id = role_id
            "#,
        )
        .bind(role_id.to_string())
        .bind(permission_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.message().contains("foreign key constraint")
                    || db_err.message().contains("Cannot add or update") {
                    return PermissionError::NotFound;
                }
            }
            PermissionError::InternalError(e.into())
        })?;

        Ok(RolePermission {
            role_id,
            permission_id,
        })
    }

    /// Remove a permission from a role
    /// Returns Ok(true) if the permission was removed, Ok(false) if it didn't exist
    pub async fn remove_permission(
        &self,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<bool, PermissionError> {
        let result = sqlx::query(
            r#"
            DELETE FROM role_permissions
            WHERE role_id = ? AND permission_id = ?
            "#,
        )
        .bind(role_id.to_string())
        .bind(permission_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| PermissionError::InternalError(e.into()))?;

        Ok(result.rows_affected() > 0)
    }

    /// Find all permission assignments for a role
    /// Requirements: 9.1
    pub async fn find_by_role(&self, role_id: Uuid) -> Result<Vec<RolePermission>, PermissionError> {
        let role_permissions = sqlx::query_as::<_, RolePermission>(
            r#"
            SELECT role_id, permission_id
            FROM role_permissions
            WHERE role_id = ?
            "#,
        )
        .bind(role_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PermissionError::InternalError(e.into()))?;

        Ok(role_permissions)
    }
}

use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::PermissionError;
use crate::models::Permission;

/// Repository for permission database operations
#[derive(Clone)]
pub struct PermissionRepository {
    pool: MySqlPool,
}

impl PermissionRepository {
    /// Create a new PermissionRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new permission for a specific app
    /// Returns PermissionError::CodeAlreadyExists if permission code already exists in the app
    /// Requirements: 7.1, 7.2
    pub async fn create_permission(&self, app_id: Uuid, code: &str) -> Result<Permission, PermissionError> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO permissions (id, app_id, code)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(app_id.to_string())
        .bind(code)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry") {
                    return PermissionError::CodeAlreadyExists;
                }
                if db_err.message().contains("foreign key constraint")
                    || db_err.message().contains("Cannot add or update") {
                    return PermissionError::AppNotFound;
                }
            }
            PermissionError::InternalError(e.into())
        })?;

        self.find_by_id(id).await?.ok_or(PermissionError::InternalError(anyhow::anyhow!("Failed to fetch created permission")))
    }

    /// Find a permission by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Permission>, PermissionError> {
        let permission = sqlx::query_as::<_, Permission>(
            r#"
            SELECT id, app_id, code
            FROM permissions
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PermissionError::InternalError(e.into()))?;

        Ok(permission)
    }

    /// Find all permissions for a specific app
    /// Requirements: 7.1
    pub async fn find_by_app_id(&self, app_id: Uuid) -> Result<Vec<Permission>, PermissionError> {
        let permissions = sqlx::query_as::<_, Permission>(
            r#"
            SELECT id, app_id, code
            FROM permissions
            WHERE app_id = ?
            ORDER BY code
            "#,
        )
        .bind(app_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| PermissionError::InternalError(e.into()))?;

        Ok(permissions)
    }

    /// Find a permission by app_id and code
    /// Requirements: 7.2
    pub async fn find_by_app_and_code(&self, app_id: Uuid, code: &str) -> Result<Option<Permission>, PermissionError> {
        let permission = sqlx::query_as::<_, Permission>(
            r#"
            SELECT id, app_id, code
            FROM permissions
            WHERE app_id = ? AND code = ?
            "#,
        )
        .bind(app_id.to_string())
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| PermissionError::InternalError(e.into()))?;

        Ok(permission)
    }
}

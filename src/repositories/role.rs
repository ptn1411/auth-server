use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::RoleError;
use crate::models::Role;

/// Repository for role database operations
#[derive(Clone)]
pub struct RoleRepository {
    pool: MySqlPool,
}

impl RoleRepository {
    /// Create a new RoleRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new role for a specific app
    /// Returns RoleError::NameAlreadyExists if role name already exists in the app
    /// Requirements: 6.1, 6.2
    pub async fn create_role(&self, app_id: Uuid, name: &str) -> Result<Role, RoleError> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO roles (id, app_id, name)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(app_id.to_string())
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry") {
                    return RoleError::NameAlreadyExists;
                }
                // MySQL foreign key violation
                if db_err.message().contains("foreign key constraint") 
                    || db_err.message().contains("Cannot add or update") {
                    return RoleError::AppNotFound;
                }
            }
            RoleError::InternalError(e.into())
        })?;

        self.find_by_id(id).await?.ok_or(RoleError::InternalError(anyhow::anyhow!("Failed to fetch created role")))
    }

    /// Find a role by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Role>, RoleError> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, app_id, name
            FROM roles
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(role)
    }

    /// Find all roles for a specific app
    /// Requirements: 6.1
    pub async fn find_by_app_id(&self, app_id: Uuid) -> Result<Vec<Role>, RoleError> {
        let roles = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, app_id, name
            FROM roles
            WHERE app_id = ?
            ORDER BY name
            "#,
        )
        .bind(app_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(roles)
    }

    /// Find a role by app_id and name
    /// Requirements: 6.2
    pub async fn find_by_app_and_name(&self, app_id: Uuid, name: &str) -> Result<Option<Role>, RoleError> {
        let role = sqlx::query_as::<_, Role>(
            r#"
            SELECT id, app_id, name
            FROM roles
            WHERE app_id = ? AND name = ?
            "#,
        )
        .bind(app_id.to_string())
        .bind(name)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(role)
    }

    /// Get role names for a user in a specific app
    /// Requirements: 6.2 - Include roles in user list response
    pub async fn get_role_names_for_user_in_app(
        &self,
        user_id: Uuid,
        app_id: Uuid,
    ) -> Result<Vec<String>, RoleError> {
        let role_names = sqlx::query_scalar::<_, String>(
            r#"
            SELECT r.name
            FROM roles r
            INNER JOIN user_app_roles uar ON r.id = uar.role_id
            WHERE uar.user_id = ? AND uar.app_id = ?
            ORDER BY r.name
            "#,
        )
        .bind(user_id.to_string())
        .bind(app_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RoleError::InternalError(e.into()))?;

        Ok(role_names)
    }
}

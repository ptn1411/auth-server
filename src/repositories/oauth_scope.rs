use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::OAuthScope;

/// Repository for OAuth scope database operations
/// Requirements: 2.1, 2.2
#[derive(Clone)]
pub struct OAuthScopeRepository {
    pool: MySqlPool,
}

impl OAuthScopeRepository {
    /// Create a new OAuthScopeRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new OAuth scope
    /// Requirements: 2.1, 2.2
    pub async fn create(
        &self,
        code: &str,
        description: &str,
    ) -> Result<OAuthScope, OAuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO oauth_scopes (id, code, description)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(code)
        .bind(description)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry")
                {
                    return OAuthError::InvalidScope(format!("Scope code '{}' already exists", code));
                }
            }
            OAuthError::ServerError(format!("Database error: {}", e))
        })?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| OAuthError::ServerError("Failed to fetch created scope".to_string()))
    }

    /// Find an OAuth scope by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OAuthScope>, OAuthError> {
        let scope = sqlx::query_as::<_, OAuthScope>(
            r#"
            SELECT id, code, description, is_active, created_at
            FROM oauth_scopes
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(scope)
    }

    /// Find an OAuth scope by its code
    /// Requirements: 2.2
    pub async fn find_by_code(&self, code: &str) -> Result<Option<OAuthScope>, OAuthError> {
        let scope = sqlx::query_as::<_, OAuthScope>(
            r#"
            SELECT id, code, description, is_active, created_at
            FROM oauth_scopes
            WHERE code = ?
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(scope)
    }

    /// Find an active OAuth scope by its code
    pub async fn find_active_by_code(&self, code: &str) -> Result<Option<OAuthScope>, OAuthError> {
        let scope = sqlx::query_as::<_, OAuthScope>(
            r#"
            SELECT id, code, description, is_active, created_at
            FROM oauth_scopes
            WHERE code = ? AND is_active = true
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(scope)
    }

    /// Find multiple scopes by their codes
    /// Requirements: 2.4
    pub async fn find_by_codes(&self, codes: &[String]) -> Result<Vec<OAuthScope>, OAuthError> {
        if codes.is_empty() {
            return Ok(vec![]);
        }

        let placeholders = codes.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let query = format!(
            r#"
            SELECT id, code, description, is_active, created_at
            FROM oauth_scopes
            WHERE code IN ({}) AND is_active = true
            "#,
            placeholders
        );

        let mut query_builder = sqlx::query_as::<_, OAuthScope>(&query);
        for code in codes {
            query_builder = query_builder.bind(code);
        }

        let scopes = query_builder
            .fetch_all(&self.pool)
            .await
            .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(scopes)
    }

    /// Validate that all scope codes exist and are active
    /// Requirements: 2.4
    pub async fn validate_scopes(&self, codes: &[String]) -> Result<bool, OAuthError> {
        if codes.is_empty() {
            return Ok(true);
        }

        let found_scopes = self.find_by_codes(codes).await?;
        Ok(found_scopes.len() == codes.len())
    }

    /// Update an OAuth scope
    pub async fn update(
        &self,
        id: Uuid,
        description: &str,
    ) -> Result<OAuthScope, OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_scopes
            SET description = ?
            WHERE id = ?
            "#,
        )
        .bind(description)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidScope("Scope not found".to_string()));
        }

        self.find_by_id(id)
            .await?
            .ok_or_else(|| OAuthError::InvalidScope("Scope not found".to_string()))
    }

    /// Deactivate an OAuth scope
    pub async fn deactivate(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_scopes
            SET is_active = false
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidScope("Scope not found".to_string()));
        }

        Ok(())
    }

    /// Activate an OAuth scope
    pub async fn activate(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_scopes
            SET is_active = true
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidScope("Scope not found".to_string()));
        }

        Ok(())
    }

    /// Delete an OAuth scope
    pub async fn delete(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_scopes
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidScope("Scope not found".to_string()));
        }

        Ok(())
    }

    /// List all OAuth scopes with pagination
    pub async fn list_all(&self, page: u32, limit: u32) -> Result<Vec<OAuthScope>, OAuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let scopes = sqlx::query_as::<_, OAuthScope>(
            r#"
            SELECT id, code, description, is_active, created_at
            FROM oauth_scopes
            ORDER BY code ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(scopes)
    }

    /// List all active OAuth scopes
    pub async fn list_active(&self) -> Result<Vec<OAuthScope>, OAuthError> {
        let scopes = sqlx::query_as::<_, OAuthScope>(
            r#"
            SELECT id, code, description, is_active, created_at
            FROM oauth_scopes
            WHERE is_active = true
            ORDER BY code ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(scopes)
    }

    /// Count total OAuth scopes
    pub async fn count_all(&self) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_scopes
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }
}

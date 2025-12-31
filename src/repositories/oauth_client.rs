use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::OAuthClient;

/// Repository for OAuth client database operations
/// Requirements: 1.1, 1.2
#[derive(Clone)]
pub struct OAuthClientRepository {
    pool: MySqlPool,
}

impl OAuthClientRepository {
    /// Create a new OAuthClientRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new OAuth client
    /// Requirements: 1.1, 1.2
    pub async fn create(
        &self,
        client_id: &str,
        client_secret_hash: &str,
        name: &str,
        owner_id: Uuid,
        redirect_uris: &[String],
        is_internal: bool,
    ) -> Result<OAuthClient, OAuthError> {
        let id = Uuid::new_v4();
        let redirect_uris_json = serde_json::to_value(redirect_uris)
            .map_err(|e| OAuthError::ServerError(format!("Failed to serialize redirect_uris: {}", e)))?;

        sqlx::query(
            r#"
            INSERT INTO oauth_clients (id, client_id, client_secret_hash, name, owner_id, redirect_uris, is_internal)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(client_id)
        .bind(client_secret_hash)
        .bind(name)
        .bind(owner_id.to_string())
        .bind(&redirect_uris_json)
        .bind(is_internal)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry")
                {
                    return OAuthError::InvalidRequest("Client ID already exists".to_string());
                }
            }
            OAuthError::ServerError(format!("Database error: {}", e))
        })?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| OAuthError::ServerError("Failed to fetch created client".to_string()))
    }

    /// Find an OAuth client by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OAuthClient>, OAuthError> {
        let client = sqlx::query_as::<_, OAuthClient>(
            r#"
            SELECT id, client_id, client_secret_hash, name, owner_id, redirect_uris, is_internal, is_active, created_at
            FROM oauth_clients
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(client)
    }

    /// Find an OAuth client by its client_id
    /// Requirements: 1.1
    pub async fn find_by_client_id(&self, client_id: &str) -> Result<Option<OAuthClient>, OAuthError> {
        let client = sqlx::query_as::<_, OAuthClient>(
            r#"
            SELECT id, client_id, client_secret_hash, name, owner_id, redirect_uris, is_internal, is_active, created_at
            FROM oauth_clients
            WHERE client_id = ?
            "#,
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(client)
    }

    /// Find an active OAuth client by its client_id
    pub async fn find_active_by_client_id(&self, client_id: &str) -> Result<Option<OAuthClient>, OAuthError> {
        let client = sqlx::query_as::<_, OAuthClient>(
            r#"
            SELECT id, client_id, client_secret_hash, name, owner_id, redirect_uris, is_internal, is_active, created_at
            FROM oauth_clients
            WHERE client_id = ? AND is_active = true
            "#,
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(client)
    }

    /// Update an OAuth client
    pub async fn update(
        &self,
        id: Uuid,
        name: &str,
        redirect_uris: &[String],
    ) -> Result<OAuthClient, OAuthError> {
        let redirect_uris_json = serde_json::to_value(redirect_uris)
            .map_err(|e| OAuthError::ServerError(format!("Failed to serialize redirect_uris: {}", e)))?;

        let result = sqlx::query(
            r#"
            UPDATE oauth_clients
            SET name = ?, redirect_uris = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(&redirect_uris_json)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidClient);
        }

        self.find_by_id(id)
            .await?
            .ok_or(OAuthError::InvalidClient)
    }

    /// Update client secret hash
    pub async fn update_secret(&self, id: Uuid, client_secret_hash: &str) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_clients
            SET client_secret_hash = ?
            WHERE id = ?
            "#,
        )
        .bind(client_secret_hash)
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidClient);
        }

        Ok(())
    }

    /// Deactivate an OAuth client
    pub async fn deactivate(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_clients
            SET is_active = false
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidClient);
        }

        Ok(())
    }

    /// Activate an OAuth client
    pub async fn activate(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            UPDATE oauth_clients
            SET is_active = true
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidClient);
        }

        Ok(())
    }

    /// Delete an OAuth client
    pub async fn delete(&self, id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM oauth_clients
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidClient);
        }

        Ok(())
    }

    /// List all OAuth clients with pagination
    pub async fn list_all_paginated(&self, page: u32, limit: u32) -> Result<Vec<OAuthClient>, OAuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let clients = sqlx::query_as::<_, OAuthClient>(
            r#"
            SELECT id, client_id, client_secret_hash, name, owner_id, redirect_uris, is_internal, is_active, created_at
            FROM oauth_clients
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(clients)
    }

    /// List all OAuth clients (no pagination)
    pub async fn list_all(&self) -> Result<Vec<OAuthClient>, OAuthError> {
        let clients = sqlx::query_as::<_, OAuthClient>(
            r#"
            SELECT id, client_id, client_secret_hash, name, owner_id, redirect_uris, is_internal, is_active, created_at
            FROM oauth_clients
            ORDER BY created_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(clients)
    }

    /// List OAuth clients by owner
    pub async fn list_by_owner(&self, owner_id: Uuid) -> Result<Vec<OAuthClient>, OAuthError> {
        let clients = sqlx::query_as::<_, OAuthClient>(
            r#"
            SELECT id, client_id, client_secret_hash, name, owner_id, redirect_uris, is_internal, is_active, created_at
            FROM oauth_clients
            WHERE owner_id = ?
            ORDER BY created_at DESC
            "#,
        )
        .bind(owner_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(clients)
    }

    /// Check if user is owner of the client
    pub async fn is_owner(&self, client_id: Uuid, user_id: Uuid) -> Result<bool, OAuthError> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_clients
            WHERE id = ? AND owner_id = ?
            "#,
        )
        .bind(client_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result > 0)
    }

    /// Count total OAuth clients
    pub async fn count_all(&self) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_clients
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }
}

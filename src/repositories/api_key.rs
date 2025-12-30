use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::AppError;
use crate::models::ApiKey;
use crate::utils::secret::hash_secret;

pub struct ApiKeyRepository {
    pool: MySqlPool,
}

impl ApiKeyRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        app_id: Uuid,
        name: &str,
        key: &str,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<ApiKey, AppError> {
        let id = Uuid::new_v4();
        let key_hash = hash_secret(key)?;
        let key_prefix = &key[..8.min(key.len())];
        let scopes_json = serde_json::to_string(&scopes)
            .map_err(|e| AppError::InternalError(e.into()))?;

        sqlx::query(
            r#"
            INSERT INTO api_keys (id, app_id, name, key_hash, key_prefix, scopes, expires_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(app_id.to_string())
        .bind(name)
        .bind(&key_hash)
        .bind(key_prefix)
        .bind(&scopes_json)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or(AppError::InternalError(
            anyhow::anyhow!("Failed to create API key"),
        ))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<ApiKey>, AppError> {
        let key = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(key)
    }

    pub async fn find_by_prefix(&self, prefix: &str) -> Result<Vec<ApiKey>, AppError> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE key_prefix = ? AND is_active = TRUE",
        )
        .bind(prefix)
        .fetch_all(&self.pool)
        .await?;

        Ok(keys)
    }

    pub async fn find_by_app(&self, app_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        let keys = sqlx::query_as::<_, ApiKey>(
            "SELECT * FROM api_keys WHERE app_id = ? ORDER BY created_at DESC",
        )
        .bind(app_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(keys)
    }

    pub async fn verify_key(&self, key: &str) -> Result<Option<ApiKey>, AppError> {
        if key.len() < 8 {
            return Ok(None);
        }

        let prefix = &key[..8];
        let candidates = self.find_by_prefix(prefix).await?;
        let key_hash = hash_secret(key)?;

        for candidate in candidates {
            if candidate.key_hash == key_hash && !candidate.is_expired() {
                // Update last used
                self.update_last_used(candidate.id).await?;
                return Ok(Some(candidate));
            }
        }

        Ok(None)
    }

    pub async fn update_last_used(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE api_keys SET last_used_at = NOW() WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update(
        &self,
        id: Uuid,
        name: Option<&str>,
        scopes: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> Result<ApiKey, AppError> {
        if let Some(name) = name {
            sqlx::query("UPDATE api_keys SET name = ? WHERE id = ?")
                .bind(name)
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;
        }

        if let Some(scopes) = scopes {
            let scopes_json = serde_json::to_string(&scopes)
                .map_err(|e| AppError::InternalError(e.into()))?;
            sqlx::query("UPDATE api_keys SET scopes = ? WHERE id = ?")
                .bind(scopes_json)
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;
        }

        if let Some(is_active) = is_active {
            sqlx::query("UPDATE api_keys SET is_active = ? WHERE id = ?")
                .bind(is_active)
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;
        }

        self.find_by_id(id).await?.ok_or(AppError::NotFound("API key not found".into()))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM api_keys WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn revoke(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE api_keys SET is_active = FALSE WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

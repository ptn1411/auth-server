use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::UserConsent;

/// Repository for user consent database operations
/// Requirements: 4.3, 9.3
#[derive(Clone)]
pub struct UserConsentRepository {
    pool: MySqlPool,
}

impl UserConsentRepository {
    /// Create a new UserConsentRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create or update user consent
    /// Requirements: 4.3
    pub async fn upsert(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<UserConsent, OAuthError> {
        let id = Uuid::new_v4();
        let scopes_json = serde_json::to_value(scopes)
            .map_err(|e| OAuthError::ServerError(format!("Failed to serialize scopes: {}", e)))?;

        // Use INSERT ... ON DUPLICATE KEY UPDATE for upsert
        sqlx::query(
            r#"
            INSERT INTO user_consents (id, user_id, client_id, scopes)
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE scopes = VALUES(scopes), granted_at = CURRENT_TIMESTAMP
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(client_id.to_string())
        .bind(&scopes_json)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        self.find_by_user_and_client(user_id, client_id)
            .await?
            .ok_or_else(|| OAuthError::ServerError("Failed to fetch consent".to_string()))
    }

    /// Find consent by user and client
    pub async fn find_by_user_and_client(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<UserConsent>, OAuthError> {
        let consent = sqlx::query_as::<_, UserConsent>(
            r#"
            SELECT id, user_id, client_id, scopes, granted_at
            FROM user_consents
            WHERE user_id = ? AND client_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(client_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(consent)
    }

    /// Check if user has consented to all requested scopes
    /// Requirements: 4.5
    pub async fn has_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        requested_scopes: &[String],
    ) -> Result<bool, OAuthError> {
        let consent = self.find_by_user_and_client(user_id, client_id).await?;
        
        match consent {
            Some(c) => Ok(requested_scopes.iter().all(|scope| c.scopes.contains(scope))),
            None => Ok(false),
        }
    }

    /// List all consents for a user
    /// Requirements: 9.1
    pub async fn list_by_user(&self, user_id: Uuid) -> Result<Vec<UserConsent>, OAuthError> {
        let consents = sqlx::query_as::<_, UserConsent>(
            r#"
            SELECT id, user_id, client_id, scopes, granted_at
            FROM user_consents
            WHERE user_id = ?
            ORDER BY granted_at DESC
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(consents)
    }

    /// List all consents for a client
    pub async fn list_by_client(&self, client_id: Uuid) -> Result<Vec<UserConsent>, OAuthError> {
        let consents = sqlx::query_as::<_, UserConsent>(
            r#"
            SELECT id, user_id, client_id, scopes, granted_at
            FROM user_consents
            WHERE client_id = ?
            ORDER BY granted_at DESC
            "#,
        )
        .bind(client_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(consents)
    }

    /// Delete consent for a user-client pair
    /// Requirements: 9.3
    pub async fn delete(&self, user_id: Uuid, client_id: Uuid) -> Result<(), OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_consents
            WHERE user_id = ? AND client_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(client_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        if result.rows_affected() == 0 {
            return Err(OAuthError::InvalidGrant("Consent not found".to_string()));
        }

        Ok(())
    }

    /// Delete all consents for a user
    pub async fn delete_all_for_user(&self, user_id: Uuid) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_consents
            WHERE user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Delete all consents for a client
    pub async fn delete_all_for_client(&self, client_id: Uuid) -> Result<u64, OAuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM user_consents
            WHERE client_id = ?
            "#,
        )
        .bind(client_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }

    /// Count total consents
    pub async fn count_all(&self) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM user_consents
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }
}

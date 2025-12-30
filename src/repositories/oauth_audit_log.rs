use chrono::{DateTime, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::{OAuthAuditLog, OAuthEventType};

/// Repository for OAuth audit log database operations
/// Requirements: 9.5, 10.6
#[derive(Clone)]
pub struct OAuthAuditLogRepository {
    pool: MySqlPool,
}

impl OAuthAuditLogRepository {
    /// Create a new OAuthAuditLogRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new audit log entry
    /// Requirements: 9.5, 10.6
    pub async fn create(
        &self,
        event_type: OAuthEventType,
        client_id: Option<Uuid>,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> Result<OAuthAuditLog, OAuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO oauth_audit_logs (id, event_type, client_id, user_id, ip_address, details)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(event_type.as_str())
        .bind(client_id.map(|c| c.to_string()))
        .bind(user_id.map(|u| u.to_string()))
        .bind(ip_address)
        .bind(&details)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| OAuthError::ServerError("Failed to fetch created audit log".to_string()))
    }

    /// Create a new audit log entry with string event type
    pub async fn create_with_event_str(
        &self,
        event_type: &str,
        client_id: Option<Uuid>,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> Result<OAuthAuditLog, OAuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO oauth_audit_logs (id, event_type, client_id, user_id, ip_address, details)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(event_type)
        .bind(client_id.map(|c| c.to_string()))
        .bind(user_id.map(|u| u.to_string()))
        .bind(ip_address)
        .bind(&details)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        self.find_by_id(id)
            .await?
            .ok_or_else(|| OAuthError::ServerError("Failed to fetch created audit log".to_string()))
    }

    /// Find an audit log entry by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<OAuthAuditLog>, OAuthError> {
        let log = sqlx::query_as::<_, OAuthAuditLog>(
            r#"
            SELECT id, event_type, client_id, user_id, ip_address, details, created_at
            FROM oauth_audit_logs
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(log)
    }

    /// List audit logs by user
    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Vec<OAuthAuditLog>, OAuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as::<_, OAuthAuditLog>(
            r#"
            SELECT id, event_type, client_id, user_id, ip_address, details, created_at
            FROM oauth_audit_logs
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(logs)
    }

    /// List audit logs by client
    pub async fn list_by_client(
        &self,
        client_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Vec<OAuthAuditLog>, OAuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as::<_, OAuthAuditLog>(
            r#"
            SELECT id, event_type, client_id, user_id, ip_address, details, created_at
            FROM oauth_audit_logs
            WHERE client_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(client_id.to_string())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(logs)
    }

    /// List audit logs by event type
    pub async fn list_by_event_type(
        &self,
        event_type: OAuthEventType,
        page: u32,
        limit: u32,
    ) -> Result<Vec<OAuthAuditLog>, OAuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as::<_, OAuthAuditLog>(
            r#"
            SELECT id, event_type, client_id, user_id, ip_address, details, created_at
            FROM oauth_audit_logs
            WHERE event_type = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(event_type.as_str())
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(logs)
    }

    /// List audit logs by date range
    pub async fn list_by_date_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        page: u32,
        limit: u32,
    ) -> Result<Vec<OAuthAuditLog>, OAuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as::<_, OAuthAuditLog>(
            r#"
            SELECT id, event_type, client_id, user_id, ip_address, details, created_at
            FROM oauth_audit_logs
            WHERE created_at >= ? AND created_at <= ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(start)
        .bind(end)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(logs)
    }

    /// List all audit logs with pagination
    pub async fn list_all(&self, page: u32, limit: u32) -> Result<Vec<OAuthAuditLog>, OAuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as::<_, OAuthAuditLog>(
            r#"
            SELECT id, event_type, client_id, user_id, ip_address, details, created_at
            FROM oauth_audit_logs
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(logs)
    }

    /// Count audit logs by user
    pub async fn count_by_user(&self, user_id: Uuid) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_audit_logs
            WHERE user_id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }

    /// Count audit logs by client
    pub async fn count_by_client(&self, client_id: Uuid) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_audit_logs
            WHERE client_id = ?
            "#,
        )
        .bind(client_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }

    /// Count total audit logs
    pub async fn count_all(&self) -> Result<u64, OAuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM oauth_audit_logs
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(count as u64)
    }

    /// Delete old audit logs (cleanup - keep last N days)
    pub async fn delete_older_than(&self, days: i64) -> Result<u64, OAuthError> {
        let cutoff = Utc::now() - chrono::Duration::days(days);

        let result = sqlx::query(
            r#"
            DELETE FROM oauth_audit_logs
            WHERE created_at < ?
            "#,
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .map_err(|e| OAuthError::ServerError(format!("Database error: {}", e)))?;

        Ok(result.rows_affected())
    }
}

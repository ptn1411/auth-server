use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::{AuditAction, AuditLog};

/// Repository for audit log database operations
#[derive(Clone)]
pub struct AuditLogRepository {
    pool: MySqlPool,
}

impl AuditLogRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new audit log entry
    pub async fn create(
        &self,
        user_id: Option<Uuid>,
        action: AuditAction,
        resource_type: &str,
        resource_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        details: Option<serde_json::Value>,
        status: &str,
    ) -> Result<AuditLog, AuthError> {
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, ip_address, user_agent, details, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.map(|u| u.to_string()))
        .bind(action.as_str())
        .bind(resource_type)
        .bind(resource_id.map(|r| r.to_string()))
        .bind(ip_address)
        .bind(user_agent)
        .bind(&details)
        .bind(status)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        self.find_by_id(id).await?.ok_or(AuthError::InternalError(anyhow::anyhow!("Failed to fetch created audit log")))
    }

    /// Find audit log by ID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<AuditLog>, AuthError> {
        let log = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, ip_address, user_agent, details, status, created_at
            FROM audit_logs
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(log)
    }

    /// List audit logs for a user with pagination
    pub async fn list_by_user(
        &self,
        user_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Vec<AuditLog>, AuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, ip_address, user_agent, details, status, created_at
            FROM audit_logs
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
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(logs)
    }

    /// List all audit logs with filters
    pub async fn list_all(
        &self,
        action: Option<&str>,
        resource_type: Option<&str>,
        page: u32,
        limit: u32,
    ) -> Result<Vec<AuditLog>, AuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let logs = sqlx::query_as::<_, AuditLog>(
            r#"
            SELECT id, user_id, action, resource_type, resource_id, ip_address, user_agent, details, status, created_at
            FROM audit_logs
            WHERE (? IS NULL OR action = ?)
              AND (? IS NULL OR resource_type = ?)
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(action)
        .bind(action.unwrap_or(""))
        .bind(resource_type)
        .bind(resource_type.unwrap_or(""))
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(logs)
    }

    /// Delete old audit logs (for cleanup)
    pub async fn delete_older_than_days(&self, days: i64) -> Result<u64, AuthError> {
        let result = sqlx::query(
            r#"
            DELETE FROM audit_logs
            WHERE created_at < DATE_SUB(NOW(), INTERVAL ? DAY)
            "#,
        )
        .bind(days)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.rows_affected())
    }
}

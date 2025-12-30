use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::{AuditAction, AuditLog};
use crate::repositories::AuditLogRepository;

/// Service for audit logging
#[derive(Clone)]
pub struct AuditService {
    repo: AuditLogRepository,
}

impl AuditService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            repo: AuditLogRepository::new(pool),
        }
    }

    /// Log an authentication event
    pub async fn log_auth_event(
        &self,
        user_id: Option<Uuid>,
        action: AuditAction,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        details: Option<serde_json::Value>,
        success: bool,
    ) -> Result<AuditLog, AuthError> {
        let status = if success { "success" } else { "failure" };
        self.repo
            .create(
                user_id,
                action,
                "auth",
                user_id,
                ip_address,
                user_agent,
                details,
                status,
            )
            .await
    }

    /// Log a user management event
    pub async fn log_user_event(
        &self,
        actor_id: Uuid,
        action: AuditAction,
        target_user_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> Result<AuditLog, AuthError> {
        self.repo
            .create(
                Some(actor_id),
                action,
                "user",
                Some(target_user_id),
                ip_address,
                user_agent,
                details,
                "success",
            )
            .await
    }

    /// Log a permission/role change event
    pub async fn log_permission_event(
        &self,
        actor_id: Uuid,
        action: AuditAction,
        resource_id: Uuid,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> Result<AuditLog, AuthError> {
        self.repo
            .create(
                Some(actor_id),
                action,
                "permission",
                Some(resource_id),
                ip_address,
                user_agent,
                details,
                "success",
            )
            .await
    }

    /// Log an MFA event
    pub async fn log_mfa_event(
        &self,
        user_id: Uuid,
        action: AuditAction,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        details: Option<serde_json::Value>,
        success: bool,
    ) -> Result<AuditLog, AuthError> {
        let status = if success { "success" } else { "failure" };
        self.repo
            .create(
                Some(user_id),
                action,
                "mfa",
                Some(user_id),
                ip_address,
                user_agent,
                details,
                status,
            )
            .await
    }

    /// Log a session event
    pub async fn log_session_event(
        &self,
        user_id: Uuid,
        action: AuditAction,
        session_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        details: Option<serde_json::Value>,
    ) -> Result<AuditLog, AuthError> {
        self.repo
            .create(
                Some(user_id),
                action,
                "session",
                session_id,
                ip_address,
                user_agent,
                details,
                "success",
            )
            .await
    }

    /// Get audit logs for a user
    pub async fn get_user_logs(
        &self,
        user_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<Vec<AuditLog>, AuthError> {
        self.repo.list_by_user(user_id, page, limit).await
    }

    /// Get all audit logs with filters (admin)
    pub async fn get_all_logs(
        &self,
        action: Option<&str>,
        resource_type: Option<&str>,
        page: u32,
        limit: u32,
    ) -> Result<Vec<AuditLog>, AuthError> {
        self.repo.list_all(action, resource_type, page, limit).await
    }

    /// Cleanup old audit logs
    pub async fn cleanup_old_logs(&self, retention_days: i64) -> Result<u64, AuthError> {
        self.repo.delete_older_than_days(retention_days).await
    }
}

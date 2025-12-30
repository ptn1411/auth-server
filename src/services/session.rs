use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::UserSession;
use crate::repositories::SessionRepository;
use crate::utils::password::hash_token;

/// Service for session management
#[derive(Clone)]
pub struct SessionService {
    repo: SessionRepository,
    session_expiry_days: i64,
}

impl SessionService {
    pub fn new(pool: MySqlPool, session_expiry_days: i64) -> Self {
        Self {
            repo: SessionRepository::new(pool),
            session_expiry_days,
        }
    }

    /// Create a new session for a user
    pub async fn create_session(
        &self,
        user_id: Uuid,
        refresh_token: &str,
        device_info: Option<DeviceInfo>,
    ) -> Result<UserSession, AuthError> {
        let token_hash = hash_token(refresh_token)?;
        let expires_at = Utc::now() + Duration::days(self.session_expiry_days);

        let (device_name, device_type, ip_address, user_agent) = match device_info {
            Some(info) => (info.device_name, info.device_type, info.ip_address, info.user_agent),
            None => (None, None, None, None),
        };

        self.repo
            .create(
                user_id,
                &token_hash,
                device_name.as_deref(),
                device_type.as_deref(),
                ip_address.as_deref(),
                user_agent.as_deref(),
                expires_at,
            )
            .await
    }

    /// Validate a session by refresh token
    pub async fn validate_session(&self, refresh_token: &str) -> Result<Option<UserSession>, AuthError> {
        let token_hash = hash_token(refresh_token)?;
        let session = self.repo.find_by_token_hash(&token_hash).await?;

        if let Some(ref s) = session {
            // Update last active timestamp
            self.repo.update_last_active(s.id).await?;
        }

        Ok(session)
    }

    /// Get all active sessions for a user
    pub async fn get_user_sessions(&self, user_id: Uuid) -> Result<Vec<UserSession>, AuthError> {
        self.repo.list_active_by_user(user_id).await
    }

    /// Revoke a specific session
    pub async fn revoke_session(&self, session_id: Uuid, user_id: Uuid) -> Result<(), AuthError> {
        // Verify the session belongs to the user
        if let Some(session) = self.repo.find_by_id(session_id).await? {
            if session.user_id != user_id {
                return Err(AuthError::InsufficientScope);
            }
            self.repo.revoke(session_id).await?;
        }
        Ok(())
    }

    /// Revoke all sessions for a user (logout everywhere)
    pub async fn revoke_all_sessions(&self, user_id: Uuid) -> Result<u64, AuthError> {
        self.repo.revoke_all_for_user(user_id).await
    }

    /// Revoke all sessions except the current one
    pub async fn revoke_other_sessions(
        &self,
        user_id: Uuid,
        current_session_id: Uuid,
    ) -> Result<u64, AuthError> {
        self.repo.revoke_all_except(user_id, current_session_id).await
    }

    /// Get session count for a user
    pub async fn get_session_count(&self, user_id: Uuid) -> Result<i64, AuthError> {
        self.repo.count_active_by_user(user_id).await
    }

    /// Cleanup expired sessions
    pub async fn cleanup_expired(&self) -> Result<u64, AuthError> {
        self.repo.delete_expired().await
    }
}

/// Device information for session tracking
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl DeviceInfo {
    pub fn new(
        device_name: Option<String>,
        device_type: Option<String>,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        Self {
            device_name,
            device_type,
            ip_address,
            user_agent,
        }
    }

    /// Parse device type from user agent string
    pub fn parse_device_type(user_agent: &str) -> String {
        let ua_lower = user_agent.to_lowercase();
        if ua_lower.contains("mobile") || ua_lower.contains("android") || ua_lower.contains("iphone") {
            "mobile".to_string()
        } else if ua_lower.contains("tablet") || ua_lower.contains("ipad") {
            "tablet".to_string()
        } else {
            "desktop".to_string()
        }
    }

    /// Extract device name from user agent
    pub fn parse_device_name(user_agent: &str) -> String {
        // Simple extraction - in production, use a proper user agent parser
        if user_agent.contains("Chrome") {
            "Chrome Browser".to_string()
        } else if user_agent.contains("Firefox") {
            "Firefox Browser".to_string()
        } else if user_agent.contains("Safari") {
            "Safari Browser".to_string()
        } else if user_agent.contains("Edge") {
            "Edge Browser".to_string()
        } else {
            "Unknown Device".to_string()
        }
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// OAuth Audit Log - records OAuth events for audit purposes
/// Requirement 9.5: Log all revocation events for audit purposes
/// Requirement 10.6: Log all authorization events for audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthAuditLog {
    pub id: Uuid,
    pub event_type: String,
    pub client_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct OAuthAuditLogRow {
    pub id: String,
    pub event_type: String,
    pub client_id: Option<String>,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub details: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl From<OAuthAuditLogRow> for OAuthAuditLog {
    fn from(row: OAuthAuditLogRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            event_type: row.event_type,
            client_id: row.client_id.and_then(|id| Uuid::parse_str(&id).ok()),
            user_id: row.user_id.and_then(|id| Uuid::parse_str(&id).ok()),
            ip_address: row.ip_address,
            details: row.details,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for OAuthAuditLog {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let log_row = OAuthAuditLogRow::from_row(row)?;
        Ok(OAuthAuditLog::from(log_row))
    }
}

/// Event types for OAuth audit logging
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OAuthEventType {
    /// Client registration
    ClientRegistered,
    /// Authorization request initiated
    AuthorizationRequested,
    /// User granted consent
    ConsentGranted,
    /// User denied consent
    ConsentDenied,
    /// Authorization code issued
    AuthorizationCodeIssued,
    /// Token issued (access + refresh)
    TokenIssued,
    /// Token refreshed
    TokenRefreshed,
    /// Token revoked
    TokenRevoked,
    /// Consent revoked
    ConsentRevoked,
    /// Invalid token attempt
    InvalidTokenAttempt,
    /// Invalid client credentials
    InvalidClientCredentials,
}

impl OAuthEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            OAuthEventType::ClientRegistered => "client_registered",
            OAuthEventType::AuthorizationRequested => "authorization_requested",
            OAuthEventType::ConsentGranted => "consent_granted",
            OAuthEventType::ConsentDenied => "consent_denied",
            OAuthEventType::AuthorizationCodeIssued => "authorization_code_issued",
            OAuthEventType::TokenIssued => "token_issued",
            OAuthEventType::TokenRefreshed => "token_refreshed",
            OAuthEventType::TokenRevoked => "token_revoked",
            OAuthEventType::ConsentRevoked => "consent_revoked",
            OAuthEventType::InvalidTokenAttempt => "invalid_token_attempt",
            OAuthEventType::InvalidClientCredentials => "invalid_client_credentials",
        }
    }
}

impl std::fmt::Display for OAuthEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ============================================================================
// Audit Log Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditAction {
    Login,
    LoginFailed,
    Logout,
    Register,
    PasswordChange,
    PasswordReset,
    PasswordResetRequest,
    TokenRefresh,
    AccountLocked,
    AccountUnlocked,
    MfaEnabled,
    MfaDisabled,
    MfaVerified,
    MfaFailed,
    SessionRevoked,
    RoleAssigned,
    RoleRemoved,
    PermissionChanged,
    ProfileUpdated,
    // Admin actions
    UserUpdated,
    UserDeleted,
    UserActivated,
    UserDeactivated,
    AppUpdated,
    AppDeleted,
}

impl AuditAction {
    pub fn as_str(&self) -> &'static str {
        match self {
            AuditAction::Login => "login",
            AuditAction::LoginFailed => "login_failed",
            AuditAction::Logout => "logout",
            AuditAction::Register => "register",
            AuditAction::PasswordChange => "password_change",
            AuditAction::PasswordReset => "password_reset",
            AuditAction::PasswordResetRequest => "password_reset_request",
            AuditAction::TokenRefresh => "token_refresh",
            AuditAction::AccountLocked => "account_locked",
            AuditAction::AccountUnlocked => "account_unlocked",
            AuditAction::MfaEnabled => "mfa_enabled",
            AuditAction::MfaDisabled => "mfa_disabled",
            AuditAction::MfaVerified => "mfa_verified",
            AuditAction::MfaFailed => "mfa_failed",
            AuditAction::SessionRevoked => "session_revoked",
            AuditAction::RoleAssigned => "role_assigned",
            AuditAction::RoleRemoved => "role_removed",
            AuditAction::PermissionChanged => "permission_changed",
            AuditAction::ProfileUpdated => "profile_updated",
            AuditAction::UserUpdated => "user_updated",
            AuditAction::UserDeleted => "user_deleted",
            AuditAction::UserActivated => "user_activated",
            AuditAction::UserDeactivated => "user_deactivated",
            AuditAction::AppUpdated => "app_updated",
            AuditAction::AppDeleted => "app_deleted",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuditLogRow {
    pub id: String,
    pub user_id: Option<String>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: Option<serde_json::Value>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

impl From<AuditLogRow> for AuditLog {
    fn from(row: AuditLogRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: row.user_id.and_then(|s| Uuid::parse_str(&s).ok()),
            action: row.action,
            resource_type: row.resource_type,
            resource_id: row.resource_id.and_then(|s| Uuid::parse_str(&s).ok()),
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            details: row.details,
            status: row.status,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for AuditLog {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let audit_row = AuditLogRow::from_row(row)?;
        Ok(AuditLog::from(audit_row))
    }
}

// ============================================================================
// User Session Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub refresh_token_hash: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserSessionRow {
    pub id: String,
    pub user_id: String,
    pub refresh_token_hash: String,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_active_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_revoked: bool,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<UserSessionRow> for UserSession {
    fn from(row: UserSessionRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            refresh_token_hash: row.refresh_token_hash,
            device_name: row.device_name,
            device_type: row.device_type,
            ip_address: row.ip_address,
            user_agent: row.user_agent,
            last_active_at: row.last_active_at,
            expires_at: row.expires_at,
            is_revoked: row.is_revoked,
            revoked_at: row.revoked_at,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for UserSession {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let session_row = UserSessionRow::from_row(row)?;
        Ok(UserSession::from(session_row))
    }
}

// ============================================================================
// Revoked Token Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokedToken {
    pub id: Uuid,
    pub token_hash: String,
    pub token_type: String,
    pub user_id: Option<Uuid>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: DateTime<Utc>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct RevokedTokenRow {
    pub id: String,
    pub token_hash: String,
    pub token_type: String,
    pub user_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl From<RevokedTokenRow> for RevokedToken {
    fn from(row: RevokedTokenRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            token_hash: row.token_hash,
            token_type: row.token_type,
            user_id: row.user_id.and_then(|s| Uuid::parse_str(&s).ok()),
            expires_at: row.expires_at,
            revoked_at: row.revoked_at,
            reason: row.reason,
        }
    }
}

// ============================================================================
// Rate Limit Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitEntry {
    pub id: Uuid,
    pub identifier: String,
    pub endpoint: String,
    pub request_count: i32,
    pub window_start: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct RateLimitEntryRow {
    pub id: String,
    pub identifier: String,
    pub endpoint: String,
    pub request_count: i32,
    pub window_start: DateTime<Utc>,
}

impl From<RateLimitEntryRow> for RateLimitEntry {
    fn from(row: RateLimitEntryRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            identifier: row.identifier,
            endpoint: row.endpoint,
            request_count: row.request_count,
            window_start: row.window_start,
        }
    }
}

// ============================================================================
// MFA Models
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MfaMethodType {
    Totp,
    Sms,
    Email,
}

impl MfaMethodType {
    pub fn as_str(&self) -> &'static str {
        match self {
            MfaMethodType::Totp => "totp",
            MfaMethodType::Sms => "sms",
            MfaMethodType::Email => "email",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "totp" => Some(MfaMethodType::Totp),
            "sms" => Some(MfaMethodType::Sms),
            "email" => Some(MfaMethodType::Email),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMfaMethod {
    pub id: Uuid,
    pub user_id: Uuid,
    pub method_type: String,
    pub secret_encrypted: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub is_verified: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserMfaMethodRow {
    pub id: String,
    pub user_id: String,
    pub method_type: String,
    pub secret_encrypted: Option<String>,
    pub phone_number: Option<String>,
    pub email: Option<String>,
    pub is_primary: bool,
    pub is_verified: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<UserMfaMethodRow> for UserMfaMethod {
    fn from(row: UserMfaMethodRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            method_type: row.method_type,
            secret_encrypted: row.secret_encrypted,
            phone_number: row.phone_number,
            email: row.email,
            is_primary: row.is_primary,
            is_verified: row.is_verified,
            last_used_at: row.last_used_at,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for UserMfaMethod {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let mfa_row = UserMfaMethodRow::from_row(row)?;
        Ok(UserMfaMethod::from(mfa_row))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMfaBackupCode {
    pub id: Uuid,
    pub user_id: Uuid,
    pub code_hash: String,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserMfaBackupCodeRow {
    pub id: String,
    pub user_id: String,
    pub code_hash: String,
    pub is_used: bool,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<UserMfaBackupCodeRow> for UserMfaBackupCode {
    fn from(row: UserMfaBackupCodeRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            code_hash: row.code_hash,
            is_used: row.is_used,
            used_at: row.used_at,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for UserMfaBackupCode {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let code_row = UserMfaBackupCodeRow::from_row(row)?;
        Ok(UserMfaBackupCode::from(code_row))
    }
}

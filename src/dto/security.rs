use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Logout / Token Revocation DTOs
// ============================================================================

/// Logout request
#[derive(Debug, Deserialize)]
pub struct LogoutRequest {
    /// Optional: revoke all sessions (logout everywhere)
    #[serde(default)]
    pub all_sessions: bool,
}

/// Logout response
#[derive(Debug, Serialize)]
pub struct LogoutResponse {
    pub message: String,
    pub sessions_revoked: u64,
}

// ============================================================================
// Session Management DTOs
// ============================================================================

/// Session info response (sanitized for API)
#[derive(Debug, Serialize)]
pub struct SessionResponse {
    pub id: Uuid,
    pub device_name: Option<String>,
    pub device_type: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub last_used_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub is_current: bool,
}

/// List sessions response
#[derive(Debug, Serialize)]
pub struct ListSessionsResponse {
    pub sessions: Vec<SessionResponse>,
    pub total: usize,
}

/// Revoke session request
#[derive(Debug, Deserialize)]
pub struct RevokeSessionRequest {
    pub session_id: Uuid,
}

/// Revoke sessions response
#[derive(Debug, Serialize)]
pub struct RevokeSessionsResponse {
    pub message: String,
    pub revoked_count: u64,
}

// ============================================================================
// MFA DTOs
// ============================================================================

/// Setup TOTP request
#[derive(Debug, Deserialize)]
pub struct SetupTotpRequest {
    // No fields needed - uses authenticated user
}

/// Setup TOTP response
#[derive(Debug, Serialize)]
pub struct SetupTotpResponse {
    pub method_id: Uuid,
    pub secret: String,
    pub provisioning_uri: String,
    pub qr_code_data: Option<String>, // Base64 encoded QR code image
}

/// Verify TOTP setup request
#[derive(Debug, Deserialize)]
pub struct VerifyTotpSetupRequest {
    pub method_id: Uuid,
    pub code: String,
}

/// Verify TOTP setup response
#[derive(Debug, Serialize)]
pub struct VerifyTotpSetupResponse {
    pub message: String,
    pub backup_codes: Vec<String>,
}

/// Verify MFA request (during login)
#[derive(Debug, Deserialize)]
pub struct VerifyMfaRequest {
    pub code: String,
    /// If true, treat code as backup code
    #[serde(default)]
    pub is_backup_code: bool,
}

/// Verify MFA response
#[derive(Debug, Serialize)]
pub struct VerifyMfaResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

/// MFA method info response
#[derive(Debug, Serialize)]
pub struct MfaMethodResponse {
    pub id: Uuid,
    pub method_type: String,
    pub is_primary: bool,
    pub is_verified: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// List MFA methods response
#[derive(Debug, Serialize)]
pub struct ListMfaMethodsResponse {
    pub methods: Vec<MfaMethodResponse>,
    pub mfa_enabled: bool,
    pub backup_codes_remaining: i64,
}

/// Disable MFA request
#[derive(Debug, Deserialize)]
pub struct DisableMfaRequest {
    /// Current password for verification
    pub password: String,
    /// Optional: specific method ID to disable (if not provided, disables all)
    pub method_id: Option<Uuid>,
}

/// Regenerate backup codes request
#[derive(Debug, Deserialize)]
pub struct RegenerateBackupCodesRequest {
    /// Current password for verification
    pub password: String,
}

/// Regenerate backup codes response
#[derive(Debug, Serialize)]
pub struct RegenerateBackupCodesResponse {
    pub backup_codes: Vec<String>,
    pub message: String,
}

// ============================================================================
// Audit Log DTOs
// ============================================================================

/// Audit log entry response
#[derive(Debug, Serialize)]
pub struct AuditLogResponse {
    pub id: Uuid,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub details: Option<serde_json::Value>,
}

/// List audit logs response
#[derive(Debug, Serialize)]
pub struct ListAuditLogsResponse {
    pub logs: Vec<AuditLogResponse>,
    pub page: u32,
    pub limit: u32,
    pub total: u64,
}

/// Audit log query parameters
#[derive(Debug, Deserialize)]
pub struct AuditLogQuery {
    pub action: Option<String>,
    pub resource_type: Option<String>,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 { 1 }
fn default_limit() -> u32 { 20 }

// ============================================================================
// Rate Limit DTOs
// ============================================================================

/// Rate limit error response
#[derive(Debug, Serialize)]
pub struct RateLimitErrorResponse {
    pub error: String,
    pub message: String,
    pub retry_after_seconds: i64,
    pub limit: i32,
    pub remaining: i32,
}

// ============================================================================
// Account Lockout DTOs
// ============================================================================

/// Account locked error response
#[derive(Debug, Serialize)]
pub struct AccountLockedResponse {
    pub error: String,
    pub message: String,
    pub locked_until: DateTime<Utc>,
    pub remaining_seconds: i64,
}

/// Unlock account request (admin)
#[derive(Debug, Deserialize)]
pub struct UnlockAccountRequest {
    pub user_id: Uuid,
}

// ============================================================================
// Login with MFA DTOs
// ============================================================================

/// Login response when MFA is required
#[derive(Debug, Serialize)]
pub struct MfaRequiredResponse {
    pub mfa_required: bool,
    pub mfa_token: String, // Temporary token to complete MFA
    pub available_methods: Vec<String>,
}

/// Complete MFA login request
#[derive(Debug, Deserialize)]
pub struct CompleteMfaLoginRequest {
    pub mfa_token: String,
    pub code: String,
    #[serde(default)]
    pub is_backup_code: bool,
}

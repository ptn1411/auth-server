use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::UserAppStatus;

/// Request to register a user to an app
#[derive(Debug, Deserialize)]
pub struct RegisterToAppRequest {
    pub app_id: Uuid,
}

/// Request to ban a user from an app
#[derive(Debug, Deserialize)]
pub struct BanUserRequest {
    pub reason: Option<String>,
}

/// User information within an app context
#[derive(Debug, Serialize)]
pub struct AppUserInfo {
    pub user_id: Uuid,
    pub email: String,
    pub status: UserAppStatus,
    pub roles: Vec<String>,
    pub banned_at: Option<DateTime<Utc>>,
    pub banned_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// User-App association response (for API Key routes)
#[derive(Debug, Serialize)]
pub struct UserAppResponse {
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub email: String,
    pub status: String,
    pub roles: Vec<String>,
    pub banned_at: Option<DateTime<Utc>>,
    pub banned_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Generic paginated response wrapper
#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub page: u32,
    pub limit: u32,
    pub total: u64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(data: Vec<T>, page: u32, limit: u32, total: u64) -> Self {
        Self {
            data,
            page,
            limit,
            total,
        }
    }
}

/// Query parameters for pagination
#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_page() -> u32 {
    1
}

fn default_limit() -> u32 {
    20
}

impl Default for PaginationQuery {
    fn default() -> Self {
        Self {
            page: default_page(),
            limit: default_limit(),
        }
    }
}

/// Query parameters for user search/filter
#[derive(Debug, Deserialize)]
pub struct UserSearchQuery {
    /// Search by email (partial match)
    pub email: Option<String>,
    /// Search by name (partial match)
    pub name: Option<String>,
    /// Filter by active status
    pub is_active: Option<bool>,
    /// Filter by email verified status
    pub email_verified: Option<bool>,
    /// Filter by system admin status
    pub is_system_admin: Option<bool>,
    /// Sort field (email, name, created_at)
    #[serde(default = "default_sort_field")]
    pub sort_by: String,
    /// Sort order (asc, desc)
    #[serde(default = "default_sort_order")]
    pub sort_order: String,
    #[serde(default = "default_page")]
    pub page: u32,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_sort_field() -> String {
    "created_at".to_string()
}

fn default_sort_order() -> String {
    "desc".to_string()
}

impl Default for UserSearchQuery {
    fn default() -> Self {
        Self {
            email: None,
            name: None,
            is_active: None,
            email_verified: None,
            is_system_admin: None,
            sort_by: default_sort_field(),
            sort_order: default_sort_order(),
            page: default_page(),
            limit: default_limit(),
        }
    }
}

/// User info for search results
#[derive(Debug, Serialize)]
pub struct UserSearchResult {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub is_system_admin: bool,
    pub created_at: DateTime<Utc>,
}

/// Bulk role assignment request
#[derive(Debug, Deserialize)]
pub struct BulkRoleAssignmentRequest {
    pub user_ids: Vec<Uuid>,
    pub role_id: Uuid,
    pub app_id: Uuid,
}

/// Bulk role assignment response
#[derive(Debug, Serialize)]
pub struct BulkOperationResponse {
    pub success_count: u32,
    pub failed_count: u32,
    pub errors: Vec<BulkOperationError>,
}

#[derive(Debug, Serialize)]
pub struct BulkOperationError {
    pub user_id: Uuid,
    pub error: String,
}

/// User export format
#[derive(Debug, Serialize, Deserialize)]
pub struct UserExportData {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub is_system_admin: bool,
    pub created_at: DateTime<Utc>,
}

/// User import request
#[derive(Debug, Deserialize)]
pub struct UserImportRequest {
    pub email: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub password: String,
}

/// Bulk import response
#[derive(Debug, Serialize)]
pub struct BulkImportResponse {
    pub imported_count: u32,
    pub failed_count: u32,
    pub errors: Vec<ImportError>,
}

#[derive(Debug, Serialize)]
pub struct ImportError {
    pub row: u32,
    pub email: String,
    pub error: String,
}

// ============================================================================
// Admin DTOs
// ============================================================================

/// Request to update user by admin
#[derive(Debug, Deserialize)]
pub struct AdminUpdateUserRequest {
    pub email: Option<String>,
    pub is_active: Option<bool>,
    pub is_system_admin: Option<bool>,
    pub email_verified: Option<bool>,
}

/// Request to update app by admin
#[derive(Debug, Deserialize)]
pub struct AdminUpdateAppRequest {
    pub name: Option<String>,
    pub owner_id: Option<Uuid>,
}

/// Detailed user response for admin
#[derive(Debug, Serialize)]
pub struct AdminUserDetailResponse {
    pub id: Uuid,
    pub email: String,
    pub name: Option<String>,
    pub phone: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub email_verified: bool,
    pub is_system_admin: bool,
    pub mfa_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
}

/// Detailed app response for admin
#[derive(Debug, Serialize)]
pub struct AdminAppDetailResponse {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub owner_id: Option<Uuid>,
    pub has_secret: bool,
}

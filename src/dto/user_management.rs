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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User-App association status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserAppStatus {
    Active,
    Banned,
}

impl UserAppStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserAppStatus::Active => "active",
            UserAppStatus::Banned => "banned",
        }
    }
}

impl std::fmt::Display for UserAppStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for UserAppStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(UserAppStatus::Active),
            "banned" => Ok(UserAppStatus::Banned),
            _ => Err(format!("Invalid UserAppStatus: {}", s)),
        }
    }
}

/// UserApp domain model - represents user-app association
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserApp {
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub status: UserAppStatus,
    pub banned_at: Option<DateTime<Utc>>,
    pub banned_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct UserAppRow {
    pub user_id: String,
    pub app_id: String,
    pub status: String,
    pub banned_at: Option<DateTime<Utc>>,
    pub banned_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<UserAppRow> for UserApp {
    fn from(row: UserAppRow) -> Self {
        Self {
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            app_id: Uuid::parse_str(&row.app_id).unwrap_or_default(),
            status: row.status.parse().unwrap_or(UserAppStatus::Active),
            banned_at: row.banned_at,
            banned_reason: row.banned_reason,
            created_at: row.created_at,
        }
    }
}

// Implement FromRow for UserApp by delegating to UserAppRow
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for UserApp {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let user_app_row = UserAppRow::from_row(row)?;
        Ok(UserApp::from(user_app_row))
    }
}

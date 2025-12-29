use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User domain model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub is_system_admin: bool,
    pub created_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub is_active: bool,
    pub email_verified: bool,
    pub is_system_admin: bool,
    pub created_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            email: row.email,
            password_hash: row.password_hash,
            is_active: row.is_active,
            email_verified: row.email_verified,
            is_system_admin: row.is_system_admin,
            created_at: row.created_at,
        }
    }
}

// Implement FromRow for User by delegating to UserRow
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for User {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let user_row = UserRow::from_row(row)?;
        Ok(User::from(user_row))
    }
}

/// Refresh token stored in database
#[derive(Debug, Clone)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Password reset token stored in database
#[derive(Debug, Clone)]
pub struct PasswordResetToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

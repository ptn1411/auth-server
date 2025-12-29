use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Role domain model - scoped to a specific App
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: Uuid,
    pub app_id: Uuid,
    pub name: String,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct RoleRow {
    pub id: String,
    pub app_id: String,
    pub name: String,
}

impl From<RoleRow> for Role {
    fn from(row: RoleRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            app_id: Uuid::parse_str(&row.app_id).unwrap_or_default(),
            name: row.name,
        }
    }
}

// Implement FromRow for Role by delegating to RoleRow
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for Role {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let role_row = RoleRow::from_row(row)?;
        Ok(Role::from(role_row))
    }
}

/// User-App-Role association
#[derive(Debug, Clone)]
pub struct UserAppRole {
    pub user_id: Uuid,
    pub app_id: Uuid,
    pub role_id: Uuid,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct UserAppRoleRow {
    pub user_id: String,
    pub app_id: String,
    pub role_id: String,
}

impl From<UserAppRoleRow> for UserAppRole {
    fn from(row: UserAppRoleRow) -> Self {
        Self {
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            app_id: Uuid::parse_str(&row.app_id).unwrap_or_default(),
            role_id: Uuid::parse_str(&row.role_id).unwrap_or_default(),
        }
    }
}

// Implement FromRow for UserAppRole by delegating to UserAppRoleRow
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for UserAppRole {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let uar_row = UserAppRoleRow::from_row(row)?;
        Ok(UserAppRole::from(uar_row))
    }
}

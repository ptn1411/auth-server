use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Permission domain model - scoped to a specific App
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: Uuid,
    pub app_id: Uuid,
    pub code: String,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct PermissionRow {
    pub id: String,
    pub app_id: String,
    pub code: String,
}

impl From<PermissionRow> for Permission {
    fn from(row: PermissionRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            app_id: Uuid::parse_str(&row.app_id).unwrap_or_default(),
            code: row.code,
        }
    }
}

// Implement FromRow for Permission by delegating to PermissionRow
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for Permission {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let perm_row = PermissionRow::from_row(row)?;
        Ok(Permission::from(perm_row))
    }
}

/// Role-Permission association
#[derive(Debug, Clone)]
pub struct RolePermission {
    pub role_id: Uuid,
    pub permission_id: Uuid,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct RolePermissionRow {
    pub role_id: String,
    pub permission_id: String,
}

impl From<RolePermissionRow> for RolePermission {
    fn from(row: RolePermissionRow) -> Self {
        Self {
            role_id: Uuid::parse_str(&row.role_id).unwrap_or_default(),
            permission_id: Uuid::parse_str(&row.permission_id).unwrap_or_default(),
        }
    }
}

// Implement FromRow for RolePermission by delegating to RolePermissionRow
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for RolePermission {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let rp_row = RolePermissionRow::from_row(row)?;
        Ok(RolePermission::from(rp_row))
    }
}

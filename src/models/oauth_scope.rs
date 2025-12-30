use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// OAuth Scope - defines a permission scope
/// Requirement 2.1: Support defining scopes with unique code and description
/// Requirement 2.2: Enforce unique scope codes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthScope {
    pub id: Uuid,
    pub code: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct OAuthScopeRow {
    pub id: String,
    pub code: String,
    pub description: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<OAuthScopeRow> for OAuthScope {
    fn from(row: OAuthScopeRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            code: row.code,
            description: row.description,
            is_active: row.is_active,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for OAuthScope {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let scope_row = OAuthScopeRow::from_row(row)?;
        Ok(OAuthScope::from(scope_row))
    }
}

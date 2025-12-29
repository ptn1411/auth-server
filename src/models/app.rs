use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// App domain model - represents a client application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct App {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub owner_id: Option<Uuid>,
    #[serde(skip_serializing)]
    pub secret_hash: Option<String>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct AppRow {
    pub id: String,
    pub code: String,
    pub name: String,
    pub owner_id: Option<String>,
    pub secret_hash: Option<String>,
}

impl From<AppRow> for App {
    fn from(row: AppRow) -> Self {
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            code: row.code,
            name: row.name,
            owner_id: row.owner_id.and_then(|id| Uuid::parse_str(&id).ok()),
            secret_hash: row.secret_hash,
        }
    }
}

// Implement FromRow for App by delegating to AppRow
impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for App {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let app_row = AppRow::from_row(row)?;
        Ok(App::from(app_row))
    }
}

impl App {
    /// Check if the app has a secret configured
    pub fn has_secret(&self) -> bool {
        self.secret_hash.is_some()
    }
}

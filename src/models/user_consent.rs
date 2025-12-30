use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User Consent - records user's consent for a client
/// Requirement 4.3: Store consent record with user_id, client_id, scopes, and timestamp
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConsent {
    pub id: Uuid,
    pub user_id: Uuid,
    pub client_id: Uuid,
    pub scopes: Vec<String>,
    pub granted_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct UserConsentRow {
    pub id: String,
    pub user_id: String,
    pub client_id: String,
    pub scopes: serde_json::Value,
    pub granted_at: DateTime<Utc>,
}

impl From<UserConsentRow> for UserConsent {
    fn from(row: UserConsentRow) -> Self {
        let scopes: Vec<String> = serde_json::from_value(row.scopes)
            .unwrap_or_default();
        
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            client_id: Uuid::parse_str(&row.client_id).unwrap_or_default(),
            scopes,
            granted_at: row.granted_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for UserConsent {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let consent_row = UserConsentRow::from_row(row)?;
        Ok(UserConsent::from(consent_row))
    }
}

impl UserConsent {
    /// Check if consent covers all requested scopes
    pub fn covers_scopes(&self, requested_scopes: &[String]) -> bool {
        requested_scopes.iter().all(|scope| self.scopes.contains(scope))
    }
}

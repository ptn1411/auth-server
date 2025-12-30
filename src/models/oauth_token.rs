use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// OAuth Token - stores issued tokens
/// Requirement 5.1: Issue access_token and refresh_token
/// Requirement 5.6: Hash tokens before storing in the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub client_id: Uuid,
    #[serde(skip_serializing)]
    pub access_token_hash: String,
    #[serde(skip_serializing)]
    pub refresh_token_hash: Option<String>,
    pub scopes: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct OAuthTokenRow {
    pub id: String,
    pub user_id: Option<String>,
    pub client_id: String,
    pub access_token_hash: String,
    pub refresh_token_hash: Option<String>,
    pub scopes: serde_json::Value,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

impl From<OAuthTokenRow> for OAuthToken {
    fn from(row: OAuthTokenRow) -> Self {
        let scopes: Vec<String> = serde_json::from_value(row.scopes)
            .unwrap_or_default();
        
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            user_id: row.user_id.and_then(|id| Uuid::parse_str(&id).ok()),
            client_id: Uuid::parse_str(&row.client_id).unwrap_or_default(),
            access_token_hash: row.access_token_hash,
            refresh_token_hash: row.refresh_token_hash,
            scopes,
            expires_at: row.expires_at,
            revoked: row.revoked,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for OAuthToken {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let token_row = OAuthTokenRow::from_row(row)?;
        Ok(OAuthToken::from(token_row))
    }
}

impl OAuthToken {
    /// Check if the token has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the token is valid (not revoked and not expired)
    pub fn is_valid(&self) -> bool {
        !self.revoked && !self.is_expired()
    }

    /// Check if the token has a specific scope
    pub fn has_scope(&self, scope: &str) -> bool {
        self.scopes.iter().any(|s| s == scope)
    }

    /// Check if the token has all required scopes
    pub fn has_all_scopes(&self, required_scopes: &[String]) -> bool {
        required_scopes.iter().all(|scope| self.has_scope(scope))
    }
}

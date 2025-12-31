use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// OAuth Client - represents an external or internal application
/// Requirement 1.1: Store client_id, client_secret, redirect_uris, and is_internal flag
/// Requirement 1.5: Distinguish between Internal_App and External_App
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthClient {
    pub id: Uuid,
    pub client_id: String,
    #[serde(skip_serializing)]
    pub client_secret_hash: String,
    pub name: String,
    pub owner_id: Option<Uuid>,
    pub redirect_uris: Vec<String>,
    pub is_internal: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct OAuthClientRow {
    pub id: String,
    pub client_id: String,
    pub client_secret_hash: String,
    pub name: String,
    pub owner_id: Option<String>,
    pub redirect_uris: serde_json::Value,
    pub is_internal: bool,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<OAuthClientRow> for OAuthClient {
    fn from(row: OAuthClientRow) -> Self {
        let redirect_uris: Vec<String> = serde_json::from_value(row.redirect_uris)
            .unwrap_or_default();
        
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            client_id: row.client_id,
            client_secret_hash: row.client_secret_hash,
            name: row.name,
            owner_id: row.owner_id.and_then(|id| Uuid::parse_str(&id).ok()),
            redirect_uris,
            is_internal: row.is_internal,
            is_active: row.is_active,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for OAuthClient {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let oauth_client_row = OAuthClientRow::from_row(row)?;
        Ok(OAuthClient::from(oauth_client_row))
    }
}

impl OAuthClient {
    /// Check if this is an external app (not internal)
    pub fn is_external(&self) -> bool {
        !self.is_internal
    }

    /// Check if a redirect URI is registered for this client
    pub fn has_redirect_uri(&self, uri: &str) -> bool {
        self.redirect_uris.iter().any(|u| u == uri)
    }

    /// Check if a user is the owner of this client
    pub fn is_owner(&self, user_id: Uuid) -> bool {
        self.owner_id == Some(user_id)
    }
}

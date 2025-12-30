use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Authorization Code - temporary code for token exchange
/// Requirement 3.4: Generate short-lived authorization code (max 10 minutes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationCode {
    pub id: Uuid,
    pub code_hash: String,
    pub client_id: Uuid,
    pub user_id: Uuid,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

/// Row type for MySQL query results
#[derive(Debug, Clone, FromRow)]
pub struct AuthorizationCodeRow {
    pub id: String,
    pub code_hash: String,
    pub client_id: String,
    pub user_id: String,
    pub redirect_uri: String,
    pub scopes: serde_json::Value,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub expires_at: DateTime<Utc>,
    pub used: bool,
    pub created_at: DateTime<Utc>,
}

impl From<AuthorizationCodeRow> for AuthorizationCode {
    fn from(row: AuthorizationCodeRow) -> Self {
        let scopes: Vec<String> = serde_json::from_value(row.scopes)
            .unwrap_or_default();
        
        Self {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            code_hash: row.code_hash,
            client_id: Uuid::parse_str(&row.client_id).unwrap_or_default(),
            user_id: Uuid::parse_str(&row.user_id).unwrap_or_default(),
            redirect_uri: row.redirect_uri,
            scopes,
            code_challenge: row.code_challenge,
            code_challenge_method: row.code_challenge_method,
            expires_at: row.expires_at,
            used: row.used,
            created_at: row.created_at,
        }
    }
}

impl<'r> sqlx::FromRow<'r, sqlx::mysql::MySqlRow> for AuthorizationCode {
    fn from_row(row: &'r sqlx::mysql::MySqlRow) -> Result<Self, sqlx::Error> {
        let code_row = AuthorizationCodeRow::from_row(row)?;
        Ok(AuthorizationCode::from(code_row))
    }
}

impl AuthorizationCode {
    /// Check if the authorization code has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if the authorization code is valid (not used and not expired)
    pub fn is_valid(&self) -> bool {
        !self.used && !self.is_expired()
    }
}

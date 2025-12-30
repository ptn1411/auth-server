use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebAuthnCredential {
    pub id: Uuid,
    pub user_id: Uuid,
    #[sqlx(default)]
    pub credential_id: Vec<u8>,
    #[sqlx(default)]
    pub public_key: Vec<u8>,
    pub counter: u32,
    #[sqlx(default)]
    pub aaguid: Option<Vec<u8>>,
    pub device_name: Option<String>,
    pub transports: Option<sqlx::types::Json<Vec<String>>>,
    pub is_active: bool,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
pub enum ChallengeType {
    Registration,
    Authentication,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebAuthnChallenge {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    #[sqlx(default)]
    pub challenge: Vec<u8>,
    pub challenge_type: ChallengeType,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl WebAuthnChallenge {
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}

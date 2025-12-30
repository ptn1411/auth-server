use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebAuthnCredential {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChallengeType {
    #[serde(rename = "registration")]
    Registration,
    #[serde(rename = "authentication")]
    Authentication,
}

impl TryFrom<String> for ChallengeType {
    type Error = String;
    
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "registration" => Ok(ChallengeType::Registration),
            "authentication" => Ok(ChallengeType::Authentication),
            _ => Ok(ChallengeType::Authentication), // Default
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebAuthnChallenge {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    pub user_id: Option<String>,
    #[sqlx(default)]
    pub challenge: Vec<u8>,
    #[sqlx(try_from = "String")]
    pub challenge_type: ChallengeType,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl WebAuthnChallenge {
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
    
    pub fn user_id_uuid(&self) -> Option<Uuid> {
        self.user_id.as_ref().and_then(|s| Uuid::parse_str(s).ok())
    }
}

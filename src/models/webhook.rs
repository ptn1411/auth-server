use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

fn parse_uuid(s: &str) -> Uuid {
    Uuid::parse_str(s).unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Webhook {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub app_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: sqlx::types::Json<Vec<String>>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    #[sqlx(try_from = "String")]
    pub id: Uuid,
    #[sqlx(try_from = "String")]
    pub webhook_id: Uuid,
    pub event_type: String,
    pub payload: sqlx::types::Json<serde_json::Value>,
    pub response_status: Option<i32>,
    pub response_body: Option<String>,
    pub attempts: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WebhookEvent {
    #[serde(rename = "user.registered")]
    UserRegistered,
    #[serde(rename = "user.login")]
    UserLogin,
    #[serde(rename = "user.logout")]
    UserLogout,
    #[serde(rename = "user.password_changed")]
    UserPasswordChanged,
    #[serde(rename = "user.password_reset")]
    UserPasswordReset,
    #[serde(rename = "user.email_verified")]
    UserEmailVerified,
    #[serde(rename = "user.mfa_enabled")]
    UserMfaEnabled,
    #[serde(rename = "user.mfa_disabled")]
    UserMfaDisabled,
    #[serde(rename = "user.locked")]
    UserLocked,
    #[serde(rename = "user.unlocked")]
    UserUnlocked,
    #[serde(rename = "user.deactivated")]
    UserDeactivated,
    #[serde(rename = "user.activated")]
    UserActivated,
    #[serde(rename = "user.app.joined")]
    UserAppJoined,
    #[serde(rename = "user.app.banned")]
    UserAppBanned,
    #[serde(rename = "user.app.unbanned")]
    UserAppUnbanned,
    #[serde(rename = "user.app.removed")]
    UserAppRemoved,
    #[serde(rename = "app.created")]
    AppCreated,
    #[serde(rename = "app.secret_regenerated")]
    AppSecretRegenerated,
    #[serde(rename = "role.assigned")]
    RoleAssigned,
    #[serde(rename = "role.removed")]
    RoleRemoved,
}

impl WebhookEvent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserRegistered => "user.registered",
            Self::UserLogin => "user.login",
            Self::UserLogout => "user.logout",
            Self::UserPasswordChanged => "user.password_changed",
            Self::UserPasswordReset => "user.password_reset",
            Self::UserEmailVerified => "user.email_verified",
            Self::UserMfaEnabled => "user.mfa_enabled",
            Self::UserMfaDisabled => "user.mfa_disabled",
            Self::UserLocked => "user.locked",
            Self::UserUnlocked => "user.unlocked",
            Self::UserDeactivated => "user.deactivated",
            Self::UserActivated => "user.activated",
            Self::UserAppJoined => "user.app.joined",
            Self::UserAppBanned => "user.app.banned",
            Self::UserAppUnbanned => "user.app.unbanned",
            Self::UserAppRemoved => "user.app.removed",
            Self::AppCreated => "app.created",
            Self::AppSecretRegenerated => "app.secret_regenerated",
            Self::RoleAssigned => "role.assigned",
            Self::RoleRemoved => "role.removed",
        }
    }
}

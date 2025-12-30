use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct CreateIpRuleRequest {
    pub ip_address: String,
    pub ip_range: Option<String>,
    pub rule_type: String, // "whitelist" or "blacklist"
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct IpRuleResponse {
    pub id: Uuid,
    pub app_id: Option<String>,
    pub ip_address: String,
    pub ip_range: Option<String>,
    pub rule_type: String,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct IpCheckResponse {
    pub ip: String,
    pub allowed: bool,
    pub rule_type: Option<String>,
}

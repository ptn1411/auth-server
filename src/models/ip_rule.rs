use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "VARCHAR")]
#[sqlx(rename_all = "lowercase")]
pub enum IpRuleType {
    #[serde(rename = "whitelist")]
    Whitelist,
    #[serde(rename = "blacklist")]
    Blacklist,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct IpRule {
    pub id: Uuid,
    pub app_id: Option<Uuid>,
    pub ip_address: String,
    pub ip_range: Option<String>,
    pub rule_type: IpRuleType,
    pub reason: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

impl IpRule {
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at < Utc::now()
        } else {
            false
        }
    }

    pub fn matches_ip(&self, ip: &str) -> bool {
        if self.ip_address == ip {
            return true;
        }
        
        // Check CIDR range if specified
        if let Some(ref range) = self.ip_range {
            return Self::ip_in_cidr(ip, range);
        }
        
        false
    }

    fn ip_in_cidr(ip: &str, cidr: &str) -> bool {
        // Simple CIDR check for IPv4
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        let network = parts[0];
        let prefix_len: u32 = match parts[1].parse() {
            Ok(p) => p,
            Err(_) => return false,
        };

        let ip_parts: Vec<u32> = ip.split('.').filter_map(|p| p.parse().ok()).collect();
        let net_parts: Vec<u32> = network.split('.').filter_map(|p| p.parse().ok()).collect();

        if ip_parts.len() != 4 || net_parts.len() != 4 {
            return false;
        }

        let ip_num = (ip_parts[0] << 24) | (ip_parts[1] << 16) | (ip_parts[2] << 8) | ip_parts[3];
        let net_num = (net_parts[0] << 24) | (net_parts[1] << 16) | (net_parts[2] << 8) | net_parts[3];
        let mask = !((1u32 << (32 - prefix_len)) - 1);

        (ip_num & mask) == (net_num & mask)
    }
}

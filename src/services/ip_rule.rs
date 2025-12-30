use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::AppError;
use crate::models::{IpRule, IpRuleType};
use crate::repositories::IpRuleRepository;

pub struct IpRuleService {
    repo: IpRuleRepository,
}

impl IpRuleService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            repo: IpRuleRepository::new(pool),
        }
    }

    pub async fn create_rule(
        &self,
        app_id: Option<Uuid>,
        ip_address: &str,
        ip_range: Option<&str>,
        rule_type: IpRuleType,
        reason: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule, AppError> {
        // Validate IP address format
        if !Self::is_valid_ip(ip_address) {
            return Err(AppError::ValidationError("Invalid IP address format".into()));
        }

        // Validate CIDR if provided
        if let Some(range) = ip_range {
            if !Self::is_valid_cidr(range) {
                return Err(AppError::ValidationError("Invalid CIDR range format".into()));
            }
        }

        self.repo.create(app_id, ip_address, ip_range, rule_type, reason, expires_at, created_by).await
    }

    pub async fn whitelist_ip(
        &self,
        ip_address: &str,
        app_id: Option<Uuid>,
        reason: Option<&str>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule, AppError> {
        self.create_rule(app_id, ip_address, None, IpRuleType::Whitelist, reason, None, created_by).await
    }

    pub async fn blacklist_ip(
        &self,
        ip_address: &str,
        app_id: Option<Uuid>,
        reason: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule, AppError> {
        self.create_rule(app_id, ip_address, None, IpRuleType::Blacklist, reason, expires_at, created_by).await
    }

    pub async fn get_rule(&self, id: Uuid) -> Result<Option<IpRule>, AppError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list_rules(&self, app_id: Option<Uuid>) -> Result<Vec<IpRule>, AppError> {
        self.repo.find_by_app(app_id).await
    }

    pub async fn check_ip_access(&self, ip: &str, app_id: Option<Uuid>) -> Result<IpAccessResult, AppError> {
        // Check blacklist first
        if self.repo.is_blacklisted(ip, app_id).await? {
            return Ok(IpAccessResult::Blocked);
        }

        // Check whitelist
        if self.repo.is_whitelisted(ip, app_id).await? {
            return Ok(IpAccessResult::Allowed);
        }

        // No rule found - allow by default
        Ok(IpAccessResult::NoRule)
    }

    pub async fn delete_rule(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }

    pub async fn cleanup_expired(&self) -> Result<u64, AppError> {
        self.repo.delete_expired().await
    }

    fn is_valid_ip(ip: &str) -> bool {
        // Simple IPv4 validation
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() == 4 {
            return parts.iter().all(|p| p.parse::<u8>().is_ok());
        }

        // Simple IPv6 validation (basic check)
        if ip.contains(':') {
            return ip.split(':').count() <= 8;
        }

        false
    }

    fn is_valid_cidr(cidr: &str) -> bool {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return false;
        }

        if !Self::is_valid_ip(parts[0]) {
            return false;
        }

        if let Ok(prefix) = parts[1].parse::<u8>() {
            return prefix <= 32;
        }

        false
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum IpAccessResult {
    Allowed,
    Blocked,
    NoRule,
}

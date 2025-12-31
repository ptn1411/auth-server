use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::error::AppError;
use crate::models::{IpRule, IpRuleType};

#[derive(Clone)]
pub struct IpRuleRepository {
    pool: MySqlPool,
}

impl IpRuleRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        app_id: Option<Uuid>,
        ip_address: &str,
        ip_range: Option<&str>,
        rule_type: IpRuleType,
        reason: Option<&str>,
        expires_at: Option<DateTime<Utc>>,
        created_by: Option<Uuid>,
    ) -> Result<IpRule, AppError> {
        let id = Uuid::new_v4();
        let rule_type_str = match rule_type {
            IpRuleType::Whitelist => "whitelist",
            IpRuleType::Blacklist => "blacklist",
        };

        sqlx::query(
            r#"
            INSERT INTO ip_rules (id, app_id, ip_address, ip_range, rule_type, reason, expires_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(app_id.map(|u| u.to_string()))
        .bind(ip_address)
        .bind(ip_range)
        .bind(rule_type_str)
        .bind(reason)
        .bind(expires_at)
        .bind(created_by.map(|u| u.to_string()))
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or(AppError::InternalError(
            anyhow::anyhow!("Failed to create IP rule"),
        ))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<IpRule>, AppError> {
        let rule = sqlx::query_as::<_, IpRule>(
            "SELECT * FROM ip_rules WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(rule)
    }

    pub async fn find_by_ip(&self, ip: &str, app_id: Option<Uuid>) -> Result<Vec<IpRule>, AppError> {
        let rules = sqlx::query_as::<_, IpRule>(
            r#"
            SELECT * FROM ip_rules 
            WHERE (ip_address = ? OR ip_range IS NOT NULL)
            AND (app_id IS NULL OR app_id = ?)
            AND (expires_at IS NULL OR expires_at > NOW())
            "#,
        )
        .bind(ip)
        .bind(app_id.map(|u| u.to_string()))
        .fetch_all(&self.pool)
        .await?;

        Ok(rules)
    }

    pub async fn find_by_app(&self, app_id: Option<Uuid>) -> Result<Vec<IpRule>, AppError> {
        let rules = if let Some(app_id) = app_id {
            sqlx::query_as::<_, IpRule>(
                "SELECT * FROM ip_rules WHERE app_id = ? ORDER BY created_at DESC",
            )
            .bind(app_id.to_string())
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query_as::<_, IpRule>(
                "SELECT * FROM ip_rules WHERE app_id IS NULL ORDER BY created_at DESC",
            )
            .fetch_all(&self.pool)
            .await?
        };

        Ok(rules)
    }

    pub async fn check_ip(&self, ip: &str, app_id: Option<Uuid>) -> Result<Option<IpRuleType>, AppError> {
        let rules = self.find_by_ip(ip, app_id).await?;

        for rule in rules {
            if rule.is_expired() {
                continue;
            }

            if rule.matches_ip(ip) {
                return Ok(Some(rule.rule_type_enum()));
            }
        }

        Ok(None)
    }

    pub async fn is_blacklisted(&self, ip: &str, app_id: Option<Uuid>) -> Result<bool, AppError> {
        match self.check_ip(ip, app_id).await? {
            Some(IpRuleType::Blacklist) => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn is_whitelisted(&self, ip: &str, app_id: Option<Uuid>) -> Result<bool, AppError> {
        match self.check_ip(ip, app_id).await? {
            Some(IpRuleType::Whitelist) => Ok(true),
            _ => Ok(false),
        }
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM ip_rules WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_expired(&self) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM ip_rules WHERE expires_at IS NOT NULL AND expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}

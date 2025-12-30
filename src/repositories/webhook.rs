use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::error::AppError;
use crate::models::{Webhook, WebhookDelivery};

pub struct WebhookRepository {
    pool: MySqlPool,
}

impl WebhookRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    pub async fn create(
        &self,
        app_id: Uuid,
        url: &str,
        secret: &str,
        events: Vec<String>,
    ) -> Result<Webhook, AppError> {
        let id = Uuid::new_v4();
        let events_json = serde_json::to_string(&events)
            .map_err(|e| AppError::InternalError(e.into()))?;

        sqlx::query(
            r#"
            INSERT INTO webhooks (id, app_id, url, secret, events)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(app_id.to_string())
        .bind(url)
        .bind(secret)
        .bind(&events_json)
        .execute(&self.pool)
        .await?;

        self.find_by_id(id).await?.ok_or(AppError::InternalError(
            anyhow::anyhow!("Failed to create webhook"),
        ))
    }

    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Webhook>, AppError> {
        let webhook = sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(webhook)
    }

    pub async fn find_by_app(&self, app_id: Uuid) -> Result<Vec<Webhook>, AppError> {
        let webhooks = sqlx::query_as::<_, Webhook>(
            "SELECT * FROM webhooks WHERE app_id = ? AND is_active = TRUE",
        )
        .bind(app_id.to_string())
        .fetch_all(&self.pool)
        .await?;

        Ok(webhooks)
    }

    pub async fn find_by_event(&self, app_id: Uuid, event: &str) -> Result<Vec<Webhook>, AppError> {
        let webhooks = sqlx::query_as::<_, Webhook>(
            r#"
            SELECT * FROM webhooks 
            WHERE app_id = ? AND is_active = TRUE 
            AND JSON_CONTAINS(events, ?)
            "#,
        )
        .bind(app_id.to_string())
        .bind(format!("\"{}\"", event))
        .fetch_all(&self.pool)
        .await?;

        Ok(webhooks)
    }

    pub async fn update(
        &self,
        id: Uuid,
        url: Option<&str>,
        events: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> Result<Webhook, AppError> {
        if let Some(url) = url {
            sqlx::query("UPDATE webhooks SET url = ? WHERE id = ?")
                .bind(url)
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;
        }

        if let Some(events) = events {
            let events_json = serde_json::to_string(&events)
                .map_err(|e| AppError::InternalError(e.into()))?;
            sqlx::query("UPDATE webhooks SET events = ? WHERE id = ?")
                .bind(events_json)
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;
        }

        if let Some(is_active) = is_active {
            sqlx::query("UPDATE webhooks SET is_active = ? WHERE id = ?")
                .bind(is_active)
                .bind(id.to_string())
                .execute(&self.pool)
                .await?;
        }

        self.find_by_id(id).await?.ok_or(AppError::NotFound("Webhook not found".into()))
    }

    pub async fn delete(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM webhooks WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Delivery methods
    pub async fn create_delivery(
        &self,
        webhook_id: Uuid,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<WebhookDelivery, AppError> {
        let id = Uuid::new_v4();
        let payload_json = serde_json::to_string(&payload)
            .map_err(|e| AppError::InternalError(e.into()))?;

        sqlx::query(
            r#"
            INSERT INTO webhook_deliveries (id, webhook_id, event_type, payload)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(webhook_id.to_string())
        .bind(event_type)
        .bind(&payload_json)
        .execute(&self.pool)
        .await?;

        self.find_delivery_by_id(id).await?.ok_or(AppError::InternalError(
            anyhow::anyhow!("Failed to create delivery"),
        ))
    }

    pub async fn find_delivery_by_id(&self, id: Uuid) -> Result<Option<WebhookDelivery>, AppError> {
        let delivery = sqlx::query_as::<_, WebhookDelivery>(
            "SELECT * FROM webhook_deliveries WHERE id = ?",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;

        Ok(delivery)
    }

    pub async fn get_pending_deliveries(&self, limit: i32) -> Result<Vec<WebhookDelivery>, AppError> {
        let deliveries = sqlx::query_as::<_, WebhookDelivery>(
            r#"
            SELECT * FROM webhook_deliveries 
            WHERE delivered_at IS NULL 
            AND (next_retry_at IS NULL OR next_retry_at <= NOW())
            AND attempts < 5
            ORDER BY created_at ASC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(deliveries)
    }

    pub async fn mark_delivered(&self, id: Uuid, status: i32, body: Option<&str>) -> Result<(), AppError> {
        sqlx::query(
            r#"
            UPDATE webhook_deliveries 
            SET delivered_at = NOW(), response_status = ?, response_body = ?, attempts = attempts + 1
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(body)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_failed(&self, id: Uuid, status: Option<i32>, body: Option<&str>) -> Result<(), AppError> {
        let next_retry = Utc::now() + Duration::minutes(5);
        
        sqlx::query(
            r#"
            UPDATE webhook_deliveries 
            SET response_status = ?, response_body = ?, attempts = attempts + 1, next_retry_at = ?
            WHERE id = ?
            "#,
        )
        .bind(status)
        .bind(body)
        .bind(next_retry)
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

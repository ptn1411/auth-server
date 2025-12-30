use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;

use crate::error::AppError;
use crate::models::{Webhook, WebhookEvent};
use crate::repositories::WebhookRepository;
use crate::utils::secret::generate_secret;

type HmacSha256 = Hmac<Sha256>;

pub struct WebhookService {
    repo: WebhookRepository,
}

impl WebhookService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            repo: WebhookRepository::new(pool),
        }
    }

    pub async fn create_webhook(
        &self,
        app_id: Uuid,
        url: &str,
        events: Vec<String>,
    ) -> Result<(Webhook, String), AppError> {
        // Validate URL
        if !url.starts_with("https://") && !url.starts_with("http://localhost") {
            return Err(AppError::ValidationError("Webhook URL must use HTTPS".into()));
        }

        // Generate secret
        let secret = generate_secret();
        
        let webhook = self.repo.create(app_id, url, &secret, events).await?;
        
        Ok((webhook, secret))
    }

    pub async fn get_webhook(&self, id: Uuid) -> Result<Option<Webhook>, AppError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list_webhooks(&self, app_id: Uuid) -> Result<Vec<Webhook>, AppError> {
        self.repo.find_by_app(app_id).await
    }

    pub async fn update_webhook(
        &self,
        id: Uuid,
        url: Option<&str>,
        events: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> Result<Webhook, AppError> {
        if let Some(url) = url {
            if !url.starts_with("https://") && !url.starts_with("http://localhost") {
                return Err(AppError::ValidationError("Webhook URL must use HTTPS".into()));
            }
        }

        self.repo.update(id, url, events, is_active).await
    }

    pub async fn delete_webhook(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }

    pub async fn trigger_event(
        &self,
        app_id: Uuid,
        event: WebhookEvent,
        payload: serde_json::Value,
    ) -> Result<(), AppError> {
        let event_str = event.as_str();
        let webhooks = self.repo.find_by_event(app_id, event_str).await?;

        for webhook in webhooks {
            self.repo.create_delivery(webhook.id, event_str, payload.clone()).await?;
        }

        Ok(())
    }

    pub fn sign_payload(secret: &str, payload: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    pub fn verify_signature(secret: &str, payload: &str, signature: &str) -> bool {
        let expected = Self::sign_payload(secret, payload);
        expected == signature
    }

    pub async fn process_pending_deliveries(&self) -> Result<u32, AppError> {
        let deliveries = self.repo.get_pending_deliveries(100).await?;
        let mut processed = 0;

        for delivery in deliveries {
            let webhook = match self.repo.find_by_id(delivery.webhook_id).await? {
                Some(w) => w,
                None => continue,
            };

            let payload_str = serde_json::to_string(&delivery.payload)
                .map_err(|e| AppError::InternalError(e.into()))?;
            
            let signature = Self::sign_payload(&webhook.secret, &payload_str);
            let timestamp = Utc::now().timestamp();

            // Build request
            let client = reqwest::Client::new();
            let result = client
                .post(&webhook.url)
                .header("Content-Type", "application/json")
                .header("X-Webhook-Signature", &signature)
                .header("X-Webhook-Timestamp", timestamp.to_string())
                .header("X-Webhook-Event", &delivery.event_type)
                .body(payload_str)
                .timeout(std::time::Duration::from_secs(30))
                .send()
                .await;

            match result {
                Ok(response) => {
                    let status = response.status().as_u16() as i32;
                    let body = response.text().await.ok();
                    
                    if status >= 200 && status < 300 {
                        self.repo.mark_delivered(delivery.id, status, body.as_deref()).await?;
                    } else {
                        self.repo.mark_failed(delivery.id, Some(status), body.as_deref()).await?;
                    }
                }
                Err(e) => {
                    self.repo.mark_failed(delivery.id, None, Some(&e.to_string())).await?;
                }
            }

            processed += 1;
        }

        Ok(processed)
    }
}

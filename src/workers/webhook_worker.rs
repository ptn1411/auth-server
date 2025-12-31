use sqlx::MySqlPool;
use std::time::Duration;
use tokio::time::interval;

use crate::services::WebhookService;

/// Background worker for processing pending webhook deliveries
/// 
/// This worker runs in the background and periodically checks for pending
/// webhook deliveries, then attempts to send them to their configured URLs.
/// 
/// Features:
/// - Configurable polling interval
/// - Automatic retry with exponential backoff (handled by WebhookService)
/// - Graceful shutdown support
/// - Error logging without crashing
pub struct WebhookWorker {
    pool: MySqlPool,
    interval_secs: u64,
}

impl WebhookWorker {
    /// Create a new webhook worker
    /// 
    /// # Arguments
    /// * `pool` - Database connection pool
    /// * `interval_secs` - How often to check for pending deliveries (in seconds)
    pub fn new(pool: MySqlPool, interval_secs: u64) -> Self {
        Self { pool, interval_secs }
    }

    /// Start the webhook worker
    /// 
    /// This method runs indefinitely until the task is cancelled.
    /// It processes pending webhook deliveries at the configured interval.
    pub async fn run(&self) {
        tracing::info!(
            "Webhook worker started, polling every {} seconds",
            self.interval_secs
        );

        let mut ticker = interval(Duration::from_secs(self.interval_secs));

        loop {
            ticker.tick().await;
            
            if let Err(e) = self.process_batch().await {
                tracing::error!("Webhook worker error: {}", e);
            }
        }
    }

    /// Process a batch of pending webhook deliveries
    async fn process_batch(&self) -> Result<(), anyhow::Error> {
        let service = WebhookService::new(self.pool.clone());
        
        match service.process_pending_deliveries().await {
            Ok(processed) => {
                if processed > 0 {
                    tracing::info!("Webhook worker processed {} deliveries", processed);
                }
                Ok(())
            }
            Err(e) => {
                tracing::error!("Failed to process webhook deliveries: {:?}", e);
                Err(anyhow::anyhow!("{:?}", e))
            }
        }
    }
}

/// Spawn the webhook worker as a background task
/// 
/// # Arguments
/// * `pool` - Database connection pool
/// * `interval_secs` - Polling interval in seconds (default: 10)
/// 
/// # Returns
/// A JoinHandle that can be used to await or abort the worker
pub fn spawn_webhook_worker(pool: MySqlPool, interval_secs: u64) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let worker = WebhookWorker::new(pool, interval_secs);
        worker.run().await;
    })
}

use sqlx::MySqlPool;

use crate::error::AuthError;
use crate::repositories::RateLimitRepository;

/// Rate limit configuration for different endpoints
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_requests: i32,
    pub window_seconds: i64,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 60,
        }
    }
}

/// Predefined rate limit configurations
impl RateLimitConfig {
    /// Login endpoint: 5 attempts per minute
    pub fn login() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 60,
        }
    }

    /// Register endpoint: 3 attempts per minute
    pub fn register() -> Self {
        Self {
            max_requests: 3,
            window_seconds: 60,
        }
    }

    /// Password reset endpoint: 3 attempts per 5 minutes
    pub fn password_reset() -> Self {
        Self {
            max_requests: 3,
            window_seconds: 300,
        }
    }

    /// MFA verification: 5 attempts per 5 minutes
    pub fn mfa_verify() -> Self {
        Self {
            max_requests: 5,
            window_seconds: 300,
        }
    }

    /// Token refresh: 10 attempts per minute
    pub fn token_refresh() -> Self {
        Self {
            max_requests: 10,
            window_seconds: 60,
        }
    }

    /// General API: 100 requests per minute
    pub fn general_api() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
        }
    }
}

/// Result of a rate limit check
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub current_count: i32,
    pub max_requests: i32,
    pub remaining: i32,
    pub retry_after_seconds: Option<i64>,
}

/// Service for rate limiting
#[derive(Clone)]
pub struct RateLimiterService {
    repo: RateLimitRepository,
}

impl RateLimiterService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            repo: RateLimitRepository::new(pool),
        }
    }

    /// Check and increment rate limit for an identifier
    /// Returns whether the request is allowed
    pub async fn check_and_increment(
        &self,
        identifier: &str,
        endpoint: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, AuthError> {
        let count = self
            .repo
            .increment(identifier, endpoint, config.window_seconds)
            .await?;

        let allowed = count <= config.max_requests;
        let remaining = (config.max_requests - count).max(0);

        let retry_after = if !allowed {
            // Get window start to calculate retry time
            if let Some(window_start) = self.repo.get_window_start(identifier, endpoint).await? {
                let window_end = window_start + chrono::Duration::seconds(config.window_seconds);
                let now = chrono::Utc::now();
                if window_end > now {
                    Some((window_end - now).num_seconds())
                } else {
                    Some(0)
                }
            } else {
                Some(config.window_seconds)
            }
        } else {
            None
        };

        Ok(RateLimitResult {
            allowed,
            current_count: count,
            max_requests: config.max_requests,
            remaining,
            retry_after_seconds: retry_after,
        })
    }

    /// Check rate limit without incrementing
    pub async fn check_only(
        &self,
        identifier: &str,
        endpoint: &str,
        config: &RateLimitConfig,
    ) -> Result<RateLimitResult, AuthError> {
        let count = self
            .repo
            .get_count(identifier, endpoint, config.window_seconds)
            .await?;

        let allowed = count < config.max_requests;
        let remaining = (config.max_requests - count).max(0);

        Ok(RateLimitResult {
            allowed,
            current_count: count,
            max_requests: config.max_requests,
            remaining,
            retry_after_seconds: None,
        })
    }

    /// Reset rate limit for an identifier
    pub async fn reset(&self, identifier: &str, endpoint: &str) -> Result<(), AuthError> {
        self.repo.reset(identifier, endpoint).await
    }

    /// Create a combined identifier from IP and email
    pub fn create_identifier(ip: Option<&str>, email: Option<&str>) -> String {
        match (ip, email) {
            (Some(ip), Some(email)) => format!("{}:{}", ip, email),
            (Some(ip), None) => ip.to_string(),
            (None, Some(email)) => email.to_string(),
            (None, None) => "unknown".to_string(),
        }
    }

    /// Cleanup expired rate limit entries
    pub async fn cleanup(&self, window_seconds: i64) -> Result<u64, AuthError> {
        self.repo.delete_expired(window_seconds).await
    }
}

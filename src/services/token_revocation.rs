use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::repositories::RevokedTokenRepository;
use crate::utils::password::hash_token;

/// Service for token revocation (logout, token blacklisting)
#[derive(Clone)]
pub struct TokenRevocationService {
    repo: RevokedTokenRepository,
}

impl TokenRevocationService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            repo: RevokedTokenRepository::new(pool),
        }
    }

    /// Revoke an access token
    pub async fn revoke_access_token(
        &self,
        token: &str,
        user_id: Option<Uuid>,
        expires_in_secs: i64,
        reason: Option<&str>,
    ) -> Result<(), AuthError> {
        let token_hash = hash_token(token)?;
        let expires_at = Utc::now() + Duration::seconds(expires_in_secs);

        self.repo
            .revoke(&token_hash, "access", user_id, expires_at, reason)
            .await
    }

    /// Revoke a refresh token
    pub async fn revoke_refresh_token(
        &self,
        token: &str,
        user_id: Option<Uuid>,
        expires_in_secs: i64,
        reason: Option<&str>,
    ) -> Result<(), AuthError> {
        let token_hash = hash_token(token)?;
        let expires_at = Utc::now() + Duration::seconds(expires_in_secs);

        self.repo
            .revoke(&token_hash, "refresh", user_id, expires_at, reason)
            .await
    }

    /// Check if an access token is revoked
    pub async fn is_access_token_revoked(&self, token: &str) -> Result<bool, AuthError> {
        let token_hash = hash_token(token)?;
        self.repo.is_revoked(&token_hash).await
    }

    /// Check if a refresh token is revoked
    pub async fn is_refresh_token_revoked(&self, token: &str) -> Result<bool, AuthError> {
        let token_hash = hash_token(token)?;
        self.repo.is_revoked(&token_hash).await
    }

    /// Revoke all tokens for a user (force logout everywhere)
    pub async fn revoke_all_user_tokens(
        &self,
        user_id: Uuid,
        expires_in_secs: i64,
        reason: &str,
    ) -> Result<(), AuthError> {
        let expires_at = Utc::now() + Duration::seconds(expires_in_secs);
        self.repo
            .revoke_all_for_user(user_id, "all", expires_at, reason)
            .await
    }

    /// Cleanup expired revoked tokens
    pub async fn cleanup_expired(&self) -> Result<u64, AuthError> {
        self.repo.delete_expired().await
    }
}

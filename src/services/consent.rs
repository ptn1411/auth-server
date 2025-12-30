use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::OAuthError;
use crate::models::{OAuthEventType, UserConsent};
use crate::repositories::{OAuthAuditLogRepository, OAuthClientRepository, UserConsentRepository};

/// Information about a connected app with consent details
/// Requirements: 9.1
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentInfo {
    pub client_id: Uuid,
    pub client_name: String,
    pub scopes: Vec<String>,
    pub granted_at: DateTime<Utc>,
}

/// Consent Service - manages user consent for OAuth clients
/// Requirements: 4.2, 4.3, 4.5, 9.1, 9.3, 9.5, 10.6
#[derive(Clone)]
pub struct ConsentService {
    consent_repo: UserConsentRepository,
    client_repo: OAuthClientRepository,
    audit_repo: OAuthAuditLogRepository,
}

impl ConsentService {
    /// Create a new ConsentService with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            consent_repo: UserConsentRepository::new(pool.clone()),
            client_repo: OAuthClientRepository::new(pool.clone()),
            audit_repo: OAuthAuditLogRepository::new(pool),
        }
    }

    /// Check if user has already consented to all requested scopes
    /// Requirements: 4.5 - Skip consent screen if user has previously consented
    /// 
    /// Returns true if user has consented to ALL requested scopes,
    /// false if any scope is missing from previous consent
    pub async fn has_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<bool, OAuthError> {
        self.consent_repo.has_consent(user_id, client_id, scopes).await
    }

    /// Store user consent for a client with specific scopes
    /// Requirements: 4.3 - Store consent record with user_id, client_id, scopes, and timestamp
    /// Requirements: 9.5, 10.6 - Log consent events for audit
    /// 
    /// If consent already exists, updates the scopes (upsert behavior)
    pub async fn grant_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<UserConsent, OAuthError> {
        // Verify client exists
        let client = self.client_repo.find_by_id(client_id).await?;
        if client.is_none() {
            return Err(OAuthError::InvalidClient);
        }

        // Store or update consent
        let consent = self.consent_repo.upsert(user_id, client_id, scopes).await?;

        // Log the consent granted event
        // Requirements: 9.5, 10.6
        self.audit_repo
            .create(
                OAuthEventType::ConsentGranted,
                Some(client_id),
                Some(user_id),
                None,
                Some(serde_json::json!({
                    "scopes": scopes,
                })),
            )
            .await
            .ok(); // Don't fail if audit logging fails

        Ok(consent)
    }

    /// Record that user denied consent
    /// Requirements: 9.5, 10.6 - Log consent events for audit
    pub async fn log_consent_denied(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<(), OAuthError> {
        self.audit_repo
            .create(
                OAuthEventType::ConsentDenied,
                Some(client_id),
                Some(user_id),
                None,
                Some(serde_json::json!({
                    "scopes": scopes,
                })),
            )
            .await
            .ok(); // Don't fail if audit logging fails

        Ok(())
    }

    /// Revoke user consent for a client
    /// Requirements: 9.3 - Delete the consent record when user revokes access
    /// Requirements: 9.5, 10.6 - Log revocation events for audit
    /// 
    /// Returns error if consent doesn't exist
    pub async fn revoke_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<(), OAuthError> {
        self.consent_repo.delete(user_id, client_id).await?;

        // Log the consent revoked event
        // Requirements: 9.5, 10.6
        self.audit_repo
            .create(
                OAuthEventType::ConsentRevoked,
                Some(client_id),
                Some(user_id),
                None,
                None,
            )
            .await
            .ok(); // Don't fail if audit logging fails

        Ok(())
    }

    /// List all connected apps with consent details for a user
    /// Requirements: 9.1 - Return list of apps with granted scopes and consent timestamps
    pub async fn list_user_consents(
        &self,
        user_id: Uuid,
    ) -> Result<Vec<ConsentInfo>, OAuthError> {
        let consents = self.consent_repo.list_by_user(user_id).await?;
        
        let mut consent_infos = Vec::with_capacity(consents.len());
        
        for consent in consents {
            // Get client details for each consent
            if let Some(client) = self.client_repo.find_by_id(consent.client_id).await? {
                consent_infos.push(ConsentInfo {
                    client_id: client.id,
                    client_name: client.name,
                    scopes: consent.scopes,
                    granted_at: consent.granted_at,
                });
            }
        }
        
        Ok(consent_infos)
    }

    /// Check if a client is internal (no consent required)
    /// Requirements: 4.6 - Internal apps don't require user consent
    pub async fn is_internal_client(&self, client_id: Uuid) -> Result<bool, OAuthError> {
        let client = self.client_repo.find_by_id(client_id).await?;
        match client {
            Some(c) => Ok(c.is_internal),
            None => Err(OAuthError::InvalidClient),
        }
    }

    /// Check if consent is required for a client and scopes
    /// Requirements: 4.2, 4.5, 4.6
    /// 
    /// Returns true if consent screen should be shown:
    /// - Internal apps never require consent (4.6)
    /// - External apps require consent if user hasn't consented to all scopes (4.2, 4.5)
    pub async fn requires_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
        scopes: &[String],
    ) -> Result<bool, OAuthError> {
        // Check if client is internal
        let client = self.client_repo.find_by_id(client_id).await?;
        let client = client.ok_or(OAuthError::InvalidClient)?;
        
        // Internal apps don't require consent (Requirement 4.6)
        if client.is_internal {
            return Ok(false);
        }
        
        // Check if user has already consented to all requested scopes
        let has_consent = self.has_consent(user_id, client_id, scopes).await?;
        
        // Consent is required if user hasn't consented to all scopes
        Ok(!has_consent)
    }

    /// Get consent details for a specific user-client pair
    pub async fn get_consent(
        &self,
        user_id: Uuid,
        client_id: Uuid,
    ) -> Result<Option<UserConsent>, OAuthError> {
        self.consent_repo.find_by_user_and_client(user_id, client_id).await
    }
}

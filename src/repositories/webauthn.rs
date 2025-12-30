use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{Utc, Duration};

use crate::error::AppError;
use crate::models::{WebAuthnCredential, WebAuthnChallenge, ChallengeType};

pub struct WebAuthnRepository {
    pool: MySqlPool,
}

impl WebAuthnRepository {
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    // Credential methods
    pub async fn create_credential(
        &self,
        user_id: Uuid,
        credential_id: &[u8],
        public_key: &[u8],
        counter: u32,
        aaguid: Option<&[u8]>,
        device_name: Option<&str>,
        transports: Option<Vec<String>>,
    ) -> Result<WebAuthnCredential, AppError> {
        let id = Uuid::new_v4();
        let transports_json = transports.map(|t| serde_json::to_string(&t).ok()).flatten();

        sqlx::query(
            r#"
            INSERT INTO webauthn_credentials 
            (id, user_id, credential_id, public_key, counter, aaguid, device_name, transports)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(credential_id)
        .bind(public_key)
        .bind(counter)
        .bind(aaguid)
        .bind(device_name)
        .bind(transports_json)
        .execute(&self.pool)
        .await?;

        self.find_credential_by_id(id).await?.ok_or(AppError::InternalError(
            anyhow::anyhow!("Failed to create credential"),
        ))
    }

    pub async fn find_credential_by_id(&self, id: Uuid) -> Result<Option<WebAuthnCredential>, AppError> {
        let cred = sqlx::query_as::<_, WebAuthnCredential>(
            "SELECT * FROM webauthn_credentials WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(cred)
    }

    pub async fn find_credential_by_credential_id(&self, credential_id: &[u8]) -> Result<Option<WebAuthnCredential>, AppError> {
        let cred = sqlx::query_as::<_, WebAuthnCredential>(
            "SELECT * FROM webauthn_credentials WHERE credential_id = ? AND is_active = TRUE",
        )
        .bind(credential_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(cred)
    }

    pub async fn find_credentials_by_user(&self, user_id: Uuid) -> Result<Vec<WebAuthnCredential>, AppError> {
        let creds = sqlx::query_as::<_, WebAuthnCredential>(
            "SELECT * FROM webauthn_credentials WHERE user_id = ? AND is_active = TRUE ORDER BY created_at DESC",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(creds)
    }

    pub async fn update_counter(&self, id: Uuid, counter: u32) -> Result<(), AppError> {
        sqlx::query("UPDATE webauthn_credentials SET counter = ?, last_used_at = NOW() WHERE id = ?")
            .bind(counter)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_device_name(&self, id: Uuid, name: &str) -> Result<(), AppError> {
        sqlx::query("UPDATE webauthn_credentials SET device_name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn deactivate_credential(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("UPDATE webauthn_credentials SET is_active = FALSE WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_credential(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM webauthn_credentials WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    // Challenge methods
    pub async fn create_challenge(
        &self,
        user_id: Option<Uuid>,
        challenge: &[u8],
        challenge_type: ChallengeType,
        ttl_seconds: i64,
    ) -> Result<WebAuthnChallenge, AppError> {
        let id = Uuid::new_v4();
        let expires_at = Utc::now() + Duration::seconds(ttl_seconds);
        let type_str = match challenge_type {
            ChallengeType::Registration => "registration",
            ChallengeType::Authentication => "authentication",
        };

        sqlx::query(
            r#"
            INSERT INTO webauthn_challenges (id, user_id, challenge, challenge_type, expires_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id)
        .bind(user_id)
        .bind(challenge)
        .bind(type_str)
        .bind(expires_at)
        .execute(&self.pool)
        .await?;

        self.find_challenge_by_id(id).await?.ok_or(AppError::InternalError(
            anyhow::anyhow!("Failed to create challenge"),
        ))
    }

    pub async fn find_challenge_by_id(&self, id: Uuid) -> Result<Option<WebAuthnChallenge>, AppError> {
        let challenge = sqlx::query_as::<_, WebAuthnChallenge>(
            "SELECT * FROM webauthn_challenges WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(challenge)
    }

    pub async fn find_valid_challenge(&self, challenge: &[u8]) -> Result<Option<WebAuthnChallenge>, AppError> {
        let result = sqlx::query_as::<_, WebAuthnChallenge>(
            "SELECT * FROM webauthn_challenges WHERE challenge = ? AND expires_at > NOW()",
        )
        .bind(challenge)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result)
    }

    pub async fn delete_challenge(&self, id: Uuid) -> Result<(), AppError> {
        sqlx::query("DELETE FROM webauthn_challenges WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn delete_expired_challenges(&self) -> Result<u64, AppError> {
        let result = sqlx::query("DELETE FROM webauthn_challenges WHERE expires_at < NOW()")
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }

    pub async fn user_has_passkeys(&self, user_id: Uuid) -> Result<bool, AppError> {
        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM webauthn_credentials WHERE user_id = ? AND is_active = TRUE",
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count.0 > 0)
    }
}

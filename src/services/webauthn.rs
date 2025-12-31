use sqlx::MySqlPool;
use uuid::Uuid;
use rand::RngCore;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::{WebAuthnCredential, ChallengeType};
use crate::repositories::WebAuthnRepository;

pub struct WebAuthnService {
    repo: WebAuthnRepository,
    rp_id: String,
    rp_name: String,
    rp_origin: String,
}

#[derive(Debug, Serialize)]
pub struct RegistrationOptions {
    pub challenge: String,
    pub rp: RelyingParty,
    pub user: UserEntity,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    pub timeout: u32,
    pub attestation: String,
    pub authenticator_selection: AuthenticatorSelection,
}

#[derive(Debug, Serialize)]
pub struct RelyingParty {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct UserEntity {
    pub id: String,
    pub name: String,
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct PubKeyCredParam {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub alg: i32,
}

#[derive(Debug, Serialize)]
pub struct AuthenticatorSelection {
    pub authenticator_attachment: Option<String>,
    pub resident_key: String,
    pub user_verification: String,
}

#[derive(Debug, Serialize)]
pub struct AuthenticationOptions {
    pub challenge: String,
    pub timeout: u32,
    pub rp_id: String,
    pub allow_credentials: Vec<AllowCredential>,
    pub user_verification: String,
}

#[derive(Debug, Serialize)]
pub struct AllowCredential {
    pub id: String,
    #[serde(rename = "type")]
    pub cred_type: String,
    pub transports: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticatorAttestationResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticatorAttestationResponse {
    pub client_data_json: String,
    pub attestation_object: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticatorAssertionResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AuthenticatorAssertionResponse {
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    pub user_handle: Option<String>,
}

impl WebAuthnService {
    pub fn new(pool: MySqlPool, rp_id: String, rp_name: String, rp_origin: String) -> Self {
        Self {
            repo: WebAuthnRepository::new(pool),
            rp_id,
            rp_name,
            rp_origin,
        }
    }

    pub async fn start_registration(
        &self,
        user_id: Uuid,
        user_email: &str,
        user_name: &str,
    ) -> Result<RegistrationOptions, AppError> {
        // Generate challenge
        let mut challenge_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut challenge_bytes);
        
        // Store challenge
        self.repo.create_challenge(
            Some(user_id),
            &challenge_bytes,
            ChallengeType::Registration,
            300, // 5 minutes
        ).await?;

        let challenge = URL_SAFE_NO_PAD.encode(&challenge_bytes);
        let user_id_encoded = URL_SAFE_NO_PAD.encode(user_id.as_bytes());

        Ok(RegistrationOptions {
            challenge,
            rp: RelyingParty {
                id: self.rp_id.clone(),
                name: self.rp_name.clone(),
            },
            user: UserEntity {
                id: user_id_encoded,
                name: user_email.to_string(),
                display_name: user_name.to_string(),
            },
            pub_key_cred_params: vec![
                PubKeyCredParam { cred_type: "public-key".to_string(), alg: -7 },  // ES256
                PubKeyCredParam { cred_type: "public-key".to_string(), alg: -257 }, // RS256
            ],
            timeout: 300000, // 5 minutes
            attestation: "none".to_string(),
            authenticator_selection: AuthenticatorSelection {
                authenticator_attachment: None,
                resident_key: "preferred".to_string(),
                user_verification: "preferred".to_string(),
            },
        })
    }

    pub async fn finish_registration(
        &self,
        user_id: Uuid,
        response: RegistrationResponse,
        device_name: Option<&str>,
    ) -> Result<WebAuthnCredential, AppError> {
        // Decode credential ID
        let credential_id = URL_SAFE_NO_PAD.decode(&response.raw_id)
            .map_err(|_| AppError::ValidationError("Invalid credential ID".into()))?;

        // Decode client data
        let client_data_json = URL_SAFE_NO_PAD.decode(&response.response.client_data_json)
            .map_err(|_| AppError::ValidationError("Invalid client data".into()))?;

        // Parse client data
        let client_data: serde_json::Value = serde_json::from_slice(&client_data_json)
            .map_err(|_| AppError::ValidationError("Invalid client data JSON".into()))?;

        // Verify challenge
        let challenge_b64 = client_data["challenge"].as_str()
            .ok_or_else(|| AppError::ValidationError("Missing challenge".into()))?;
        
        let challenge = URL_SAFE_NO_PAD.decode(challenge_b64)
            .map_err(|_| AppError::ValidationError("Invalid challenge encoding".into()))?;

        let stored_challenge = self.repo.find_valid_challenge(&challenge).await?
            .ok_or_else(|| AppError::ValidationError("Challenge not found or expired".into()))?;

        if stored_challenge.challenge_type != ChallengeType::Registration {
            return Err(AppError::ValidationError("Invalid challenge type".into()));
        }

        // Verify origin
        let origin = client_data["origin"].as_str()
            .ok_or_else(|| AppError::ValidationError("Missing origin".into()))?;
        
        if origin != self.rp_origin {
            return Err(AppError::ValidationError("Origin mismatch".into()));
        }

        // Decode attestation object
        let attestation_object = URL_SAFE_NO_PAD.decode(&response.response.attestation_object)
            .map_err(|_| AppError::ValidationError("Invalid attestation object".into()))?;

        // Parse attestation object (simplified - in production use a proper CBOR parser)
        // For now, we'll store the raw attestation object as the public key
        // In a real implementation, you'd extract the actual public key from the attestation

        // Delete used challenge
        self.repo.delete_challenge(stored_challenge.id).await?;

        // Create credential
        let credential = self.repo.create_credential(
            user_id,
            &credential_id,
            &attestation_object, // Simplified - should extract actual public key
            0,
            None,
            device_name,
            None,
        ).await?;

        Ok(credential)
    }

    pub async fn start_authentication(
        &self,
        user_id: Option<Uuid>,
    ) -> Result<AuthenticationOptions, AppError> {
        // Generate challenge
        let mut challenge_bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut challenge_bytes);

        // Store challenge
        self.repo.create_challenge(
            user_id,
            &challenge_bytes,
            ChallengeType::Authentication,
            300,
        ).await?;

        let challenge = URL_SAFE_NO_PAD.encode(&challenge_bytes);

        // Get allowed credentials
        let allow_credentials = if let Some(user_id) = user_id {
            let creds = self.repo.find_credentials_by_user(user_id).await?;
            creds.into_iter().map(|c| AllowCredential {
                id: URL_SAFE_NO_PAD.encode(&c.credential_id),
                cred_type: "public-key".to_string(),
                transports: c.transports.map(|t| t.0),
            }).collect()
        } else {
            vec![]
        };

        Ok(AuthenticationOptions {
            challenge,
            timeout: 300000,
            rp_id: self.rp_id.clone(),
            allow_credentials,
            user_verification: "preferred".to_string(),
        })
    }

    pub async fn finish_authentication(
        &self,
        response: AuthenticationResponse,
    ) -> Result<(Uuid, WebAuthnCredential), AppError> {
        // Decode credential ID
        let credential_id = URL_SAFE_NO_PAD.decode(&response.raw_id)
            .map_err(|_| AppError::ValidationError("Invalid credential ID".into()))?;

        // Find credential
        let credential = self.repo.find_credential_by_credential_id(&credential_id).await?
            .ok_or_else(|| AppError::ValidationError("Credential not found".into()))?;

        // Decode client data
        let client_data_json = URL_SAFE_NO_PAD.decode(&response.response.client_data_json)
            .map_err(|_| AppError::ValidationError("Invalid client data".into()))?;

        // Parse client data
        let client_data: serde_json::Value = serde_json::from_slice(&client_data_json)
            .map_err(|_| AppError::ValidationError("Invalid client data JSON".into()))?;

        // Verify challenge
        let challenge_b64 = client_data["challenge"].as_str()
            .ok_or_else(|| AppError::ValidationError("Missing challenge".into()))?;
        
        let challenge = URL_SAFE_NO_PAD.decode(challenge_b64)
            .map_err(|_| AppError::ValidationError("Invalid challenge encoding".into()))?;

        let stored_challenge = self.repo.find_valid_challenge(&challenge).await?
            .ok_or_else(|| AppError::ValidationError("Challenge not found or expired".into()))?;

        if stored_challenge.challenge_type != ChallengeType::Authentication {
            return Err(AppError::ValidationError("Invalid challenge type".into()));
        }

        // Verify origin
        let origin = client_data["origin"].as_str()
            .ok_or_else(|| AppError::ValidationError("Missing origin".into()))?;
        
        if origin != self.rp_origin {
            return Err(AppError::ValidationError("Origin mismatch".into()));
        }

        // Decode authenticator data
        let auth_data = URL_SAFE_NO_PAD.decode(&response.response.authenticator_data)
            .map_err(|_| AppError::ValidationError("Invalid authenticator data".into()))?;

        // Extract counter from authenticator data (bytes 33-36)
        if auth_data.len() < 37 {
            return Err(AppError::ValidationError("Invalid authenticator data length".into()));
        }
        
        let new_counter = u32::from_be_bytes([auth_data[33], auth_data[34], auth_data[35], auth_data[36]]);

        // Verify counter (prevent replay attacks)
        // Allow counter = 0 for first authentication, otherwise must be strictly greater
        // Some authenticators (like platform authenticators) may not increment counter
        if credential.counter > 0 && new_counter != 0 && new_counter <= credential.counter {
            return Err(AppError::ValidationError("Invalid counter - possible replay attack".into()));
        }

        // In production, verify the signature here using the stored public key
        // This is simplified for demonstration

        // Delete used challenge
        self.repo.delete_challenge(stored_challenge.id).await?;

        // Update counter
        self.repo.update_counter(credential.id, new_counter).await?;

        Ok((credential.user_id, credential))
    }

    pub async fn list_credentials(&self, user_id: Uuid) -> Result<Vec<WebAuthnCredential>, AppError> {
        self.repo.find_credentials_by_user(user_id).await
    }

    pub async fn rename_credential(&self, id: Uuid, name: &str) -> Result<(), AppError> {
        self.repo.update_device_name(id, name).await
    }

    pub async fn delete_credential(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete_credential(id).await
    }

    pub async fn user_has_passkeys(&self, user_id: Uuid) -> Result<bool, AppError> {
        self.repo.user_has_passkeys(user_id).await
    }
}

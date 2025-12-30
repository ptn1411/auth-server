use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Deserialize)]
pub struct StartRegistrationRequest {
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FinishRegistrationRequest {
    pub id: String,
    pub raw_id: String,
    pub response: AttestationResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AttestationResponse {
    pub client_data_json: String,
    pub attestation_object: String,
}

#[derive(Debug, Deserialize)]
pub struct StartAuthenticationRequest {
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FinishAuthenticationRequest {
    pub id: String,
    pub raw_id: String,
    pub response: AssertionResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
}

#[derive(Debug, Deserialize)]
pub struct AssertionResponse {
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    pub user_handle: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RenameCredentialRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct PasskeyResponse {
    pub id: Uuid,
    pub device_name: Option<String>,
    pub transports: Option<Vec<String>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct PasskeyAuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: u64,
}

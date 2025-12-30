use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::config::AppState;
use crate::dto::{
    StartRegistrationRequest, FinishRegistrationRequest, StartAuthenticationRequest,
    FinishAuthenticationRequest, RenameCredentialRequest, PasskeyResponse, PasskeyAuthResponse,
};
use crate::error::AppError;
use crate::services::{WebAuthnService, RegistrationResponse, AuthenticationResponse};
use crate::utils::jwt::Claims;
use crate::repositories::UserRepository;

fn get_webauthn_service(state: &AppState) -> WebAuthnService {
    let rp_id = std::env::var("WEBAUTHN_RP_ID").unwrap_or_else(|_| "localhost".to_string());
    let rp_name = std::env::var("WEBAUTHN_RP_NAME").unwrap_or_else(|_| "Auth Server".to_string());
    let rp_origin = std::env::var("WEBAUTHN_RP_ORIGIN").unwrap_or_else(|_| "http://localhost:3000".to_string());
    
    WebAuthnService::new(state.pool.clone(), rp_id, rp_name, rp_origin)
}

/// POST /auth/webauthn/register/start - Start passkey registration
pub async fn start_registration_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(_req): Json<StartRegistrationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = claims.user_id()?;
    
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AppError::NotFound("User not found".into()))?;

    let service = get_webauthn_service(&state);
    let options = service.start_registration(
        user_id,
        &user.email,
        user.name.as_deref().unwrap_or(&user.email),
    ).await?;

    Ok(Json(serde_json::to_value(options).unwrap()))
}

/// POST /auth/webauthn/register/finish - Complete passkey registration
pub async fn finish_registration_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(req): Json<FinishRegistrationRequest>,
) -> Result<(StatusCode, Json<PasskeyResponse>), AppError> {
    let user_id = claims.user_id()?;

    let service = get_webauthn_service(&state);
    let response = RegistrationResponse {
        id: req.id,
        raw_id: req.raw_id,
        response: crate::services::AuthenticatorAttestationResponse {
            client_data_json: req.response.client_data_json,
            attestation_object: req.response.attestation_object,
        },
        cred_type: req.cred_type,
    };

    let credential = service.finish_registration(
        user_id,
        response,
        req.device_name.as_deref(),
    ).await?;

    Ok((
        StatusCode::CREATED,
        Json(PasskeyResponse {
            id: credential.id,
            device_name: credential.device_name,
            transports: credential.transports.map(|t| t.0),
            last_used_at: credential.last_used_at,
            created_at: credential.created_at,
        }),
    ))
}

/// POST /auth/webauthn/authenticate/start - Start passkey authentication
pub async fn start_authentication_handler(
    State(state): State<AppState>,
    Json(req): Json<StartAuthenticationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = if let Some(email) = &req.email {
        let user_repo = UserRepository::new(state.pool.clone());
        // If user not found, return None (will return empty credentials)
        user_repo.find_by_email(email).await?.map(|u| u.id)
    } else {
        None
    };

    let service = get_webauthn_service(&state);
    let options = service.start_authentication(user_id).await?;

    Ok(Json(serde_json::to_value(options).unwrap()))
}

/// POST /auth/webauthn/authenticate/finish - Complete passkey authentication
pub async fn finish_authentication_handler(
    State(state): State<AppState>,
    Json(req): Json<FinishAuthenticationRequest>,
) -> Result<Json<PasskeyAuthResponse>, AppError> {
    let service = get_webauthn_service(&state);
    let response = AuthenticationResponse {
        id: req.id,
        raw_id: req.raw_id,
        response: crate::services::AuthenticatorAssertionResponse {
            client_data_json: req.response.client_data_json,
            authenticator_data: req.response.authenticator_data,
            signature: req.response.signature,
            user_handle: req.response.user_handle,
        },
        cred_type: req.cred_type,
    };

    let (user_id, _credential) = service.finish_authentication(response).await?;

    // Get user info
    let user_repo = UserRepository::new(state.pool.clone());
    let user = user_repo.find_by_id(user_id).await?
        .ok_or(AppError::NotFound("User not found".into()))?;

    // Generate tokens using jwt_manager
    let apps = std::collections::HashMap::new();
    let token_pair = state.jwt_manager.create_token_pair(user.id, apps)
        .map_err(|e| AppError::InternalError(e.into()))?;

    Ok(Json(PasskeyAuthResponse {
        access_token: token_pair.access_token,
        refresh_token: token_pair.refresh_token,
        token_type: token_pair.token_type,
        expires_in: token_pair.expires_in as u64,
    }))
}

/// GET /auth/webauthn/credentials - List user's passkeys
pub async fn list_credentials_handler(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<PasskeyResponse>>, AppError> {
    let user_id = claims.user_id()?;

    let service = get_webauthn_service(&state);
    let credentials = service.list_credentials(user_id).await?;

    let response: Vec<PasskeyResponse> = credentials
        .into_iter()
        .map(|c| PasskeyResponse {
            id: c.id,
            device_name: c.device_name,
            transports: c.transports.map(|t| t.0),
            last_used_at: c.last_used_at,
            created_at: c.created_at,
        })
        .collect();

    Ok(Json(response))
}

/// PUT /auth/webauthn/credentials/:credential_id - Rename passkey
pub async fn rename_credential_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(credential_id): Path<Uuid>,
    Json(req): Json<RenameCredentialRequest>,
) -> Result<StatusCode, AppError> {
    let service = get_webauthn_service(&state);
    service.rename_credential(credential_id, &req.name).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /auth/webauthn/credentials/:credential_id - Delete passkey
pub async fn delete_credential_handler(
    State(state): State<AppState>,
    Extension(_claims): Extension<Claims>,
    Path(credential_id): Path<Uuid>,
) -> Result<StatusCode, AppError> {
    let service = get_webauthn_service(&state);
    service.delete_credential(credential_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

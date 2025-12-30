use sqlx::MySqlPool;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use rand::Rng;

use crate::error::AppError;
use crate::models::ApiKey;
use crate::repositories::ApiKeyRepository;

pub struct ApiKeyService {
    repo: ApiKeyRepository,
}

impl ApiKeyService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            repo: ApiKeyRepository::new(pool),
        }
    }

    /// Generate a new API key
    /// Returns (ApiKey, plain_text_key) - plain text key is only returned once
    pub async fn create_api_key(
        &self,
        app_id: Uuid,
        name: &str,
        scopes: Vec<String>,
        expires_at: Option<DateTime<Utc>>,
    ) -> Result<(ApiKey, String), AppError> {
        // Generate a secure random key
        let key = Self::generate_key();
        
        let api_key = self.repo.create(app_id, name, &key, scopes, expires_at).await?;
        
        Ok((api_key, key))
    }

    fn generate_key() -> String {
        let mut rng = rand::thread_rng();
        let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
        format!("ak_{}", base64::Engine::encode(&base64::engine::general_purpose::URL_SAFE_NO_PAD, &bytes))
    }

    pub async fn get_api_key(&self, id: Uuid) -> Result<Option<ApiKey>, AppError> {
        self.repo.find_by_id(id).await
    }

    pub async fn list_api_keys(&self, app_id: Uuid) -> Result<Vec<ApiKey>, AppError> {
        self.repo.find_by_app(app_id).await
    }

    pub async fn verify_api_key(&self, key: &str) -> Result<Option<ApiKey>, AppError> {
        self.repo.verify_key(key).await
    }

    pub async fn update_api_key(
        &self,
        id: Uuid,
        name: Option<&str>,
        scopes: Option<Vec<String>>,
        is_active: Option<bool>,
    ) -> Result<ApiKey, AppError> {
        self.repo.update(id, name, scopes, is_active).await
    }

    pub async fn revoke_api_key(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.revoke(id).await
    }

    pub async fn delete_api_key(&self, id: Uuid) -> Result<(), AppError> {
        self.repo.delete(id).await
    }

    /// Check if API key has required scope
    pub fn check_scope(api_key: &ApiKey, required_scope: &str) -> bool {
        api_key.has_scope(required_scope)
    }
}

// Common API key scopes
pub mod scopes {
    pub const READ_USERS: &str = "read:users";
    pub const WRITE_USERS: &str = "write:users";
    pub const READ_ROLES: &str = "read:roles";
    pub const WRITE_ROLES: &str = "write:roles";
    pub const READ_PERMISSIONS: &str = "read:permissions";
    pub const WRITE_PERMISSIONS: &str = "write:permissions";
    pub const ADMIN: &str = "admin";
    pub const ALL: &str = "*";
}

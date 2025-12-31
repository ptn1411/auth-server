use sqlx::MySqlPool;
use std::sync::Arc;

use crate::utils::jwt::JwtManager;

/// Application configuration loaded from environment variables
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Config {
    // Database
    pub database_url: String,
    
    // JWT
    pub jwt_private_key: String,
    pub jwt_public_key: String,
    pub access_token_expiry_secs: i64,
    pub refresh_token_expiry_secs: i64,
    
    // Server
    pub server_host: String,
    pub server_port: u16,

    // Background Workers
    pub webhook_worker_interval_secs: u64,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let jwt_private_key = std::env::var("JWT_PRIVATE_KEY").unwrap_or_else(|_| {
            std::fs::read_to_string("keys/private.pem")
                .unwrap_or_else(|_| Self::default_private_key().to_string())
        });

        let jwt_public_key = std::env::var("JWT_PUBLIC_KEY").unwrap_or_else(|_| {
            std::fs::read_to_string("keys/public.pem")
                .unwrap_or_else(|_| Self::default_public_key().to_string())
        });

        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "mysql://root:password@localhost/auth_server".to_string()),
            jwt_private_key,
            jwt_public_key,
            access_token_expiry_secs: std::env::var("ACCESS_TOKEN_EXPIRY_SECS")
                .unwrap_or_else(|_| "900".to_string()) // 15 minutes
                .parse()?,
            refresh_token_expiry_secs: std::env::var("REFRESH_TOKEN_EXPIRY_SECS")
                .unwrap_or_else(|_| "604800".to_string()) // 7 days
                .parse()?,
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "0.0.0.0".to_string()),
            server_port: std::env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()?,
            webhook_worker_interval_secs: std::env::var("WEBHOOK_WORKER_INTERVAL_SECS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()?,
        })
    }

    /// Get the socket address for the server
    #[allow(dead_code)]
    pub fn socket_addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.server_host, self.server_port)
            .parse()
            .expect("Invalid socket address")
    }

    // Default development keys - DO NOT USE IN PRODUCTION
    fn default_private_key() -> &'static str {
        r#"-----BEGIN RSA PRIVATE KEY-----
MIIEowIBAAKCAQEA0Z3qX2BTLS4e0ek55tJqNnFXRjCxLJQGxKHHKFpSgaQPkEkN
OPWgWnLZHYHGLSqMdLOqoFgKg7wMHFoVrYHGwXsZBGfn+0XBqJlIUGTpMKKbKcLh
wFtYgRxq8O5VBqdkgySgNByCMIaQJoQTFPmgR7azMgFcqaJmyTqo2RCHQJ8oKbQJ
xzgMPBj+0dL0MYchlwLPhAQcWnqBwCFG4lYwTN+sBD1nQqeAIaGsLfPNBD8znTIC
CPWDXQOV1WVXQFN5K3PqfuBDLmnnApGGf/RZuACin8aBxdj1LmBPTqHLpVPHCUre
F5aEdkWOD5QlKJFFKFpIp9TP3bueshBKkxYRUQIDAQABAoIBAC5RgZ+hBx7xHnFZ
nQmY436CjazfrHpOzjsek4OgVnFrG5KQ7EMwqYIkahFKmbH2sFwJVc1q5PL0wLTo
MKkaBQKJthMBBFWNIToKhELULJkMKRhXfB1iQzfpli0SqfOBc7V1GiGpMIgHe5MG
VWPH0MRUUP8sHBfGFKPzCqew8pLNWzPjdGB6ZrJUfKHpWbCdDkaTs3gNzVgStqZB
jQST9GNlPuBJOYB4fBMr0XPSbEIM0KFzXOqfMPpO9CrTsqfWIyfTsxORbBMFYxBz
bPqLwAfJmiMdPLhCXlPgTXyRhMG4fwPixna5XPWB0VQqqH1lolYfpGIp8QBhLnJR
ZpYfBaECgYEA7/4pZ+bLNXPHtAKRSQMvzpM5KCWB0rewHBBfVxfPDTfLrpKgmhxH
ZcAVMdLYfMPJQiMJBDyQKoFNwHmanUHgHfrj7lYNs7OPbPDKoe5vPPPRAoGJE7sq
r0DnZTq0J7xqpttYHmPaHanP+bGMhL1xBqI5Wk2e5K8GFUj9GZBctYkCgYEA5wXL
H3ZNMWUV7KCWB0rewHBBfVxfPDTfLrpKgmhxHZcAVMdLYfMPJQiMJBDyQKoFNwHm
anUHgHfrj7lYNs7OPbPDKoe5vPPPRAoGJE7sqr0DnZTq0J7xqpttYHmPaHanP+bG
MhL1xBqI5Wk2e5K8GFUj9GZBctYkCgYBN5K3PqfuBDLmnnApGGf/RZuACin8aBxd
j1LmBPTqHLpVPHCUreF5aEdkWOD5QlKJFFKFpIp9TP3bueshBKkxYRUQKBgHe5MG
VWPH0MRUUP8sHBfGFKPzCqew8pLNWzPjdGB6ZrJUfKHpWbCdDkaTs3gNzVgStqZB
jQST9GNlPuBJOYB4fBMr0XPSbEIM0KFzXOqfMPpO9CrTsqfWIyfTsxORbBMFYxBz
bPqLwAfJmiMdPLhCXlPgTXyRhMG4fwPixna5XPWB0VQqqH1lolYfpGIp8QBhLnJR
-----END RSA PRIVATE KEY-----"#
    }

    fn default_public_key() -> &'static str {
        r#"-----BEGIN PUBLIC KEY-----
MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA0Z3qX2BTLS4e0ek55tJq
NnFXRjCxLJQGxKHHKFpSgaQPkEkNOPWgWnLZHYHGLSqMdLOqoFgKg7wMHFoVrYHG
wXsZBGfn+0XBqJlIUGTpMKKbKcLhwFtYgRxq8O5VBqdkgySgNByCMIaQJoQTFPmg
R7azMgFcqaJmyTqo2RCHQJ8oKbQJxzgMPBj+0dL0MYchlwLPhAQcWnqBwCFG4lYw
TN+sBD1nQqeAIaGsLfPNBD8znTICCPWDXQOV1WVXQFN5K3PqfuBDLmnnApGGf/RZ
uACin8aBxdj1LmBPTqHLpVPHCUreF5aEdkWOD5QlKJFFKFpIp9TP3bueshBKkxYR
UQIDAQAB
-----END PUBLIC KEY-----"#
    }
}

/// Shared application state
#[allow(dead_code)]
#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
    pub jwt_manager: JwtManager,
}

impl AppState {
    pub fn new(pool: MySqlPool, config: Config) -> Self {
        let jwt_manager = JwtManager::new(
            &config.jwt_private_key,
            &config.jwt_public_key,
            config.access_token_expiry_secs,
            config.refresh_token_expiry_secs,
        ).expect("Failed to create JWT manager");
        
        Self {
            pool,
            config: Arc::new(config),
            jwt_manager,
        }
    }
}

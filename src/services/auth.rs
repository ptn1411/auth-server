use std::collections::HashMap;

use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::User;
use crate::repositories::{UserAppRepository, UserRepository};
use crate::utils::email::validate_email;
use crate::utils::jwt::{AppClaims, JwtManager, TokenPair};
use crate::utils::password::{hash_password, verify_password};

/// Minimum password length requirement
const MIN_PASSWORD_LENGTH: usize = 8;

/// Refresh token expiry in days
const REFRESH_TOKEN_EXPIRY_DAYS: i64 = 7;

/// Password reset token expiry in hours
const PASSWORD_RESET_TOKEN_EXPIRY_HOURS: i64 = 1;

/// Authentication service handling user registration, login, token refresh, and password reset
#[derive(Clone)]
pub struct AuthService {
    pool: MySqlPool,
    user_repo: UserRepository,
    user_app_repo: UserAppRepository,
    jwt_manager: JwtManager,
}

impl AuthService {
    /// Create a new AuthService
    pub fn new(pool: MySqlPool, jwt_manager: JwtManager) -> Self {
        let user_repo = UserRepository::new(pool.clone());
        let user_app_repo = UserAppRepository::new(pool.clone());
        Self {
            pool,
            user_repo,
            user_app_repo,
            jwt_manager,
        }
    }

    /// Register a new user with email and password
    pub async fn register(&self, email: &str, password: &str) -> Result<User, AuthError> {
        // Validate email format (Requirement 1.3)
        validate_email(email)?;

        // Validate password strength (Requirement 1.4)
        self.validate_password(password)?;

        // Hash password using argon2 (Requirement 1.1, 1.5)
        let password_hash = hash_password(password)?;

        // Create user (Requirement 1.2 - uniqueness enforced by database)
        let user = self.user_repo.create_user(email, &password_hash).await?;

        Ok(user)
    }

    /// Login a user with email and password
    /// If app_id is provided, checks if user is banned from that app (Requirement 3.4)
    pub async fn login(&self, email: &str, password: &str, app_id: Option<Uuid>) -> Result<TokenPair, AuthError> {
        // Find user by email (Requirement 2.2)
        let user = self.user_repo
            .find_by_email(email)
            .await?
            .ok_or(AuthError::InvalidCredentials)?;

        // Verify password (Requirement 2.1, 2.2)
        let is_valid = verify_password(password, &user.password_hash)?;
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Check if user is active (Requirement 2.3)
        if !user.is_active {
            return Err(AuthError::UserInactive);
        }

        // Check if user is banned from the specified app (Requirement 3.4)
        if let Some(app_id) = app_id {
            if let Some(user_app) = self.user_app_repo
                .find(user.id, app_id)
                .await
                .map_err(|e| AuthError::InternalError(anyhow::anyhow!("{}", e)))?
            {
                if user_app.status == crate::models::user_app::UserAppStatus::Banned {
                    return Err(AuthError::UserBanned {
                        reason: user_app.banned_reason,
                    });
                }
            }
        }

        // Get user's apps, roles, and permissions for token payload
        let apps = self.get_user_app_claims(user.id).await?;

        // Generate token pair (Requirement 2.4, 2.5)
        let token_pair = self.jwt_manager.create_token_pair(user.id, apps)?;

        // Store refresh token hash in database
        self.store_refresh_token(user.id, &token_pair.refresh_token).await?;

        Ok(token_pair)
    }

    /// Get user's app claims (roles and permissions) for JWT token
    async fn get_user_app_claims(&self, user_id: Uuid) -> Result<HashMap<String, AppClaims>, AuthError> {
        // Query to get all apps, roles, and permissions for a user
        let rows = sqlx::query_as::<_, (String, String, Option<String>)>(
            r#"
            SELECT 
                a.code as app_code,
                r.name as role_name,
                p.code as permission_code
            FROM user_app_roles uar
            JOIN apps a ON uar.app_id = a.id
            JOIN roles r ON uar.role_id = r.id
            LEFT JOIN role_permissions rp ON r.id = rp.role_id
            LEFT JOIN permissions p ON rp.permission_id = p.id
            WHERE uar.user_id = ?
            ORDER BY a.code, r.name, p.code
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Build app claims map
        let mut apps: HashMap<String, AppClaims> = HashMap::new();
        
        for (app_code, role_name, permission_code) in rows {
            let app_claims = apps.entry(app_code).or_insert_with(|| AppClaims {
                roles: Vec::new(),
                permissions: Vec::new(),
            });
            
            // Add role if not already present
            if !app_claims.roles.contains(&role_name) {
                app_claims.roles.push(role_name);
            }
            
            // Add permission if present and not already added
            if let Some(perm) = permission_code {
                if !app_claims.permissions.contains(&perm) {
                    app_claims.permissions.push(perm);
                }
            }
        }

        Ok(apps)
    }

    /// Store refresh token hash in database
    async fn store_refresh_token(&self, user_id: Uuid, refresh_token: &str) -> Result<(), AuthError> {
        let token_hash = hash_password(refresh_token)?;
        let expires_at = Utc::now() + Duration::days(REFRESH_TOKEN_EXPIRY_DAYS);
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO refresh_tokens (id, user_id, token_hash, expires_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }

    /// Validate password meets requirements
    fn validate_password(&self, password: &str) -> Result<(), AuthError> {
        if password.len() < MIN_PASSWORD_LENGTH {
            return Err(AuthError::WeakPassword);
        }
        
        // Check for at least one uppercase, one lowercase, and one digit
        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());
        
        if !has_uppercase || !has_lowercase || !has_digit {
            return Err(AuthError::WeakPassword);
        }
        
        Ok(())
    }

    /// Refresh access token using a valid refresh token
    pub async fn refresh(&self, refresh_token: &str) -> Result<TokenPair, AuthError> {
        // Verify the refresh token JWT (Requirement 3.2)
        let claims = self.jwt_manager.verify_token(refresh_token)?;

        // Get user_id from claims
        let user_id = claims.user_id()?;

        // Verify user still exists and is active
        let user = self.user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::InvalidToken)?;

        if !user.is_active {
            return Err(AuthError::UserInactive);
        }

        // Get updated roles and permissions (Requirement 3.3)
        let apps = self.get_user_app_claims(user_id).await?;

        // Generate new token pair (Requirement 3.1)
        let token_pair = self.jwt_manager.create_token_pair(user_id, apps)?;

        // Store new refresh token hash
        self.store_refresh_token(user_id, &token_pair.refresh_token).await?;

        Ok(token_pair)
    }

    /// Request password reset for an email address
    pub async fn forgot_password(&self, email: &str) -> Result<Option<String>, AuthError> {
        // Try to find user by email
        let user = self.user_repo.find_by_email(email).await?;

        // If user doesn't exist, return Ok(None) without revealing this fact (Requirement 4.2)
        let user = match user {
            Some(u) => u,
            None => return Ok(None),
        };

        // Generate a secure random reset token
        let reset_token = Uuid::new_v4().to_string();

        // Hash the token before storing (Requirement 4.1)
        let token_hash = hash_password(&reset_token)?;
        let expires_at = Utc::now() + Duration::hours(PASSWORD_RESET_TOKEN_EXPIRY_HOURS);
        let id = Uuid::new_v4();

        // Store the hashed token in database
        sqlx::query(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, token_hash, expires_at)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(user.id.to_string())
        .bind(token_hash)
        .bind(expires_at)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Return the token (in production, this would be sent via email)
        Ok(Some(reset_token))
    }

    /// Reset password using a valid reset token
    pub async fn reset_password(&self, token: &str, new_password: &str) -> Result<(), AuthError> {
        // Validate new password strength
        self.validate_password(new_password)?;

        // Find all non-used, non-expired reset tokens
        let reset_tokens = sqlx::query_as::<_, (String, String, String)>(
            r#"
            SELECT id, user_id, token_hash
            FROM password_reset_tokens
            WHERE used = false AND expires_at > NOW()
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Find the matching token by verifying the hash
        let mut matching_token: Option<(String, String)> = None;
        for (token_id, user_id, token_hash) in reset_tokens {
            if verify_password(token, &token_hash)? {
                matching_token = Some((token_id, user_id));
                break;
            }
        }

        // If no matching token found, return error (Requirement 4.4)
        let (token_id, user_id_str) = matching_token.ok_or(AuthError::InvalidToken)?;
        let user_id = Uuid::parse_str(&user_id_str)
            .map_err(|e| AuthError::InternalError(e.into()))?;

        // Hash the new password (Requirement 4.3)
        let new_password_hash = hash_password(new_password)?;

        // Update user's password
        self.user_repo.update_password(user_id, &new_password_hash).await?;

        // Mark the reset token as used
        sqlx::query(
            r#"
            UPDATE password_reset_tokens
            SET used = true
            WHERE id = ?
            "#,
        )
        .bind(token_id)
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(())
    }
}

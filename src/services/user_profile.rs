use chrono::{Duration, Utc};
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::auth::{ChangePasswordRequest, UpdateProfileRequest, UserProfileResponse};
use crate::dto::user_management::{
    BulkImportResponse, BulkOperationError, BulkOperationResponse, BulkRoleAssignmentRequest,
    ImportError, PaginatedResponse, UserExportData, UserImportRequest, UserSearchQuery,
    UserSearchResult,
};
use crate::error::AuthError;
use crate::repositories::UserRepository;
use crate::utils::password::{hash_password, verify_password};

/// Email verification token expiry in hours
const EMAIL_VERIFICATION_TOKEN_EXPIRY_HOURS: i64 = 24;

/// Service for user profile management
#[derive(Clone)]
pub struct UserProfileService {
    pool: MySqlPool,
    user_repo: UserRepository,
}

impl UserProfileService {
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            pool,
        }
    }

    /// Get current user's profile
    pub async fn get_profile(&self, user_id: Uuid) -> Result<UserProfileResponse, AuthError> {
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        Ok(UserProfileResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            phone: user.phone,
            is_active: user.is_active,
            email_verified: user.email_verified,
            is_system_admin: user.is_system_admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    /// Update user profile
    pub async fn update_profile(
        &self,
        user_id: Uuid,
        req: UpdateProfileRequest,
    ) -> Result<UserProfileResponse, AuthError> {
        let user = self
            .user_repo
            .update_profile(user_id, req.name, req.avatar_url, req.phone)
            .await?;

        Ok(UserProfileResponse {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            phone: user.phone,
            is_active: user.is_active,
            email_verified: user.email_verified,
            is_system_admin: user.is_system_admin,
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
    }

    /// Change password (when logged in)
    pub async fn change_password(
        &self,
        user_id: Uuid,
        req: ChangePasswordRequest,
    ) -> Result<(), AuthError> {
        // Get current user
        let user = self
            .user_repo
            .find_by_id(user_id)
            .await?
            .ok_or(AuthError::UserNotFound)?;

        // Verify current password
        let is_valid = verify_password(&req.current_password, &user.password_hash)?;
        if !is_valid {
            return Err(AuthError::InvalidCredentials);
        }

        // Validate new password
        Self::validate_password(&req.new_password)?;

        // Hash and update new password
        let new_hash = hash_password(&req.new_password)?;
        self.user_repo.update_password(user_id, &new_hash).await?;

        Ok(())
    }

    /// Validate password meets requirements
    fn validate_password(password: &str) -> Result<(), AuthError> {
        if password.len() < 8 {
            return Err(AuthError::WeakPassword);
        }

        let has_uppercase = password.chars().any(|c| c.is_uppercase());
        let has_lowercase = password.chars().any(|c| c.is_lowercase());
        let has_digit = password.chars().any(|c| c.is_ascii_digit());

        if !has_uppercase || !has_lowercase || !has_digit {
            return Err(AuthError::WeakPassword);
        }

        Ok(())
    }

    /// Create email verification token
    pub async fn create_verification_token(&self, user_id: Uuid) -> Result<String, AuthError> {
        let token = Uuid::new_v4().to_string();
        let token_hash = hash_password(&token)?;
        let expires_at = Utc::now() + Duration::hours(EMAIL_VERIFICATION_TOKEN_EXPIRY_HOURS);
        let id = Uuid::new_v4();

        sqlx::query(
            r#"
            INSERT INTO email_verification_tokens (id, user_id, token_hash, expires_at)
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

        Ok(token)
    }

    /// Verify email with token
    pub async fn verify_email(&self, token: &str) -> Result<(), AuthError> {
        // Find all non-used, non-expired verification tokens
        let tokens = sqlx::query_as::<_, (String, String, String)>(
            r#"
            SELECT id, user_id, token_hash
            FROM email_verification_tokens
            WHERE used = false AND expires_at > NOW()
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        // Find matching token
        let mut matching: Option<(String, String)> = None;
        for (token_id, user_id, token_hash) in tokens {
            if verify_password(token, &token_hash)? {
                matching = Some((token_id, user_id));
                break;
            }
        }

        let (token_id, user_id_str) = matching.ok_or(AuthError::InvalidToken)?;
        let user_id =
            Uuid::parse_str(&user_id_str).map_err(|e| AuthError::InternalError(e.into()))?;

        // Mark email as verified
        self.user_repo.set_email_verified(user_id, true).await?;

        // Mark token as used
        sqlx::query(
            r#"
            UPDATE email_verification_tokens
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

    /// Resend verification email
    pub async fn resend_verification(&self, email: &str) -> Result<Option<String>, AuthError> {
        let user = self.user_repo.find_by_email(email).await?;

        match user {
            Some(u) if !u.email_verified => {
                let token = self.create_verification_token(u.id).await?;
                Ok(Some(token))
            }
            _ => Ok(None), // Don't reveal if user exists or is already verified
        }
    }

    /// Search users with filters (admin only)
    pub async fn search_users(
        &self,
        query: UserSearchQuery,
    ) -> Result<PaginatedResponse<UserSearchResult>, AuthError> {
        let users = self
            .user_repo
            .search(
                query.email.as_deref(),
                query.name.as_deref(),
                query.is_active,
                query.email_verified,
                query.is_system_admin,
                &query.sort_by,
                &query.sort_order,
                query.page,
                query.limit,
            )
            .await?;

        let total = self
            .user_repo
            .count_search(
                query.email.as_deref(),
                query.name.as_deref(),
                query.is_active,
                query.email_verified,
                query.is_system_admin,
            )
            .await?;

        let results: Vec<UserSearchResult> = users
            .into_iter()
            .map(|u| UserSearchResult {
                id: u.id,
                email: u.email,
                name: u.name,
                is_active: u.is_active,
                email_verified: u.email_verified,
                is_system_admin: u.is_system_admin,
                created_at: u.created_at,
            })
            .collect();

        Ok(PaginatedResponse::new(results, query.page, query.limit, total))
    }

    /// Export users (admin only)
    pub async fn export_users(&self) -> Result<Vec<UserExportData>, AuthError> {
        let mut page = 1u32;
        let limit = 100u32;
        let mut all_users = Vec::new();

        loop {
            let users = self.user_repo.list_all(page, limit).await?;
            if users.is_empty() {
                break;
            }

            for u in users {
                all_users.push(UserExportData {
                    id: u.id,
                    email: u.email,
                    name: u.name,
                    phone: u.phone,
                    is_active: u.is_active,
                    email_verified: u.email_verified,
                    is_system_admin: u.is_system_admin,
                    created_at: u.created_at,
                });
            }

            page += 1;
        }

        Ok(all_users)
    }

    /// Import users (admin only)
    pub async fn import_users(
        &self,
        users: Vec<UserImportRequest>,
    ) -> Result<BulkImportResponse, AuthError> {
        let mut imported_count = 0u32;
        let mut failed_count = 0u32;
        let mut errors = Vec::new();

        for (idx, user_req) in users.into_iter().enumerate() {
            // Validate password
            if let Err(_) = Self::validate_password(&user_req.password) {
                failed_count += 1;
                errors.push(ImportError {
                    row: idx as u32 + 1,
                    email: user_req.email,
                    error: "Password does not meet requirements".to_string(),
                });
                continue;
            }

            // Hash password
            let password_hash = match hash_password(&user_req.password) {
                Ok(h) => h,
                Err(_) => {
                    failed_count += 1;
                    errors.push(ImportError {
                        row: idx as u32 + 1,
                        email: user_req.email,
                        error: "Failed to hash password".to_string(),
                    });
                    continue;
                }
            };

            // Create user
            match self
                .user_repo
                .create_user_with_profile(
                    &user_req.email,
                    &password_hash,
                    user_req.name.as_deref(),
                    user_req.phone.as_deref(),
                )
                .await
            {
                Ok(_) => imported_count += 1,
                Err(AuthError::EmailAlreadyExists) => {
                    failed_count += 1;
                    errors.push(ImportError {
                        row: idx as u32 + 1,
                        email: user_req.email,
                        error: "Email already exists".to_string(),
                    });
                }
                Err(e) => {
                    failed_count += 1;
                    errors.push(ImportError {
                        row: idx as u32 + 1,
                        email: user_req.email,
                        error: format!("Failed to create user: {}", e),
                    });
                }
            }
        }

        Ok(BulkImportResponse {
            imported_count,
            failed_count,
            errors,
        })
    }

    /// Bulk assign role to users (admin only)
    pub async fn bulk_assign_role(
        &self,
        req: BulkRoleAssignmentRequest,
    ) -> Result<BulkOperationResponse, AuthError> {
        let mut success_count = 0u32;
        let mut failed_count = 0u32;
        let mut errors = Vec::new();

        for user_id in req.user_ids {
            // Check if user exists
            let user = self.user_repo.find_by_id(user_id).await?;
            if user.is_none() {
                failed_count += 1;
                errors.push(BulkOperationError {
                    user_id,
                    error: "User not found".to_string(),
                });
                continue;
            }

            // Try to assign role
            let result = sqlx::query(
                r#"
                INSERT IGNORE INTO user_app_roles (user_id, app_id, role_id)
                VALUES (?, ?, ?)
                "#,
            )
            .bind(user_id.to_string())
            .bind(req.app_id.to_string())
            .bind(req.role_id.to_string())
            .execute(&self.pool)
            .await;

            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    failed_count += 1;
                    errors.push(BulkOperationError {
                        user_id,
                        error: format!("Failed to assign role: {}", e),
                    });
                }
            }
        }

        Ok(BulkOperationResponse {
            success_count,
            failed_count,
            errors,
        })
    }
}

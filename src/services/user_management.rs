use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::user_management::{AppUserInfo, PaginatedResponse};
use crate::error::UserManagementError;
use crate::models::user_app::{UserApp, UserAppStatus};
use crate::models::WebhookEvent;
use crate::repositories::{AppRepository, RoleRepository, UserAppRepository, UserAppRoleRepository, UserRepository, WebhookRepository};
use crate::services::WebhookService;

/// Service for user management within apps
/// 
/// Handles user registration, banning, unbanning, and removal from apps.
/// Requirements: 2.1-2.4, 3.1-3.5, 4.1-4.3, 5.1-5.4, 6.1-6.4
#[derive(Clone)]
pub struct UserManagementService {
    pool: MySqlPool,
    user_repo: UserRepository,
    app_repo: AppRepository,
    user_app_repo: UserAppRepository,
    user_app_role_repo: UserAppRoleRepository,
    role_repo: RoleRepository,
    webhook_service: WebhookService,
}

impl UserManagementService {
    /// Create a new UserManagementService with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            pool: pool.clone(),
            user_repo: UserRepository::new(pool.clone()),
            app_repo: AppRepository::new(pool.clone()),
            user_app_repo: UserAppRepository::new(pool.clone()),
            user_app_role_repo: UserAppRoleRepository::new(pool.clone()),
            role_repo: RoleRepository::new(pool.clone()),
            webhook_service: WebhookService::new(pool),
        }
    }

    /// Check if actor has permission to manage users in an app
    /// Actor must be either the app owner OR a system admin
    /// 
    /// # Arguments
    /// * `actor_id` - The user performing the action
    /// * `app_id` - The app being managed
    /// 
    /// # Returns
    /// * `Ok(())` - If actor has permission
    /// * `Err(UserManagementError::NotAppOwner)` - If actor lacks permission
    /// * `Err(UserManagementError::AppNotFound)` - If app doesn't exist
    /// 
    /// # Requirements
    /// - 3.3, 4.2, 5.2, 6.3: Non-owner rejection
    /// - 7.2: System admin override
    pub async fn check_permission(&self, actor_id: Uuid, app_id: Uuid) -> Result<(), UserManagementError> {
        // First check if app exists
        let app = self.app_repo.find_by_id(app_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if app.is_none() {
            return Err(UserManagementError::AppNotFound);
        }

        // Check if actor is system admin (has override permission)
        let is_admin = self.user_repo.is_system_admin(actor_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if is_admin {
            return Ok(());
        }

        // Check if actor is app owner
        let is_owner = self.app_repo.is_owner(app_id, actor_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if is_owner {
            return Ok(());
        }

        Err(UserManagementError::NotAppOwner)
    }

    /// Register a user to an app
    /// 
    /// Creates a user-app association with status "active".
    /// Rejects if user is banned or already registered.
    /// 
    /// # Arguments
    /// * `user_id` - The user to register
    /// * `app_id` - The app to register to
    /// 
    /// # Returns
    /// * `Ok(UserApp)` - The created association
    /// * `Err(UserManagementError::UserBanned)` - If user is banned from app
    /// * `Err(UserManagementError::UserAlreadyRegistered)` - If already registered
    /// * `Err(UserManagementError::AppNotFound)` - If app doesn't exist
    /// 
    /// # Requirements
    /// - 2.1: Create user_app association with status "active"
    /// - 2.2: Reject banned users
    /// - 2.3: Reject duplicate registration
    pub async fn register_to_app(&self, user_id: Uuid, app_id: Uuid) -> Result<UserApp, UserManagementError> {
        // Check if app exists
        let app = self.app_repo.find_by_id(app_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if app.is_none() {
            return Err(UserManagementError::AppNotFound);
        }

        // Check if user exists
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(UserManagementError::UserNotFound);
        }

        // Check if user is banned from this app
        // Requirements: 2.2
        let existing = self.user_app_repo.find(user_id, app_id).await?;
        if let Some(ref user_app) = existing {
            if user_app.status == UserAppStatus::Banned {
                return Err(UserManagementError::UserBanned {
                    reason: user_app.banned_reason.clone(),
                });
            }
            // User already registered (active)
            // Requirements: 2.3
            return Err(UserManagementError::UserAlreadyRegistered);
        }

        // Create user-app association with status "active"
        // Requirements: 2.1
        let user_app = self.user_app_repo.create(user_id, app_id).await?;

        // Trigger webhook for user.app.joined event
        let webhook_service = self.webhook_service.clone();
        let payload = serde_json::json!({
            "event": "user.app.joined",
            "user_id": user_id.to_string(),
            "app_id": app_id.to_string(),
            "status": "active",
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        tokio::spawn(async move {
            let _ = webhook_service.trigger_event(app_id, WebhookEvent::UserAppJoined, payload).await;
        });

        Ok(user_app)
    }
}


impl UserManagementService {
    /// Ban a user from an app
    /// 
    /// Updates user_app status to "banned" with timestamp.
    /// Creates a banned record if user is not yet registered.
    /// 
    /// # Arguments
    /// * `actor_id` - The user performing the ban (must be owner or admin)
    /// * `user_id` - The user to ban
    /// * `app_id` - The app to ban from
    /// * `reason` - Optional ban reason
    /// 
    /// # Returns
    /// * `Ok(UserApp)` - The updated/created association
    /// * `Err(UserManagementError::NotAppOwner)` - If actor lacks permission
    /// * `Err(UserManagementError::AppNotFound)` - If app doesn't exist
    /// 
    /// # Requirements
    /// - 3.1: Update status to "banned" with timestamp
    /// - 3.2: Store optional ban reason
    /// - 3.3: Reject non-owner/non-admin
    /// - 3.5: Create banned record if not registered
    pub async fn ban_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
        app_id: Uuid,
        reason: Option<String>,
    ) -> Result<UserApp, UserManagementError> {
        // Check permission (owner or admin)
        // Requirements: 3.3
        self.check_permission(actor_id, app_id).await?;

        // Check if user exists
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(UserManagementError::UserNotFound);
        }

        // Check if user is already registered to this app
        let existing = self.user_app_repo.find(user_id, app_id).await?;
        
        match existing {
            Some(_) => {
                // User is registered, update status to banned
                // Requirements: 3.1, 3.2
                let user_app = self.user_app_repo.update_status(user_id, app_id, UserAppStatus::Banned, reason.clone()).await?;

                // Trigger webhook for user.app.banned event
                let webhook_service = self.webhook_service.clone();
                let payload = serde_json::json!({
                    "event": "user.app.banned",
                    "user_id": user_id.to_string(),
                    "app_id": app_id.to_string(),
                    "banned_by": actor_id.to_string(),
                    "reason": reason,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                tokio::spawn(async move {
                    let _ = webhook_service.trigger_event(app_id, WebhookEvent::UserAppBanned, payload).await;
                });

                Ok(user_app)
            }
            None => {
                // User not registered, create banned record to prevent future registration
                // Requirements: 3.5
                let user_app = self.user_app_repo.create_banned(user_id, app_id, reason.clone()).await?;

                // Trigger webhook for user.app.banned event
                let webhook_service = self.webhook_service.clone();
                let payload = serde_json::json!({
                    "event": "user.app.banned",
                    "user_id": user_id.to_string(),
                    "app_id": app_id.to_string(),
                    "banned_by": actor_id.to_string(),
                    "reason": reason,
                    "pre_registered": false,
                    "timestamp": chrono::Utc::now().to_rfc3339()
                });
                tokio::spawn(async move {
                    let _ = webhook_service.trigger_event(app_id, WebhookEvent::UserAppBanned, payload).await;
                });

                Ok(user_app)
            }
        }
    }
}


impl UserManagementService {
    /// Unban a user from an app
    /// 
    /// Updates user_app status to "active" and clears banned_at.
    /// Idempotent: succeeds without changes if user is not banned.
    /// 
    /// # Arguments
    /// * `actor_id` - The user performing the unban (must be owner or admin)
    /// * `user_id` - The user to unban
    /// * `app_id` - The app to unban from
    /// 
    /// # Returns
    /// * `Ok(UserApp)` - The updated association
    /// * `Err(UserManagementError::NotAppOwner)` - If actor lacks permission
    /// * `Err(UserManagementError::AppNotFound)` - If app doesn't exist
    /// * `Err(UserManagementError::UserNotRegistered)` - If user has no association
    /// 
    /// # Requirements
    /// - 4.1: Update status to "active", clear banned_at
    /// - 4.2: Reject non-owner/non-admin
    /// - 4.3: Idempotent - succeed without changes if not banned
    pub async fn unban_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
        app_id: Uuid,
    ) -> Result<UserApp, UserManagementError> {
        // Check permission (owner or admin)
        // Requirements: 4.2
        self.check_permission(actor_id, app_id).await?;

        // Check if user has an association with this app
        let existing = self.user_app_repo.find(user_id, app_id).await?;
        
        match existing {
            Some(user_app) => {
                if user_app.status == UserAppStatus::Active {
                    // Already active, return success (idempotent)
                    // Requirements: 4.3
                    Ok(user_app)
                } else {
                    // Update status to active, clear banned_at
                    // Requirements: 4.1
                    let updated_user_app = self.user_app_repo.update_status(user_id, app_id, UserAppStatus::Active, None).await?;

                    // Trigger webhook for user.app.unbanned event
                    let webhook_service = self.webhook_service.clone();
                    let payload = serde_json::json!({
                        "event": "user.app.unbanned",
                        "user_id": user_id.to_string(),
                        "app_id": app_id.to_string(),
                        "unbanned_by": actor_id.to_string(),
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    tokio::spawn(async move {
                        let _ = webhook_service.trigger_event(app_id, WebhookEvent::UserAppUnbanned, payload).await;
                    });

                    Ok(updated_user_app)
                }
            }
            None => {
                // User not registered to this app
                Err(UserManagementError::UserNotRegistered)
            }
        }
    }
}


impl UserManagementService {
    /// Remove a user from an app
    /// 
    /// Deletes user_app association and all user_app_roles for that user in the app.
    /// Idempotent: succeeds without changes if user is not registered.
    /// 
    /// # Arguments
    /// * `actor_id` - The user performing the removal (must be owner or admin)
    /// * `user_id` - The user to remove
    /// * `app_id` - The app to remove from
    /// 
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(UserManagementError::NotAppOwner)` - If actor lacks permission
    /// * `Err(UserManagementError::AppNotFound)` - If app doesn't exist
    /// 
    /// # Requirements
    /// - 5.1: Delete user_app association and user_app_roles
    /// - 5.2: Reject non-owner/non-admin
    /// - 5.3: Idempotent - succeed without changes if not registered
    pub async fn remove_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
        app_id: Uuid,
    ) -> Result<(), UserManagementError> {
        // Check permission (owner or admin)
        // Requirements: 5.2
        self.check_permission(actor_id, app_id).await?;

        // Check if user was registered (for webhook)
        let was_registered = self.user_app_repo.find(user_id, app_id).await?.is_some();

        // Delete user_app_roles for this user in this app
        // Requirements: 5.1
        self.user_app_role_repo.delete_by_user_and_app(user_id, app_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;

        // Delete user_app association
        // Requirements: 5.1, 5.3 (idempotent - delete succeeds even if not exists)
        self.user_app_repo.delete(user_id, app_id).await?;

        // Trigger webhook for user.app.removed event (only if user was registered)
        if was_registered {
            let webhook_service = self.webhook_service.clone();
            let payload = serde_json::json!({
                "event": "user.app.removed",
                "user_id": user_id.to_string(),
                "app_id": app_id.to_string(),
                "removed_by": actor_id.to_string(),
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            tokio::spawn(async move {
                let _ = webhook_service.trigger_event(app_id, WebhookEvent::UserAppRemoved, payload).await;
            });
        }

        Ok(())
    }
}


impl UserManagementService {
    /// List all users registered to an app with pagination
    /// 
    /// Returns user info including email, status, roles, banned_at, and banned_reason.
    /// 
    /// # Arguments
    /// * `actor_id` - The user requesting the list (must be owner or admin)
    /// * `app_id` - The app to list users for
    /// * `page` - Page number (1-indexed)
    /// * `limit` - Number of items per page
    /// 
    /// # Returns
    /// * `Ok(PaginatedResponse<AppUserInfo>)` - Paginated list of users
    /// * `Err(UserManagementError::NotAppOwner)` - If actor lacks permission
    /// * `Err(UserManagementError::AppNotFound)` - If app doesn't exist
    /// 
    /// # Requirements
    /// - 6.1: Return all users registered to the app with status
    /// - 6.2: Include email, status, roles, banned_at, banned_reason
    /// - 6.3: Reject non-owner/non-admin
    /// - 6.4: Support pagination
    pub async fn list_app_users(
        &self,
        actor_id: Uuid,
        app_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<AppUserInfo>, UserManagementError> {
        // Check permission (owner or admin)
        // Requirements: 6.3
        self.check_permission(actor_id, app_id).await?;

        // Get total count for pagination
        let total = self.user_app_repo.count_by_app(app_id).await?;

        // Get user_apps for this page
        let user_apps = self.user_app_repo.list_by_app(app_id, page, limit).await?;

        // Build AppUserInfo for each user_app
        let mut app_users = Vec::with_capacity(user_apps.len());
        for user_app in user_apps {
            // Get user email
            let user = self.user_repo.find_by_id(user_app.user_id).await
                .map_err(|e| UserManagementError::InternalError(e.into()))?;
            
            let email = user.map(|u| u.email).unwrap_or_default();

            // Get role names for this user in this app
            // Requirements: 6.2
            let roles = self.role_repo.get_role_names_for_user_in_app(user_app.user_id, app_id).await
                .map_err(|e| UserManagementError::InternalError(e.into()))?;

            app_users.push(AppUserInfo {
                user_id: user_app.user_id,
                email,
                status: user_app.status,
                roles,
                banned_at: user_app.banned_at,
                banned_reason: user_app.banned_reason,
                created_at: user_app.created_at,
            });
        }

        Ok(PaginatedResponse::new(app_users, page, limit, total))
    }

    /// List all users registered to an app with pagination (without permission check)
    /// Used by API Key authentication where permission is checked via scopes
    pub async fn list_app_users_by_api_key(
        &self,
        app_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<(Vec<crate::dto::UserAppResponse>, i64), UserManagementError> {
        // Get total count for pagination
        let total = self.user_app_repo.count_by_app(app_id).await?;

        // Get user_apps for this page
        let user_apps = self.user_app_repo.list_by_app(app_id, page, limit).await?;

        // Build response for each user_app
        let mut users = Vec::with_capacity(user_apps.len());
        for user_app in user_apps {
            // Get user email
            let user = self.user_repo.find_by_id(user_app.user_id).await
                .map_err(|e| UserManagementError::InternalError(e.into()))?;
            
            let email = user.map(|u| u.email).unwrap_or_default();

            // Get role names for this user in this app
            let roles = self.role_repo.get_role_names_for_user_in_app(user_app.user_id, app_id).await
                .map_err(|e| UserManagementError::InternalError(e.into()))?;

            users.push(crate::dto::UserAppResponse {
                user_id: user_app.user_id,
                app_id: user_app.app_id,
                email,
                status: user_app.status.to_string(),
                roles,
                banned_at: user_app.banned_at,
                banned_reason: user_app.banned_reason,
                created_at: user_app.created_at,
            });
        }

        Ok((users, total as i64))
    }

    /// Get a specific user in an app (without permission check)
    /// Used by API Key authentication
    pub async fn get_user_in_app(
        &self,
        app_id: Uuid,
        user_id: Uuid,
    ) -> Result<crate::dto::UserAppResponse, UserManagementError> {
        // Get user_app association
        let user_app = self.user_app_repo.find(user_id, app_id).await?
            .ok_or(UserManagementError::UserNotRegistered)?;

        // Get user email
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?
            .ok_or(UserManagementError::UserNotFound)?;

        // Get role names
        let roles = self.role_repo.get_role_names_for_user_in_app(user_id, app_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(crate::dto::UserAppResponse {
            user_id: user_app.user_id,
            app_id: user_app.app_id,
            email: user.email,
            status: user_app.status.to_string(),
            roles,
            banned_at: user_app.banned_at,
            banned_reason: user_app.banned_reason,
            created_at: user_app.created_at,
        })
    }

    /// Ban a user from an app (without actor permission check)
    /// Used by API Key authentication where permission is checked via scopes
    pub async fn ban_user_by_api_key(
        &self,
        app_id: Uuid,
        user_id: Uuid,
        reason: Option<String>,
    ) -> Result<UserApp, UserManagementError> {
        // Check if user exists
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(UserManagementError::UserNotFound);
        }

        // Check if user is already registered to this app
        let existing = self.user_app_repo.find(user_id, app_id).await?;
        
        let user_app = match existing {
            Some(_) => {
                self.user_app_repo.update_status(user_id, app_id, UserAppStatus::Banned, reason.clone()).await?
            }
            None => {
                self.user_app_repo.create_banned(user_id, app_id, reason.clone()).await?
            }
        };

        // Trigger webhook for user.app.banned event
        let webhook_service = self.webhook_service.clone();
        let payload = serde_json::json!({
            "event": "user.app.banned",
            "user_id": user_id.to_string(),
            "app_id": app_id.to_string(),
            "reason": reason,
            "via_api_key": true,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        tokio::spawn(async move {
            let _ = webhook_service.trigger_event(app_id, WebhookEvent::UserAppBanned, payload).await;
        });

        Ok(user_app)
    }

    /// Unban a user from an app (without actor permission check)
    /// Used by API Key authentication where permission is checked via scopes
    pub async fn unban_user_by_api_key(
        &self,
        app_id: Uuid,
        user_id: Uuid,
    ) -> Result<UserApp, UserManagementError> {
        // Check if user has an association with this app
        let existing = self.user_app_repo.find(user_id, app_id).await?;
        
        match existing {
            Some(user_app) => {
                if user_app.status == UserAppStatus::Active {
                    Ok(user_app)
                } else {
                    let updated_user_app = self.user_app_repo.update_status(user_id, app_id, UserAppStatus::Active, None).await?;

                    // Trigger webhook for user.app.unbanned event
                    let webhook_service = self.webhook_service.clone();
                    let payload = serde_json::json!({
                        "event": "user.app.unbanned",
                        "user_id": user_id.to_string(),
                        "app_id": app_id.to_string(),
                        "via_api_key": true,
                        "timestamp": chrono::Utc::now().to_rfc3339()
                    });
                    tokio::spawn(async move {
                        let _ = webhook_service.trigger_event(app_id, WebhookEvent::UserAppUnbanned, payload).await;
                    });

                    Ok(updated_user_app)
                }
            }
            None => {
                Err(UserManagementError::UserNotRegistered)
            }
        }
    }
}

use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::user_management::PaginatedResponse;
use crate::error::UserManagementError;
use crate::models::{App, User};
use crate::repositories::{AppRepository, UserRepository};

/// Service for system admin operations
/// 
/// Handles admin-only operations like listing all users/apps and global user deactivation.
/// Requirements: 7.2, 7.3, 7.4, 7.5
#[derive(Clone)]
pub struct AdminService {
    user_repo: UserRepository,
    app_repo: AppRepository,
}

impl AdminService {
    /// Create a new AdminService with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            app_repo: AppRepository::new(pool),
        }
    }

    /// Verify that the actor is a system admin
    /// 
    /// # Arguments
    /// * `actor_id` - The user to verify
    /// 
    /// # Returns
    /// * `Ok(())` - If actor is a system admin
    /// * `Err(UserManagementError::NotSystemAdmin)` - If actor is not a system admin
    /// * `Err(UserManagementError::UserNotFound)` - If actor doesn't exist
    /// 
    /// # Requirements
    /// - 7.2: System admin override for app management actions
    pub async fn verify_admin(&self, actor_id: Uuid) -> Result<(), UserManagementError> {
        // Check if user exists
        let user = self.user_repo.find_by_id(actor_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(UserManagementError::UserNotFound);
        }

        // Check if user is system admin
        let is_admin = self.user_repo.is_system_admin(actor_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if !is_admin {
            return Err(UserManagementError::NotSystemAdmin);
        }

        Ok(())
    }


    /// List all users with pagination (admin only)
    /// 
    /// # Arguments
    /// * `actor_id` - The admin user requesting the list
    /// * `page` - Page number (1-indexed)
    /// * `limit` - Number of items per page
    /// 
    /// # Returns
    /// * `Ok(PaginatedResponse<User>)` - Paginated list of all users
    /// * `Err(UserManagementError::NotSystemAdmin)` - If actor is not a system admin
    /// 
    /// # Requirements
    /// - 7.4: System admin can list all users
    pub async fn list_all_users(
        &self,
        actor_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<User>, UserManagementError> {
        // Verify actor is system admin
        self.verify_admin(actor_id).await?;

        // Get total count for pagination
        let total = self.user_repo.count_all().await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;

        // Get users for this page
        let users = self.user_repo.list_all(page, limit).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(PaginatedResponse::new(users, page, limit, total))
    }

    /// List all apps with pagination (admin only)
    /// 
    /// # Arguments
    /// * `actor_id` - The admin user requesting the list
    /// * `page` - Page number (1-indexed)
    /// * `limit` - Number of items per page
    /// 
    /// # Returns
    /// * `Ok(PaginatedResponse<App>)` - Paginated list of all apps
    /// * `Err(UserManagementError::NotSystemAdmin)` - If actor is not a system admin
    /// 
    /// # Requirements
    /// - 7.4: System admin can list all apps
    pub async fn list_all_apps(
        &self,
        actor_id: Uuid,
        page: u32,
        limit: u32,
    ) -> Result<PaginatedResponse<App>, UserManagementError> {
        // Verify actor is system admin
        self.verify_admin(actor_id).await?;

        // Get total count for pagination
        let total = self.app_repo.count_all().await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;

        // Get apps for this page
        let apps = self.app_repo.list_all(page, limit).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(PaginatedResponse::new(apps, page, limit, total))
    }

    /// Deactivate a user globally (admin only)
    /// 
    /// Sets the user's is_active flag to false, preventing them from logging in
    /// to any app.
    /// 
    /// # Arguments
    /// * `actor_id` - The admin user performing the deactivation
    /// * `user_id` - The user to deactivate
    /// 
    /// # Returns
    /// * `Ok(())` - Success
    /// * `Err(UserManagementError::NotSystemAdmin)` - If actor is not a system admin
    /// * `Err(UserManagementError::UserNotFound)` - If target user doesn't exist
    /// 
    /// # Requirements
    /// - 7.5: System admin can deactivate any user globally
    pub async fn deactivate_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), UserManagementError> {
        // Verify actor is system admin
        self.verify_admin(actor_id).await?;

        // Check if target user exists
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(UserManagementError::UserNotFound);
        }

        // Deactivate the user
        self.user_repo.deactivate(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;

        Ok(())
    }
}

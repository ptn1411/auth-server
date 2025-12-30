use sqlx::MySqlPool;
use uuid::Uuid;

use crate::dto::user_management::PaginatedResponse;
use crate::error::UserManagementError;
use crate::models::{App, User};
use crate::repositories::{AppRepository, UserRepository, UserAppRoleRepository};

/// User roles info across all apps
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserRolesInfo {
    pub user_id: Uuid,
    pub apps: Vec<AppRoleInfo>,
}

/// Role info for a specific app
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppRoleInfo {
    pub app_id: Uuid,
    pub app_code: String,
    pub app_name: String,
    pub roles: Vec<RoleInfo>,
}

/// Basic role info
#[derive(Debug, Clone, serde::Serialize)]
pub struct RoleInfo {
    pub role_id: Uuid,
    pub role_name: String,
}

/// Service for system admin operations
/// 
/// Handles admin-only operations like listing all users/apps and global user deactivation.
/// Requirements: 7.2, 7.3, 7.4, 7.5
#[derive(Clone)]
pub struct AdminService {
    pool: MySqlPool,
    user_repo: UserRepository,
    app_repo: AppRepository,
    user_app_role_repo: UserAppRoleRepository,
}

impl AdminService {
    /// Create a new AdminService with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            user_repo: UserRepository::new(pool.clone()),
            app_repo: AppRepository::new(pool.clone()),
            user_app_role_repo: UserAppRoleRepository::new(pool.clone()),
            pool,
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

    /// Get user details by ID (admin only)
    pub async fn get_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
    ) -> Result<User, UserManagementError> {
        self.verify_admin(actor_id).await?;

        self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?
            .ok_or(UserManagementError::UserNotFound)
    }

    /// Update user by admin
    pub async fn update_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
        email: Option<&str>,
        is_active: Option<bool>,
        is_system_admin: Option<bool>,
        email_verified: Option<bool>,
    ) -> Result<User, UserManagementError> {
        self.verify_admin(actor_id).await?;

        // Prevent admin from removing their own admin status
        if is_system_admin == Some(false) && actor_id == user_id {
            return Err(UserManagementError::InternalError(
                anyhow::anyhow!("Cannot remove your own admin status")
            ));
        }

        self.user_repo.admin_update(user_id, email, is_active, is_system_admin, email_verified).await
            .map_err(|e| UserManagementError::InternalError(e.into()))
    }

    /// Delete user permanently (admin only)
    pub async fn delete_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), UserManagementError> {
        self.verify_admin(actor_id).await?;

        // Prevent admin from deleting themselves
        if actor_id == user_id {
            return Err(UserManagementError::InternalError(
                anyhow::anyhow!("Cannot delete your own account")
            ));
        }

        // Check if user exists
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(UserManagementError::UserNotFound);
        }

        self.user_repo.delete(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))
    }

    /// Activate a user (admin only)
    pub async fn activate_user(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
    ) -> Result<(), UserManagementError> {
        self.verify_admin(actor_id).await?;

        self.user_repo.set_active(user_id, true).await
            .map_err(|e| UserManagementError::InternalError(e.into()))
    }

    /// Get app details by ID (admin only)
    pub async fn get_app(
        &self,
        actor_id: Uuid,
        app_id: Uuid,
    ) -> Result<App, UserManagementError> {
        self.verify_admin(actor_id).await?;

        self.app_repo.find_by_id(app_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?
            .ok_or(UserManagementError::AppNotFound)
    }

    /// Update app by admin
    pub async fn update_app(
        &self,
        actor_id: Uuid,
        app_id: Uuid,
        name: Option<&str>,
        owner_id: Option<Uuid>,
    ) -> Result<App, UserManagementError> {
        self.verify_admin(actor_id).await?;

        self.app_repo.update(app_id, name, owner_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))
    }

    /// Delete app permanently (admin only)
    pub async fn delete_app(
        &self,
        actor_id: Uuid,
        app_id: Uuid,
    ) -> Result<(), UserManagementError> {
        self.verify_admin(actor_id).await?;

        self.app_repo.delete(app_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))
    }

    /// Get all roles for a user across all apps (admin only)
    pub async fn get_user_roles(
        &self,
        actor_id: Uuid,
        user_id: Uuid,
    ) -> Result<UserRolesInfo, UserManagementError> {
        self.verify_admin(actor_id).await?;

        // Check if user exists
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| UserManagementError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(UserManagementError::UserNotFound);
        }

        // Get all roles for user with app info
        let rows = sqlx::query_as::<_, (String, String, String, String, String)>(
            r#"
            SELECT 
                a.id as app_id,
                a.code as app_code,
                a.name as app_name,
                r.id as role_id,
                r.name as role_name
            FROM user_app_roles uar
            JOIN apps a ON uar.app_id = a.id
            JOIN roles r ON uar.role_id = r.id
            WHERE uar.user_id = ?
            ORDER BY a.code, r.name
            "#,
        )
        .bind(user_id.to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| UserManagementError::InternalError(e.into()))?;

        // Group by app
        let mut apps_map: std::collections::HashMap<Uuid, AppRoleInfo> = std::collections::HashMap::new();
        
        for (app_id_str, app_code, app_name, role_id_str, role_name) in rows {
            let app_id = Uuid::parse_str(&app_id_str)
                .map_err(|e| UserManagementError::InternalError(e.into()))?;
            let role_id = Uuid::parse_str(&role_id_str)
                .map_err(|e| UserManagementError::InternalError(e.into()))?;

            let app_info = apps_map.entry(app_id).or_insert_with(|| AppRoleInfo {
                app_id,
                app_code: app_code.clone(),
                app_name: app_name.clone(),
                roles: Vec::new(),
            });

            app_info.roles.push(RoleInfo {
                role_id,
                role_name,
            });
        }

        Ok(UserRolesInfo {
            user_id,
            apps: apps_map.into_values().collect(),
        })
    }
}

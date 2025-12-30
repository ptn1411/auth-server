use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::RoleError;
use crate::models::Role;
use crate::repositories::{AppRepository, RoleRepository, UserAppRoleRepository, UserRepository};

/// Service for role management operations
/// 
/// Handles creating roles scoped to apps and assigning roles to users.
#[derive(Clone)]
pub struct RoleService {
    role_repo: RoleRepository,
    app_repo: AppRepository,
    user_repo: UserRepository,
    user_app_role_repo: UserAppRoleRepository,
}

impl RoleService {
    /// Create a new RoleService with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            role_repo: RoleRepository::new(pool.clone()),
            app_repo: AppRepository::new(pool.clone()),
            user_repo: UserRepository::new(pool.clone()),
            user_app_role_repo: UserAppRoleRepository::new(pool),
        }
    }

    /// Create a new role for a specific app
    /// 
    /// # Arguments
    /// * `app_id` - The UUID of the app this role belongs to
    /// * `name` - The name of the role (must be unique within the app)
    /// 
    /// # Returns
    /// * `Ok(Role)` - The created role
    /// * `Err(RoleError::AppNotFound)` - If the app doesn't exist
    /// * `Err(RoleError::NameAlreadyExists)` - If role name already exists in this app
    /// 
    /// # Requirements
    /// - 6.1: Create role scoped to specific app only
    /// - 6.2: Reject duplicate role name within the same app
    pub async fn create_role(&self, app_id: Uuid, name: &str) -> Result<Role, RoleError> {
        // Verify app exists (Requirement 6.1)
        let app = self.app_repo.find_by_id(app_id).await
            .map_err(|e| RoleError::InternalError(e.into()))?;
        
        if app.is_none() {
            return Err(RoleError::AppNotFound);
        }

        // Create role - name uniqueness within app is enforced by database constraint
        // Requirements: 6.1, 6.2
        self.role_repo.create_role(app_id, name).await
    }

    /// Get all roles for a specific app
    /// 
    /// # Arguments
    /// * `app_id` - The UUID of the app
    /// 
    /// # Returns
    /// * `Ok(Vec<Role>)` - List of roles for the app
    pub async fn get_roles_by_app(&self, app_id: Uuid) -> Result<Vec<Role>, RoleError> {
        self.role_repo.find_by_app_id(app_id).await
    }

    /// Assign a role to a user for a specific app
    /// 
    /// # Arguments
    /// * `user_id` - The UUID of the user
    /// * `app_id` - The UUID of the app
    /// * `role_id` - The UUID of the role to assign
    /// 
    /// # Returns
    /// * `Ok(())` - Role was successfully assigned
    /// * `Err(RoleError::UserNotFound)` - If user doesn't exist
    /// * `Err(RoleError::AppNotFound)` - If app doesn't exist
    /// * `Err(RoleError::NotFound)` - If role doesn't exist or doesn't belong to the app
    /// 
    /// # Requirements
    /// - 8.1: Create user-app-role association
    /// - 8.2: Reject assignment with non-existent user, app, or role
    pub async fn assign_role_to_user(
        &self,
        user_id: Uuid,
        app_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), RoleError> {
        // Verify user exists (Requirement 8.2)
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| RoleError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(RoleError::UserNotFound);
        }

        // Verify app exists (Requirement 8.2)
        let app = self.app_repo.find_by_id(app_id).await
            .map_err(|e| RoleError::InternalError(e.into()))?;
        
        if app.is_none() {
            return Err(RoleError::AppNotFound);
        }

        // Verify role exists and belongs to the app (Requirement 8.2)
        let role = self.role_repo.find_by_id(role_id).await?;
        
        match role {
            None => return Err(RoleError::NotFound),
            Some(r) if r.app_id != app_id => return Err(RoleError::NotFound),
            _ => {}
        }

        // Create the user-app-role association (Requirement 8.1)
        self.user_app_role_repo.assign_role(user_id, app_id, role_id).await?;

        Ok(())
    }

    /// Remove a role from a user for a specific app
    pub async fn remove_role_from_user(
        &self,
        user_id: Uuid,
        app_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), RoleError> {
        // Verify user exists
        let user = self.user_repo.find_by_id(user_id).await
            .map_err(|e| RoleError::InternalError(e.into()))?;
        
        if user.is_none() {
            return Err(RoleError::UserNotFound);
        }

        // Verify role exists and belongs to the app
        let role = self.role_repo.find_by_id(role_id).await?;
        
        match role {
            None => return Err(RoleError::NotFound),
            Some(r) if r.app_id != app_id => return Err(RoleError::NotFound),
            _ => {}
        }

        // Remove the role
        self.user_app_role_repo.remove_role(user_id, app_id, role_id).await?;

        Ok(())
    }

    /// Get all roles for a user in a specific app
    pub async fn get_user_roles_in_app(
        &self,
        user_id: Uuid,
        app_id: Uuid,
    ) -> Result<Vec<Role>, RoleError> {
        let user_app_roles = self.user_app_role_repo.find_by_user_and_app(user_id, app_id).await?;
        
        let mut roles = Vec::new();
        for uar in user_app_roles {
            if let Some(role) = self.role_repo.find_by_id(uar.role_id).await? {
                roles.push(role);
            }
        }
        
        Ok(roles)
    }
}

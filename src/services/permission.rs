use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::PermissionError;
use crate::models::Permission;
use crate::repositories::{AppRepository, PermissionRepository, RolePermissionRepository, RoleRepository};

/// Service for permission management operations
/// 
/// Handles creating permissions scoped to apps and assigning permissions to roles.
#[derive(Clone)]
pub struct PermissionService {
    permission_repo: PermissionRepository,
    app_repo: AppRepository,
    role_repo: RoleRepository,
    role_permission_repo: RolePermissionRepository,
}

impl PermissionService {
    /// Create a new PermissionService with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self {
            permission_repo: PermissionRepository::new(pool.clone()),
            app_repo: AppRepository::new(pool.clone()),
            role_repo: RoleRepository::new(pool.clone()),
            role_permission_repo: RolePermissionRepository::new(pool),
        }
    }

    /// Create a new permission for a specific app
    /// 
    /// # Arguments
    /// * `app_id` - The UUID of the app this permission belongs to
    /// * `code` - The code of the permission (must be unique within the app)
    /// 
    /// # Returns
    /// * `Ok(Permission)` - The created permission
    /// * `Err(PermissionError::AppNotFound)` - If the app doesn't exist
    /// * `Err(PermissionError::CodeAlreadyExists)` - If permission code already exists in this app
    /// 
    /// # Requirements
    /// - 7.1: Create permission scoped to specific app only
    /// - 7.2: Reject duplicate permission code within the same app
    pub async fn create_permission(&self, app_id: Uuid, code: &str) -> Result<Permission, PermissionError> {
        // Verify app exists (Requirement 7.1)
        let app = self.app_repo.find_by_id(app_id).await
            .map_err(|e| PermissionError::InternalError(e.into()))?;
        
        if app.is_none() {
            return Err(PermissionError::AppNotFound);
        }

        // Create permission - code uniqueness within app is enforced by database constraint
        // Requirements: 7.1, 7.2
        self.permission_repo.create_permission(app_id, code).await
    }

    /// Get all permissions for a specific app
    /// 
    /// # Arguments
    /// * `app_id` - The UUID of the app
    /// 
    /// # Returns
    /// * `Ok(Vec<Permission>)` - List of permissions for the app
    pub async fn get_permissions_by_app(&self, app_id: Uuid) -> Result<Vec<Permission>, PermissionError> {
        self.permission_repo.find_by_app_id(app_id).await
    }

    /// Assign a permission to a role
    /// 
    /// # Arguments
    /// * `role_id` - The UUID of the role
    /// * `permission_id` - The UUID of the permission to assign
    /// 
    /// # Returns
    /// * `Ok(())` - Permission was successfully assigned
    /// * `Err(PermissionError::NotFound)` - If role or permission doesn't exist
    /// * `Err(PermissionError::CrossAppAssignment)` - If permission and role belong to different apps
    /// 
    /// # Requirements
    /// - 9.1: Create role-permission association
    /// - 9.2: Reject cross-app permission assignment
    pub async fn assign_permission_to_role(
        &self,
        role_id: Uuid,
        permission_id: Uuid,
    ) -> Result<(), PermissionError> {
        // Verify role exists
        let role = self.role_repo.find_by_id(role_id).await
            .map_err(|e| PermissionError::InternalError(e.into()))?;
        
        let role = match role {
            Some(r) => r,
            None => return Err(PermissionError::NotFound),
        };

        // Verify permission exists
        let permission = self.permission_repo.find_by_id(permission_id).await?;
        
        let permission = match permission {
            Some(p) => p,
            None => return Err(PermissionError::NotFound),
        };

        // Verify role and permission belong to the same app (Requirement 9.2)
        if role.app_id != permission.app_id {
            return Err(PermissionError::CrossAppAssignment);
        }

        // Create the role-permission association (Requirement 9.1)
        self.role_permission_repo.assign_permission(role_id, permission_id).await?;

        Ok(())
    }
}

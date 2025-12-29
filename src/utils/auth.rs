use crate::utils::jwt::Claims;

/// Check if a user has a specific permission within an app scope
/// 
/// # Arguments
/// * `claims` - The JWT claims containing user's apps, roles, and permissions
/// * `app_code` - The app code to check permission in
/// * `permission` - The permission code to check for
/// 
/// # Returns
/// * `true` - If the user has the specified permission in the specified app
/// * `false` - If the user does not have the permission or the app doesn't exist in claims
/// 
/// # Requirements
/// - 12.1: Provide a mechanism to check if a user has a specific permission within an app scope
/// - 12.2: Ensure an app can only read permissions within its own scope
/// 
/// # Examples
/// ```
/// use std::collections::HashMap;
/// use auth_server::utils::jwt::{Claims, AppClaims};
/// use auth_server::utils::auth::can;
/// use uuid::Uuid;
/// 
/// let mut apps = HashMap::new();
/// apps.insert(
///     "my_app".to_string(),
///     AppClaims {
///         roles: vec!["admin".to_string()],
///         permissions: vec!["read".to_string(), "write".to_string()],
///     },
/// );
/// 
/// let claims = Claims::new(Uuid::new_v4(), apps, 900);
/// 
/// assert!(can(&claims, "my_app", "read"));
/// assert!(can(&claims, "my_app", "write"));
/// assert!(!can(&claims, "my_app", "delete"));
/// assert!(!can(&claims, "other_app", "read"));
/// ```
pub fn can(claims: &Claims, app_code: &str, permission: &str) -> bool {
    // Get the app claims for the specified app code
    // If the app doesn't exist in claims, return false (cross-app isolation)
    match claims.apps.get(app_code) {
        Some(app_claims) => {
            // Check if the permission exists in the app's permissions list
            app_claims.permissions.contains(&permission.to_string())
        }
        None => false,
    }
}

/// Check if a user has a specific role within an app scope
/// 
/// # Arguments
/// * `claims` - The JWT claims containing user's apps, roles, and permissions
/// * `app_code` - The app code to check role in
/// * `role` - The role name to check for
/// 
/// # Returns
/// * `true` - If the user has the specified role in the specified app
/// * `false` - If the user does not have the role or the app doesn't exist in claims
pub fn has_role(claims: &Claims, app_code: &str, role: &str) -> bool {
    match claims.apps.get(app_code) {
        Some(app_claims) => {
            app_claims.roles.contains(&role.to_string())
        }
        None => false,
    }
}

/// Check if a user has any of the specified permissions within an app scope
/// 
/// # Arguments
/// * `claims` - The JWT claims containing user's apps, roles, and permissions
/// * `app_code` - The app code to check permissions in
/// * `permissions` - The list of permission codes to check for (any match returns true)
/// 
/// # Returns
/// * `true` - If the user has any of the specified permissions in the specified app
/// * `false` - If the user has none of the permissions or the app doesn't exist in claims
pub fn can_any(claims: &Claims, app_code: &str, permissions: &[&str]) -> bool {
    match claims.apps.get(app_code) {
        Some(app_claims) => {
            permissions.iter().any(|p| app_claims.permissions.contains(&p.to_string()))
        }
        None => false,
    }
}

/// Check if a user has all of the specified permissions within an app scope
/// 
/// # Arguments
/// * `claims` - The JWT claims containing user's apps, roles, and permissions
/// * `app_code` - The app code to check permissions in
/// * `permissions` - The list of permission codes to check for (all must match)
/// 
/// # Returns
/// * `true` - If the user has all of the specified permissions in the specified app
/// * `false` - If the user is missing any permission or the app doesn't exist in claims
pub fn can_all(claims: &Claims, app_code: &str, permissions: &[&str]) -> bool {
    match claims.apps.get(app_code) {
        Some(app_claims) => {
            permissions.iter().all(|p| app_claims.permissions.contains(&p.to_string()))
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::jwt::AppClaims;
    use std::collections::HashMap;
    use uuid::Uuid;

    fn create_test_claims() -> Claims {
        let mut apps = HashMap::new();
        apps.insert(
            "app_a".to_string(),
            AppClaims {
                roles: vec!["admin".to_string(), "user".to_string()],
                permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string()],
            },
        );
        apps.insert(
            "app_b".to_string(),
            AppClaims {
                roles: vec!["viewer".to_string()],
                permissions: vec!["read".to_string()],
            },
        );
        Claims::new(Uuid::new_v4(), apps, 900)
    }

    // Property 28: Authorization Check Correctness
    // For any user with permission P in app A, can(claims, "A", "P") SHALL return true.
    // For any user without permission P in app A, can(claims, "A", "P") SHALL return false.

    #[test]
    fn test_can_returns_true_for_existing_permission() {
        let claims = create_test_claims();
        
        // User has "read" permission in app_a
        assert!(can(&claims, "app_a", "read"));
        assert!(can(&claims, "app_a", "write"));
        assert!(can(&claims, "app_a", "delete"));
        
        // User has "read" permission in app_b
        assert!(can(&claims, "app_b", "read"));
    }

    #[test]
    fn test_can_returns_false_for_missing_permission() {
        let claims = create_test_claims();
        
        // User does not have "admin" permission in app_a
        assert!(!can(&claims, "app_a", "admin"));
        
        // User does not have "write" or "delete" permission in app_b
        assert!(!can(&claims, "app_b", "write"));
        assert!(!can(&claims, "app_b", "delete"));
    }

    // Property 29: Cross-App Permission Isolation
    // For any user with permission P in app A but not in app B, can(claims, "B", "P") SHALL return false.

    #[test]
    fn test_cross_app_permission_isolation() {
        let claims = create_test_claims();
        
        // User has "write" in app_a but NOT in app_b
        assert!(can(&claims, "app_a", "write"));
        assert!(!can(&claims, "app_b", "write"));
        
        // User has "delete" in app_a but NOT in app_b
        assert!(can(&claims, "app_a", "delete"));
        assert!(!can(&claims, "app_b", "delete"));
    }

    #[test]
    fn test_can_returns_false_for_nonexistent_app() {
        let claims = create_test_claims();
        
        // App doesn't exist in claims
        assert!(!can(&claims, "nonexistent_app", "read"));
        assert!(!can(&claims, "app_c", "write"));
    }

    #[test]
    fn test_has_role_returns_true_for_existing_role() {
        let claims = create_test_claims();
        
        assert!(has_role(&claims, "app_a", "admin"));
        assert!(has_role(&claims, "app_a", "user"));
        assert!(has_role(&claims, "app_b", "viewer"));
    }

    #[test]
    fn test_has_role_returns_false_for_missing_role() {
        let claims = create_test_claims();
        
        assert!(!has_role(&claims, "app_a", "viewer"));
        assert!(!has_role(&claims, "app_b", "admin"));
        assert!(!has_role(&claims, "nonexistent_app", "admin"));
    }

    #[test]
    fn test_can_any_returns_true_if_any_permission_exists() {
        let claims = create_test_claims();
        
        // Has "read" but not "execute"
        assert!(can_any(&claims, "app_a", &["read", "execute"]));
        
        // Has both "read" and "write"
        assert!(can_any(&claims, "app_a", &["read", "write"]));
    }

    #[test]
    fn test_can_any_returns_false_if_no_permission_exists() {
        let claims = create_test_claims();
        
        // Has neither "execute" nor "admin"
        assert!(!can_any(&claims, "app_a", &["execute", "admin_perm"]));
        
        // App doesn't exist
        assert!(!can_any(&claims, "nonexistent", &["read"]));
    }

    #[test]
    fn test_can_all_returns_true_if_all_permissions_exist() {
        let claims = create_test_claims();
        
        // Has all: read, write, delete
        assert!(can_all(&claims, "app_a", &["read", "write", "delete"]));
        
        // Has read in app_b
        assert!(can_all(&claims, "app_b", &["read"]));
    }

    #[test]
    fn test_can_all_returns_false_if_any_permission_missing() {
        let claims = create_test_claims();
        
        // Missing "execute"
        assert!(!can_all(&claims, "app_a", &["read", "execute"]));
        
        // Missing "write" in app_b
        assert!(!can_all(&claims, "app_b", &["read", "write"]));
        
        // App doesn't exist
        assert!(!can_all(&claims, "nonexistent", &["read"]));
    }

    #[test]
    fn test_empty_permissions_list() {
        let claims = create_test_claims();
        
        // Empty list for can_any should return false
        assert!(!can_any(&claims, "app_a", &[]));
        
        // Empty list for can_all should return true (vacuous truth)
        assert!(can_all(&claims, "app_a", &[]));
    }

    #[test]
    fn test_claims_with_no_apps() {
        let claims = Claims::new(Uuid::new_v4(), HashMap::new(), 900);
        
        assert!(!can(&claims, "any_app", "any_permission"));
        assert!(!has_role(&claims, "any_app", "any_role"));
        assert!(!can_any(&claims, "any_app", &["read"]));
        assert!(!can_all(&claims, "any_app", &["read"]));
    }

    #[test]
    fn test_app_with_empty_permissions() {
        let mut apps = HashMap::new();
        apps.insert(
            "empty_app".to_string(),
            AppClaims {
                roles: vec!["user".to_string()],
                permissions: vec![],
            },
        );
        let claims = Claims::new(Uuid::new_v4(), apps, 900);
        
        assert!(!can(&claims, "empty_app", "read"));
        assert!(has_role(&claims, "empty_app", "user"));
    }
}

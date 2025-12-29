use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create role request
#[derive(Debug, Deserialize)]
pub struct CreateRoleRequest {
    pub name: String,
}

/// Role response
#[derive(Debug, Serialize)]
pub struct RoleResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub name: String,
}

/// Assign role to user request
#[derive(Debug, Deserialize)]
pub struct AssignRoleRequest {
    pub role_id: Uuid,
}

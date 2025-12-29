use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Create permission request
#[derive(Debug, Deserialize)]
pub struct CreatePermissionRequest {
    pub code: String,
}

/// Permission response
#[derive(Debug, Serialize)]
pub struct PermissionResponse {
    pub id: Uuid,
    pub app_id: Uuid,
    pub code: String,
}

/// Assign permission to role request
#[derive(Debug, Deserialize)]
pub struct AssignPermissionRequest {
    pub permission_id: Uuid,
}

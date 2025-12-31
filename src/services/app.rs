use chrono::Utc;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::App;
use crate::repositories::AppRepository;
use crate::utils::jwt::JwtManager;
use crate::utils::secret::{generate_secret, hash_secret, verify_secret};

/// Generate code with timestamp suffix (code_timestamp) like JS Date.now()
fn generate_code_with_timestamp(code: &str) -> String {
    let timestamp = Utc::now().timestamp_millis();
    format!("{}_{}", code, timestamp)
}

/// Service for app management operations
/// 
/// Handles creating and retrieving apps with code uniqueness validation.
#[derive(Clone)]
pub struct AppService {
    app_repo: AppRepository,
    jwt_manager: JwtManager,
}

impl AppService {
    /// Create a new AppService with the given database pool and JWT manager
    pub fn new(pool: MySqlPool, jwt_manager: JwtManager) -> Self {
        let app_repo = AppRepository::new(pool);
        Self { app_repo, jwt_manager }
    }

    /// Create a new app with unique code
    /// 
    /// # Arguments
    /// * `code` - Unique identifier code for the app
    /// * `name` - Display name for the app
    /// 
    /// # Returns
    /// * `Ok(App)` - The created app
    /// * `Err(AppError::CodeAlreadyExists)` - If code is already taken
    /// 
    /// # Requirements
    /// - 5.1: Register the app in the system with unique code and name
    /// - 5.2: Reject creation if code already exists
    pub async fn create_app(&self, code: &str, name: &str) -> Result<App, AppError> {
        // Create app - uniqueness is enforced by database constraint
        // Requirements: 5.1, 5.2
        self.app_repo.create_app(code, name).await
    }

    /// Create a new app with unique code and assign owner
    /// 
    /// # Arguments
    /// * `code` - Unique identifier code for the app
    /// * `name` - Display name for the app
    /// * `owner_id` - The user ID of the app owner
    /// 
    /// # Returns
    /// * `Ok(App)` - The created app with owner assigned
    /// * `Err(AppError::CodeAlreadyExists)` - If code is already taken
    /// 
    /// # Requirements
    /// - 1.1: Assign user as App_Owner when creating app
    /// - 5.1: Register the app in the system with unique code and name
    /// - 5.2: Reject creation if code already exists
    pub async fn create_app_with_owner(&self, code: &str, name: &str, owner_id: Uuid) -> Result<App, AppError> {
        // Create app with owner - uniqueness is enforced by database constraint
        // Requirements: 1.1, 5.1, 5.2
        self.app_repo.create_with_owner(code, name, owner_id).await
    }

    /// Get an app by its UUID
    /// 
    /// # Arguments
    /// * `id` - The app's UUID
    /// 
    /// # Returns
    /// * `Ok(Some(App))` - The app if found
    /// * `Ok(None)` - If no app exists with this ID
    pub async fn get_app(&self, id: Uuid) -> Result<Option<App>, AppError> {
        self.app_repo.find_by_id(id).await
    }

    /// Get an app by its unique code
    /// 
    /// # Arguments
    /// * `code` - The app's unique code
    /// 
    /// # Returns
    /// * `Ok(Some(App))` - The app if found
    /// * `Ok(None)` - If no app exists with this code
    pub async fn get_app_by_code(&self, code: &str) -> Result<Option<App>, AppError> {
        self.app_repo.find_by_code(code).await
    }

    /// Create a new app with a generated secret
    /// 
    /// # Arguments
    /// * `code` - Unique identifier code for the app (will be suffixed with current date)
    /// * `name` - Display name for the app
    /// * `owner_id` - The user ID of the app owner
    /// 
    /// # Returns
    /// * `Ok((App, String))` - The created app and the plain-text secret (returned only once)
    /// * `Err(AppError::CodeAlreadyExists)` - If code is already taken
    /// 
    /// # Requirements
    /// - 1.1: Generate a cryptographically secure random App_Secret
    /// - 1.2: Return the plain-text secret only once during creation
    /// - 1.3: Store only the hashed value using bcrypt
    pub async fn create_app_with_secret(
        &self,
        code: &str,
        name: &str,
        owner_id: Uuid,
    ) -> Result<(App, String), AppError> {
        // Generate code with timestamp suffix (code_timestamp)
        let code_with_timestamp = generate_code_with_timestamp(code);
        
        // Generate a cryptographically secure secret (Requirements: 1.1, 1.4)
        let plain_secret = generate_secret();
        
        // Hash the secret using bcrypt (Requirements: 1.3, 9.2)
        let secret_hash = hash_secret(&plain_secret)?;
        
        // Store the app with the hashed secret
        let app = self.app_repo.create_with_secret(&code_with_timestamp, name, owner_id, &secret_hash).await?;
        
        // Return the app and plain-text secret (Requirements: 1.2 - only returned once)
        Ok((app, plain_secret))
    }

    /// Authenticate an app using App ID and Secret
    /// 
    /// # Arguments
    /// * `app_id` - The app's UUID
    /// * `secret` - The plain-text secret to verify
    /// 
    /// # Returns
    /// * `Ok(String)` - The access token if authentication succeeds
    /// * `Err(AppError::InvalidCredentials)` - If app_id doesn't exist or secret is invalid
    /// 
    /// # Requirements
    /// - 3.1: Authenticate the request when valid App_ID and App_Secret are provided
    /// - 3.3: Reject with 401 Unauthorized if App_Secret is invalid
    /// - 3.4: Reject with 401 Unauthorized if App_ID does not exist
    /// - 9.3: Not reveal whether the App_ID or Secret was incorrect
    pub async fn authenticate_app(&self, app_id: Uuid, secret: &str) -> Result<String, AppError> {
        // Get the app's secret hash (Requirements: 3.4 - generic error if app doesn't exist)
        let secret_hash = self.app_repo.get_secret_hash(app_id).await?;
        
        // If app doesn't exist or has no secret, return generic error (Requirements: 9.3)
        let hash = match secret_hash {
            Some(h) => h,
            None => return Err(AppError::InvalidCredentials),
        };
        
        // Verify the secret using bcrypt (constant-time comparison) (Requirements: 3.5)
        let is_valid = verify_secret(secret, &hash)?;
        
        if !is_valid {
            // Return generic error - don't reveal if app_id or secret was wrong (Requirements: 9.3)
            return Err(AppError::InvalidCredentials);
        }
        
        // Generate and return an app token (Requirements: 3.1, 3.2)
        self.jwt_manager.create_app_token(app_id)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("Token creation failed: {}", e)))
    }

    /// Regenerate the secret for an app (owner only)
    /// 
    /// # Arguments
    /// * `app_id` - The app's UUID
    /// * `requester_id` - The user ID of the requester
    /// 
    /// # Returns
    /// * `Ok(String)` - The new plain-text secret (returned only once)
    /// * `Err(AppError::NotAppOwner)` - If requester is not the app owner
    /// * `Err(AppError::NotFound)` - If app doesn't exist
    /// 
    /// # Requirements
    /// - 2.1: Generate a new App_Secret when owner requests regeneration
    /// - 2.2: Invalidate the previous secret immediately
    /// - 2.4: Reject with 403 Forbidden if non-owner attempts regeneration
    pub async fn regenerate_secret(
        &self,
        app_id: Uuid,
        requester_id: Uuid,
    ) -> Result<String, AppError> {
        // Verify the requester is the app owner (Requirements: 2.4)
        let is_owner = self.app_repo.is_owner(app_id, requester_id).await?;
        
        if !is_owner {
            return Err(AppError::NotAppOwner);
        }
        
        // Generate a new cryptographically secure secret (Requirements: 2.1)
        let plain_secret = generate_secret();
        
        // Hash the new secret
        let secret_hash = hash_secret(&plain_secret)?;
        
        // Update the secret hash in the database (Requirements: 2.2 - invalidates previous)
        self.app_repo.update_secret_hash(app_id, &secret_hash).await?;
        
        // Return the new plain-text secret (returned only once)
        Ok(plain_secret)
    }
}

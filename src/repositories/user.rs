use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::User;


/// Repository for user database operations
#[derive(Clone)]
pub struct UserRepository {
    pool: MySqlPool,
}

impl UserRepository {
    /// Create a new UserRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new user with the given email and password hash
    /// Returns AuthError::EmailAlreadyExists if email is taken
    /// Requirements: 1.1, 1.2
    pub async fn create_user(&self, email: &str, password_hash: &str) -> Result<User, AuthError> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO users (id, email, password_hash)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(email)
        .bind(password_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                // MySQL duplicate entry error code is 1062
                if db_err.code().map(|c| c == "23000").unwrap_or(false) 
                    || db_err.message().contains("Duplicate entry") {
                    return AuthError::EmailAlreadyExists;
                }
            }
            AuthError::InternalError(e.into())
        })?;

        // Fetch the created user
        self.find_by_id(id).await?.ok_or(AuthError::InternalError(anyhow::anyhow!("Failed to fetch created user")))
    }

    /// Find a user by their email address
    /// Requirements: 2.1
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, is_active, email_verified, is_system_admin, created_at
            FROM users
            WHERE email = ?
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(user)
    }

    /// Find a user by their UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, is_active, email_verified, is_system_admin, created_at
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(user)
    }

    /// Update a user's password hash
    /// Requirements: 4.3
    pub async fn update_password(&self, user_id: Uuid, new_password_hash: &str) -> Result<(), AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET password_hash = ?
            WHERE id = ?
            "#,
        )
        .bind(new_password_hash)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }

        Ok(())
    }

    /// Set a user's active status
    pub async fn set_active(&self, user_id: Uuid, is_active: bool) -> Result<(), AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET is_active = ?
            WHERE id = ?
            "#,
        )
        .bind(is_active)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }

        Ok(())
    }

    /// Check if a user is a system admin
    /// Requirements: 7.1
    pub async fn is_system_admin(&self, user_id: Uuid) -> Result<bool, AuthError> {
        let result = sqlx::query_scalar::<_, bool>(
            r#"
            SELECT is_system_admin
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(user_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(result.unwrap_or(false))
    }

    /// Set a user's system admin status
    /// Requirements: 7.1
    pub async fn set_system_admin(&self, user_id: Uuid, is_admin: bool) -> Result<(), AuthError> {
        let result = sqlx::query(
            r#"
            UPDATE users
            SET is_system_admin = ?
            WHERE id = ?
            "#,
        )
        .bind(is_admin)
        .bind(user_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AuthError::UserNotFound);
        }

        Ok(())
    }

    /// Deactivate a user globally (set is_active = false)
    /// Requirements: 7.5
    pub async fn deactivate(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.set_active(user_id, false).await
    }

    /// List all users with pagination (for admin)
    /// Requirements: 7.4
    pub async fn list_all(&self, page: u32, limit: u32) -> Result<Vec<User>, AuthError> {
        let offset = (page.saturating_sub(1)) * limit;

        let users = sqlx::query_as::<_, User>(
            r#"
            SELECT id, email, password_hash, is_active, email_verified, is_system_admin, created_at
            FROM users
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(users)
    }

    /// Count total users (for pagination)
    pub async fn count_all(&self) -> Result<u64, AuthError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM users
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::InternalError(e.into()))?;

        Ok(count as u64)
    }
}


#[cfg(test)]
mod tests {
    // Feature: auth-server, Property 2: Email Uniqueness
    // Feature: auth-server, Property 4: Valid Registration Creates User
    // Validates: Requirements 1.1, 1.2

    use proptest::prelude::*;
    use sqlx::MySqlPool;

    use super::UserRepository;
    use crate::error::AuthError;

    /// Generate valid email addresses for property testing
    fn email_strategy() -> impl Strategy<Value = String> {
        ("[a-z]{3,10}", "[a-z]{3,8}")
            .prop_map(|(local, domain)| format!("test_{}@{}.com", local, domain))
    }

    /// Generate password hash strings (simulating argon2 hashes)
    fn password_hash_strategy() -> impl Strategy<Value = String> {
        "[a-zA-Z0-9]{60,80}".prop_map(|s| format!("$argon2id$v=19$m=19456,t=2,p=1${}", s))
    }

    /// Setup test database connection
    async fn setup_test_db() -> MySqlPool {
        dotenvy::dotenv().ok();
        
        let database_url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set for tests");

        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
            .expect("Failed to create pool");

        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    /// Clean up test data after each test
    async fn cleanup_test_data(pool: &MySqlPool, emails: &[String]) {
        for email in emails {
            let _ = sqlx::query("DELETE FROM users WHERE email = ?")
                .bind(email)
                .execute(pool)
                .await;
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        /// Property 2: Email Uniqueness
        #[test]
        fn prop_email_uniqueness(
            email in email_strategy(),
            password_hash1 in password_hash_strategy(),
            password_hash2 in password_hash_strategy()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = setup_test_db().await;
                let repo = UserRepository::new(pool.clone());

                cleanup_test_data(&pool, &[email.clone()]).await;

                let result1 = repo.create_user(&email, &password_hash1).await;
                prop_assert!(result1.is_ok(), "First user creation should succeed");

                let result2 = repo.create_user(&email, &password_hash2).await;
                prop_assert!(result2.is_err(), "Second user creation with same email should fail");
                
                match result2 {
                    Err(AuthError::EmailAlreadyExists) => {}
                    Err(e) => {
                        prop_assert!(false, "Expected EmailAlreadyExists error, got: {:?}", e);
                    }
                    Ok(_) => {
                        prop_assert!(false, "Should not allow duplicate email");
                    }
                }

                cleanup_test_data(&pool, &[email]).await;
                Ok(())
            })?;
        }

        /// Property 4: Valid Registration Creates User
        #[test]
        fn prop_valid_registration_creates_user(
            email in email_strategy(),
            password_hash in password_hash_strategy()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = setup_test_db().await;
                let repo = UserRepository::new(pool.clone());

                cleanup_test_data(&pool, &[email.clone()]).await;

                let create_result = repo.create_user(&email, &password_hash).await;
                prop_assert!(create_result.is_ok(), "User creation should succeed");

                let created_user = create_result.unwrap();

                let find_by_email_result = repo.find_by_email(&email).await;
                prop_assert!(find_by_email_result.is_ok(), "Find by email should succeed");
                
                let found_user = find_by_email_result.unwrap();
                prop_assert!(found_user.is_some(), "User should be found by email");
                
                let found_user = found_user.unwrap();
                prop_assert_eq!(found_user.id, created_user.id, "User IDs should match");
                prop_assert_eq!(&found_user.email, &email, "Email should match");

                cleanup_test_data(&pool, &[email.clone()]).await;
                Ok(())
            })?;
        }
    }
}

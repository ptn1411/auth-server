use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::App;
use crate::models::User;

/// Repository for app database operations
#[derive(Clone)]
pub struct AppRepository {
    pool: MySqlPool,
}

impl AppRepository {
    /// Create a new AppRepository with the given database pool
    pub fn new(pool: MySqlPool) -> Self {
        Self { pool }
    }

    /// Create a new app with the given code and name
    /// Returns AppError::CodeAlreadyExists if code is taken
    /// Requirements: 5.1, 5.2
    pub async fn create_app(&self, code: &str, name: &str) -> Result<App, AppError> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO apps (id, code, name)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(code)
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry") {
                    return AppError::CodeAlreadyExists;
                }
            }
            AppError::InternalError(e.into())
        })?;

        self.find_by_id(id).await?.ok_or(AppError::InternalError(anyhow::anyhow!("Failed to fetch created app")))
    }

    /// Create a new app with the given code, name, and owner
    /// Returns AppError::CodeAlreadyExists if code is taken
    /// Requirements: 1.1, 1.3
    pub async fn create_with_owner(&self, code: &str, name: &str, owner_id: Uuid) -> Result<App, AppError> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO apps (id, code, name, owner_id)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(code)
        .bind(name)
        .bind(owner_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry") {
                    return AppError::CodeAlreadyExists;
                }
            }
            AppError::InternalError(e.into())
        })?;

        self.find_by_id(id).await?.ok_or(AppError::InternalError(anyhow::anyhow!("Failed to fetch created app")))
    }

    /// Find an app by its UUID
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<App>, AppError> {
        let app = sqlx::query_as::<_, App>(
            r#"
            SELECT id, code, name, owner_id, secret_hash
            FROM apps
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(app)
    }

    /// Find an app by its unique code
    /// Requirements: 5.1
    pub async fn find_by_code(&self, code: &str) -> Result<Option<App>, AppError> {
        let app = sqlx::query_as::<_, App>(
            r#"
            SELECT id, code, name, owner_id, secret_hash
            FROM apps
            WHERE code = ?
            "#,
        )
        .bind(code)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(app)
    }

    /// Check if a user is the owner of an app
    /// Requirements: 1.3
    pub async fn is_owner(&self, app_id: Uuid, user_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM apps
            WHERE id = ? AND owner_id = ?
            "#,
        )
        .bind(app_id.to_string())
        .bind(user_id.to_string())
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(result > 0)
    }

    /// Get the owner of an app
    /// Requirements: 1.3
    pub async fn get_owner(&self, app_id: Uuid) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as::<_, User>(
            r#"
            SELECT u.id, u.email, u.password_hash, u.is_active, u.email_verified, u.is_system_admin, u.created_at
            FROM users u
            INNER JOIN apps a ON a.owner_id = u.id
            WHERE a.id = ?
            "#,
        )
        .bind(app_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(user)
    }

    /// List all apps with pagination (for admin)
    /// Requirements: 7.4
    pub async fn list_all(&self, page: u32, limit: u32) -> Result<Vec<App>, AppError> {
        let offset = (page.saturating_sub(1)) * limit;

        let apps = sqlx::query_as::<_, App>(
            r#"
            SELECT id, code, name, owner_id, secret_hash
            FROM apps
            ORDER BY code ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(apps)
    }

    /// Count total apps (for pagination)
    pub async fn count_all(&self) -> Result<u64, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*) as count
            FROM apps
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        Ok(count as u64)
    }

    /// Create a new app with the given code, name, owner, and secret hash
    /// Returns AppError::CodeAlreadyExists if code is taken
    /// Requirements: 1.1, 1.3, 2.1
    pub async fn create_with_secret(
        &self,
        code: &str,
        name: &str,
        owner_id: Uuid,
        secret_hash: &str,
    ) -> Result<App, AppError> {
        let id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO apps (id, code, name, owner_id, secret_hash)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(id.to_string())
        .bind(code)
        .bind(name)
        .bind(owner_id.to_string())
        .bind(secret_hash)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e {
                if db_err.code().map(|c| c == "23000").unwrap_or(false)
                    || db_err.message().contains("Duplicate entry") {
                    return AppError::CodeAlreadyExists;
                }
            }
            AppError::InternalError(e.into())
        })?;

        self.find_by_id(id).await?.ok_or(AppError::InternalError(anyhow::anyhow!("Failed to fetch created app")))
    }

    /// Update the secret hash for an app
    /// Requirements: 2.1, 2.2
    pub async fn update_secret_hash(&self, app_id: Uuid, secret_hash: &str) -> Result<(), AppError> {
        let result = sqlx::query(
            r#"
            UPDATE apps
            SET secret_hash = ?
            WHERE id = ?
            "#,
        )
        .bind(secret_hash)
        .bind(app_id.to_string())
        .execute(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    /// Get the secret hash for an app (for verification)
    /// Requirements: 1.3
    pub async fn get_secret_hash(&self, app_id: Uuid) -> Result<Option<String>, AppError> {
        let hash = sqlx::query_scalar::<_, Option<String>>(
            r#"
            SELECT secret_hash
            FROM apps
            WHERE id = ?
            "#,
        )
        .bind(app_id.to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| AppError::InternalError(e.into()))?;

        // Flatten Option<Option<String>> to Option<String>
        Ok(hash.flatten())
    }

    /// Update app details
    pub async fn update(&self, app_id: Uuid, name: Option<&str>, owner_id: Option<Uuid>) -> Result<App, AppError> {
        let mut updates = Vec::new();
        
        if name.is_some() {
            updates.push("name = ?");
        }
        if owner_id.is_some() {
            updates.push("owner_id = ?");
        }

        if updates.is_empty() {
            return self.find_by_id(app_id).await?.ok_or(AppError::NotFound);
        }

        let query = format!(
            "UPDATE apps SET {} WHERE id = ?",
            updates.join(", ")
        );

        let mut q = sqlx::query(&query);
        
        if let Some(n) = name {
            q = q.bind(n);
        }
        if let Some(o) = owner_id {
            q = q.bind(o.to_string());
        }
        q = q.bind(app_id.to_string());

        let result = q.execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        self.find_by_id(app_id).await?.ok_or(AppError::NotFound)
    }

    /// Delete an app
    pub async fn delete(&self, app_id: Uuid) -> Result<(), AppError> {
        let result = sqlx::query("DELETE FROM apps WHERE id = ?")
            .bind(app_id.to_string())
            .execute(&self.pool)
            .await
            .map_err(|e| AppError::InternalError(e.into()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use sqlx::MySqlPool;

    use super::AppRepository;
    use crate::error::AppError;

    fn app_code_strategy() -> impl Strategy<Value = String> {
        "[a-z]{3,10}".prop_map(|s| format!("test_app_{}", s))
    }

    fn app_name_strategy() -> impl Strategy<Value = String> {
        "[A-Za-z ]{5,20}".prop_map(|s| format!("Test App {}", s.trim()))
    }

    async fn setup_test_db() -> MySqlPool {
        dotenvy::dotenv().ok();

        let database_url =
            std::env::var("DATABASE_URL").expect("DATABASE_URL must be set for tests");

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

    async fn cleanup_test_data(pool: &MySqlPool, codes: &[String]) {
        for code in codes {
            let _ = sqlx::query("DELETE FROM apps WHERE code = ?")
                .bind(code)
                .execute(pool)
                .await;
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10))]

        /// Property 16: App Code Uniqueness
        #[test]
        fn prop_app_code_uniqueness(
            code in app_code_strategy(),
            name1 in app_name_strategy(),
            name2 in app_name_strategy()
        ) {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let pool = setup_test_db().await;
                let repo = AppRepository::new(pool.clone());

                cleanup_test_data(&pool, &[code.clone()]).await;

                let result1 = repo.create_app(&code, &name1).await;
                prop_assert!(result1.is_ok(), "First app creation should succeed");

                let created_app = result1.unwrap();
                prop_assert_eq!(&created_app.code, &code, "Created app code should match");

                let result2 = repo.create_app(&code, &name2).await;
                prop_assert!(result2.is_err(), "Second app creation with same code should fail");

                match result2 {
                    Err(AppError::CodeAlreadyExists) => {}
                    Err(e) => {
                        prop_assert!(false, "Expected CodeAlreadyExists error, got: {:?}", e);
                    }
                    Ok(_) => {
                        prop_assert!(false, "Should not allow duplicate app code");
                    }
                }

                cleanup_test_data(&pool, &[code]).await;
                Ok(())
            })?;
        }
    }
}

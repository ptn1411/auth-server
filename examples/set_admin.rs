use sqlx::mysql::MySqlPoolOptions;
use argon2::{password_hash::{SaltString, PasswordHasher}, Argon2};
use rand::rngs::OsRng;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = MySqlPoolOptions::new().connect(&database_url).await?;
    
    let email = "admin@test.com";
    let password = "Admin123!@#";
    
    // Check if user exists
    let exists: Option<(String,)> = sqlx::query_as("SELECT id FROM users WHERE email = ?")
        .bind(email)
        .fetch_optional(&pool)
        .await?;
    
    if exists.is_none() {
        // Create user
        let id = Uuid::new_v4();
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("Failed to hash password: {}", e))?
            .to_string();
        
        sqlx::query(
            "INSERT INTO users (id, email, password_hash, is_active, is_system_admin) VALUES (?, ?, ?, TRUE, TRUE)"
        )
        .bind(id.to_string())
        .bind(email)
        .bind(&password_hash)
        .execute(&pool)
        .await?;
        
        println!("User {} created as system admin", email);
    } else {
        // Update existing user
        sqlx::query("UPDATE users SET is_system_admin = TRUE, failed_login_attempts = 0, locked_until = NULL WHERE email = ?")
            .bind(email)
            .execute(&pool)
            .await?;
        
        println!("User {} updated to system admin and unlocked", email);
    }
    
    Ok(())
}

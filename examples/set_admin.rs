use sqlx::mysql::MySqlPoolOptions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let database_url = std::env::var("DATABASE_URL")?;
    let pool = MySqlPoolOptions::new().connect(&database_url).await?;
    
    // Set as system admin and unlock account
    sqlx::query("UPDATE users SET is_system_admin = TRUE, failed_login_attempts = 0, locked_until = NULL WHERE email = ?")
        .bind("admin@test.com")
        .execute(&pool)
        .await?;
    
    println!("User admin@test.com updated to system admin and unlocked");
    Ok(())
}

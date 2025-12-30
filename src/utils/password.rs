use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use sha2::{Digest, Sha256};

use crate::error::AuthError;

/// Hash a password using Argon2id algorithm
/// 
/// # Arguments
/// * `password` - The plain text password to hash
/// 
/// # Returns
/// * `Ok(String)` - The hashed password as a PHC string
/// * `Err(AuthError)` - If hashing fails
/// 
/// # Requirements
/// - 1.1: Create user with hashed password using argon2
/// - 1.5: Never store passwords in plain text
pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Password hashing failed: {}", e)))
}

/// Verify a password against a stored hash
/// 
/// # Arguments
/// * `password` - The plain text password to verify
/// * `hash` - The stored password hash (PHC string format)
/// 
/// # Returns
/// * `Ok(true)` - If the password matches
/// * `Ok(false)` - If the password does not match
/// * `Err(AuthError)` - If verification fails due to invalid hash format
/// 
/// # Requirements
/// - 2.1: Verify credentials during login
pub fn verify_password(password: &str, hash: &str) -> Result<bool, AuthError> {
    let parsed_hash = PasswordHash::new(hash)
        .map_err(|e| AuthError::InternalError(anyhow::anyhow!("Invalid password hash format: {}", e)))?;
    
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// Hash a token using SHA-256 for storage
/// Used for refresh tokens, session tokens, etc.
/// 
/// # Arguments
/// * `token` - The token to hash
/// 
/// # Returns
/// * `Ok(String)` - The hex-encoded SHA-256 hash
/// * `Err(AuthError)` - If hashing fails
pub fn hash_token(token: &str) -> Result<String, AuthError> {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password_produces_valid_hash() {
        let password = "secure_password123";
        let hash = hash_password(password).unwrap();
        
        // Hash should not be empty
        assert!(!hash.is_empty());
        
        // Hash should be in PHC format (starts with $argon2)
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_password_not_equal_to_plain_text() {
        // Property 1: Password Storage Security
        // For any registered user, the stored password_hash SHALL NOT equal the original plain-text password
        let password = "my_secret_password";
        let hash = hash_password(password).unwrap();
        
        assert_ne!(password, hash);
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password_123";
        let hash = hash_password(password).unwrap();
        
        let result = verify_password(password, &hash).unwrap();
        assert!(result);
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "correct_password";
        let wrong_password = "wrong_password";
        let hash = hash_password(password).unwrap();
        
        let result = verify_password(wrong_password, &hash).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_hash_password_different_salts() {
        // Each hash should be unique due to random salt
        let password = "same_password";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();
        
        assert_ne!(hash1, hash2);
        
        // But both should verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_verify_password_invalid_hash_format() {
        let password = "test";
        let invalid_hash = "not_a_valid_hash";
        
        let result = verify_password(password, invalid_hash);
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_empty_password() {
        // Empty password should still hash successfully
        let password = "";
        let hash = hash_password(password).unwrap();
        
        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("not_empty", &hash).unwrap());
    }
}

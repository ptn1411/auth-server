use rand::Rng;

use crate::error::AppError;

/// Minimum length for generated secrets
pub const MIN_SECRET_LENGTH: usize = 32;

/// Default length for generated secrets
pub const DEFAULT_SECRET_LENGTH: usize = 48;

/// Bcrypt cost factor for hashing secrets
/// Requirements: 9.2 - Use bcrypt with cost factor of at least 10
pub const BCRYPT_COST: u32 = 12;

/// Character set for secret generation (alphanumeric + special chars)
const SECRET_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*-_=+";

/// Generate a cryptographically secure random secret
/// 
/// # Returns
/// A random string of at least 32 characters containing alphanumeric and special characters
/// 
/// # Requirements
/// - 1.1: Generate a cryptographically secure random App_Secret
/// - 1.4: App_Secret SHALL be at least 32 characters long with alphanumeric and special characters
pub fn generate_secret() -> String {
    generate_secret_with_length(DEFAULT_SECRET_LENGTH)
}

/// Generate a cryptographically secure random secret with specified length
/// 
/// # Arguments
/// * `length` - The desired length of the secret (minimum 32)
/// 
/// # Returns
/// A random string of the specified length (or MIN_SECRET_LENGTH if length < MIN_SECRET_LENGTH)
pub fn generate_secret_with_length(length: usize) -> String {
    let length = length.max(MIN_SECRET_LENGTH);
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..SECRET_CHARSET.len());
            SECRET_CHARSET[idx] as char
        })
        .collect()
}

/// Hash a secret using bcrypt
/// 
/// # Arguments
/// * `secret` - The plain text secret to hash
/// 
/// # Returns
/// * `Ok(String)` - The bcrypt hash of the secret
/// * `Err(AppError)` - If hashing fails
/// 
/// # Requirements
/// - 1.3: Store only the hashed value using bcrypt
/// - 9.2: Use bcrypt with cost factor of at least 10
pub fn hash_secret(secret: &str) -> Result<String, AppError> {
    bcrypt::hash(secret, BCRYPT_COST)
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Secret hashing failed: {}", e)))
}

/// Verify a secret against a stored bcrypt hash
/// 
/// # Arguments
/// * `secret` - The plain text secret to verify
/// * `hash` - The stored bcrypt hash
/// 
/// # Returns
/// * `Ok(true)` - If the secret matches
/// * `Ok(false)` - If the secret does not match
/// * `Err(AppError)` - If verification fails
/// 
/// # Requirements
/// - 3.5: Use constant-time comparison when verifying App_Secret
pub fn verify_secret(secret: &str, hash: &str) -> Result<bool, AppError> {
    bcrypt::verify(secret, hash)
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("Secret verification failed: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret_minimum_length() {
        let secret = generate_secret();
        assert!(secret.len() >= MIN_SECRET_LENGTH, 
            "Secret length {} should be at least {}", secret.len(), MIN_SECRET_LENGTH);
    }

    #[test]
    fn test_generate_secret_default_length() {
        let secret = generate_secret();
        assert_eq!(secret.len(), DEFAULT_SECRET_LENGTH,
            "Default secret length should be {}", DEFAULT_SECRET_LENGTH);
    }

    #[test]
    fn test_generate_secret_with_custom_length() {
        let length = 64;
        let secret = generate_secret_with_length(length);
        assert_eq!(secret.len(), length);
    }

    #[test]
    fn test_generate_secret_enforces_minimum() {
        let secret = generate_secret_with_length(10); // Less than minimum
        assert!(secret.len() >= MIN_SECRET_LENGTH,
            "Secret should enforce minimum length of {}", MIN_SECRET_LENGTH);
    }

    #[test]
    fn test_generate_secret_contains_valid_chars() {
        let secret = generate_secret();
        let valid_chars: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*-_=+";
        
        for c in secret.chars() {
            assert!(valid_chars.contains(c), 
                "Secret contains invalid character: {}", c);
        }
    }

    #[test]
    fn test_generate_secret_uniqueness() {
        let secret1 = generate_secret();
        let secret2 = generate_secret();
        assert_ne!(secret1, secret2, "Generated secrets should be unique");
    }

    #[test]
    fn test_hash_secret_produces_bcrypt_hash() {
        let secret = "test_secret_123";
        let hash = hash_secret(secret).unwrap();
        
        // Bcrypt hashes start with $2b$ or $2a$
        assert!(hash.starts_with("$2"), 
            "Hash should be bcrypt format, got: {}", hash);
    }

    #[test]
    fn test_hash_secret_not_equal_to_plain_text() {
        let secret = generate_secret();
        let hash = hash_secret(&secret).unwrap();
        
        assert_ne!(secret, hash, "Hash should not equal plain text secret");
    }

    #[test]
    fn test_verify_secret_correct() {
        let secret = generate_secret();
        let hash = hash_secret(&secret).unwrap();
        
        let result = verify_secret(&secret, &hash).unwrap();
        assert!(result, "Correct secret should verify successfully");
    }

    #[test]
    fn test_verify_secret_incorrect() {
        let secret = generate_secret();
        let wrong_secret = generate_secret();
        let hash = hash_secret(&secret).unwrap();
        
        let result = verify_secret(&wrong_secret, &hash).unwrap();
        assert!(!result, "Wrong secret should not verify");
    }

    #[test]
    fn test_hash_secret_different_hashes() {
        // Each hash should be unique due to random salt
        let secret = "same_secret";
        let hash1 = hash_secret(secret).unwrap();
        let hash2 = hash_secret(secret).unwrap();
        
        assert_ne!(hash1, hash2, "Same secret should produce different hashes");
        
        // But both should verify correctly
        assert!(verify_secret(secret, &hash1).unwrap());
        assert!(verify_secret(secret, &hash2).unwrap());
    }
}

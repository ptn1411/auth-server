use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::Rng;
use sha2::{Digest, Sha256};

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

// ============================================================================
// OAuth Token Hashing Utilities
// ============================================================================
// These functions are used for hashing OAuth tokens (access_token, refresh_token)
// before storing them in the database. Unlike passwords/secrets which use bcrypt,
// tokens use SHA256 because:
// 1. Tokens are already cryptographically random (high entropy)
// 2. SHA256 is faster, allowing efficient token lookups
// 3. No need for salt since tokens are unique and random
//
// Requirements:
// - 5.6: THE Authorization_Server SHALL hash tokens before storing in the database
// ============================================================================

/// Character set for OAuth token generation (URL-safe base64 characters)
const TOKEN_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Default length for generated OAuth tokens (256 bits of entropy)
pub const DEFAULT_TOKEN_LENGTH: usize = 43;

/// Generate a cryptographically secure random OAuth token
/// 
/// # Returns
/// A random URL-safe string suitable for use as an OAuth token
/// 
/// # Requirements
/// - 5.1: Generate secure access_token and refresh_token
pub fn generate_oauth_token() -> String {
    generate_oauth_token_with_length(DEFAULT_TOKEN_LENGTH)
}

/// Generate a cryptographically secure random OAuth token with specified length
/// 
/// # Arguments
/// * `length` - The desired length of the token
/// 
/// # Returns
/// A random URL-safe string of the specified length
pub fn generate_oauth_token_with_length(length: usize) -> String {
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..TOKEN_CHARSET.len());
            TOKEN_CHARSET[idx] as char
        })
        .collect()
}

/// Hash an OAuth token using SHA256 for storage
/// 
/// Uses SHA256 instead of bcrypt because:
/// - Tokens are already high-entropy random values
/// - Faster lookup performance for token validation
/// - No salt needed since tokens are unique
/// 
/// # Arguments
/// * `token` - The plain text token to hash
/// 
/// # Returns
/// The base64url-encoded SHA256 hash of the token
/// 
/// # Requirements
/// - 5.6: Hash tokens before storing in the database
pub fn hash_oauth_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Verify an OAuth token against a stored hash
/// 
/// # Arguments
/// * `token` - The plain text token to verify
/// * `hash` - The stored SHA256 hash (base64url-encoded)
/// 
/// # Returns
/// `true` if the token matches the hash, `false` otherwise
/// 
/// # Requirements
/// - 5.6: Verify tokens against stored hashes
pub fn verify_oauth_token(token: &str, hash: &str) -> bool {
    let computed_hash = hash_oauth_token(token);
    // Use constant-time comparison to prevent timing attacks
    constant_time_compare(&computed_hash, hash)
}

/// Constant-time string comparison to prevent timing attacks
fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
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

    // ========================================================================
    // OAuth Token Hashing Tests
    // ========================================================================

    #[test]
    fn test_generate_oauth_token_default_length() {
        let token = generate_oauth_token();
        assert_eq!(token.len(), DEFAULT_TOKEN_LENGTH,
            "Default token length should be {}", DEFAULT_TOKEN_LENGTH);
    }

    #[test]
    fn test_generate_oauth_token_with_custom_length() {
        let length = 64;
        let token = generate_oauth_token_with_length(length);
        assert_eq!(token.len(), length);
    }

    #[test]
    fn test_generate_oauth_token_url_safe_chars() {
        let token = generate_oauth_token();
        let valid_chars: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        
        for c in token.chars() {
            assert!(valid_chars.contains(c), 
                "Token contains invalid character: {}", c);
        }
    }

    #[test]
    fn test_generate_oauth_token_uniqueness() {
        let token1 = generate_oauth_token();
        let token2 = generate_oauth_token();
        assert_ne!(token1, token2, "Generated tokens should be unique");
    }

    #[test]
    fn test_hash_oauth_token_not_equal_to_plain_text() {
        let token = generate_oauth_token();
        let hash = hash_oauth_token(&token);
        
        assert_ne!(token, hash, "Hash should not equal plain text token");
    }

    #[test]
    fn test_hash_oauth_token_deterministic() {
        // Unlike bcrypt, SHA256 is deterministic (no salt)
        let token = "test_token_12345";
        let hash1 = hash_oauth_token(token);
        let hash2 = hash_oauth_token(token);
        
        assert_eq!(hash1, hash2, "Same token should produce same hash");
    }

    #[test]
    fn test_hash_oauth_token_produces_base64url() {
        let token = generate_oauth_token();
        let hash = hash_oauth_token(&token);
        
        // SHA256 produces 32 bytes = 43 base64 chars (without padding)
        assert_eq!(hash.len(), 43, "Hash should be 43 base64url characters");
        
        // Check that it's valid base64url characters
        for c in hash.chars() {
            assert!(c.is_ascii_alphanumeric() || c == '-' || c == '_',
                "Hash contains invalid base64url character: {}", c);
        }
    }

    #[test]
    fn test_verify_oauth_token_correct() {
        let token = generate_oauth_token();
        let hash = hash_oauth_token(&token);
        
        assert!(verify_oauth_token(&token, &hash), 
            "Correct token should verify successfully");
    }

    #[test]
    fn test_verify_oauth_token_incorrect() {
        let token = generate_oauth_token();
        let wrong_token = generate_oauth_token();
        let hash = hash_oauth_token(&token);
        
        assert!(!verify_oauth_token(&wrong_token, &hash), 
            "Wrong token should not verify");
    }

    #[test]
    fn test_verify_oauth_token_tampered_hash() {
        let token = generate_oauth_token();
        let hash = hash_oauth_token(&token);
        
        // Tamper with the hash
        let mut tampered_hash = hash.clone();
        if tampered_hash.ends_with('a') {
            tampered_hash.pop();
            tampered_hash.push('b');
        } else {
            tampered_hash.pop();
            tampered_hash.push('a');
        }
        
        assert!(!verify_oauth_token(&token, &tampered_hash), 
            "Token should not verify against tampered hash");
    }

    #[test]
    fn test_constant_time_compare_equal() {
        assert!(constant_time_compare("hello", "hello"));
        assert!(constant_time_compare("", ""));
    }

    #[test]
    fn test_constant_time_compare_not_equal() {
        assert!(!constant_time_compare("hello", "world"));
        assert!(!constant_time_compare("hello", "hello!"));
        assert!(!constant_time_compare("hello", "hell"));
    }
}

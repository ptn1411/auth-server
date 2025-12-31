use rand::Rng;
use sqlx::MySqlPool;
use uuid::Uuid;

use crate::error::AuthError;
use crate::models::UserMfaMethod;
use crate::repositories::MfaRepository;
use crate::utils::password::hash_token;

/// Number of backup codes to generate
const BACKUP_CODE_COUNT: usize = 10;

/// Length of each backup code
const BACKUP_CODE_LENGTH: usize = 8;

/// TOTP configuration
const TOTP_DIGITS: u32 = 6;
const TOTP_PERIOD: u64 = 30;

/// Service for MFA operations
#[derive(Clone)]
pub struct MfaService {
    repo: MfaRepository,
    totp_issuer: String,
}

impl MfaService {
    pub fn new(pool: MySqlPool, totp_issuer: String) -> Self {
        Self {
            repo: MfaRepository::new(pool),
            totp_issuer,
        }
    }

    // ========================================================================
    // TOTP Setup
    // ========================================================================

    /// Setup TOTP for a user - returns secret and provisioning URI
    pub async fn setup_totp(&self, user_id: Uuid, email: &str) -> Result<TotpSetupResponse, AuthError> {
        // Generate a random secret (20 bytes = 160 bits)
        let secret = generate_totp_secret();
        let secret_base32 = base32_encode(&secret);

        // Create the MFA method (not verified yet)
        let method = self
            .repo
            .create_method(user_id, "totp", Some(&secret_base32), None, None, true)
            .await?;

        // Generate provisioning URI for authenticator apps
        let provisioning_uri = format!(
            "otpauth://totp/{}:{}?secret={}&issuer={}&digits={}&period={}",
            self.totp_issuer, email, secret_base32, self.totp_issuer, TOTP_DIGITS, TOTP_PERIOD
        );

        Ok(TotpSetupResponse {
            method_id: method.id,
            secret: secret_base32,
            provisioning_uri,
        })
    }

    /// Verify TOTP setup with a code from the authenticator app
    pub async fn verify_totp_setup(
        &self,
        user_id: Uuid,
        method_id: Uuid,
        code: &str,
    ) -> Result<Vec<String>, AuthError> {
        // Get the MFA method
        let method = self
            .repo
            .find_method_by_id(method_id)
            .await?
            .ok_or(AuthError::InvalidMfaCode)?;

        // Verify it belongs to the user and is TOTP
        if method.user_id != user_id || method.method_type != "totp" {
            return Err(AuthError::InvalidMfaCode);
        }

        // Get the secret
        let secret = method
            .secret_encrypted
            .ok_or(AuthError::InternalError(anyhow::anyhow!("TOTP secret not found")))?;

        // Verify the code
        if !verify_totp_code(&secret, code)? {
            return Err(AuthError::InvalidMfaCode);
        }

        // Mark as verified
        self.repo.verify_method(method_id).await?;

        // Generate backup codes
        let backup_codes = self.generate_backup_codes(user_id).await?;

        Ok(backup_codes)
    }

    /// Verify a TOTP code during login
    pub async fn verify_totp(&self, user_id: Uuid, code: &str) -> Result<bool, AuthError> {
        // Get the primary TOTP method
        let method = self
            .repo
            .get_primary_method(user_id)
            .await?
            .ok_or(AuthError::InvalidMfaCode)?;

        if method.method_type != "totp" {
            return Err(AuthError::InvalidMfaCode);
        }

        let secret = method
            .secret_encrypted
            .ok_or(AuthError::InternalError(anyhow::anyhow!("TOTP secret not found")))?;

        let is_valid = verify_totp_code(&secret, code)?;

        if is_valid {
            self.repo.update_last_used(method.id).await?;
        }

        Ok(is_valid)
    }

    // ========================================================================
    // Backup Codes
    // ========================================================================

    /// Generate new backup codes for a user
    pub async fn generate_backup_codes(&self, user_id: Uuid) -> Result<Vec<String>, AuthError> {
        let mut codes = Vec::with_capacity(BACKUP_CODE_COUNT);
        let mut code_hashes = Vec::with_capacity(BACKUP_CODE_COUNT);

        for _ in 0..BACKUP_CODE_COUNT {
            let code = generate_backup_code();
            let hash = hash_token(&code)?;
            codes.push(code);
            code_hashes.push(hash);
        }

        self.repo.create_backup_codes(user_id, code_hashes).await?;

        Ok(codes)
    }

    /// Verify a backup code
    pub async fn verify_backup_code(&self, user_id: Uuid, code: &str) -> Result<bool, AuthError> {
        let code_hash = hash_token(code)?;

        if let Some(backup_code) = self.repo.find_unused_backup_code(user_id, &code_hash).await? {
            self.repo.use_backup_code(backup_code.id).await?;
            return Ok(true);
        }

        Ok(false)
    }

    /// Get remaining backup code count
    pub async fn get_remaining_backup_codes(&self, user_id: Uuid) -> Result<i64, AuthError> {
        self.repo.count_unused_backup_codes(user_id).await
    }

    // ========================================================================
    // MFA Method Management
    // ========================================================================

    /// Get all MFA methods for a user
    pub async fn get_user_methods(&self, user_id: Uuid) -> Result<Vec<UserMfaMethod>, AuthError> {
        self.repo.list_methods_by_user(user_id).await
    }

    /// Check if user has MFA enabled
    pub async fn is_mfa_enabled(&self, user_id: Uuid) -> Result<bool, AuthError> {
        let methods = self.repo.list_methods_by_user(user_id).await?;
        Ok(methods.iter().any(|m| m.is_verified))
    }

    /// Delete an MFA method
    pub async fn delete_method(&self, user_id: Uuid, method_id: Uuid) -> Result<(), AuthError> {
        self.repo.delete_method(method_id, user_id).await
    }

    /// Disable all MFA for a user
    pub async fn disable_mfa(&self, user_id: Uuid) -> Result<(), AuthError> {
        self.repo.delete_all_methods(user_id).await?;
        Ok(())
    }

    /// Record MFA verification attempt
    pub async fn record_attempt(
        &self,
        user_id: Uuid,
        attempt_type: &str,
        is_successful: bool,
        ip_address: Option<&str>,
    ) -> Result<(), AuthError> {
        self.repo
            .record_attempt(user_id, attempt_type, is_successful, ip_address)
            .await
    }

    /// Check if MFA is rate limited
    pub async fn is_rate_limited(&self, user_id: Uuid, max_attempts: i64, window_minutes: i64) -> Result<bool, AuthError> {
        let count = self
            .repo
            .count_recent_failed_attempts(user_id, window_minutes)
            .await?;
        Ok(count >= max_attempts)
    }
}

/// Response for TOTP setup
#[derive(Debug, Clone)]
pub struct TotpSetupResponse {
    pub method_id: Uuid,
    pub secret: String,
    pub provisioning_uri: String,
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generate a random TOTP secret (20 bytes)
fn generate_totp_secret() -> Vec<u8> {
    let mut rng = rand::thread_rng();
    (0..20).map(|_| rng.gen::<u8>()).collect()
}

/// Base32 encode bytes
fn base32_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for &byte in data {
        buffer = (buffer << 8) | byte as u64;
        bits_left += 8;

        while bits_left >= 5 {
            bits_left -= 5;
            let index = ((buffer >> bits_left) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
    }

    if bits_left > 0 {
        let index = ((buffer << (5 - bits_left)) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }

    result
}

/// Base32 decode string
fn base32_decode(encoded: &str) -> Result<Vec<u8>, AuthError> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = Vec::new();
    let mut buffer: u64 = 0;
    let mut bits_left = 0;

    for c in encoded.chars() {
        let c_upper = c.to_ascii_uppercase();
        let value = ALPHABET
            .iter()
            .position(|&x| x as char == c_upper)
            .ok_or_else(|| AuthError::InternalError(anyhow::anyhow!("Invalid base32 character")))?;

        buffer = (buffer << 5) | value as u64;
        bits_left += 5;

        if bits_left >= 8 {
            bits_left -= 8;
            result.push((buffer >> bits_left) as u8);
        }
    }

    Ok(result)
}

/// Verify a TOTP code
fn verify_totp_code(secret_base32: &str, code: &str) -> Result<bool, AuthError> {
    let secret = base32_decode(secret_base32)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| AuthError::InternalError(anyhow::anyhow!("{}", e)))?
        .as_secs();

    let counter = now / TOTP_PERIOD;

    // Check current time step and adjacent ones (to handle clock drift)
    for offset in -1i64..=1 {
        let check_counter = (counter as i64 + offset) as u64;
        let expected = generate_totp(&secret, check_counter)?;
        if expected == code {
            return Ok(true);
        }
    }

    Ok(false)
}

/// Generate TOTP code for a given counter
fn generate_totp(secret: &[u8], counter: u64) -> Result<String, AuthError> {
    use hmac::{Hmac, Mac};
    use sha1::Sha1;
    use std::convert::TryInto;

    type HmacSha1 = Hmac<Sha1>;

    // HMAC-SHA1 (RFC 6238 standard)
    let counter_bytes = counter.to_be_bytes();
    
    let mut mac = HmacSha1::new_from_slice(secret)
        .map_err(|e| AuthError::InternalError(anyhow::anyhow!("HMAC error: {}", e)))?;
    mac.update(&counter_bytes);
    let hmac_result = mac.finalize().into_bytes();

    // Dynamic truncation
    let offset = (hmac_result[hmac_result.len() - 1] & 0x0F) as usize;
    let binary = u32::from_be_bytes(
        hmac_result[offset..offset + 4]
            .try_into()
            .map_err(|_| AuthError::InternalError(anyhow::anyhow!("TOTP generation failed")))?,
    ) & 0x7FFFFFFF;

    let otp = binary % 10u32.pow(TOTP_DIGITS);
    Ok(format!("{:0>width$}", otp, width = TOTP_DIGITS as usize))
}

/// Generate a random backup code
fn generate_backup_code() -> String {
    let mut rng = rand::thread_rng();
    const CHARSET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // Excluding confusing chars
    
    (0..BACKUP_CODE_LENGTH)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

//! PKCE (Proof Key for Code Exchange) utilities for OAuth2 Authorization Code Flow
//!
//! This module provides functions for PKCE verification as specified in RFC 7636.
//! PKCE is required for External Apps to prevent authorization code interception attacks.
//!
//! # Requirements
//! - 3.5: WHEN exchanging code for token, THE Authorization_Server SHALL verify
//!        code_verifier matches the original code_challenge

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use sha2::{Digest, Sha256};

/// Minimum length for code_verifier as per RFC 7636
pub const CODE_VERIFIER_MIN_LENGTH: usize = 43;

/// Maximum length for code_verifier as per RFC 7636
pub const CODE_VERIFIER_MAX_LENGTH: usize = 128;

/// Supported PKCE methods
pub const PKCE_METHOD_S256: &str = "S256";
pub const PKCE_METHOD_PLAIN: &str = "plain";

/// Verify code_verifier against code_challenge using the specified method
///
/// # Arguments
/// * `code_verifier` - The code verifier sent during token exchange
/// * `code_challenge` - The code challenge sent during authorization request
/// * `method` - The code challenge method ("S256" or "plain")
///
/// # Returns
/// * `true` if the code_verifier matches the code_challenge
/// * `false` if verification fails or method is unsupported
///
/// # Requirements
/// - 3.5: Verify code_verifier matches the original code_challenge
///
/// # Example
/// ```
/// use auth_server::utils::pkce::verify_pkce;
///
/// let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
/// let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
/// assert!(verify_pkce(verifier, challenge, "S256"));
/// ```
pub fn verify_pkce(code_verifier: &str, code_challenge: &str, method: &str) -> bool {
    match method {
        PKCE_METHOD_S256 => {
            let computed_challenge = compute_s256_challenge(code_verifier);
            // Use constant-time comparison to prevent timing attacks
            constant_time_compare(&computed_challenge, code_challenge)
        }
        PKCE_METHOD_PLAIN => {
            // Plain method: code_verifier == code_challenge
            // Use constant-time comparison to prevent timing attacks
            constant_time_compare(code_verifier, code_challenge)
        }
        _ => false, // Unsupported method
    }
}

/// Compute S256 code_challenge from code_verifier
///
/// S256: code_challenge = BASE64URL(SHA256(code_verifier))
///
/// # Arguments
/// * `code_verifier` - The code verifier to hash
///
/// # Returns
/// The base64url-encoded SHA256 hash of the code_verifier
pub fn compute_s256_challenge(code_verifier: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code_verifier.as_bytes());
    let hash = hasher.finalize();
    URL_SAFE_NO_PAD.encode(hash)
}

/// Validate code_verifier format according to RFC 7636
///
/// The code_verifier must be:
/// - Between 43 and 128 characters long
/// - Contain only unreserved URI characters: [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
///
/// # Arguments
/// * `verifier` - The code verifier to validate
///
/// # Returns
/// * `true` if the verifier is valid
/// * `false` if the verifier is invalid
///
/// # Requirements
/// - 3.5: Validate code_verifier format
pub fn validate_code_verifier(verifier: &str) -> bool {
    let len = verifier.len();

    // Check length constraints
    if len < CODE_VERIFIER_MIN_LENGTH || len > CODE_VERIFIER_MAX_LENGTH {
        return false;
    }

    // Check character set: unreserved URI characters only
    verifier
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_' || c == '~')
}

/// Validate code_challenge format
///
/// The code_challenge for S256 method should be a valid base64url-encoded string
/// of exactly 43 characters (256 bits / 6 bits per base64 char = ~43 chars)
///
/// # Arguments
/// * `challenge` - The code challenge to validate
///
/// # Returns
/// * `true` if the challenge appears valid
/// * `false` if the challenge is invalid
pub fn validate_code_challenge(challenge: &str) -> bool {
    // S256 challenge is base64url-encoded SHA256 hash (32 bytes = 43 base64 chars)
    if challenge.len() != 43 {
        return false;
    }

    // Check that it's valid base64url characters
    challenge
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
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

    // Known test vector from RFC 7636 Appendix B
    // code_verifier: dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk
    // code_challenge (S256): E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM

    #[test]
    fn test_verify_pkce_s256_valid() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        assert!(verify_pkce(verifier, challenge, "S256"));
    }

    #[test]
    fn test_verify_pkce_s256_invalid_verifier() {
        let verifier = "wrong_verifier_that_is_long_enough_to_pass_length_check";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        assert!(!verify_pkce(verifier, challenge, "S256"));
    }

    #[test]
    fn test_verify_pkce_plain_valid() {
        let verifier = "my_plain_code_verifier_that_is_at_least_43_chars";
        let challenge = "my_plain_code_verifier_that_is_at_least_43_chars";

        assert!(verify_pkce(verifier, challenge, "plain"));
    }

    #[test]
    fn test_verify_pkce_plain_invalid() {
        let verifier = "my_plain_code_verifier_that_is_at_least_43_chars";
        let challenge = "different_challenge_value_that_is_also_long_enough";

        assert!(!verify_pkce(verifier, challenge, "plain"));
    }

    #[test]
    fn test_verify_pkce_unsupported_method() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        assert!(!verify_pkce(verifier, challenge, "unsupported"));
    }

    #[test]
    fn test_compute_s256_challenge() {
        let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
        let expected = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

        let computed = compute_s256_challenge(verifier);
        assert_eq!(computed, expected);
    }

    #[test]
    fn test_validate_code_verifier_valid() {
        // Exactly 43 characters (minimum)
        let verifier_min = "abcdefghijklmnopqrstuvwxyz0123456789-._~abc";
        assert!(validate_code_verifier(verifier_min));

        // 128 characters (maximum)
        let verifier_max = "a".repeat(128);
        assert!(validate_code_verifier(&verifier_max));

        // With all allowed special characters
        let verifier_special = "abcABC123-._~abcABC123-._~abcABC123-._~abcde";
        assert!(validate_code_verifier(verifier_special));
    }

    #[test]
    fn test_validate_code_verifier_too_short() {
        let verifier = "a".repeat(42); // One less than minimum
        assert!(!validate_code_verifier(&verifier));
    }

    #[test]
    fn test_validate_code_verifier_too_long() {
        let verifier = "a".repeat(129); // One more than maximum
        assert!(!validate_code_verifier(&verifier));
    }

    #[test]
    fn test_validate_code_verifier_invalid_chars() {
        // Contains space
        let verifier_space = "abcdefghijklmnopqrstuvwxyz0123456789 abcdef";
        assert!(!validate_code_verifier(verifier_space));

        // Contains +
        let verifier_plus = "abcdefghijklmnopqrstuvwxyz0123456789+abcdef";
        assert!(!validate_code_verifier(verifier_plus));

        // Contains /
        let verifier_slash = "abcdefghijklmnopqrstuvwxyz0123456789/abcdef";
        assert!(!validate_code_verifier(verifier_slash));

        // Contains =
        let verifier_equals = "abcdefghijklmnopqrstuvwxyz0123456789=abcdef";
        assert!(!validate_code_verifier(verifier_equals));
    }

    #[test]
    fn test_validate_code_challenge_valid() {
        // Valid S256 challenge (43 base64url chars)
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
        assert!(validate_code_challenge(challenge));
    }

    #[test]
    fn test_validate_code_challenge_wrong_length() {
        let challenge_short = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw";
        assert!(!validate_code_challenge(challenge_short));

        let challenge_long = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cMx";
        assert!(!validate_code_challenge(challenge_long));
    }

    #[test]
    fn test_validate_code_challenge_invalid_chars() {
        // Contains + (not valid base64url)
        let challenge = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw+cM";
        assert!(!validate_code_challenge(challenge));
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

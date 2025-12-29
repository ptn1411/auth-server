use regex::Regex;
use std::sync::LazyLock;

use crate::error::AuthError;

// Email regex pattern - practical validation for common email formats
// Allows: alphanumeric, dots, underscores, hyphens, plus signs in local part
// Requires: at least one dot in domain part (e.g., example.com)
static EMAIL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$"
    ).expect("Invalid email regex pattern")
});

/// Validate an email address format
/// 
/// # Arguments
/// * `email` - The email address to validate
/// 
/// # Returns
/// * `Ok(())` - If the email format is valid
/// * `Err(AuthError::InvalidEmailFormat)` - If the email format is invalid
/// 
/// # Requirements
/// - 1.3: Reject registration with invalid email format
pub fn validate_email(email: &str) -> Result<(), AuthError> {
    // Check for empty email
    if email.is_empty() {
        return Err(AuthError::InvalidEmailFormat);
    }
    
    // Check length constraints (max 254 characters per RFC 5321)
    if email.len() > 254 {
        return Err(AuthError::InvalidEmailFormat);
    }
    
    // Check local part length (max 64 characters)
    if let Some(at_pos) = email.find('@') {
        let local_part = &email[..at_pos];
        if local_part.len() > 64 {
            return Err(AuthError::InvalidEmailFormat);
        }
        
        // Check for leading/trailing dots in local part
        if local_part.starts_with('.') || local_part.ends_with('.') {
            return Err(AuthError::InvalidEmailFormat);
        }
        
        // Check for consecutive dots in local part
        if local_part.contains("..") {
            return Err(AuthError::InvalidEmailFormat);
        }
    }
    
    // Validate against regex pattern
    if !EMAIL_REGEX.is_match(email) {
        return Err(AuthError::InvalidEmailFormat);
    }
    
    Ok(())
}

/// Check if an email address is valid (returns boolean)
/// 
/// # Arguments
/// * `email` - The email address to check
/// 
/// # Returns
/// * `true` - If the email format is valid
/// * `false` - If the email format is invalid
pub fn is_valid_email(email: &str) -> bool {
    validate_email(email).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_emails() {
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.org",
            "user+tag@example.co.uk",
            "firstname.lastname@company.com",
            "email@subdomain.domain.com",
            "1234567890@example.com",
            "email@domain-one.com",
            "_______@example.com",
            "email@example.name",
            "email@example.museum",
            "email@example.co.jp",
        ];

        for email in valid_emails {
            assert!(
                validate_email(email).is_ok(),
                "Expected '{}' to be valid",
                email
            );
        }
    }

    #[test]
    fn test_invalid_emails() {
        let invalid_emails = vec![
            "",                           // empty
            "plainaddress",               // no @ symbol
            "@no-local-part.com",         // no local part
            "no-at-sign.com",             // no @ symbol
            "no-domain@",                 // no domain
            "spaces in@email.com",        // spaces not allowed
            ".email@domain.com",          // leading dot in local part
            "email.@domain.com",          // trailing dot in local part
            "email..email@domain.com",    // double dots
            "email@-domain.com",          // domain starts with hyphen
        ];

        for email in invalid_emails {
            assert!(
                validate_email(email).is_err(),
                "Expected '{}' to be invalid",
                email
            );
        }
    }

    #[test]
    fn test_email_too_long() {
        // Create an email longer than 254 characters
        let long_local = "a".repeat(65);
        let long_email = format!("{}@example.com", long_local);
        
        assert!(validate_email(&long_email).is_err());
    }

    #[test]
    fn test_local_part_too_long() {
        // Local part > 64 characters
        let long_local = "a".repeat(65);
        let email = format!("{}@example.com", long_local);
        
        assert!(validate_email(&email).is_err());
    }

    #[test]
    fn test_is_valid_email_helper() {
        assert!(is_valid_email("valid@email.com"));
        assert!(!is_valid_email("invalid-email"));
    }

    #[test]
    fn test_email_with_special_characters() {
        // Valid special characters in local part
        assert!(validate_email("user+tag@example.com").is_ok());
        assert!(validate_email("user.name@example.com").is_ok());
        assert!(validate_email("user_name@example.com").is_ok());
    }
}

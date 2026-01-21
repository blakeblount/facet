//! Password hashing utilities for employee PINs.
//!
//! This module provides argon2-based password hashing for secure PIN storage.

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};

/// Error type for password hashing operations.
#[derive(Debug)]
pub enum HashError {
    /// Failed to hash the password.
    HashFailed(String),
    /// Failed to verify the password.
    VerifyFailed(String),
}

impl std::fmt::Display for HashError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HashError::HashFailed(msg) => write!(f, "Hash failed: {}", msg),
            HashError::VerifyFailed(msg) => write!(f, "Verify failed: {}", msg),
        }
    }
}

impl std::error::Error for HashError {}

/// Hash a PIN using argon2.
///
/// Returns the hashed PIN as a PHC-formatted string that includes
/// the salt and all parameters needed for verification.
///
/// # Example
///
/// ```
/// use api::auth::hash_pin;
///
/// let pin = "1234";
/// let hash = hash_pin(pin).unwrap();
/// assert!(hash.starts_with("$argon2"));
/// ```
pub fn hash_pin(pin: &str) -> Result<String, HashError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(pin.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| HashError::HashFailed(e.to_string()))
}

/// Result of PIN validation.
#[derive(Debug, Clone)]
pub struct PinValidationResult {
    /// Whether the PIN is valid.
    pub valid: bool,
    /// Error message if invalid.
    pub error: Option<String>,
}

impl PinValidationResult {
    /// Create a valid result.
    pub fn valid() -> Self {
        Self {
            valid: true,
            error: None,
        }
    }

    /// Create an invalid result with an error message.
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            valid: false,
            error: Some(error.into()),
        }
    }
}

/// Common weak PIN patterns that should be rejected.
const WEAK_PATTERNS: &[&str] = &[
    "000000", "111111", "222222", "333333", "444444", "555555", "666666", "777777", "888888",
    "999999", "123456", "654321", "012345", "543210", "123123", "111222", "112233", "121212",
    "abcdef", "qwerty", "password", "changeme",
];

/// Validate PIN complexity requirements.
///
/// Checks:
/// - Minimum length
/// - No weak patterns
///
/// # Arguments
/// * `pin` - The PIN to validate
/// * `min_length` - Minimum required length
///
/// # Returns
/// A `PinValidationResult` indicating whether the PIN is valid.
pub fn validate_pin_complexity(pin: &str, min_length: i32) -> PinValidationResult {
    let min_length = min_length as usize;

    // Check minimum length
    if pin.len() < min_length {
        return PinValidationResult::invalid(format!(
            "PIN must be at least {} characters",
            min_length
        ));
    }

    // Check for weak patterns (case-insensitive)
    let pin_lower = pin.to_lowercase();
    for pattern in WEAK_PATTERNS {
        if pin_lower == *pattern || pin_lower.starts_with(*pattern) {
            return PinValidationResult::invalid(
                "PIN is too common or easy to guess. Please choose a stronger PIN.",
            );
        }
    }

    PinValidationResult::valid()
}

/// Verify a PIN against a stored hash.
///
/// Returns `true` if the PIN matches the hash, `false` otherwise.
///
/// # Example
///
/// ```
/// use api::auth::{hash_pin, verify_pin};
///
/// let pin = "1234";
/// let hash = hash_pin(pin).unwrap();
/// assert!(verify_pin(pin, &hash).unwrap());
/// assert!(!verify_pin("wrong", &hash).unwrap());
/// ```
pub fn verify_pin(pin: &str, hash: &str) -> Result<bool, HashError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| HashError::VerifyFailed(e.to_string()))?;

    let argon2 = Argon2::default();

    match argon2.verify_password(pin.as_bytes(), &parsed_hash) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(e) => Err(HashError::VerifyFailed(e.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_pin_produces_argon2_format() {
        let hash = hash_pin("1234").unwrap();
        assert!(hash.starts_with("$argon2"));
    }

    #[test]
    fn test_hash_pin_produces_unique_hashes() {
        let hash1 = hash_pin("1234").unwrap();
        let hash2 = hash_pin("1234").unwrap();
        // Same PIN should produce different hashes due to random salt
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_pin_correct() {
        let pin = "1234";
        let hash = hash_pin(pin).unwrap();
        assert!(verify_pin(pin, &hash).unwrap());
    }

    #[test]
    fn test_verify_pin_incorrect() {
        let pin = "1234";
        let hash = hash_pin(pin).unwrap();
        assert!(!verify_pin("wrong", &hash).unwrap());
    }

    #[test]
    fn test_verify_pin_empty() {
        let hash = hash_pin("1234").unwrap();
        assert!(!verify_pin("", &hash).unwrap());
    }

    #[test]
    fn test_hash_empty_pin() {
        // Empty PIN should still hash successfully
        let hash = hash_pin("").unwrap();
        assert!(verify_pin("", &hash).unwrap());
        assert!(!verify_pin("any", &hash).unwrap());
    }

    #[test]
    fn test_verify_invalid_hash_format() {
        let result = verify_pin("1234", "not-a-valid-hash");
        assert!(result.is_err());
    }

    #[test]
    fn test_hash_with_special_characters() {
        let pin = "!@#$%^&*()";
        let hash = hash_pin(pin).unwrap();
        assert!(verify_pin(pin, &hash).unwrap());
    }

    #[test]
    fn test_hash_with_unicode() {
        let pin = "密码123";
        let hash = hash_pin(pin).unwrap();
        assert!(verify_pin(pin, &hash).unwrap());
    }

    #[test]
    fn test_hash_long_pin() {
        let pin = "a".repeat(1000);
        let hash = hash_pin(&pin).unwrap();
        assert!(verify_pin(&pin, &hash).unwrap());
    }

    #[test]
    fn test_error_display() {
        let err = HashError::HashFailed("test".to_string());
        assert_eq!(err.to_string(), "Hash failed: test");

        let err = HashError::VerifyFailed("test".to_string());
        assert_eq!(err.to_string(), "Verify failed: test");
    }

    // PIN complexity validation tests

    #[test]
    fn test_validate_pin_too_short() {
        let result = validate_pin_complexity("12345", 6);
        assert!(!result.valid);
        assert!(result.error.unwrap().contains("at least 6 characters"));
    }

    #[test]
    fn test_validate_pin_minimum_length() {
        let result = validate_pin_complexity("123456", 6);
        // Still invalid because 123456 is a weak pattern
        assert!(!result.valid);
    }

    #[test]
    fn test_validate_pin_weak_patterns() {
        let weak_pins = vec![
            "000000", "111111", "123456", "654321", "qwerty", "password", "changeme",
        ];

        for pin in weak_pins {
            let result = validate_pin_complexity(pin, 6);
            assert!(!result.valid, "PIN '{}' should be rejected as weak", pin);
            assert!(result.error.unwrap().contains("too common"));
        }
    }

    #[test]
    fn test_validate_pin_valid() {
        let valid_pins = vec!["abc123xyz", "mySecure9", "j3w3lry!", "Repair2024"];

        for pin in valid_pins {
            let result = validate_pin_complexity(pin, 6);
            assert!(result.valid, "PIN '{}' should be valid", pin);
            assert!(result.error.is_none());
        }
    }

    #[test]
    fn test_validate_pin_case_insensitive() {
        // Weak patterns should be detected case-insensitively
        let result = validate_pin_complexity("CHANGEME", 6);
        assert!(!result.valid);

        let result = validate_pin_complexity("PassWord", 6);
        assert!(!result.valid);
    }

    #[test]
    fn test_validate_pin_custom_min_length() {
        let result = validate_pin_complexity("abc", 4);
        assert!(!result.valid);

        let result = validate_pin_complexity("abcd", 4);
        assert!(result.valid);
    }
}

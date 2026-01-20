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
}

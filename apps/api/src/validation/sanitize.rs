//! Text sanitization and validation functions.
//!
//! Provides functions to sanitize and validate user input:
//! - Trim whitespace
//! - Normalize line endings
//! - Validate length constraints
//! - Validate phone numbers
//! - Validate email format
//! - Sanitize for logging

use crate::error::AppError;

/// Trim whitespace and normalize newlines in text input.
///
/// This function:
/// - Trims leading and trailing whitespace
/// - Normalizes Windows-style line endings (\r\n and \r) to Unix-style (\n)
pub fn sanitize_text(input: &str) -> String {
    input
        .trim()
        .replace("\r\n", "\n") // Windows -> Unix
        .replace('\r', "\n") // Old Mac -> Unix
}

/// Sanitize and validate text length.
///
/// Returns the sanitized text if within length limits.
///
/// # Errors
/// Returns `AppError::ValidationError` if the text exceeds the maximum length.
pub fn validate_text(input: &str, field_name: &str, max_length: usize) -> Result<String, AppError> {
    let sanitized = sanitize_text(input);

    if sanitized.len() > max_length {
        return Err(AppError::validation(format!(
            "{} exceeds maximum length of {} characters",
            field_name, max_length
        )));
    }

    Ok(sanitized)
}

/// Validate that a required field is present and within length limits.
///
/// # Errors
/// - Returns `AppError::ValidationError` if the text is empty after trimming
/// - Returns `AppError::ValidationError` if the text exceeds the maximum length
pub fn validate_required(
    input: &str,
    field_name: &str,
    max_length: usize,
) -> Result<String, AppError> {
    let sanitized = validate_text(input, field_name, max_length)?;

    if sanitized.is_empty() {
        return Err(AppError::validation(format!("{} is required", field_name)));
    }

    Ok(sanitized)
}

/// Validate an optional text field (can be empty but must respect length if provided).
///
/// # Errors
/// Returns `AppError::ValidationError` if the non-empty text exceeds the maximum length.
pub fn validate_optional(
    input: Option<&str>,
    field_name: &str,
    max_length: usize,
) -> Result<Option<String>, AppError> {
    match input {
        None => Ok(None),
        Some(s) => {
            let sanitized = validate_text(s, field_name, max_length)?;
            if sanitized.is_empty() {
                Ok(None)
            } else {
                Ok(Some(sanitized))
            }
        }
    }
}

/// Validate a phone number format.
///
/// Phone numbers can contain:
/// - Digits (0-9)
/// - Spaces
/// - Hyphens (-)
/// - Parentheses ()
/// - Plus sign (+)
/// - Periods (.)
///
/// # Errors
/// - Returns `AppError::ValidationError` if the phone contains invalid characters
/// - Returns `AppError::ValidationError` if the phone exceeds the maximum length
pub fn validate_phone(input: Option<&str>, max_length: usize) -> Result<Option<String>, AppError> {
    match input {
        None => Ok(None),
        Some(s) => {
            let sanitized = sanitize_text(s);

            if sanitized.is_empty() {
                return Ok(None);
            }

            if sanitized.len() > max_length {
                return Err(AppError::validation(format!(
                    "phone exceeds maximum length of {} characters",
                    max_length
                )));
            }

            // Allow only digits, spaces, dashes, parentheses, plus sign
            if !sanitized
                .chars()
                .all(|c| c.is_ascii_digit() || " -+().".contains(c))
            {
                return Err(AppError::validation(
                    "phone contains invalid characters (only digits, spaces, dashes, parentheses, and + are allowed)",
                ));
            }

            Ok(Some(sanitized))
        }
    }
}

/// Validate an email address format.
///
/// Performs basic validation:
/// - Must contain @ if not empty
/// - Converted to lowercase
/// - Length must not exceed maximum
///
/// # Errors
/// - Returns `AppError::ValidationError` if the email is missing @ symbol
/// - Returns `AppError::ValidationError` if the email exceeds the maximum length
pub fn validate_email(input: Option<&str>, max_length: usize) -> Result<Option<String>, AppError> {
    match input {
        None => Ok(None),
        Some(s) => {
            let sanitized = sanitize_text(s).to_lowercase();

            if sanitized.is_empty() {
                return Ok(None);
            }

            if sanitized.len() > max_length {
                return Err(AppError::validation(format!(
                    "email exceeds maximum length of {} characters",
                    max_length
                )));
            }

            // Basic email format check - must contain @
            if !sanitized.contains('@') {
                return Err(AppError::validation("email must contain @ symbol"));
            }

            // Check for at least something before and after @
            let parts: Vec<&str> = sanitized.splitn(2, '@').collect();
            if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
                return Err(AppError::validation("invalid email format"));
            }

            Ok(Some(sanitized))
        }
    }
}

/// Sanitize text for safe logging.
///
/// Replaces control characters (except newlines and tabs) with '?'
/// to prevent log injection attacks.
pub fn sanitize_for_log(input: &str) -> String {
    input
        .chars()
        .map(|c| {
            if c.is_control() && c != '\n' && c != '\t' {
                '?' // Replace control chars with placeholder
            } else {
                c
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // =========================================================================
    // sanitize_text tests
    // =========================================================================

    #[test]
    fn test_sanitize_text_trims_whitespace() {
        assert_eq!(sanitize_text("  hello  "), "hello");
        assert_eq!(sanitize_text("\t\nworld\t\n"), "world");
        assert_eq!(sanitize_text("   "), "");
    }

    #[test]
    fn test_sanitize_text_normalizes_newlines() {
        assert_eq!(sanitize_text("line1\r\nline2"), "line1\nline2");
        assert_eq!(sanitize_text("line1\rline2"), "line1\nline2");
        assert_eq!(sanitize_text("a\r\nb\rc\nd"), "a\nb\nc\nd");
    }

    #[test]
    fn test_sanitize_text_preserves_content() {
        assert_eq!(sanitize_text("hello world"), "hello world");
        assert_eq!(
            sanitize_text("Unicode: 日本語 émojis"),
            "Unicode: 日本語 émojis"
        );
    }

    // =========================================================================
    // validate_text tests
    // =========================================================================

    #[test]
    fn test_validate_text_within_limit() {
        let result = validate_text("hello", "field", 10);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_validate_text_at_limit() {
        let result = validate_text("hello", "field", 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_validate_text_exceeds_limit() {
        let result = validate_text("hello world", "test_field", 5);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("test_field"));
        assert!(err.message().contains("5"));
    }

    #[test]
    fn test_validate_text_trims_before_checking() {
        // "  hi  " trims to "hi" (2 chars), should be under limit
        let result = validate_text("  hi  ", "field", 5);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hi");
    }

    // =========================================================================
    // validate_required tests
    // =========================================================================

    #[test]
    fn test_validate_required_with_content() {
        let result = validate_required("hello", "field", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello");
    }

    #[test]
    fn test_validate_required_empty_string() {
        let result = validate_required("", "test_field", 100);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("test_field"));
        assert!(err.message().contains("required"));
    }

    #[test]
    fn test_validate_required_whitespace_only() {
        let result = validate_required("   ", "test_field", 100);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("required"));
    }

    #[test]
    fn test_validate_required_too_long() {
        let result = validate_required("hello world", "field", 5);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("exceeds"));
    }

    // =========================================================================
    // validate_optional tests
    // =========================================================================

    #[test]
    fn test_validate_optional_none() {
        let result = validate_optional(None, "field", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_optional_empty_string() {
        let result = validate_optional(Some(""), "field", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_optional_whitespace_only() {
        let result = validate_optional(Some("   "), "field", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_optional_with_content() {
        let result = validate_optional(Some("hello"), "field", 100);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("hello".to_string()));
    }

    #[test]
    fn test_validate_optional_too_long() {
        let result = validate_optional(Some("hello world"), "field", 5);
        assert!(result.is_err());
    }

    // =========================================================================
    // validate_phone tests
    // =========================================================================

    #[test]
    fn test_validate_phone_valid_formats() {
        assert!(validate_phone(Some("5551234567"), 50).is_ok());
        assert!(validate_phone(Some("555-123-4567"), 50).is_ok());
        assert!(validate_phone(Some("(555) 123-4567"), 50).is_ok());
        assert!(validate_phone(Some("+1-555-123-4567"), 50).is_ok());
        assert!(validate_phone(Some("+1 (555) 123.4567"), 50).is_ok());
    }

    #[test]
    fn test_validate_phone_none() {
        let result = validate_phone(None, 50);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_phone_empty() {
        let result = validate_phone(Some(""), 50);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_phone_whitespace_only() {
        let result = validate_phone(Some("   "), 50);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_phone_invalid_characters() {
        let result = validate_phone(Some("abc-def-ghij"), 50);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("invalid characters"));
    }

    #[test]
    fn test_validate_phone_too_long() {
        let long_phone = "1".repeat(100);
        let result = validate_phone(Some(&long_phone), 50);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("exceeds"));
    }

    // =========================================================================
    // validate_email tests
    // =========================================================================

    #[test]
    fn test_validate_email_valid() {
        let result = validate_email(Some("user@example.com"), 255);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("user@example.com".to_string()));
    }

    #[test]
    fn test_validate_email_lowercase_conversion() {
        let result = validate_email(Some("USER@EXAMPLE.COM"), 255);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), Some("user@example.com".to_string()));
    }

    #[test]
    fn test_validate_email_none() {
        let result = validate_email(None, 255);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_email_empty() {
        let result = validate_email(Some(""), 255);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_email_whitespace_only() {
        let result = validate_email(Some("   "), 255);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }

    #[test]
    fn test_validate_email_missing_at() {
        let result = validate_email(Some("notanemail"), 255);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("@"));
    }

    #[test]
    fn test_validate_email_nothing_before_at() {
        let result = validate_email(Some("@example.com"), 255);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_nothing_after_at() {
        let result = validate_email(Some("user@"), 255);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_email_too_long() {
        let long_email = format!("{}@example.com", "a".repeat(300));
        let result = validate_email(Some(&long_email), 255);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.message().contains("exceeds"));
    }

    // =========================================================================
    // sanitize_for_log tests
    // =========================================================================

    #[test]
    fn test_sanitize_for_log_normal_text() {
        assert_eq!(sanitize_for_log("hello world"), "hello world");
    }

    #[test]
    fn test_sanitize_for_log_preserves_newlines_and_tabs() {
        assert_eq!(sanitize_for_log("line1\nline2"), "line1\nline2");
        assert_eq!(sanitize_for_log("col1\tcol2"), "col1\tcol2");
    }

    #[test]
    fn test_sanitize_for_log_removes_control_chars() {
        assert_eq!(sanitize_for_log("hello\x00world"), "hello?world");
        assert_eq!(sanitize_for_log("test\x1b[31mred"), "test?[31mred");
    }

    #[test]
    fn test_sanitize_for_log_preserves_unicode() {
        assert_eq!(sanitize_for_log("Unicode: 日本語"), "Unicode: 日本語");
    }
}

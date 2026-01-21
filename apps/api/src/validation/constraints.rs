//! Field length constraints for text validation.
//!
//! These constants define maximum lengths for various text fields
//! to prevent excessive storage usage and ensure consistent behavior.

/// Maximum length for name fields (customer name, employee name, location name).
pub const MAX_NAME_LENGTH: usize = 255;

/// Maximum length for phone number fields.
pub const MAX_PHONE_LENGTH: usize = 50;

/// Maximum length for email fields.
pub const MAX_EMAIL_LENGTH: usize = 255;

/// Maximum length for item type field.
pub const MAX_ITEM_TYPE_LENGTH: usize = 100;

/// Maximum length for description fields (item_description, condition_notes, requested_work).
pub const MAX_DESCRIPTION_LENGTH: usize = 2000;

/// Maximum length for note content.
pub const MAX_NOTE_LENGTH: usize = 5000;

/// Maximum length for address fields.
pub const MAX_ADDRESS_LENGTH: usize = 500;

/// Maximum length for ticket prefix (e.g., "JR").
pub const MAX_TICKET_PREFIX_LENGTH: usize = 10;

/// Maximum length for currency code (e.g., "USD").
pub const MAX_CURRENCY_LENGTH: usize = 10;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraints_are_reasonable() {
        // Sanity checks to ensure constraints are set to reasonable values
        assert!(MAX_NAME_LENGTH > 0 && MAX_NAME_LENGTH <= 1000);
        assert!(MAX_PHONE_LENGTH > 0 && MAX_PHONE_LENGTH <= 100);
        assert!(MAX_EMAIL_LENGTH > 0 && MAX_EMAIL_LENGTH <= 500);
        assert!(MAX_DESCRIPTION_LENGTH > 0 && MAX_DESCRIPTION_LENGTH <= 10000);
        assert!(MAX_NOTE_LENGTH > 0 && MAX_NOTE_LENGTH <= 50000);
        assert!(MAX_ADDRESS_LENGTH > 0 && MAX_ADDRESS_LENGTH <= 2000);
    }
}

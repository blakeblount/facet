//! Store settings model.
//!
//! Store settings contain configuration for the jewelry store,
//! including store info, ticket numbering, and admin PIN.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full store settings entity (internal use only).
///
/// The admin_pin_hash is excluded from serialization for security.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct StoreSettings {
    pub setting_id: Uuid,
    pub store_name: String,
    pub store_phone: Option<String>,
    pub store_address: Option<String>,
    pub ticket_prefix: String,
    pub next_ticket_number: i32,
    pub currency: String,
    pub max_photos_per_ticket: i32,
    pub admin_pin_hash: String,
    pub setup_complete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Public view of store settings (without admin PIN hash).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StoreSettingsPublic {
    pub setting_id: Uuid,
    pub store_name: String,
    pub store_phone: Option<String>,
    pub store_address: Option<String>,
    pub ticket_prefix: String,
    pub next_ticket_number: i32,
    pub currency: String,
    pub max_photos_per_ticket: i32,
    pub setup_complete: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<StoreSettings> for StoreSettingsPublic {
    fn from(settings: StoreSettings) -> Self {
        Self {
            setting_id: settings.setting_id,
            store_name: settings.store_name,
            store_phone: settings.store_phone,
            store_address: settings.store_address,
            ticket_prefix: settings.ticket_prefix,
            next_ticket_number: settings.next_ticket_number,
            currency: settings.currency,
            max_photos_per_ticket: settings.max_photos_per_ticket,
            setup_complete: settings.setup_complete,
            created_at: settings.created_at,
            updated_at: settings.updated_at,
        }
    }
}

/// Input for updating store settings.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStoreSettings {
    pub store_name: Option<String>,
    pub store_phone: Option<String>,
    pub store_address: Option<String>,
    pub ticket_prefix: Option<String>,
    pub currency: Option<String>,
    pub max_photos_per_ticket: Option<i32>,
}

/// Result of ticket number increment operation.
#[derive(Debug, Clone)]
pub struct TicketNumberResult {
    pub prefix: String,
    pub number: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_store_settings_partial() {
        let json = r#"{"store_name": "New Name"}"#;
        let input: UpdateStoreSettings = serde_json::from_str(json).unwrap();
        assert_eq!(input.store_name, Some("New Name".to_string()));
        assert!(input.store_phone.is_none());
        assert!(input.store_address.is_none());
        assert!(input.ticket_prefix.is_none());
        assert!(input.currency.is_none());
        assert!(input.max_photos_per_ticket.is_none());
    }

    #[test]
    fn test_update_store_settings_full() {
        let json = r#"{
            "store_name": "Test Store",
            "store_phone": "555-1234",
            "store_address": "123 Main St",
            "ticket_prefix": "TS",
            "currency": "EUR",
            "max_photos_per_ticket": 5
        }"#;
        let input: UpdateStoreSettings = serde_json::from_str(json).unwrap();
        assert_eq!(input.store_name, Some("Test Store".to_string()));
        assert_eq!(input.store_phone, Some("555-1234".to_string()));
        assert_eq!(input.store_address, Some("123 Main St".to_string()));
        assert_eq!(input.ticket_prefix, Some("TS".to_string()));
        assert_eq!(input.currency, Some("EUR".to_string()));
        assert_eq!(input.max_photos_per_ticket, Some(5));
    }

    #[test]
    fn test_store_settings_public_serialization() {
        let public = StoreSettingsPublic {
            setting_id: Uuid::nil(),
            store_name: "Test".to_string(),
            store_phone: Some("555-1234".to_string()),
            store_address: None,
            ticket_prefix: "JR".to_string(),
            next_ticket_number: 1,
            currency: "USD".to_string(),
            max_photos_per_ticket: 10,
            setup_complete: false,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let json = serde_json::to_string(&public).unwrap();
        assert!(json.contains("store_name"));
        assert!(json.contains("ticket_prefix"));
        // Should NOT contain admin_pin_hash
        assert!(!json.contains("admin_pin_hash"));
    }
}

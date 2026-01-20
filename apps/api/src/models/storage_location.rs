//! Storage location model and related types.
//!
//! Storage locations represent physical places where jewelry items are kept
//! during the repair process (e.g., safe drawers, workbenches, display cases).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full storage location entity with all fields.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StorageLocation {
    pub location_id: Uuid,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Summary view of a storage location.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StorageLocationSummary {
    pub location_id: Uuid,
    pub name: String,
    pub is_active: bool,
}

/// Input for creating a new storage location.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateStorageLocation {
    pub name: String,
}

/// Input for updating a storage location.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateStorageLocation {
    pub name: Option<String>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_storage_location_deserialize() {
        let json = r#"{"name": "Safe Drawer 1"}"#;
        let input: CreateStorageLocation = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "Safe Drawer 1");
    }

    #[test]
    fn test_update_storage_location_partial() {
        let json = r#"{"name": "Workbench A"}"#;
        let input: UpdateStorageLocation = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("Workbench A".to_string()));
        assert!(input.is_active.is_none());
    }

    #[test]
    fn test_update_storage_location_full() {
        let json = r#"{"name": "Display Case", "is_active": false}"#;
        let input: UpdateStorageLocation = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("Display Case".to_string()));
        assert_eq!(input.is_active, Some(false));
    }

    #[test]
    fn test_update_storage_location_only_is_active() {
        let json = r#"{"is_active": true}"#;
        let input: UpdateStorageLocation = serde_json::from_str(json).unwrap();
        assert!(input.name.is_none());
        assert_eq!(input.is_active, Some(true));
    }
}

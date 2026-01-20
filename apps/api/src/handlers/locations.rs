//! Storage location request handlers.

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::storage_location::{
    CreateStorageLocation, StorageLocationSummary, UpdateStorageLocation,
};
use crate::repositories::{StorageLocationRepository, StoreSettingsRepository};
use crate::response::{created, ApiResponse};
use crate::routes::AppState;
use uuid::Uuid;

// =============================================================================
// Admin PIN Verification Helper
// =============================================================================

/// Extract and verify admin PIN from X-Admin-PIN header.
///
/// Returns an error if the header is missing or the PIN is invalid.
async fn verify_admin_pin_header(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    let pin = headers
        .get("X-Admin-PIN")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::invalid_pin("Missing X-Admin-PIN header"))?;

    let is_valid = StoreSettingsRepository::verify_admin_pin(&state.db, pin).await?;

    if !is_valid {
        return Err(AppError::invalid_pin("Invalid admin PIN"));
    }

    Ok(())
}

// =============================================================================
// GET /locations - List Storage Locations
// =============================================================================

/// Query parameters for listing storage locations.
#[derive(Debug, Clone, Deserialize)]
pub struct ListLocationsQuery {
    /// Include inactive locations in the list.
    /// Defaults to false (only active locations returned).
    #[serde(default)]
    pub include_inactive: bool,
}

/// Response for listing storage locations.
#[derive(Debug, Clone, Serialize)]
pub struct ListLocationsResponse {
    /// List of storage locations
    pub locations: Vec<StorageLocationSummary>,
    /// Total count of locations returned
    pub count: usize,
}

/// GET /api/v1/locations - List all storage locations.
///
/// This endpoint is public and does not require authentication.
/// Returns a list of storage locations.
/// By default only active locations are returned.
/// Use `?include_inactive=true` to include inactive locations.
pub async fn list_locations(
    State(state): State<AppState>,
    Query(query): Query<ListLocationsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Fetch locations from repository
    let locations = StorageLocationRepository::list(&state.db, query.include_inactive).await?;

    let response = ListLocationsResponse {
        count: locations.len(),
        locations,
    };

    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /locations (admin) - Create Storage Location
// =============================================================================

/// POST /api/v1/locations - Create a new storage location (admin only).
///
/// Requires X-Admin-PIN header for authorization.
/// Creates a storage location with the provided name.
/// Name must be unique (case-insensitive).
///
/// Returns the created location.
pub async fn create_location(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateStorageLocation>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin PIN
    verify_admin_pin_header(&state, &headers).await?;

    // Validate input
    let name = body.name.trim();
    if name.is_empty() {
        return Err(AppError::validation("Name is required"));
    }

    // Check for duplicate name
    let existing = StorageLocationRepository::find_by_name(&state.db, name).await?;
    if existing.is_some() {
        return Err(AppError::validation(
            "A location with this name already exists",
        ));
    }

    // Create the location
    let location = StorageLocationRepository::create(
        &state.db,
        CreateStorageLocation {
            name: name.to_string(),
        },
    )
    .await?;

    // Return as StorageLocationSummary
    let summary = StorageLocationSummary {
        location_id: location.location_id,
        name: location.name,
        is_active: location.is_active,
    };

    Ok(created(summary))
}

// =============================================================================
// PUT /locations/:location_id (admin) - Update Storage Location
// =============================================================================

/// PUT /api/v1/locations/:location_id - Update a storage location (admin only).
///
/// Requires X-Admin-PIN header for authorization.
/// Updates the location with the provided fields.
/// Name must be unique (case-insensitive) if changed.
///
/// Returns the updated location.
pub async fn update_location(
    State(state): State<AppState>,
    headers: HeaderMap,
    axum::extract::Path(location_id): axum::extract::Path<Uuid>,
    Json(body): Json<UpdateStorageLocation>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin PIN
    verify_admin_pin_header(&state, &headers).await?;

    // Find the existing location
    let existing = StorageLocationRepository::find_by_id(&state.db, location_id).await?;
    let existing = existing.ok_or_else(|| AppError::not_found("Location not found"))?;

    // If name is being changed, check for duplicates
    if let Some(ref new_name) = body.name {
        let trimmed_name = new_name.trim();
        if trimmed_name.is_empty() {
            return Err(AppError::validation("Name cannot be empty"));
        }

        // Check if another location has this name (case-insensitive)
        let duplicate = StorageLocationRepository::find_by_name(&state.db, trimmed_name).await?;
        if let Some(dup) = duplicate {
            if dup.location_id != existing.location_id {
                return Err(AppError::validation(
                    "A location with this name already exists",
                ));
            }
        }
    }

    // Build the update input with trimmed name if provided
    let update_input = UpdateStorageLocation {
        name: body.name.map(|n| n.trim().to_string()),
        is_active: body.is_active,
    };

    // Update the location
    let location = StorageLocationRepository::update(&state.db, location_id, update_input)
        .await?
        .ok_or_else(|| AppError::not_found("Location not found"))?;

    // Return as StorageLocationSummary
    let summary = StorageLocationSummary {
        location_id: location.location_id,
        name: location.name,
        is_active: location.is_active,
    };

    Ok(Json(ApiResponse::success(summary)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_locations_query_deserialize_empty() {
        let json = r#"{}"#;
        let query: ListLocationsQuery = serde_json::from_str(json).unwrap();
        // include_inactive should default to false
        assert!(!query.include_inactive);
    }

    #[test]
    fn test_list_locations_query_deserialize_include_inactive_true() {
        let json = r#"{"include_inactive": true}"#;
        let query: ListLocationsQuery = serde_json::from_str(json).unwrap();
        assert!(query.include_inactive);
    }

    #[test]
    fn test_list_locations_query_deserialize_include_inactive_false() {
        let json = r#"{"include_inactive": false}"#;
        let query: ListLocationsQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);
    }

    #[test]
    fn test_list_locations_response_serialization_empty() {
        let response = ListLocationsResponse {
            locations: vec![],
            count: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"locations\":[]"));
        assert!(json.contains("\"count\":0"));
    }

    #[test]
    fn test_list_locations_response_serialization_with_locations() {
        let response = ListLocationsResponse {
            locations: vec![
                StorageLocationSummary {
                    location_id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
                        .unwrap(),
                    name: "Safe Drawer 1".to_string(),
                    is_active: true,
                },
                StorageLocationSummary {
                    location_id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001")
                        .unwrap(),
                    name: "Workbench A".to_string(),
                    is_active: true,
                },
            ],
            count: 2,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"name\":\"Safe Drawer 1\""));
        assert!(json.contains("\"name\":\"Workbench A\""));
        assert!(json.contains("\"is_active\":true"));
    }

    #[test]
    fn test_list_locations_response_includes_inactive() {
        let response = ListLocationsResponse {
            locations: vec![
                StorageLocationSummary {
                    location_id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000")
                        .unwrap(),
                    name: "Active Location".to_string(),
                    is_active: true,
                },
                StorageLocationSummary {
                    location_id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001")
                        .unwrap(),
                    name: "Inactive Location".to_string(),
                    is_active: false,
                },
            ],
            count: 2,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"is_active\":true"));
        assert!(json.contains("\"is_active\":false"));
    }

    // Tests for CreateStorageLocation deserialization

    #[test]
    fn test_create_storage_location_deserialize() {
        let json = r#"{"name": "Safe Drawer 1"}"#;
        let input: CreateStorageLocation = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "Safe Drawer 1");
    }

    #[test]
    fn test_create_storage_location_deserialize_with_whitespace() {
        let json = r#"{"name": "  Workbench A  "}"#;
        let input: CreateStorageLocation = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "  Workbench A  ");
    }

    #[test]
    fn test_create_storage_location_missing_name() {
        let json = r#"{}"#;
        let result: Result<CreateStorageLocation, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // Tests for StorageLocationSummary serialization

    #[test]
    fn test_storage_location_summary_serialization() {
        let summary = StorageLocationSummary {
            location_id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Display Case".to_string(),
            is_active: true,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"location_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"name\":\"Display Case\""));
        assert!(json.contains("\"is_active\":true"));
    }

    #[test]
    fn test_storage_location_summary_serialization_inactive() {
        let summary = StorageLocationSummary {
            location_id: uuid::Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Old Storage".to_string(),
            is_active: false,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"is_active\":false"));
    }

    // Tests for UpdateStorageLocation deserialization

    #[test]
    fn test_update_storage_location_deserialize_name_only() {
        let json = r#"{"name": "New Name"}"#;
        let input: UpdateStorageLocation = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("New Name".to_string()));
        assert!(input.is_active.is_none());
    }

    #[test]
    fn test_update_storage_location_deserialize_is_active_only() {
        let json = r#"{"is_active": false}"#;
        let input: UpdateStorageLocation = serde_json::from_str(json).unwrap();
        assert!(input.name.is_none());
        assert_eq!(input.is_active, Some(false));
    }

    #[test]
    fn test_update_storage_location_deserialize_full() {
        let json = r#"{"name": "Updated Name", "is_active": true}"#;
        let input: UpdateStorageLocation = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("Updated Name".to_string()));
        assert_eq!(input.is_active, Some(true));
    }

    #[test]
    fn test_update_storage_location_deserialize_empty() {
        let json = r#"{}"#;
        let input: UpdateStorageLocation = serde_json::from_str(json).unwrap();
        assert!(input.name.is_none());
        assert!(input.is_active.is_none());
    }
}

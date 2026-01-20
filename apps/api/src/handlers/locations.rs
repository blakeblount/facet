//! Storage location request handlers.

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::storage_location::StorageLocationSummary;
use crate::repositories::StorageLocationRepository;
use crate::response::ApiResponse;
use crate::routes::AppState;

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
}

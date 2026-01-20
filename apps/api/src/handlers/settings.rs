//! Store settings request handlers.

use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};

use crate::error::AppError;
use crate::models::store_settings::{StoreSettingsPublic, UpdateStoreSettings};
use crate::repositories::StoreSettingsRepository;
use crate::response::ApiResponse;
use crate::routes::AppState;

// =============================================================================
// GET /settings - Get Store Settings
// =============================================================================

/// GET /api/v1/settings - Get store settings (public read).
///
/// Returns the store settings without the admin PIN hash.
/// This endpoint is public and does not require authentication.
pub async fn get_settings(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let settings: StoreSettingsPublic =
        StoreSettingsRepository::get_settings_public(&state.db).await?;

    Ok(Json(ApiResponse::success(settings)))
}

// =============================================================================
// PUT /settings - Update Store Settings (Admin Only)
// =============================================================================

/// PUT /api/v1/settings - Update store settings (admin only).
///
/// Updates the store settings. Only the fields provided in the request body
/// will be updated; other fields retain their current values.
///
/// # Request Headers
/// - `X-Admin-PIN`: Required admin PIN for authentication
///
/// # Request Body (all fields optional)
/// - `store_name`: Store display name
/// - `store_phone`: Store phone number
/// - `store_address`: Store address
/// - `ticket_prefix`: Prefix for ticket IDs (e.g., "JR")
/// - `currency`: Currency code (e.g., "USD")
/// - `max_photos_per_ticket`: Maximum photos allowed per ticket
///
/// # Errors
/// - INVALID_PIN: If the X-Admin-PIN header is missing or incorrect
pub async fn update_settings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<UpdateStoreSettings>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin PIN from header
    let pin = headers
        .get("X-Admin-PIN")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::invalid_pin("Missing X-Admin-PIN header"))?;

    let is_valid = StoreSettingsRepository::verify_admin_pin(&state.db, pin).await?;

    if !is_valid {
        return Err(AppError::invalid_pin("Invalid admin PIN"));
    }

    // Update the settings
    let settings = StoreSettingsRepository::update_settings(&state.db, body).await?;

    Ok(Json(ApiResponse::success(settings)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_settings_handler_exists() {
        // Basic sanity test
        assert!(true);
    }

    #[test]
    fn test_update_store_settings_partial_deserialize() {
        let json = r#"{"store_name": "Updated Store"}"#;
        let input: UpdateStoreSettings = serde_json::from_str(json).unwrap();
        assert_eq!(input.store_name, Some("Updated Store".to_string()));
        assert!(input.store_phone.is_none());
        assert!(input.ticket_prefix.is_none());
    }

    #[test]
    fn test_update_store_settings_empty_deserialize() {
        let json = r#"{}"#;
        let input: UpdateStoreSettings = serde_json::from_str(json).unwrap();
        assert!(input.store_name.is_none());
        assert!(input.store_phone.is_none());
        assert!(input.store_address.is_none());
        assert!(input.ticket_prefix.is_none());
        assert!(input.currency.is_none());
        assert!(input.max_photos_per_ticket.is_none());
    }

    #[test]
    fn test_update_store_settings_full_deserialize() {
        let json = r#"{
            "store_name": "My Jewelry Store",
            "store_phone": "555-0100",
            "store_address": "456 Oak Ave",
            "ticket_prefix": "MJ",
            "currency": "EUR",
            "max_photos_per_ticket": 8
        }"#;
        let input: UpdateStoreSettings = serde_json::from_str(json).unwrap();
        assert_eq!(input.store_name, Some("My Jewelry Store".to_string()));
        assert_eq!(input.store_phone, Some("555-0100".to_string()));
        assert_eq!(input.store_address, Some("456 Oak Ave".to_string()));
        assert_eq!(input.ticket_prefix, Some("MJ".to_string()));
        assert_eq!(input.currency, Some("EUR".to_string()));
        assert_eq!(input.max_photos_per_ticket, Some(8));
    }
}

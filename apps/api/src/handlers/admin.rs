//! Admin request handlers.

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::store_settings::StoreSettingsPublic;
use crate::repositories::StoreSettingsRepository;
use crate::response::ApiResponse;
use crate::routes::AppState;

// =============================================================================
// POST /admin/setup - Initial Admin Setup
// =============================================================================

/// Request body for initial admin setup.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminSetupRequest {
    /// The current PIN (should be the default "changeme" on first use)
    pub current_pin: String,
    /// The new PIN to set
    pub new_pin: String,
}

/// Response for successful admin setup.
#[derive(Debug, Clone, Serialize)]
pub struct AdminSetupResponse {
    /// The updated store settings
    pub settings: StoreSettingsPublic,
}

/// POST /api/v1/admin/setup - Initial admin setup (force password change).
///
/// This endpoint is used for first-time setup to change the default admin PIN.
/// It can only be called once (when setup_complete is false).
///
/// # Request Body
/// - `current_pin`: The current PIN (default: "changeme")
/// - `new_pin`: The new PIN to set
///
/// # Errors
/// - FORBIDDEN: If setup is already complete
/// - INVALID_PIN: If the current PIN is incorrect
/// - VALIDATION_ERROR: If the new PIN is empty
pub async fn admin_setup(
    State(state): State<AppState>,
    Json(body): Json<AdminSetupRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Check if setup is already complete
    let is_complete = StoreSettingsRepository::is_setup_complete(&state.db).await?;
    if is_complete {
        return Err(AppError::forbidden("Setup has already been completed"));
    }

    // Verify the current PIN
    let is_valid = StoreSettingsRepository::verify_admin_pin(&state.db, &body.current_pin).await?;
    if !is_valid {
        return Err(AppError::invalid_pin("Invalid current PIN"));
    }

    // Validate new PIN
    if body.new_pin.is_empty() {
        return Err(AppError::validation("New PIN is required"));
    }

    // Change the admin PIN
    StoreSettingsRepository::change_admin_pin(&state.db, &body.new_pin).await?;

    // Mark setup as complete
    let settings = StoreSettingsRepository::mark_setup_complete(&state.db).await?;

    let response = AdminSetupResponse { settings };
    Ok(Json(ApiResponse::success(response)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_admin_setup_request_deserialize() {
        let json = r#"{"current_pin": "changeme", "new_pin": "secure1234"}"#;
        let request: AdminSetupRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.current_pin, "changeme");
        assert_eq!(request.new_pin, "secure1234");
    }

    #[test]
    fn test_admin_setup_request_missing_current_pin() {
        let json = r#"{"new_pin": "secure1234"}"#;
        let result: Result<AdminSetupRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_admin_setup_request_missing_new_pin() {
        let json = r#"{"current_pin": "changeme"}"#;
        let result: Result<AdminSetupRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_admin_setup_response_serialization() {
        let response = AdminSetupResponse {
            settings: StoreSettingsPublic {
                setting_id: uuid::Uuid::nil(),
                store_name: "Test Store".to_string(),
                store_phone: None,
                store_address: None,
                ticket_prefix: "JR".to_string(),
                next_ticket_number: 1,
                currency: "USD".to_string(),
                max_photos_per_ticket: 10,
                setup_complete: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"setup_complete\":true"));
        assert!(json.contains("\"store_name\":\"Test Store\""));
        // Should NOT contain admin_pin_hash
        assert!(!json.contains("admin_pin_hash"));
    }
}

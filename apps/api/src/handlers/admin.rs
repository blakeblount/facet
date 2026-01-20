//! Admin request handlers.

use axum::{extract::State, http::HeaderMap, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::models::store_settings::StoreSettingsPublic;
use crate::repositories::StoreSettingsRepository;
use crate::response::ApiResponse;
use crate::routes::AppState;

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

// =============================================================================
// POST /admin/change-pin - Change Admin PIN
// =============================================================================

/// Request body for changing admin PIN.
#[derive(Debug, Clone, Deserialize)]
pub struct ChangePinRequest {
    /// The new PIN to set
    pub new_pin: String,
}

/// Response for successful PIN change.
#[derive(Debug, Clone, Serialize)]
pub struct ChangePinResponse {
    /// The updated store settings
    pub settings: StoreSettingsPublic,
}

// =============================================================================
// POST /admin/verify - Verify Admin PIN
// =============================================================================

/// Request body for admin PIN verification.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminVerifyRequest {
    /// The PIN to verify
    pub pin: String,
}

/// Response for successful admin verification.
#[derive(Debug, Clone, Serialize)]
pub struct AdminVerifyResponse {
    /// Whether the PIN was valid
    pub valid: bool,
}

/// POST /api/v1/admin/verify - Verify the admin PIN.
///
/// This endpoint verifies that a given PIN matches the admin PIN.
/// Used by the UI to unlock admin features.
///
/// # Request Body
/// - `pin`: The PIN to verify
///
/// # Returns
/// - Success: `{ "valid": true }`
///
/// # Errors
/// - INVALID_PIN: If the PIN is incorrect
pub async fn verify_admin(
    State(state): State<AppState>,
    Json(body): Json<AdminVerifyRequest>,
) -> Result<impl IntoResponse, AppError> {
    let is_valid = StoreSettingsRepository::verify_admin_pin(&state.db, &body.pin).await?;

    if !is_valid {
        return Err(AppError::invalid_pin("Invalid admin PIN"));
    }

    let response = AdminVerifyResponse { valid: true };
    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /admin/change-pin - Change Admin PIN
// =============================================================================

/// POST /api/v1/admin/change-pin - Change the admin PIN.
///
/// This endpoint allows changing the admin PIN after initial setup.
/// Requires the current admin PIN in the X-Admin-PIN header.
///
/// # Request Headers
/// - `X-Admin-PIN`: The current admin PIN
///
/// # Request Body
/// - `new_pin`: The new PIN to set
///
/// # Errors
/// - INVALID_PIN: If the X-Admin-PIN header is missing or incorrect
/// - VALIDATION_ERROR: If the new PIN is empty
pub async fn change_pin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChangePinRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verify the current admin PIN
    verify_admin_pin_header(&state, &headers).await?;

    // Validate new PIN
    if body.new_pin.is_empty() {
        return Err(AppError::validation("New PIN is required"));
    }

    // Change the admin PIN
    let settings = StoreSettingsRepository::change_admin_pin(&state.db, &body.new_pin).await?;

    let response = ChangePinResponse { settings };
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

    // Tests for ChangePinRequest

    #[test]
    fn test_change_pin_request_deserialize() {
        let json = r#"{"new_pin": "newpin1234"}"#;
        let request: ChangePinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.new_pin, "newpin1234");
    }

    #[test]
    fn test_change_pin_request_missing_new_pin() {
        let json = r#"{}"#;
        let result: Result<ChangePinRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_change_pin_request_empty_new_pin() {
        let json = r#"{"new_pin": ""}"#;
        let request: ChangePinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.new_pin, "");
    }

    // Tests for AdminVerifyRequest

    #[test]
    fn test_admin_verify_request_deserialize() {
        let json = r#"{"pin": "secret1234"}"#;
        let request: AdminVerifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.pin, "secret1234");
    }

    #[test]
    fn test_admin_verify_request_missing_pin() {
        let json = r#"{}"#;
        let result: Result<AdminVerifyRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_admin_verify_request_empty_pin() {
        let json = r#"{"pin": ""}"#;
        let request: AdminVerifyRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.pin, "");
    }

    // Tests for AdminVerifyResponse

    #[test]
    fn test_admin_verify_response_serialization() {
        let response = AdminVerifyResponse { valid: true };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"valid\":true"));
    }

    #[test]
    fn test_admin_verify_response_valid_false() {
        let response = AdminVerifyResponse { valid: false };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"valid\":false"));
    }

    // Tests for ChangePinResponse

    #[test]
    fn test_change_pin_response_serialization() {
        let response = ChangePinResponse {
            settings: StoreSettingsPublic {
                setting_id: uuid::Uuid::nil(),
                store_name: "Test Store".to_string(),
                store_phone: Some("555-1234".to_string()),
                store_address: Some("123 Main St".to_string()),
                ticket_prefix: "JR".to_string(),
                next_ticket_number: 42,
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
        assert!(json.contains("\"store_phone\":\"555-1234\""));
        // Should NOT contain admin_pin_hash
        assert!(!json.contains("admin_pin_hash"));
    }
}

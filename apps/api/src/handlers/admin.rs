//! Admin request handlers.

use axum::{
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

use crate::auth::validate_pin_complexity;
use crate::error::AppError;
use crate::middleware::extract_client_ip;
use crate::models::store_settings::StoreSettingsPublic;
use crate::repositories::{AdminSessionRepository, StoreSettingsRepository};
use crate::response::ApiResponse;
use crate::routes::AppState;

// =============================================================================
// Admin Authentication Helpers
// =============================================================================

/// Verify admin authentication via session token (X-Admin-Session header).
///
/// This is the preferred authentication method. Session tokens are issued
/// after successful PIN verification and avoid sending the PIN on every request.
///
/// Returns an error if the header is missing, or the session is invalid/expired.
pub async fn verify_admin_session_header(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<(), AppError> {
    let token = headers
        .get("X-Admin-Session")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::unauthorized("Missing X-Admin-Session header"))?;

    let session = AdminSessionRepository::verify_and_touch(&state.db, token).await?;

    if session.is_none() {
        return Err(AppError::unauthorized("Invalid or expired session"));
    }

    Ok(())
}

/// Extract and verify admin PIN from X-Admin-PIN header.
///
/// DEPRECATED: Use session-based authentication instead.
/// This is kept for backwards compatibility but should not be used for new code.
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

/// Verify admin authentication via either session token or PIN.
///
/// Prefers session token (X-Admin-Session) but falls back to PIN (X-Admin-PIN)
/// for backwards compatibility. New clients should use session-based auth.
pub async fn verify_admin_auth(state: &AppState, headers: &HeaderMap) -> Result<(), AppError> {
    // Try session token first (preferred)
    if headers.contains_key("X-Admin-Session") {
        return verify_admin_session_header(state, headers).await;
    }

    // Fall back to PIN header (deprecated, for backwards compatibility)
    if headers.contains_key("X-Admin-PIN") {
        return verify_admin_pin_header(state, headers).await;
    }

    Err(AppError::unauthorized(
        "Missing authentication. Provide X-Admin-Session header.",
    ))
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
/// It can only be called once (when setup_complete is false) and within the setup deadline.
///
/// # Request Body
/// - `current_pin`: The current PIN (default: "changeme")
/// - `new_pin`: The new PIN to set
///
/// # Errors
/// - FORBIDDEN: If setup is already complete
/// - SETUP_EXPIRED: If setup deadline has passed
/// - INVALID_PIN: If the current PIN is incorrect
/// - VALIDATION_ERROR: If the new PIN is empty or too weak
/// - RATE_LIMITED: If too many attempts from the same IP
pub async fn admin_setup(
    State(state): State<AppState>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(body): Json<AdminSetupRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Extract client IP for rate limiting
    let client_ip = extract_client_ip(&headers, connect_info.map(|c| c.0));

    // Check rate limit
    if let Err(retry_after) = state.rate_limit.check_rate_limit(client_ip).await {
        return Err(AppError::rate_limited(
            "Too many authentication attempts. Please wait before trying again.",
            retry_after,
        ));
    }

    // Check if setup is already complete
    let is_complete = StoreSettingsRepository::is_setup_complete(&state.db).await?;
    if is_complete {
        return Err(AppError::forbidden("Setup has already been completed"));
    }

    // Check if setup deadline has expired
    let is_expired = StoreSettingsRepository::is_setup_expired(&state.db).await?;
    if is_expired {
        tracing::warn!("Setup attempted after deadline expiration");
        return Err(AppError::setup_expired(
            "Initial setup deadline has passed. Please contact system administrator.",
        ));
    }

    // Verify the current PIN
    let is_valid = StoreSettingsRepository::verify_admin_pin(&state.db, &body.current_pin).await?;
    if !is_valid {
        // Record failure for exponential backoff
        state.rate_limit.record_failure(client_ip).await;
        // Log default PIN usage attempt
        tracing::warn!("Invalid PIN attempt during setup - setup incomplete");
        return Err(AppError::invalid_pin("Invalid current PIN"));
    }

    // Validate new PIN - check if empty first
    if body.new_pin.is_empty() {
        return Err(AppError::validation("New PIN is required"));
    }

    // Validate new PIN complexity
    let min_pin_length = StoreSettingsRepository::get_min_pin_length(&state.db).await?;
    let validation_result = validate_pin_complexity(&body.new_pin, min_pin_length);
    if !validation_result.valid {
        return Err(AppError::validation(
            validation_result
                .error
                .unwrap_or_else(|| "Invalid PIN".to_string()),
        ));
    }

    // Change the admin PIN
    StoreSettingsRepository::change_admin_pin(&state.db, &body.new_pin).await?;

    // Mark setup as complete
    let settings = StoreSettingsRepository::mark_setup_complete(&state.db).await?;

    // Record success to reset backoff
    state.rate_limit.record_success(client_ip).await;

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
// POST /admin/verify - Verify Admin PIN and Get Session Token
// =============================================================================

/// Request body for admin PIN verification.
#[derive(Debug, Clone, Deserialize)]
pub struct AdminVerifyRequest {
    /// The PIN to verify
    pub pin: String,
}

/// Response for successful admin verification.
///
/// Returns a session token that should be used for subsequent admin requests
/// via the X-Admin-Session header instead of sending the PIN repeatedly.
#[derive(Debug, Clone, Serialize)]
pub struct AdminVerifyResponse {
    /// Whether the PIN was valid
    pub valid: bool,
    /// Session token for subsequent requests (use in X-Admin-Session header)
    pub session_token: String,
    /// When the session expires (ISO 8601 format)
    pub expires_at: DateTime<Utc>,
}

/// POST /api/v1/admin/verify - Verify the admin PIN and get a session token.
///
/// This endpoint verifies that a given PIN matches the admin PIN.
/// On success, it creates a new admin session and returns a session token
/// that should be used for subsequent admin API requests.
///
/// # Request Body
/// - `pin`: The PIN to verify
///
/// # Returns
/// - Success: `{ "valid": true, "session_token": "...", "expires_at": "..." }`
///
/// # Session Usage
/// Use the returned `session_token` in the `X-Admin-Session` header for all
/// subsequent admin requests. This avoids sending the PIN on every request.
///
/// # Errors
/// - INVALID_PIN: If the PIN is incorrect
/// - SETUP_EXPIRED: If setup deadline has passed without completing setup
/// - RATE_LIMITED: If too many attempts from the same IP
pub async fn verify_admin(
    State(state): State<AppState>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(body): Json<AdminVerifyRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Extract client IP for rate limiting
    let client_ip = extract_client_ip(&headers, connect_info.map(|c| c.0));

    // Check rate limit
    if let Err(retry_after) = state.rate_limit.check_rate_limit(client_ip).await {
        return Err(AppError::rate_limited(
            "Too many authentication attempts. Please wait before trying again.",
            retry_after,
        ));
    }

    // Check if setup deadline has expired without completion
    let is_expired = StoreSettingsRepository::is_setup_expired(&state.db).await?;
    if is_expired {
        tracing::warn!("Admin verification attempted after setup deadline expiration");
        return Err(AppError::setup_expired(
            "Initial setup deadline has passed. Please contact system administrator.",
        ));
    }

    let is_valid = StoreSettingsRepository::verify_admin_pin(&state.db, &body.pin).await?;

    if !is_valid {
        // Record failure for exponential backoff
        state.rate_limit.record_failure(client_ip).await;
        return Err(AppError::invalid_pin("Invalid admin PIN"));
    }

    // Log warning if using default PIN (setup not complete)
    let is_complete = StoreSettingsRepository::is_setup_complete(&state.db).await?;
    if !is_complete {
        tracing::warn!("Admin operation attempted with default PIN - setup incomplete");
    }

    // Record success to reset backoff
    state.rate_limit.record_success(client_ip).await;

    // Create a new admin session
    let session = AdminSessionRepository::create(&state.db).await?;

    let response = AdminVerifyResponse {
        valid: true,
        session_token: session.session_token,
        expires_at: session.expires_at,
    };
    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /admin/change-pin - Change Admin PIN
// =============================================================================

/// POST /api/v1/admin/change-pin - Change the admin PIN.
///
/// This endpoint allows changing the admin PIN after initial setup.
/// Requires admin authentication via X-Admin-Session header (preferred)
/// or X-Admin-PIN header (deprecated).
///
/// # Request Headers
/// - `X-Admin-Session`: Session token (preferred)
/// - `X-Admin-PIN`: The current admin PIN (deprecated)
///
/// # Request Body
/// - `new_pin`: The new PIN to set
///
/// # Errors
/// - UNAUTHORIZED: If not authenticated
/// - INVALID_PIN: If the X-Admin-PIN header is incorrect (deprecated auth)
/// - VALIDATION_ERROR: If the new PIN is empty or too weak
pub async fn change_pin(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ChangePinRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin authentication (session or PIN)
    verify_admin_auth(&state, &headers).await?;

    // Validate new PIN - check if empty first
    if body.new_pin.is_empty() {
        return Err(AppError::validation("New PIN is required"));
    }

    // Validate new PIN complexity
    let min_pin_length = StoreSettingsRepository::get_min_pin_length(&state.db).await?;
    let validation_result = validate_pin_complexity(&body.new_pin, min_pin_length);
    if !validation_result.valid {
        return Err(AppError::validation(
            validation_result
                .error
                .unwrap_or_else(|| "Invalid PIN".to_string()),
        ));
    }

    // Change the admin PIN
    let settings = StoreSettingsRepository::change_admin_pin(&state.db, &body.new_pin).await?;

    let response = ChangePinResponse { settings };
    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /admin/logout - End Admin Session
// =============================================================================

/// Response for successful admin logout.
#[derive(Debug, Clone, Serialize)]
pub struct AdminLogoutResponse {
    /// Whether the logout was successful
    pub success: bool,
}

/// POST /api/v1/admin/logout - End the admin session.
///
/// Invalidates the current admin session token. After calling this endpoint,
/// the session token can no longer be used for authentication.
///
/// # Request Headers
/// - `X-Admin-Session`: The session token to invalidate
///
/// # Returns
/// - Success: `{ "success": true }`
///
/// # Notes
/// - Does not return an error if the session is already invalid/expired
/// - If no session header is provided, returns success (idempotent)
pub async fn admin_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, AppError> {
    // Get the session token if present
    if let Some(token) = headers.get("X-Admin-Session").and_then(|v| v.to_str().ok()) {
        AdminSessionRepository::delete_by_token(&state.db, token).await?;
    }

    // Always return success (idempotent)
    let response = AdminLogoutResponse { success: true };
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
                setup_required: false,
                min_pin_length: 6,
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
        let response = AdminVerifyResponse {
            valid: true,
            session_token: "test_token_abc123".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(30),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"valid\":true"));
        assert!(json.contains("\"session_token\":\"test_token_abc123\""));
        assert!(json.contains("\"expires_at\":"));
    }

    #[test]
    fn test_admin_verify_response_contains_session_info() {
        let response = AdminVerifyResponse {
            valid: true,
            session_token: "secure_random_token".to_string(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(30),
        };
        let json = serde_json::to_string(&response).unwrap();
        // Should contain all required fields
        assert!(json.contains("session_token"));
        assert!(json.contains("expires_at"));
    }

    // Tests for AdminLogoutResponse

    #[test]
    fn test_admin_logout_response_serialization() {
        let response = AdminLogoutResponse { success: true };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
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
                setup_required: false,
                min_pin_length: 6,
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

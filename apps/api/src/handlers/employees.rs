//! Employee request handlers.

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::verify_pin;
use crate::error::AppError;
use crate::models::employee::EmployeeRole;
use crate::repositories::EmployeeRepository;
use crate::response::ApiResponse;
use crate::routes::AppState;

// =============================================================================
// POST /employees/verify - Verify Employee PIN
// =============================================================================

/// Request body for verifying an employee PIN.
#[derive(Debug, Clone, Deserialize)]
pub struct VerifyPinRequest {
    /// The PIN to verify
    pub pin: String,
}

/// Response for a successful PIN verification.
#[derive(Debug, Clone, Serialize)]
pub struct VerifyPinResponse {
    /// The employee's unique identifier
    pub employee_id: Uuid,
    /// The employee's name
    pub name: String,
    /// The employee's role
    pub role: EmployeeRole,
}

/// POST /api/v1/employees/verify - Verify an employee PIN.
///
/// Accepts a PIN in the request body and returns the employee's
/// ID, name, and role if the PIN is valid.
///
/// Returns INVALID_PIN error if no active employee matches the PIN.
pub async fn verify_employee_pin(
    State(state): State<AppState>,
    Json(body): Json<VerifyPinRequest>,
) -> Result<impl IntoResponse, AppError> {
    // Get all active employees for PIN verification
    let employees = EmployeeRepository::find_active_for_pin_verification(&state.db).await?;

    // Find an employee whose PIN matches
    for employee in employees {
        if verify_pin(&body.pin, &employee.pin_hash)? {
            let response = VerifyPinResponse {
                employee_id: employee.employee_id,
                name: employee.name,
                role: employee.role,
            };
            return Ok(Json(ApiResponse::success(response)));
        }
    }

    // No matching PIN found
    Err(AppError::invalid_pin("Invalid PIN"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verify_pin_request_deserialize() {
        let json = r#"{"pin": "1234"}"#;
        let request: VerifyPinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.pin, "1234");
    }

    #[test]
    fn test_verify_pin_request_deserialize_empty_pin() {
        let json = r#"{"pin": ""}"#;
        let request: VerifyPinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.pin, "");
    }

    #[test]
    fn test_verify_pin_request_missing_pin() {
        let json = r#"{}"#;
        let result: Result<VerifyPinRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_pin_response_serialization() {
        let response = VerifyPinResponse {
            employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Alice".to_string(),
            role: EmployeeRole::Staff,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"employee_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"name\":\"Alice\""));
        assert!(json.contains("\"role\":\"staff\""));
    }

    #[test]
    fn test_verify_pin_response_admin_role() {
        let response = VerifyPinResponse {
            employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Admin User".to_string(),
            role: EmployeeRole::Admin,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"role\":\"admin\""));
    }
}

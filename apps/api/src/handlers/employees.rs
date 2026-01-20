//! Employee request handlers.

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::verify_pin;
use crate::error::AppError;
use crate::models::employee::{CreateEmployee, EmployeeRole, EmployeeSummary, UpdateEmployee};
use crate::repositories::{EmployeeRepository, StoreSettingsRepository};
use crate::response::{created, ApiResponse};
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

// =============================================================================
// POST /employees (admin) - Create Employee
// =============================================================================

/// POST /api/v1/employees - Create a new employee (admin only).
///
/// Requires X-Admin-PIN header for authorization.
/// Creates an employee with the provided name, PIN, and role.
/// The PIN is hashed before storage using argon2.
///
/// Returns the created employee (without pin_hash).
pub async fn create_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateEmployee>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin PIN
    verify_admin_pin_header(&state, &headers).await?;

    // Validate input
    if body.name.trim().is_empty() {
        return Err(AppError::validation("Name is required"));
    }

    if body.pin.is_empty() {
        return Err(AppError::validation("PIN is required"));
    }

    // Create the employee (PIN is hashed in the repository)
    let employee = EmployeeRepository::create(&state.db, body).await?;

    // Return as EmployeeSummary (without pin_hash)
    let summary = EmployeeSummary {
        employee_id: employee.employee_id,
        name: employee.name,
        role: employee.role,
        is_active: employee.is_active,
    };

    Ok(created(summary))
}

// =============================================================================
// PUT /employees/:employee_id (admin) - Update Employee
// =============================================================================

/// PUT /api/v1/employees/:employee_id - Update an employee (admin only).
///
/// Requires X-Admin-PIN header for authorization.
/// Updates employee fields: name, role, is_active.
/// If PIN is provided, it's re-hashed before storage.
///
/// Returns the updated employee (without pin_hash).
pub async fn update_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
    Json(body): Json<UpdateEmployee>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin PIN
    verify_admin_pin_header(&state, &headers).await?;

    // Validate input - if name is provided, it shouldn't be empty
    if let Some(ref name) = body.name {
        if name.trim().is_empty() {
            return Err(AppError::validation("Name cannot be empty"));
        }
    }

    // Validate input - if PIN is provided, it shouldn't be empty
    if let Some(ref pin) = body.pin {
        if pin.is_empty() {
            return Err(AppError::validation("PIN cannot be empty"));
        }
    }

    // Update the employee
    let employee = EmployeeRepository::update(&state.db, employee_id, body).await?;

    match employee {
        Some(emp) => {
            // Return as EmployeeSummary (without pin_hash)
            let summary = EmployeeSummary {
                employee_id: emp.employee_id,
                name: emp.name,
                role: emp.role,
                is_active: emp.is_active,
            };
            Ok(Json(ApiResponse::success(summary)))
        }
        None => Err(AppError::not_found("Employee not found")),
    }
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

    // Tests for CreateEmployee deserialization

    #[test]
    fn test_create_employee_deserialize_minimal() {
        let json = r#"{"name": "John Doe", "pin": "1234"}"#;
        let input: CreateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "John Doe");
        assert_eq!(input.pin, "1234");
        assert!(input.role.is_none());
    }

    #[test]
    fn test_create_employee_deserialize_with_role() {
        let json = r#"{"name": "Jane Admin", "pin": "5678", "role": "admin"}"#;
        let input: CreateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "Jane Admin");
        assert_eq!(input.pin, "5678");
        assert_eq!(input.role, Some(EmployeeRole::Admin));
    }

    #[test]
    fn test_create_employee_deserialize_staff_role() {
        let json = r#"{"name": "Staff User", "pin": "9999", "role": "staff"}"#;
        let input: CreateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.role, Some(EmployeeRole::Staff));
    }

    #[test]
    fn test_create_employee_missing_name() {
        let json = r#"{"pin": "1234"}"#;
        let result: Result<CreateEmployee, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_create_employee_missing_pin() {
        let json = r#"{"name": "John"}"#;
        let result: Result<CreateEmployee, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    // Tests for EmployeeSummary serialization

    #[test]
    fn test_employee_summary_serialization() {
        let summary = EmployeeSummary {
            employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Test User".to_string(),
            role: EmployeeRole::Staff,
            is_active: true,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"employee_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"name\":\"Test User\""));
        assert!(json.contains("\"role\":\"staff\""));
        assert!(json.contains("\"is_active\":true"));
        // Importantly, should NOT contain pin_hash
        assert!(!json.contains("pin_hash"));
    }

    #[test]
    fn test_employee_summary_serialization_inactive() {
        let summary = EmployeeSummary {
            employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Inactive User".to_string(),
            role: EmployeeRole::Admin,
            is_active: false,
        };

        let json = serde_json::to_string(&summary).unwrap();
        assert!(json.contains("\"is_active\":false"));
        assert!(json.contains("\"role\":\"admin\""));
    }

    // Tests for UpdateEmployee deserialization

    #[test]
    fn test_update_employee_deserialize_empty_body() {
        let json = r#"{}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert!(input.name.is_none());
        assert!(input.pin.is_none());
        assert!(input.role.is_none());
        assert!(input.is_active.is_none());
    }

    #[test]
    fn test_update_employee_deserialize_name_only() {
        let json = r#"{"name": "New Name"}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("New Name".to_string()));
        assert!(input.pin.is_none());
        assert!(input.role.is_none());
        assert!(input.is_active.is_none());
    }

    #[test]
    fn test_update_employee_deserialize_with_pin() {
        let json = r#"{"pin": "9999"}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert!(input.name.is_none());
        assert_eq!(input.pin, Some("9999".to_string()));
    }

    #[test]
    fn test_update_employee_deserialize_role_change() {
        let json = r#"{"role": "admin"}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert!(input.name.is_none());
        assert_eq!(input.role, Some(EmployeeRole::Admin));
    }

    #[test]
    fn test_update_employee_deserialize_deactivate() {
        let json = r#"{"is_active": false}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.is_active, Some(false));
    }

    #[test]
    fn test_update_employee_deserialize_full_update() {
        let json = r#"{"name": "Updated Name", "pin": "5678", "role": "staff", "is_active": true}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("Updated Name".to_string()));
        assert_eq!(input.pin, Some("5678".to_string()));
        assert_eq!(input.role, Some(EmployeeRole::Staff));
        assert_eq!(input.is_active, Some(true));
    }
}

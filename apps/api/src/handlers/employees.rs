//! Employee request handlers.

use axum::{
    extract::{ConnectInfo, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use uuid::Uuid;

use crate::auth::verify_pin;
use crate::error::AppError;
use crate::handlers::verify_admin_auth;
use crate::middleware::extract_client_ip;
use crate::models::employee::{CreateEmployee, EmployeeRole, EmployeeSummary, UpdateEmployee};
use crate::repositories::EmployeeRepository;
use crate::response::{created, ApiResponse};
use crate::routes::AppState;

// =============================================================================
// GET /employees (admin) - List Employees
// =============================================================================

/// Query parameters for listing employees.
#[derive(Debug, Clone, Deserialize)]
pub struct ListEmployeesQuery {
    /// Include inactive employees in the list.
    /// Defaults to false (only active employees returned).
    #[serde(default)]
    pub include_inactive: bool,
}

/// Response for listing employees.
#[derive(Debug, Clone, Serialize)]
pub struct ListEmployeesResponse {
    /// List of employees
    pub employees: Vec<EmployeeSummary>,
    /// Total count of employees returned
    pub count: usize,
}

/// GET /api/v1/employees - List all employees (admin only).
///
/// Requires admin authentication via X-Admin-Session header (preferred)
/// or X-Admin-PIN header (deprecated).
/// Returns a list of employees (without pin_hash).
/// By default only active employees are returned.
/// Use `?include_inactive=true` to include inactive employees.
pub async fn list_employees(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListEmployeesQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin authentication (session or PIN)
    verify_admin_auth(&state, &headers).await?;

    // Fetch employees from repository
    let employees = EmployeeRepository::list(&state.db, query.include_inactive).await?;

    let response = ListEmployeesResponse {
        count: employees.len(),
        employees,
    };

    Ok(Json(ApiResponse::success(response)))
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
/// Returns RATE_LIMITED error (429) if too many attempts from the same IP.
pub async fn verify_employee_pin(
    State(state): State<AppState>,
    headers: HeaderMap,
    connect_info: Option<ConnectInfo<SocketAddr>>,
    Json(body): Json<VerifyPinRequest>,
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

    // Get all active employees for PIN verification
    let employees = EmployeeRepository::find_active_for_pin_verification(&state.db).await?;

    // Find an employee whose PIN matches
    for employee in employees {
        if verify_pin(&body.pin, &employee.pin_hash)? {
            // Record success to reset backoff
            state.rate_limit.record_success(client_ip).await;

            let response = VerifyPinResponse {
                employee_id: employee.employee_id,
                name: employee.name,
                role: employee.role,
            };
            return Ok(Json(ApiResponse::success(response)));
        }
    }

    // Record failure for exponential backoff
    state.rate_limit.record_failure(client_ip).await;

    // No matching PIN found
    Err(AppError::invalid_pin("Invalid PIN"))
}

// =============================================================================
// POST /employees (admin) - Create Employee
// =============================================================================

/// POST /api/v1/employees - Create a new employee (admin only).
///
/// Requires admin authentication via X-Admin-Session header (preferred)
/// or X-Admin-PIN header (deprecated).
/// Creates an employee with the provided name, PIN, and role.
/// The PIN is hashed before storage using argon2.
///
/// Returns the created employee (without pin_hash).
pub async fn create_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateEmployee>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin authentication (session or PIN)
    verify_admin_auth(&state, &headers).await?;

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
/// Requires admin authentication via X-Admin-Session header (preferred)
/// or X-Admin-PIN header (deprecated).
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
    // Verify admin authentication (session or PIN)
    verify_admin_auth(&state, &headers).await?;

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

// =============================================================================
// DELETE /employees/:employee_id (admin) - Delete Employee
// =============================================================================

/// Response for employee deletion.
#[derive(Debug, Clone, Serialize)]
pub struct DeleteEmployeeResponse {
    /// Whether the employee was deleted
    pub deleted: bool,
    /// Warning message if the employee had attribution history
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// DELETE /api/v1/employees/:employee_id - Delete an employee (admin only).
///
/// Requires admin authentication via X-Admin-Session header (preferred)
/// or X-Admin-PIN header (deprecated).
/// Checks for attribution history and includes a warning if history exists.
/// Performs a hard delete (employee is permanently removed).
///
/// Returns success with optional warning about history loss.
pub async fn delete_employee(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(employee_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Verify admin authentication (session or PIN)
    verify_admin_auth(&state, &headers).await?;

    // Check if employee exists
    let employee = EmployeeRepository::find_by_id(&state.db, employee_id).await?;
    if employee.is_none() {
        return Err(AppError::not_found("Employee not found"));
    }

    // Check for attribution history
    let attribution_count = EmployeeRepository::count_attributions(&state.db, employee_id).await?;

    // Build warning message if attributions exist
    let warning = if attribution_count > 0 {
        Some(format!(
            "Employee had {} attribution(s) in history. This data has been deleted.",
            attribution_count
        ))
    } else {
        None
    };

    // Perform hard delete
    let deleted = EmployeeRepository::hard_delete(&state.db, employee_id).await?;

    if !deleted {
        return Err(AppError::not_found("Employee not found"));
    }

    let response = DeleteEmployeeResponse { deleted, warning };

    Ok(Json(ApiResponse::success(response)))
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

    // Tests for DeleteEmployeeResponse serialization

    #[test]
    fn test_delete_employee_response_serialization_no_warning() {
        let response = DeleteEmployeeResponse {
            deleted: true,
            warning: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"deleted\":true"));
        // warning should be omitted when None
        assert!(!json.contains("warning"));
    }

    #[test]
    fn test_delete_employee_response_serialization_with_warning() {
        let response = DeleteEmployeeResponse {
            deleted: true,
            warning: Some(
                "Employee had 5 attribution(s) in history. This data has been deleted.".to_string(),
            ),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"deleted\":true"));
        assert!(json.contains("\"warning\":"));
        assert!(json.contains("5 attribution(s)"));
    }

    // Tests for ListEmployeesQuery deserialization

    #[test]
    fn test_list_employees_query_deserialize_empty() {
        let json = r#"{}"#;
        let query: ListEmployeesQuery = serde_json::from_str(json).unwrap();
        // include_inactive should default to false
        assert!(!query.include_inactive);
    }

    #[test]
    fn test_list_employees_query_deserialize_include_inactive_true() {
        let json = r#"{"include_inactive": true}"#;
        let query: ListEmployeesQuery = serde_json::from_str(json).unwrap();
        assert!(query.include_inactive);
    }

    #[test]
    fn test_list_employees_query_deserialize_include_inactive_false() {
        let json = r#"{"include_inactive": false}"#;
        let query: ListEmployeesQuery = serde_json::from_str(json).unwrap();
        assert!(!query.include_inactive);
    }

    // Tests for ListEmployeesResponse serialization

    #[test]
    fn test_list_employees_response_serialization_empty() {
        let response = ListEmployeesResponse {
            employees: vec![],
            count: 0,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"employees\":[]"));
        assert!(json.contains("\"count\":0"));
    }

    #[test]
    fn test_list_employees_response_serialization_with_employees() {
        let response = ListEmployeesResponse {
            employees: vec![
                EmployeeSummary {
                    employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
                    name: "Alice".to_string(),
                    role: EmployeeRole::Staff,
                    is_active: true,
                },
                EmployeeSummary {
                    employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440001").unwrap(),
                    name: "Bob".to_string(),
                    role: EmployeeRole::Admin,
                    is_active: true,
                },
            ],
            count: 2,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"name\":\"Alice\""));
        assert!(json.contains("\"name\":\"Bob\""));
        assert!(json.contains("\"role\":\"staff\""));
        assert!(json.contains("\"role\":\"admin\""));
        // Should NOT contain pin_hash
        assert!(!json.contains("pin_hash"));
    }
}

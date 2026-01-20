//! Employee model and related types.
//!
//! Employees are staff members who can perform actions in the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

/// Employee role enum matching the database type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "employee_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EmployeeRole {
    Staff,
    Admin,
}

/// Full employee entity with all fields.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Employee {
    pub employee_id: Uuid,
    pub name: String,
    #[serde(skip_serializing)]
    pub pin_hash: String,
    pub role: EmployeeRole,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Summary view of an employee (without PIN hash).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EmployeeSummary {
    pub employee_id: Uuid,
    pub name: String,
    pub role: EmployeeRole,
    pub is_active: bool,
}

/// Input for creating a new employee.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateEmployee {
    pub name: String,
    pub pin: String,
    #[serde(default)]
    pub role: Option<EmployeeRole>,
}

/// Input for updating an employee.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateEmployee {
    pub name: Option<String>,
    pub pin: Option<String>,
    pub role: Option<EmployeeRole>,
    pub is_active: Option<bool>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_employee_role_serialization() {
        let role = EmployeeRole::Admin;
        let json = serde_json::to_string(&role).unwrap();
        assert_eq!(json, "\"admin\"");

        let parsed: EmployeeRole = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, EmployeeRole::Admin);
    }

    #[test]
    fn test_create_employee_deserialize() {
        let json = r#"{"name": "John Doe", "pin": "1234"}"#;
        let input: CreateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "John Doe");
        assert_eq!(input.pin, "1234");
        assert!(input.role.is_none());
    }

    #[test]
    fn test_create_employee_with_role() {
        let json = r#"{"name": "Jane Admin", "pin": "5678", "role": "admin"}"#;
        let input: CreateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, "Jane Admin");
        assert_eq!(input.pin, "5678");
        assert_eq!(input.role, Some(EmployeeRole::Admin));
    }

    #[test]
    fn test_update_employee_partial() {
        let json = r#"{"name": "New Name"}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("New Name".to_string()));
        assert!(input.pin.is_none());
        assert!(input.role.is_none());
        assert!(input.is_active.is_none());
    }

    #[test]
    fn test_update_employee_full() {
        let json = r#"{"name": "New Name", "pin": "9999", "role": "staff", "is_active": false}"#;
        let input: UpdateEmployee = serde_json::from_str(json).unwrap();
        assert_eq!(input.name, Some("New Name".to_string()));
        assert_eq!(input.pin, Some("9999".to_string()));
        assert_eq!(input.role, Some(EmployeeRole::Staff));
        assert_eq!(input.is_active, Some(false));
    }
}

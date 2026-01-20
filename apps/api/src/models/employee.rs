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
}

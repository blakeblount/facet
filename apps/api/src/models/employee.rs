//! Employee model and related types.
//!
//! Employees are staff members who can perform actions in the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

/// Permission types for role-based access control.
///
/// These define the specific actions that can be performed in the system.
/// Admin has all permissions; Staff has a restricted subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    /// Create new tickets
    CreateTicket,
    /// View any ticket
    ViewTicket,
    /// Modify tickets the employee owns (taken_in_by or worked_by)
    ModifyOwnTicket,
    /// Modify any ticket regardless of ownership
    ModifyAnyTicket,
    /// Add notes to tickets the employee is authorized for
    AddNotes,
    /// Upload photos to tickets the employee is authorized for
    UploadPhotos,
    /// Delete photos (admin only)
    DeletePhotos,
    /// Close any ticket (admin only)
    CloseAnyTicket,
    /// Manage employees (admin only)
    ManageEmployees,
    /// Manage store settings (admin only)
    ManageSettings,
    /// Manage storage locations (admin only)
    ManageLocations,
}

/// Employee role enum matching the database type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "employee_role", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EmployeeRole {
    Staff,
    Admin,
}

impl EmployeeRole {
    /// Check if this role has the specified permission.
    ///
    /// Admin has all permissions. Staff has a limited set of permissions
    /// focused on day-to-day operations without destructive capabilities.
    pub fn has_permission(&self, permission: Permission) -> bool {
        match self {
            // Admin has all permissions
            EmployeeRole::Admin => true,
            // Staff has limited permissions
            EmployeeRole::Staff => matches!(
                permission,
                Permission::CreateTicket
                    | Permission::ViewTicket
                    | Permission::ModifyOwnTicket
                    | Permission::AddNotes
                    | Permission::UploadPhotos
            ),
        }
    }
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

    // Permission tests

    #[test]
    fn test_admin_has_all_permissions() {
        let admin = EmployeeRole::Admin;

        // Admin has all permissions
        assert!(admin.has_permission(Permission::CreateTicket));
        assert!(admin.has_permission(Permission::ViewTicket));
        assert!(admin.has_permission(Permission::ModifyOwnTicket));
        assert!(admin.has_permission(Permission::ModifyAnyTicket));
        assert!(admin.has_permission(Permission::AddNotes));
        assert!(admin.has_permission(Permission::UploadPhotos));
        assert!(admin.has_permission(Permission::DeletePhotos));
        assert!(admin.has_permission(Permission::CloseAnyTicket));
        assert!(admin.has_permission(Permission::ManageEmployees));
        assert!(admin.has_permission(Permission::ManageSettings));
        assert!(admin.has_permission(Permission::ManageLocations));
    }

    #[test]
    fn test_staff_has_basic_permissions() {
        let staff = EmployeeRole::Staff;

        // Staff has basic permissions
        assert!(staff.has_permission(Permission::CreateTicket));
        assert!(staff.has_permission(Permission::ViewTicket));
        assert!(staff.has_permission(Permission::ModifyOwnTicket));
        assert!(staff.has_permission(Permission::AddNotes));
        assert!(staff.has_permission(Permission::UploadPhotos));
    }

    #[test]
    fn test_staff_lacks_admin_permissions() {
        let staff = EmployeeRole::Staff;

        // Staff does NOT have admin-only permissions
        assert!(!staff.has_permission(Permission::ModifyAnyTicket));
        assert!(!staff.has_permission(Permission::DeletePhotos));
        assert!(!staff.has_permission(Permission::CloseAnyTicket));
        assert!(!staff.has_permission(Permission::ManageEmployees));
        assert!(!staff.has_permission(Permission::ManageSettings));
        assert!(!staff.has_permission(Permission::ManageLocations));
    }
}

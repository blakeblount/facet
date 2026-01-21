//! Role-based access control (RBAC) helpers.
//!
//! Provides functions for checking employee permissions on tickets
//! and other resources based on their role and relationship to the resource.

use crate::error::AppError;
use crate::models::{Employee, EmployeeRole, Permission, Ticket};

/// Check if an employee has the required permission.
///
/// Returns `Ok(())` if the employee has the permission, or an error if not.
pub fn require_permission(employee: &Employee, permission: Permission) -> Result<(), AppError> {
    if employee.role.has_permission(permission) {
        Ok(())
    } else {
        Err(AppError::forbidden(
            "You do not have permission to perform this action",
        ))
    }
}

/// Check if an employee has permission to access/modify a ticket.
///
/// For staff employees, this checks ownership (taken_in_by or worked_by).
/// Admin employees always have access.
///
/// # Arguments
/// * `employee` - The employee requesting access
/// * `ticket` - The ticket being accessed
/// * `permission` - The type of access being requested
///
/// # Returns
/// * `Ok(())` if access is allowed
/// * `Err(AppError::Forbidden)` if access is denied
pub fn require_ticket_access(
    employee: &Employee,
    ticket: &Ticket,
    permission: Permission,
) -> Result<(), AppError> {
    // Admin always has full access
    if employee.role == EmployeeRole::Admin {
        return Ok(());
    }

    // For non-admins, check if they have the base permission first
    if !employee.role.has_permission(permission) {
        return Err(AppError::forbidden(
            "You do not have permission to perform this action",
        ));
    }

    // For modify operations, check ownership
    if matches!(permission, Permission::ModifyOwnTicket) {
        if is_ticket_owner(employee, ticket) {
            return Ok(());
        }
        // Staff can't modify tickets they don't own
        return Err(AppError::forbidden(
            "You do not have permission to modify this ticket",
        ));
    }

    // For other permissions (ViewTicket, AddNotes, UploadPhotos),
    // staff can access any ticket they're authorized for
    Ok(())
}

/// Check if an employee is an owner of a ticket.
///
/// An employee is considered an owner if:
/// - They took in the ticket (taken_in_by)
/// - They are assigned to work on the ticket (worked_by)
pub fn is_ticket_owner(employee: &Employee, ticket: &Ticket) -> bool {
    ticket.taken_in_by == employee.employee_id || ticket.worked_by == Some(employee.employee_id)
}

/// Check if an employee can close a specific ticket.
///
/// Only admins can close tickets. Staff cannot close any ticket,
/// even ones they own.
pub fn can_close_ticket(employee: &Employee) -> Result<(), AppError> {
    if employee.role == EmployeeRole::Admin {
        Ok(())
    } else {
        Err(AppError::forbidden("Only administrators can close tickets"))
    }
}

/// Check if an employee can delete photos.
///
/// Only admins can delete photos. Staff cannot delete any photos.
pub fn can_delete_photo(employee: &Employee) -> Result<(), AppError> {
    if employee.role.has_permission(Permission::DeletePhotos) {
        Ok(())
    } else {
        Err(AppError::forbidden("Only administrators can delete photos"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::TicketStatus;
    use chrono::Utc;
    use uuid::Uuid;

    fn create_test_employee(role: EmployeeRole) -> Employee {
        Employee {
            employee_id: Uuid::new_v4(),
            name: "Test Employee".to_string(),
            pin_hash: "hash".to_string(),
            role,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_ticket(taken_in_by: Uuid, worked_by: Option<Uuid>) -> Ticket {
        Ticket {
            ticket_id: Uuid::new_v4(),
            friendly_code: "JR-TEST".to_string(),
            customer_id: Uuid::new_v4(),
            status: TicketStatus::Intake,
            item_type: None,
            item_description: "Test item".to_string(),
            condition_notes: "Test notes".to_string(),
            requested_work: "Test work".to_string(),
            is_rush: false,
            promise_date: None,
            storage_location_id: Uuid::new_v4(),
            quote_amount: None,
            actual_amount: None,
            taken_in_by,
            worked_by,
            closed_by: None,
            last_modified_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
            queue_position: None,
        }
    }

    #[test]
    fn test_require_permission_admin_succeeds() {
        let admin = create_test_employee(EmployeeRole::Admin);
        assert!(require_permission(&admin, Permission::DeletePhotos).is_ok());
        assert!(require_permission(&admin, Permission::CloseAnyTicket).is_ok());
        assert!(require_permission(&admin, Permission::ManageEmployees).is_ok());
    }

    #[test]
    fn test_require_permission_staff_limited() {
        let staff = create_test_employee(EmployeeRole::Staff);
        assert!(require_permission(&staff, Permission::CreateTicket).is_ok());
        assert!(require_permission(&staff, Permission::ViewTicket).is_ok());
        assert!(require_permission(&staff, Permission::DeletePhotos).is_err());
        assert!(require_permission(&staff, Permission::CloseAnyTicket).is_err());
    }

    #[test]
    fn test_require_ticket_access_admin_always_allowed() {
        let admin = create_test_employee(EmployeeRole::Admin);
        let other_employee_id = Uuid::new_v4();
        let ticket = create_test_ticket(other_employee_id, None);

        // Admin can modify any ticket
        assert!(require_ticket_access(&admin, &ticket, Permission::ModifyOwnTicket).is_ok());
    }

    #[test]
    fn test_require_ticket_access_staff_can_modify_own() {
        let staff = create_test_employee(EmployeeRole::Staff);
        let ticket = create_test_ticket(staff.employee_id, None);

        // Staff can modify their own ticket
        assert!(require_ticket_access(&staff, &ticket, Permission::ModifyOwnTicket).is_ok());
    }

    #[test]
    fn test_require_ticket_access_staff_can_modify_assigned() {
        let staff = create_test_employee(EmployeeRole::Staff);
        let other_employee_id = Uuid::new_v4();
        let ticket = create_test_ticket(other_employee_id, Some(staff.employee_id));

        // Staff can modify ticket assigned to them
        assert!(require_ticket_access(&staff, &ticket, Permission::ModifyOwnTicket).is_ok());
    }

    #[test]
    fn test_require_ticket_access_staff_cannot_modify_others() {
        let staff = create_test_employee(EmployeeRole::Staff);
        let other_employee_id = Uuid::new_v4();
        let ticket = create_test_ticket(other_employee_id, None);

        // Staff cannot modify someone else's ticket
        assert!(require_ticket_access(&staff, &ticket, Permission::ModifyOwnTicket).is_err());
    }

    #[test]
    fn test_is_ticket_owner_taken_in_by() {
        let employee = create_test_employee(EmployeeRole::Staff);
        let ticket = create_test_ticket(employee.employee_id, None);

        assert!(is_ticket_owner(&employee, &ticket));
    }

    #[test]
    fn test_is_ticket_owner_worked_by() {
        let employee = create_test_employee(EmployeeRole::Staff);
        let other_id = Uuid::new_v4();
        let ticket = create_test_ticket(other_id, Some(employee.employee_id));

        assert!(is_ticket_owner(&employee, &ticket));
    }

    #[test]
    fn test_is_ticket_owner_neither() {
        let employee = create_test_employee(EmployeeRole::Staff);
        let other_id = Uuid::new_v4();
        let ticket = create_test_ticket(other_id, None);

        assert!(!is_ticket_owner(&employee, &ticket));
    }

    #[test]
    fn test_can_close_ticket_admin() {
        let admin = create_test_employee(EmployeeRole::Admin);
        assert!(can_close_ticket(&admin).is_ok());
    }

    #[test]
    fn test_can_close_ticket_staff_denied() {
        let staff = create_test_employee(EmployeeRole::Staff);
        assert!(can_close_ticket(&staff).is_err());
    }

    #[test]
    fn test_can_delete_photo_admin() {
        let admin = create_test_employee(EmployeeRole::Admin);
        assert!(can_delete_photo(&admin).is_ok());
    }

    #[test]
    fn test_can_delete_photo_staff_denied() {
        let staff = create_test_employee(EmployeeRole::Staff);
        assert!(can_delete_photo(&staff).is_err());
    }
}

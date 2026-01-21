//! Reference validation for foreign key relationships.
//!
//! Provides consistent validation for referenced entities (storage locations, employees,
//! customers) before database operations. This ensures clear 404 errors instead of
//! cryptic foreign key constraint violations.

use crate::error::AppError;
use crate::repositories::{CustomerRepository, EmployeeRepository, StorageLocationRepository};
use sqlx::PgPool;
use uuid::Uuid;

/// Validate that a storage location exists and is active.
///
/// Returns an error if the storage location does not exist or is inactive.
pub async fn validate_storage_location(pool: &PgPool, location_id: Uuid) -> Result<(), AppError> {
    if !StorageLocationRepository::exists_active(pool, location_id).await? {
        return Err(AppError::not_found(
            "Storage location not found or inactive",
        ));
    }
    Ok(())
}

/// Validate that an employee exists and is active.
///
/// Returns an error if the employee does not exist or is inactive.
pub async fn validate_employee(pool: &PgPool, employee_id: Uuid) -> Result<(), AppError> {
    if !EmployeeRepository::exists_active(pool, employee_id).await? {
        return Err(AppError::not_found("Employee not found or inactive"));
    }
    Ok(())
}

/// Validate that a customer exists.
///
/// Returns an error if the customer does not exist.
pub async fn validate_customer(pool: &PgPool, customer_id: Uuid) -> Result<(), AppError> {
    if !CustomerRepository::exists(pool, customer_id).await? {
        return Err(AppError::not_found("Customer not found"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    // Unit tests would require mocking the database connection
    // Integration tests are more appropriate and are in apps/api/tests/

    #[test]
    fn test_module_compiles() {
        // Basic sanity check that the module is properly structured
        assert!(true);
    }
}

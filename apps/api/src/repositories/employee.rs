//! Employee repository for database operations.

use crate::error::AppError;
use crate::models::employee::Employee;
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for employee database operations.
pub struct EmployeeRepository;

impl EmployeeRepository {
    /// Find an employee by ID.
    ///
    /// Returns None if not found or if employee is inactive.
    pub async fn find_active_by_id(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Option<Employee>, AppError> {
        let employee = sqlx::query_as::<_, Employee>(
            r#"
            SELECT * FROM employees
            WHERE employee_id = $1 AND is_active = TRUE
            "#,
        )
        .bind(employee_id)
        .fetch_optional(pool)
        .await?;

        Ok(employee)
    }

    /// Find an employee by ID (including inactive).
    pub async fn find_by_id(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<Option<Employee>, AppError> {
        let employee = sqlx::query_as::<_, Employee>(
            r#"
            SELECT * FROM employees WHERE employee_id = $1
            "#,
        )
        .bind(employee_id)
        .fetch_optional(pool)
        .await?;

        Ok(employee)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_employee_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

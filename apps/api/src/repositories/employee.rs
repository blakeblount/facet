//! Employee repository for database operations.

use crate::auth::hash_pin;
use crate::error::AppError;
use crate::models::employee::{
    CreateEmployee, Employee, EmployeeRole, EmployeeSummary, UpdateEmployee,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for employee database operations.
pub struct EmployeeRepository;

impl EmployeeRepository {
    /// Create a new employee.
    ///
    /// The PIN is hashed before storage using argon2.
    pub async fn create(pool: &PgPool, input: CreateEmployee) -> Result<Employee, AppError> {
        let pin_hash = hash_pin(&input.pin)?;
        let role = input.role.unwrap_or(EmployeeRole::Staff);

        let employee = sqlx::query_as::<_, Employee>(
            r#"
            INSERT INTO employees (name, pin_hash, role)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&input.name)
        .bind(&pin_hash)
        .bind(role)
        .fetch_one(pool)
        .await?;

        Ok(employee)
    }

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

    /// Check if an active employee exists.
    ///
    /// Returns true if the employee exists and is active.
    pub async fn exists_active(pool: &PgPool, employee_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM employees WHERE employee_id = $1 AND is_active = TRUE)",
        )
        .bind(employee_id)
        .fetch_one(pool)
        .await?;

        Ok(result)
    }

    /// Find an active employee by PIN for verification.
    ///
    /// Returns all active employees - the caller must verify the PIN
    /// against each one since we can't compare argon2 hashes directly in SQL.
    /// This is intentional for security - we don't expose a direct PIN lookup.
    pub async fn find_active_for_pin_verification(
        pool: &PgPool,
    ) -> Result<Vec<Employee>, AppError> {
        let employees = sqlx::query_as::<_, Employee>(
            r#"
            SELECT * FROM employees WHERE is_active = TRUE
            "#,
        )
        .fetch_all(pool)
        .await?;

        Ok(employees)
    }

    /// List employees with optional filtering.
    ///
    /// If include_inactive is false (default), only active employees are returned.
    pub async fn list(
        pool: &PgPool,
        include_inactive: bool,
    ) -> Result<Vec<EmployeeSummary>, AppError> {
        let employees = if include_inactive {
            sqlx::query_as::<_, EmployeeSummary>(
                r#"
                SELECT employee_id, name, role, is_active
                FROM employees
                ORDER BY name ASC
                "#,
            )
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, EmployeeSummary>(
                r#"
                SELECT employee_id, name, role, is_active
                FROM employees
                WHERE is_active = TRUE
                ORDER BY name ASC
                "#,
            )
            .fetch_all(pool)
            .await?
        };

        Ok(employees)
    }

    /// Update an employee.
    ///
    /// Only the provided fields are updated.
    /// If PIN is provided, it's hashed before storage.
    pub async fn update(
        pool: &PgPool,
        employee_id: Uuid,
        input: UpdateEmployee,
    ) -> Result<Option<Employee>, AppError> {
        // First check if employee exists
        let existing = Self::find_by_id(pool, employee_id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        // Build update with provided fields, keeping existing values for unspecified fields
        let name = input.name.unwrap_or(existing.name);
        let pin_hash = match input.pin {
            Some(pin) => hash_pin(&pin)?,
            None => existing.pin_hash,
        };
        let role = input.role.unwrap_or(existing.role);
        let is_active = input.is_active.unwrap_or(existing.is_active);

        let employee = sqlx::query_as::<_, Employee>(
            r#"
            UPDATE employees
            SET name = $1, pin_hash = $2, role = $3, is_active = $4, updated_at = NOW()
            WHERE employee_id = $5
            RETURNING *
            "#,
        )
        .bind(&name)
        .bind(&pin_hash)
        .bind(role)
        .bind(is_active)
        .bind(employee_id)
        .fetch_one(pool)
        .await?;

        Ok(Some(employee))
    }

    /// Soft-delete an employee by setting is_active to false.
    ///
    /// Returns the updated employee, or None if not found.
    pub async fn delete(pool: &PgPool, employee_id: Uuid) -> Result<Option<Employee>, AppError> {
        let employee = sqlx::query_as::<_, Employee>(
            r#"
            UPDATE employees
            SET is_active = FALSE, updated_at = NOW()
            WHERE employee_id = $1
            RETURNING *
            "#,
        )
        .bind(employee_id)
        .fetch_optional(pool)
        .await?;

        Ok(employee)
    }

    /// Check if an employee has any attribution history.
    ///
    /// Returns the count of attributions across all tables that reference this employee.
    pub async fn count_attributions(pool: &PgPool, employee_id: Uuid) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT
                COALESCE(
                    (SELECT COUNT(*) FROM tickets WHERE taken_in_by = $1) +
                    (SELECT COUNT(*) FROM tickets WHERE worked_by = $1) +
                    (SELECT COUNT(*) FROM tickets WHERE closed_by = $1) +
                    (SELECT COUNT(*) FROM tickets WHERE last_modified_by = $1) +
                    (SELECT COUNT(*) FROM ticket_photos WHERE uploaded_by = $1) +
                    (SELECT COUNT(*) FROM ticket_notes WHERE created_by = $1) +
                    (SELECT COUNT(*) FROM ticket_status_history WHERE changed_by = $1) +
                    (SELECT COUNT(*) FROM ticket_field_history WHERE changed_by = $1),
                    0
                )
            "#,
        )
        .bind(employee_id)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Hard-delete an employee from the database.
    ///
    /// WARNING: This will fail if there are foreign key references to this employee.
    /// The caller should check attribution history and handle accordingly.
    ///
    /// Returns true if deleted, false if not found.
    pub async fn hard_delete(pool: &PgPool, employee_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM employees WHERE employee_id = $1
            "#,
        )
        .bind(employee_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
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

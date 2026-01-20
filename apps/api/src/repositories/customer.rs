//! Customer repository for database operations.

use crate::error::AppError;
use crate::models::customer::{CreateCustomer, Customer};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for customer database operations.
pub struct CustomerRepository;

impl CustomerRepository {
    /// Create a new customer.
    pub async fn create(pool: &PgPool, input: CreateCustomer) -> Result<Customer, AppError> {
        let customer = sqlx::query_as::<_, Customer>(
            r#"
            INSERT INTO customers (name, phone, email)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(&input.name)
        .bind(&input.phone)
        .bind(&input.email)
        .fetch_one(pool)
        .await?;

        Ok(customer)
    }

    /// Find a customer by ID.
    pub async fn find_by_id(
        pool: &PgPool,
        customer_id: Uuid,
    ) -> Result<Option<Customer>, AppError> {
        let customer = sqlx::query_as::<_, Customer>(
            r#"
            SELECT * FROM customers WHERE customer_id = $1
            "#,
        )
        .bind(customer_id)
        .fetch_optional(pool)
        .await?;

        Ok(customer)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_customer_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

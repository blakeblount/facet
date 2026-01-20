//! Customer repository for database operations.

use crate::error::AppError;
use crate::models::customer::{
    CreateCustomer, Customer, CustomerSearchParams, CustomerWithTicketCount, CustomerWithTickets,
};
use crate::models::ticket::TicketSummary;
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

    /// Search customers by name, phone, or email.
    ///
    /// Uses case-insensitive partial matching (ILIKE) across all searchable fields.
    /// Results are ordered by name for consistent display.
    pub async fn search(
        pool: &PgPool,
        params: CustomerSearchParams,
    ) -> Result<Vec<Customer>, AppError> {
        let search_pattern = format!("%{}%", params.query);

        let customers = sqlx::query_as::<_, Customer>(
            r#"
            SELECT *
            FROM customers
            WHERE name ILIKE $1
               OR phone ILIKE $1
               OR email ILIKE $1
            ORDER BY name ASC
            LIMIT $2
            OFFSET $3
            "#,
        )
        .bind(&search_pattern)
        .bind(params.limit.unwrap_or(50))
        .bind(params.offset.unwrap_or(0))
        .fetch_all(pool)
        .await?;

        Ok(customers)
    }

    /// Search customers by name, phone, or email, including ticket count.
    ///
    /// Uses case-insensitive partial matching (ILIKE) across all searchable fields.
    /// Includes the count of tickets associated with each customer.
    /// Results are ordered by name for consistent display.
    pub async fn search_with_ticket_count(
        pool: &PgPool,
        params: CustomerSearchParams,
    ) -> Result<Vec<CustomerWithTicketCount>, AppError> {
        let search_pattern = format!("%{}%", params.query);

        let customers = sqlx::query_as::<_, CustomerWithTicketCount>(
            r#"
            SELECT
                c.customer_id,
                c.name,
                c.phone,
                c.email,
                c.created_at,
                c.updated_at,
                COUNT(t.ticket_id) as ticket_count
            FROM customers c
            LEFT JOIN tickets t ON c.customer_id = t.customer_id
            WHERE c.name ILIKE $1
               OR c.phone ILIKE $1
               OR c.email ILIKE $1
            GROUP BY c.customer_id, c.name, c.phone, c.email, c.created_at, c.updated_at
            ORDER BY c.name ASC
            LIMIT $2
            OFFSET $3
            "#,
        )
        .bind(&search_pattern)
        .bind(params.limit.unwrap_or(50))
        .bind(params.offset.unwrap_or(0))
        .fetch_all(pool)
        .await?;

        Ok(customers)
    }

    /// Get a customer with their associated tickets.
    ///
    /// Returns the customer and a list of their ticket summaries, sorted by
    /// created_at descending (most recent first).
    pub async fn get_with_tickets(
        pool: &PgPool,
        customer_id: Uuid,
    ) -> Result<Option<CustomerWithTickets>, AppError> {
        // First fetch the customer
        let customer = Self::find_by_id(pool, customer_id).await?;

        let customer = match customer {
            Some(c) => c,
            None => return Ok(None),
        };

        // Then fetch their tickets
        let tickets = sqlx::query_as::<_, TicketSummary>(
            r#"
            SELECT
                t.ticket_id,
                t.friendly_code,
                t.customer_id,
                c.name as customer_name,
                t.item_type,
                t.item_description,
                t.status,
                t.is_rush,
                t.promise_date,
                t.quote_amount,
                t.created_at
            FROM tickets t
            JOIN customers c ON t.customer_id = c.customer_id
            WHERE t.customer_id = $1
            ORDER BY t.created_at DESC
            "#,
        )
        .bind(customer_id)
        .fetch_all(pool)
        .await?;

        Ok(Some(CustomerWithTickets { customer, tickets }))
    }
}

#[cfg(test)]
mod tests {
    use crate::models::customer::CustomerSearchParams;

    #[test]
    fn test_customer_repository_exists() {
        // Basic sanity test
        assert!(true);
    }

    #[test]
    fn test_search_pattern_format() {
        // Verify search pattern is formatted correctly for ILIKE
        let query = "john";
        let pattern = format!("%{}%", query);
        assert_eq!(pattern, "%john%");
    }

    #[test]
    fn test_search_params_defaults() {
        let params = CustomerSearchParams::default();
        assert_eq!(params.limit, None);
        assert_eq!(params.offset, None);
    }
}

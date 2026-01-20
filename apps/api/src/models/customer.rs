//! Customer model and related types.
//!
//! Customers are the people who bring in items for repair.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full customer entity with all fields.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Customer {
    pub customer_id: Uuid,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new customer.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateCustomer {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Search parameters for customer lookup.
#[derive(Debug, Clone, Default)]
pub struct CustomerSearchParams {
    /// Search query (matches name, phone, or email)
    pub query: String,
    /// Maximum results to return
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Customer with their associated tickets.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerWithTickets {
    #[serde(flatten)]
    pub customer: Customer,
    pub tickets: Vec<crate::models::ticket::TicketSummary>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_customer_deserialize() {
        let json = r#"{"name": "John Doe", "phone": "555-1234", "email": null}"#;
        let customer: CreateCustomer = serde_json::from_str(json).unwrap();
        assert_eq!(customer.name, "John Doe");
        assert_eq!(customer.phone, Some("555-1234".to_string()));
        assert!(customer.email.is_none());
    }

    #[test]
    fn test_customer_search_params_default() {
        let params = CustomerSearchParams::default();
        assert!(params.query.is_empty());
        assert!(params.limit.is_none());
        assert!(params.offset.is_none());
    }

    #[test]
    fn test_customer_search_params() {
        let params = CustomerSearchParams {
            query: "john".to_string(),
            limit: Some(10),
            offset: Some(20),
        };
        assert_eq!(params.query, "john");
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.offset, Some(20));
    }
}

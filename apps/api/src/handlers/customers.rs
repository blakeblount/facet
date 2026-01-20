//! Customer request handlers.

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;

use crate::error::AppError;
use crate::models::customer::CustomerSearchParams;
use crate::repositories::CustomerRepository;
use crate::response::ApiResponse;
use crate::routes::AppState;

// =============================================================================
// GET /customers - Search Customers
// =============================================================================

/// Query parameters for customer search.
#[derive(Debug, Clone, Deserialize)]
pub struct CustomerSearchQuery {
    /// Search query (matches name, phone, or email)
    pub search: Option<String>,
    /// Maximum results to return (default: 50)
    pub limit: Option<i64>,
    /// Offset for pagination (default: 0)
    pub offset: Option<i64>,
}

/// GET /api/v1/customers - Search customers for autocomplete.
///
/// Searches customers by name, phone, or email using case-insensitive
/// partial matching. Returns customers with their ticket counts.
///
/// # Query Parameters
/// - `search`: Search query to match against name, phone, or email
/// - `limit`: Maximum number of results (default: 50)
/// - `offset`: Offset for pagination (default: 0)
///
/// # Returns
/// List of matching customers with ticket_count included.
pub async fn search_customers(
    State(state): State<AppState>,
    Query(query): Query<CustomerSearchQuery>,
) -> Result<impl IntoResponse, AppError> {
    let search_query = query.search.unwrap_or_default();

    let params = CustomerSearchParams {
        query: search_query,
        limit: query.limit,
        offset: query.offset,
    };

    let customers = CustomerRepository::search_with_ticket_count(&state.db, params).await?;

    Ok(Json(ApiResponse::success(customers)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customer_search_query_deserialize_empty() {
        let json = r#"{}"#;
        let query: CustomerSearchQuery = serde_json::from_str(json).unwrap();
        assert!(query.search.is_none());
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
    }

    #[test]
    fn test_customer_search_query_deserialize_with_search() {
        let json = r#"{"search": "john"}"#;
        let query: CustomerSearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.search, Some("john".to_string()));
    }

    #[test]
    fn test_customer_search_query_deserialize_with_pagination() {
        let json = r#"{"search": "jane", "limit": 10, "offset": 20}"#;
        let query: CustomerSearchQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.search, Some("jane".to_string()));
        assert_eq!(query.limit, Some(10));
        assert_eq!(query.offset, Some(20));
    }

    #[test]
    fn test_customer_search_query_deserialize_limit_only() {
        let json = r#"{"limit": 25}"#;
        let query: CustomerSearchQuery = serde_json::from_str(json).unwrap();
        assert!(query.search.is_none());
        assert_eq!(query.limit, Some(25));
    }
}

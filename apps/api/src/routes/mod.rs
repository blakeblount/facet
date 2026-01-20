//! API route modules.
//!
//! Routes are organized by domain:
//! - `/health` - Health check endpoint
//! - `/api/v1/tickets` - Ticket management
//! - `/api/v1/customers` - Customer management
//! - `/api/v1/employees` - Employee management
//! - `/api/v1/locations` - Storage location management
//! - `/api/v1/queue` - Workboard queue
//! - `/api/v1/settings` - Store settings
//! - `/api/v1/admin` - Admin operations

mod health;

use axum::Router;

pub use health::health_check;

/// Build the API router with all routes.
pub fn api_router() -> Router {
    // API v1 routes - will be expanded as handlers are implemented
    let api_v1 = Router::new();
    // Future routes:
    // .nest("/tickets", tickets::router())
    // .nest("/customers", customers::router())
    // .nest("/employees", employees::router())
    // .nest("/locations", locations::router())
    // .nest("/queue", queue::router())
    // .nest("/settings", settings::router())
    // .nest("/admin", admin::router())

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/v1", api_v1)
}

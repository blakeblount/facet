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

use axum::{routing::post, Router};
use sqlx::postgres::PgPool;

use crate::handlers;

pub use health::health_check;

/// Application state shared across all handlers.
///
/// Contains the database connection pool and other shared resources.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db: PgPool,
}

impl AppState {
    /// Create a new AppState with the given database pool.
    pub fn new(db: PgPool) -> Self {
        Self { db }
    }
}

/// Build the API router with all routes.
///
/// The router is configured with shared application state containing
/// the database pool.
pub fn api_router(state: AppState) -> Router {
    // Ticket routes
    let tickets_routes = Router::new().route("/", post(handlers::create_ticket));

    // API v1 routes
    let api_v1 = Router::new().nest("/tickets", tickets_routes);

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/v1", api_v1)
        .with_state(state)
}

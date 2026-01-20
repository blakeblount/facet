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

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPool;

use crate::handlers;

pub use health::health_check;

use crate::storage::StorageClient;

/// Application state shared across all handlers.
///
/// Contains the database connection pool and other shared resources.
#[derive(Clone)]
pub struct AppState {
    /// PostgreSQL connection pool
    pub db: PgPool,
    /// S3-compatible storage client for photo uploads (optional for dev environments)
    pub storage: Option<StorageClient>,
}

impl AppState {
    /// Create a new AppState with the given database pool.
    pub fn new(db: PgPool) -> Self {
        Self { db, storage: None }
    }

    /// Create a new AppState with database pool and storage client.
    pub fn with_storage(db: PgPool, storage: StorageClient) -> Self {
        Self {
            db,
            storage: Some(storage),
        }
    }
}

/// Build the API router with all routes.
///
/// The router is configured with shared application state containing
/// the database pool.
pub fn api_router(state: AppState) -> Router {
    // Ticket routes
    let tickets_routes = Router::new()
        .route(
            "/",
            get(handlers::list_tickets).post(handlers::create_ticket),
        )
        .route(
            "/{ticket_id}",
            get(handlers::get_ticket).put(handlers::update_ticket),
        )
        .route("/{ticket_id}/receipt.pdf", get(handlers::get_receipt_pdf))
        .route("/{ticket_id}/label.pdf", get(handlers::get_label_pdf))
        .route("/{ticket_id}/status", post(handlers::change_status))
        .route("/{ticket_id}/close", post(handlers::close_ticket))
        .route("/{ticket_id}/rush", post(handlers::toggle_rush))
        .route("/{ticket_id}/notes", post(handlers::add_note))
        .route("/{ticket_id}/photos", post(handlers::upload_photo))
        .route(
            "/{ticket_id}/photos/{photo_id}",
            delete(handlers::delete_photo),
        );

    // Queue route
    let queue_route = Router::new().route("/", get(handlers::get_queue));

    // Employee routes
    let employees_routes = Router::new()
        .route("/", post(handlers::create_employee))
        .route(
            "/{employee_id}",
            put(handlers::update_employee).delete(handlers::delete_employee),
        )
        .route("/verify", post(handlers::verify_employee_pin));

    // Customer routes
    let customers_routes = Router::new()
        .route("/", get(handlers::search_customers))
        .route("/{customer_id}", get(handlers::get_customer));

    // Admin routes
    let admin_routes = Router::new()
        .route("/setup", post(handlers::admin_setup))
        .route("/verify", post(handlers::verify_admin))
        .route("/change-pin", post(handlers::change_pin));

    // API v1 routes
    let api_v1 = Router::new()
        .nest("/tickets", tickets_routes)
        .nest("/queue", queue_route)
        .nest("/employees", employees_routes)
        .nest("/customers", customers_routes)
        .nest("/admin", admin_routes);

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/v1", api_v1)
        .with_state(state)
}

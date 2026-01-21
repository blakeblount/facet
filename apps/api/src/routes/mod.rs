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
    middleware,
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPool;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::services::ServeDir;

use crate::config::{DEFAULT_MAX_BODY_SIZE, DEFAULT_MAX_PHOTO_SIZE};
use crate::handlers;
use crate::middleware::{json_payload_error, RateLimitState};

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
    /// Rate limiter state for PIN verification endpoints
    pub rate_limit: RateLimitState,
}

impl AppState {
    /// Create a new AppState with the given database pool.
    pub fn new(db: PgPool) -> Self {
        Self {
            db,
            storage: None,
            rate_limit: RateLimitState::new(),
        }
    }

    /// Create a new AppState with database pool and storage client.
    pub fn with_storage(db: PgPool, storage: StorageClient) -> Self {
        Self {
            db,
            storage: Some(storage),
            rate_limit: RateLimitState::new(),
        }
    }
}

/// Configuration for request body size limits.
#[derive(Debug, Clone)]
pub struct BodyLimitConfig {
    /// Maximum body size for JSON endpoints (default: 1MB)
    pub max_body_size: usize,
    /// Maximum body size for photo uploads (default: 10MB)
    pub max_photo_size: usize,
}

impl Default for BodyLimitConfig {
    fn default() -> Self {
        Self {
            max_body_size: DEFAULT_MAX_BODY_SIZE,
            max_photo_size: DEFAULT_MAX_PHOTO_SIZE,
        }
    }
}

/// Build the API router with all routes.
///
/// The router is configured with shared application state containing
/// the database pool.
pub fn api_router(state: AppState) -> Router {
    api_router_with_limits(state, BodyLimitConfig::default())
}

/// Build the API router with custom body size limits.
///
/// The router is configured with shared application state and custom
/// request body size limits.
pub fn api_router_with_limits(state: AppState, limits: BodyLimitConfig) -> Router {
    // Photo upload route with larger limit
    let photo_upload_route = Router::new()
        .route("/", post(handlers::upload_photo))
        .layer(RequestBodyLimitLayer::new(limits.max_photo_size));

    // Ticket routes (without photo upload, which has its own limit)
    let tickets_routes = Router::new()
        .route(
            "/",
            get(handlers::list_tickets).post(handlers::create_ticket),
        )
        .route(
            "/:ticket_id",
            get(handlers::get_ticket).put(handlers::update_ticket),
        )
        .route("/:ticket_id/receipt.pdf", get(handlers::get_receipt_pdf))
        .route("/:ticket_id/label.pdf", get(handlers::get_label_pdf))
        .route("/:ticket_id/status", post(handlers::change_status))
        .route("/:ticket_id/close", post(handlers::close_ticket))
        .route("/:ticket_id/rush", post(handlers::toggle_rush))
        .route("/:ticket_id/notes", post(handlers::add_note))
        .nest("/:ticket_id/photos", photo_upload_route)
        .route(
            "/:ticket_id/photos/:photo_id",
            delete(handlers::delete_photo),
        );

    // Queue route
    let queue_route = Router::new().route("/", get(handlers::get_queue));

    // Employee routes
    let employees_routes = Router::new()
        .route(
            "/",
            get(handlers::list_employees).post(handlers::create_employee),
        )
        .route(
            "/:employee_id",
            put(handlers::update_employee).delete(handlers::delete_employee),
        )
        .route("/verify", post(handlers::verify_employee_pin));

    // Customer routes
    let customers_routes = Router::new()
        .route("/", get(handlers::search_customers))
        .route("/:customer_id", get(handlers::get_customer));

    // Admin routes
    let admin_routes = Router::new()
        .route("/setup", post(handlers::admin_setup))
        .route("/verify", post(handlers::verify_admin))
        .route("/change-pin", post(handlers::change_pin))
        .route("/logout", post(handlers::admin_logout));

    // Settings routes
    let settings_routes = Router::new().route(
        "/",
        get(handlers::get_settings).put(handlers::update_settings),
    );

    // Storage location routes
    let locations_routes = Router::new()
        .route(
            "/",
            get(handlers::list_locations).post(handlers::create_location),
        )
        .route("/:location_id", put(handlers::update_location));

    // API v1 routes with default body limit
    let api_v1 = Router::new()
        .nest("/tickets", tickets_routes)
        .nest("/queue", queue_route)
        .nest("/employees", employees_routes)
        .nest("/customers", customers_routes)
        .nest("/admin", admin_routes)
        .nest("/settings", settings_routes)
        .nest("/locations", locations_routes)
        // Apply default body size limit to all API routes (except photo upload which has its own)
        .layer(RequestBodyLimitLayer::new(limits.max_body_size))
        // Convert 413 responses to JSON format
        .layer(middleware::from_fn(json_payload_error));

    Router::new()
        .route("/health", axum::routing::get(health::health_check))
        .nest("/api/v1", api_v1)
        // Serve uploaded files from local storage (dev only)
        .nest_service("/uploads", ServeDir::new("uploads"))
        .with_state(state)
}

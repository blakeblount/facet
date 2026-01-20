//! Facet API - Jewelry repair intake and ticketing system backend
//!
//! This crate provides the REST API for the Facet application.

pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod models;
pub mod response;
pub mod routes;
pub mod services;
pub mod storage;

pub use config::Config;
pub use db::{create_pool, test_connection, DbConfig};
pub use error::{codes as error_codes, AppError};
pub use response::{created, empty, no_content, ok, ApiResponse, ApiResult};
pub use routes::{api_router, AppState};
pub use storage::{StorageClient, StorageConfig, StorageError, StorageResult};

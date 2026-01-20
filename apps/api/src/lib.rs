//! Facet API - Jewelry repair intake and ticketing system backend
//!
//! This crate provides the REST API for the Facet application.

pub mod config;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod services;

pub use config::Config;
pub use routes::api_router;

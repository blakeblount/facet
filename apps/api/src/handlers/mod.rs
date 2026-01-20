//! HTTP request handlers.
//!
//! Handlers process incoming requests and return responses.
//! Business logic is delegated to services.

pub mod tickets;

pub use tickets::{create_ticket, get_receipt_pdf, update_ticket};

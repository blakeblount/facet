//! HTTP request handlers.
//!
//! Handlers process incoming requests and return responses.
//! Business logic is delegated to services.

pub mod tickets;

pub use tickets::{
    create_ticket, get_label_pdf, get_queue, get_receipt_pdf, get_ticket, list_tickets,
    update_ticket,
};

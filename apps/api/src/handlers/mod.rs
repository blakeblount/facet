//! HTTP request handlers.
//!
//! Handlers process incoming requests and return responses.
//! Business logic is delegated to services.

pub mod admin;
pub mod customers;
pub mod employees;
pub mod tickets;

pub use admin::{admin_setup, change_pin, verify_admin};
pub use customers::{get_customer, search_customers};
pub use employees::{create_employee, delete_employee, update_employee, verify_employee_pin};
pub use tickets::{
    add_note, change_status, close_ticket, create_ticket, delete_photo, get_label_pdf, get_queue,
    get_receipt_pdf, get_ticket, list_tickets, toggle_rush, update_ticket, upload_photo,
};

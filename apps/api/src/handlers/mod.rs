//! HTTP request handlers.
//!
//! Handlers process incoming requests and return responses.
//! Business logic is delegated to services.

pub mod admin;
pub mod customers;
pub mod employees;
pub mod locations;
pub mod settings;
pub mod tickets;

pub use admin::{admin_logout, admin_setup, change_pin, verify_admin, verify_admin_auth};
pub use customers::{get_customer, search_customers};
pub use employees::{
    create_employee, delete_employee, list_employees, update_employee, verify_employee_pin,
};
pub use locations::{create_location, list_locations, update_location};
pub use settings::{get_settings, update_settings};
pub use tickets::{
    add_note, change_status, close_ticket, create_ticket, delete_photo, get_label_pdf, get_queue,
    get_receipt_pdf, get_ticket, list_tickets, toggle_rush, update_ticket, upload_photo,
};

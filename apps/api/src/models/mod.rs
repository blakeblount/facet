//! Domain models and database entities.
//!
//! Models represent the core business entities used throughout the application.

pub mod ticket;

pub use ticket::{CreateTicket, Ticket, TicketFilters, TicketStatus, TicketSummary, UpdateTicket};

// Future model modules:
// pub mod customer;
// pub mod employee;
// pub mod location;
// pub mod photo;
// pub mod audit;

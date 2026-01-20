//! Domain models and database entities.
//!
//! Models represent the core business entities used throughout the application.

pub mod customer;
pub mod employee;
pub mod status_history;
pub mod ticket;

pub use customer::{CreateCustomer, Customer};
pub use employee::{Employee, EmployeeRole, EmployeeSummary};
pub use status_history::{CreateStatusHistory, StatusHistoryEntry};
pub use ticket::{
    CreateTicket, QueueTicket, Ticket, TicketFilters, TicketSearchParams, TicketStatus,
    TicketSummary, UpdateTicket, WorkboardQueue,
};

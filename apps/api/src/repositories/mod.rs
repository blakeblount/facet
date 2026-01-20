//! Data access layer (repositories).
//!
//! Repositories handle database operations and provide a clean interface
//! for data access. Each repository is responsible for a specific domain entity.

pub mod customer;
pub mod employee;
pub mod status_history;
pub mod ticket;

pub use customer::CustomerRepository;
pub use employee::EmployeeRepository;
pub use status_history::StatusHistoryRepository;
pub use ticket::TicketRepository;

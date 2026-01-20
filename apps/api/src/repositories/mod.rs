//! Data access layer (repositories).
//!
//! Repositories handle database operations and provide a clean interface
//! for data access. Each repository is responsible for a specific domain entity.

pub mod ticket;

pub use ticket::TicketRepository;

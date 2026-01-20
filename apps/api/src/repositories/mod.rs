//! Data access layer (repositories).
//!
//! Repositories handle database operations and provide a clean interface
//! for data access. Each repository is responsible for a specific domain entity.

pub mod customer;
pub mod employee;
pub mod field_history;
pub mod status_history;
pub mod storage_location;
pub mod store_settings;
pub mod ticket;
pub mod ticket_note;
pub mod ticket_photo;

pub use customer::CustomerRepository;
pub use employee::EmployeeRepository;
pub use field_history::FieldHistoryRepository;
pub use status_history::StatusHistoryRepository;
pub use storage_location::StorageLocationRepository;
pub use store_settings::StoreSettingsRepository;
pub use ticket::TicketRepository;
pub use ticket_note::TicketNoteRepository;
pub use ticket_photo::TicketPhotoRepository;

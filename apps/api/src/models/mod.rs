//! Domain models and database entities.
//!
//! Models represent the core business entities used throughout the application.

pub mod customer;
pub mod employee;
pub mod field_history;
pub mod status_history;
pub mod storage_location;
pub mod ticket;
pub mod ticket_photo;

pub use customer::{CreateCustomer, Customer};
pub use employee::{CreateEmployee, Employee, EmployeeRole, EmployeeSummary, UpdateEmployee};
pub use field_history::{CreateFieldHistory, FieldHistoryEntry};
pub use status_history::{CreateStatusHistory, StatusHistoryEntry};
pub use storage_location::{
    CreateStorageLocation, StorageLocation, StorageLocationSummary, UpdateStorageLocation,
};
pub use ticket::{
    CreateTicket, QueueTicket, Ticket, TicketFilters, TicketSearchParams, TicketStatus,
    TicketSummary, UpdateTicket, WorkboardQueue,
};
pub use ticket_photo::{CreateTicketPhoto, TicketPhoto, TicketPhotoSummary};

//! Domain models and database entities.
//!
//! Models represent the core business entities used throughout the application.

pub mod admin_session;
pub mod customer;
pub mod employee;
pub mod field_history;
pub mod status_history;
pub mod storage_location;
pub mod store_settings;
pub mod ticket;
pub mod ticket_note;
pub mod ticket_photo;

pub use admin_session::{AdminSession, AdminSessionResponse, CreateAdminSession};
pub use customer::{CreateCustomer, Customer};
pub use employee::{
    CreateEmployee, Employee, EmployeeRole, EmployeeSummary, Permission, UpdateEmployee,
};
pub use field_history::{CreateFieldHistory, FieldHistoryEntry};
pub use status_history::{CreateStatusHistory, StatusHistoryEntry};
pub use storage_location::{
    CreateStorageLocation, StorageLocation, StorageLocationSummary, UpdateStorageLocation,
};
pub use store_settings::{
    StoreSettings, StoreSettingsPublic, TicketNumberResult, UpdateStoreSettings,
};
pub use ticket::{
    CreateTicket, QueueTicket, Ticket, TicketFilters, TicketSearchParams, TicketStatus,
    TicketSummary, UpdateTicket, WorkboardQueue,
};
pub use ticket_note::{CreateTicketNote, TicketNote};
pub use ticket_photo::{CreateTicketPhoto, TicketPhoto, TicketPhotoSummary};

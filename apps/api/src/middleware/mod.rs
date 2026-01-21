//! Middleware modules for the API.

pub mod rate_limit;
pub mod rbac;

pub use rate_limit::{extract_client_ip, RateLimitState, RateLimiter};
pub use rbac::{
    can_close_ticket, can_delete_photo, is_ticket_owner, require_permission, require_ticket_access,
};

//! Ticket note model.
//!
//! Internal notes on tickets. Append-only - no edit or delete.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A note attached to a ticket.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TicketNote {
    pub note_id: Uuid,
    pub ticket_id: Uuid,
    pub content: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a ticket note.
#[derive(Debug, Clone)]
pub struct CreateTicketNote {
    pub ticket_id: Uuid,
    pub content: String,
    pub created_by: Uuid,
}

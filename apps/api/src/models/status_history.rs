//! Ticket status history model.
//!
//! Records status changes for audit trail.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::ticket::TicketStatus;

/// A status change history entry.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct StatusHistoryEntry {
    pub history_id: Uuid,
    pub ticket_id: Uuid,
    pub from_status: Option<TicketStatus>,
    pub to_status: TicketStatus,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}

/// Input for creating a status history entry.
#[derive(Debug, Clone)]
pub struct CreateStatusHistory {
    pub ticket_id: Uuid,
    pub from_status: Option<TicketStatus>,
    pub to_status: TicketStatus,
    pub changed_by: Uuid,
}

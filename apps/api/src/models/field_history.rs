//! Ticket field history model.
//!
//! Records field changes for audit trail.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A field change history entry.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FieldHistoryEntry {
    pub history_id: Uuid,
    pub ticket_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
    pub changed_at: DateTime<Utc>,
}

/// Input for creating a field history entry.
#[derive(Debug, Clone)]
pub struct CreateFieldHistory {
    pub ticket_id: Uuid,
    pub field_name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub changed_by: Uuid,
}

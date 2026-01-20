//! Ticket model and related types.
//!
//! Tickets represent repair jobs in the system.

use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

/// Ticket status enum matching the database type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "ticket_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    Intake,
    InProgress,
    WaitingOnParts,
    ReadyForPickup,
    Closed,
    Archived,
}

impl TicketStatus {
    /// Returns all active (non-closed/archived) statuses.
    pub fn active_statuses() -> &'static [TicketStatus] {
        &[
            TicketStatus::Intake,
            TicketStatus::InProgress,
            TicketStatus::WaitingOnParts,
            TicketStatus::ReadyForPickup,
        ]
    }

    /// Returns true if this status represents an open ticket.
    pub fn is_open(&self) -> bool {
        !matches!(self, TicketStatus::Closed | TicketStatus::Archived)
    }
}

/// Full ticket entity with all fields.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Ticket {
    pub ticket_id: Uuid,
    pub friendly_code: String,

    // Customer reference
    pub customer_id: Uuid,

    // Item details
    pub item_type: Option<String>,
    pub item_description: String,
    pub condition_notes: String,
    pub requested_work: String,

    // Operational
    pub status: TicketStatus,
    pub is_rush: bool,
    pub promise_date: Option<NaiveDate>,
    pub storage_location_id: Uuid,

    // Pricing
    pub quote_amount: Option<Decimal>,
    pub actual_amount: Option<Decimal>,

    // Employee attribution
    pub taken_in_by: Uuid,
    pub worked_by: Option<Uuid>,
    pub closed_by: Option<Uuid>,
    pub last_modified_by: Option<Uuid>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,

    // Queue ordering
    pub queue_position: Option<i32>,
}

/// Summary view of a ticket for list views.
///
/// Contains just the essential fields needed for queue/list display.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TicketSummary {
    pub ticket_id: Uuid,
    pub friendly_code: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub item_type: Option<String>,
    pub item_description: String,
    pub status: TicketStatus,
    pub is_rush: bool,
    pub promise_date: Option<NaiveDate>,
    pub quote_amount: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new ticket.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTicket {
    pub customer_id: Uuid,
    pub item_type: Option<String>,
    pub item_description: String,
    pub condition_notes: String,
    pub requested_work: String,
    pub is_rush: bool,
    pub promise_date: Option<NaiveDate>,
    pub storage_location_id: Uuid,
    pub quote_amount: Option<Decimal>,
    pub taken_in_by: Uuid,
}

/// Input for updating an existing ticket.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateTicket {
    pub item_type: Option<String>,
    pub item_description: Option<String>,
    pub condition_notes: Option<String>,
    pub requested_work: Option<String>,
    pub is_rush: Option<bool>,
    pub promise_date: Option<Option<NaiveDate>>,
    pub storage_location_id: Option<Uuid>,
    pub quote_amount: Option<Option<Decimal>>,
    pub actual_amount: Option<Option<Decimal>>,
    pub worked_by: Option<Option<Uuid>>,
    pub last_modified_by: Option<Uuid>,
}

/// Filters for listing tickets.
#[derive(Debug, Clone, Default)]
pub struct TicketFilters {
    /// Filter by status (can include multiple)
    pub statuses: Option<Vec<TicketStatus>>,
    /// Filter by rush flag
    pub is_rush: Option<bool>,
    /// Filter by customer
    pub customer_id: Option<Uuid>,
    /// Filter by created date range (start)
    pub created_after: Option<DateTime<Utc>>,
    /// Filter by created date range (end)
    pub created_before: Option<DateTime<Utc>>,
    /// Limit results
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Extended ticket summary for queue/workboard views.
///
/// Includes overdue calculation for visual indicators.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct QueueTicket {
    pub ticket_id: Uuid,
    pub friendly_code: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub item_type: Option<String>,
    pub item_description: String,
    pub status: TicketStatus,
    pub is_rush: bool,
    pub promise_date: Option<NaiveDate>,
    pub quote_amount: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    /// True if promise_date is in the past and ticket is still open.
    pub is_overdue: bool,
}

/// Search parameters for full-text ticket search.
#[derive(Debug, Clone)]
pub struct TicketSearchParams {
    /// Search query string (searches across ticket, customer, and notes)
    pub query: String,
    /// Filter by statuses (empty = all statuses including archived)
    pub statuses: Option<Vec<TicketStatus>>,
    /// Limit results
    pub limit: Option<i64>,
    /// Offset for pagination
    pub offset: Option<i64>,
}

/// Workboard queue response grouped by status lanes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkboardQueue {
    pub intake: Vec<QueueTicket>,
    pub in_progress: Vec<QueueTicket>,
    pub waiting_on_parts: Vec<QueueTicket>,
    pub ready_for_pickup: Vec<QueueTicket>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ticket_status_active() {
        assert!(TicketStatus::Intake.is_open());
        assert!(TicketStatus::InProgress.is_open());
        assert!(TicketStatus::WaitingOnParts.is_open());
        assert!(TicketStatus::ReadyForPickup.is_open());
        assert!(!TicketStatus::Closed.is_open());
        assert!(!TicketStatus::Archived.is_open());
    }

    #[test]
    fn test_active_statuses() {
        let active = TicketStatus::active_statuses();
        assert_eq!(active.len(), 4);
        assert!(!active.contains(&TicketStatus::Closed));
        assert!(!active.contains(&TicketStatus::Archived));
    }

    #[test]
    fn test_ticket_status_serialization() {
        let status = TicketStatus::WaitingOnParts;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"waiting_on_parts\"");

        let parsed: TicketStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, TicketStatus::WaitingOnParts);
    }

    #[test]
    fn test_workboard_queue_serialization() {
        let queue = WorkboardQueue {
            intake: vec![],
            in_progress: vec![],
            waiting_on_parts: vec![],
            ready_for_pickup: vec![],
        };
        let json = serde_json::to_string(&queue).unwrap();
        assert!(json.contains("\"intake\":[]"));
        assert!(json.contains("\"in_progress\":[]"));
        assert!(json.contains("\"waiting_on_parts\":[]"));
        assert!(json.contains("\"ready_for_pickup\":[]"));
    }
}

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

    /// Check if a status transition is valid.
    ///
    /// Allowed transitions:
    /// - Intake → InProgress, WaitingOnParts, ReadyForPickup
    /// - InProgress → WaitingOnParts, ReadyForPickup
    /// - WaitingOnParts → InProgress, ReadyForPickup
    /// - ReadyForPickup → Closed
    /// - Closed → Archived (admin only, but allowed by this check)
    ///
    /// Note: Closing is handled separately via the close endpoint.
    /// This method validates transitions for the status change endpoint.
    pub fn can_transition_to(&self, new_status: TicketStatus) -> bool {
        match (self, new_status) {
            // Cannot stay on the same status
            (from, to) if *from == to => false,

            // Intake can move to any active status
            (TicketStatus::Intake, TicketStatus::InProgress)
            | (TicketStatus::Intake, TicketStatus::WaitingOnParts)
            | (TicketStatus::Intake, TicketStatus::ReadyForPickup) => true,

            // InProgress can move to waiting or ready
            (TicketStatus::InProgress, TicketStatus::WaitingOnParts)
            | (TicketStatus::InProgress, TicketStatus::ReadyForPickup) => true,

            // WaitingOnParts can go back to in_progress or move to ready
            (TicketStatus::WaitingOnParts, TicketStatus::InProgress)
            | (TicketStatus::WaitingOnParts, TicketStatus::ReadyForPickup) => true,

            // ReadyForPickup can only be closed (via close endpoint, not status change)
            // So no valid transitions from ReadyForPickup via status endpoint
            (TicketStatus::ReadyForPickup, _) => false,

            // Closed tickets can only be archived (admin operation)
            (TicketStatus::Closed, TicketStatus::Archived) => true,
            (TicketStatus::Closed, _) => false,

            // Archived tickets cannot change status
            (TicketStatus::Archived, _) => false,

            // Catch-all: reject any other transitions
            _ => false,
        }
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

    // Soft-delete fields
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
}

impl Ticket {
    /// Returns true if this ticket has been soft-deleted.
    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }
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

    #[test]
    fn test_status_transition_same_status() {
        // Cannot transition to the same status
        assert!(!TicketStatus::Intake.can_transition_to(TicketStatus::Intake));
        assert!(!TicketStatus::InProgress.can_transition_to(TicketStatus::InProgress));
        assert!(!TicketStatus::WaitingOnParts.can_transition_to(TicketStatus::WaitingOnParts));
        assert!(!TicketStatus::ReadyForPickup.can_transition_to(TicketStatus::ReadyForPickup));
        assert!(!TicketStatus::Closed.can_transition_to(TicketStatus::Closed));
        assert!(!TicketStatus::Archived.can_transition_to(TicketStatus::Archived));
    }

    #[test]
    fn test_status_transition_from_intake() {
        // Intake can move to in_progress, waiting_on_parts, or ready_for_pickup
        assert!(TicketStatus::Intake.can_transition_to(TicketStatus::InProgress));
        assert!(TicketStatus::Intake.can_transition_to(TicketStatus::WaitingOnParts));
        assert!(TicketStatus::Intake.can_transition_to(TicketStatus::ReadyForPickup));
        // Cannot skip to closed/archived
        assert!(!TicketStatus::Intake.can_transition_to(TicketStatus::Closed));
        assert!(!TicketStatus::Intake.can_transition_to(TicketStatus::Archived));
    }

    #[test]
    fn test_status_transition_from_in_progress() {
        // InProgress can move to waiting_on_parts or ready_for_pickup
        assert!(TicketStatus::InProgress.can_transition_to(TicketStatus::WaitingOnParts));
        assert!(TicketStatus::InProgress.can_transition_to(TicketStatus::ReadyForPickup));
        // Cannot go back to intake
        assert!(!TicketStatus::InProgress.can_transition_to(TicketStatus::Intake));
        // Cannot skip to closed/archived
        assert!(!TicketStatus::InProgress.can_transition_to(TicketStatus::Closed));
        assert!(!TicketStatus::InProgress.can_transition_to(TicketStatus::Archived));
    }

    #[test]
    fn test_status_transition_from_waiting_on_parts() {
        // WaitingOnParts can go back to in_progress or move to ready_for_pickup
        assert!(TicketStatus::WaitingOnParts.can_transition_to(TicketStatus::InProgress));
        assert!(TicketStatus::WaitingOnParts.can_transition_to(TicketStatus::ReadyForPickup));
        // Cannot go back to intake
        assert!(!TicketStatus::WaitingOnParts.can_transition_to(TicketStatus::Intake));
        // Cannot skip to closed/archived
        assert!(!TicketStatus::WaitingOnParts.can_transition_to(TicketStatus::Closed));
        assert!(!TicketStatus::WaitingOnParts.can_transition_to(TicketStatus::Archived));
    }

    #[test]
    fn test_status_transition_from_ready_for_pickup() {
        // ReadyForPickup can only be closed (via close endpoint, not status change)
        // So no valid transitions via status endpoint
        assert!(!TicketStatus::ReadyForPickup.can_transition_to(TicketStatus::Intake));
        assert!(!TicketStatus::ReadyForPickup.can_transition_to(TicketStatus::InProgress));
        assert!(!TicketStatus::ReadyForPickup.can_transition_to(TicketStatus::WaitingOnParts));
        assert!(!TicketStatus::ReadyForPickup.can_transition_to(TicketStatus::Closed));
        assert!(!TicketStatus::ReadyForPickup.can_transition_to(TicketStatus::Archived));
    }

    #[test]
    fn test_status_transition_from_closed() {
        // Closed can only be archived
        assert!(TicketStatus::Closed.can_transition_to(TicketStatus::Archived));
        // Cannot reopen
        assert!(!TicketStatus::Closed.can_transition_to(TicketStatus::Intake));
        assert!(!TicketStatus::Closed.can_transition_to(TicketStatus::InProgress));
        assert!(!TicketStatus::Closed.can_transition_to(TicketStatus::WaitingOnParts));
        assert!(!TicketStatus::Closed.can_transition_to(TicketStatus::ReadyForPickup));
    }

    #[test]
    fn test_status_transition_from_archived() {
        // Archived cannot change status
        assert!(!TicketStatus::Archived.can_transition_to(TicketStatus::Intake));
        assert!(!TicketStatus::Archived.can_transition_to(TicketStatus::InProgress));
        assert!(!TicketStatus::Archived.can_transition_to(TicketStatus::WaitingOnParts));
        assert!(!TicketStatus::Archived.can_transition_to(TicketStatus::ReadyForPickup));
        assert!(!TicketStatus::Archived.can_transition_to(TicketStatus::Closed));
    }

    #[test]
    fn test_ticket_is_deleted_when_deleted_at_is_some() {
        use chrono::Utc;
        use rust_decimal::Decimal;

        let ticket = Ticket {
            ticket_id: Uuid::new_v4(),
            friendly_code: "JR-0001".to_string(),
            customer_id: Uuid::new_v4(),
            item_type: None,
            item_description: "Test".to_string(),
            condition_notes: "Test".to_string(),
            requested_work: "Test".to_string(),
            status: TicketStatus::Intake,
            is_rush: false,
            promise_date: None,
            storage_location_id: Uuid::new_v4(),
            quote_amount: Some(Decimal::new(100, 2)),
            actual_amount: None,
            taken_in_by: Uuid::new_v4(),
            worked_by: None,
            closed_by: None,
            last_modified_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
            queue_position: None,
            deleted_at: Some(Utc::now()),
            deleted_by: Some(Uuid::new_v4()),
        };

        assert!(ticket.is_deleted());
    }

    #[test]
    fn test_ticket_is_not_deleted_when_deleted_at_is_none() {
        use chrono::Utc;
        use rust_decimal::Decimal;

        let ticket = Ticket {
            ticket_id: Uuid::new_v4(),
            friendly_code: "JR-0001".to_string(),
            customer_id: Uuid::new_v4(),
            item_type: None,
            item_description: "Test".to_string(),
            condition_notes: "Test".to_string(),
            requested_work: "Test".to_string(),
            status: TicketStatus::Intake,
            is_rush: false,
            promise_date: None,
            storage_location_id: Uuid::new_v4(),
            quote_amount: Some(Decimal::new(100, 2)),
            actual_amount: None,
            taken_in_by: Uuid::new_v4(),
            worked_by: None,
            closed_by: None,
            last_modified_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
            queue_position: None,
            deleted_at: None,
            deleted_by: None,
        };

        assert!(!ticket.is_deleted());
    }
}

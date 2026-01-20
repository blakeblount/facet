//! Ticket repository for database operations.

use crate::error::AppError;
use crate::models::ticket::{
    CreateTicket, QueueTicket, Ticket, TicketFilters, TicketSearchParams, TicketStatus,
    TicketSummary, UpdateTicket, WorkboardQueue,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for ticket database operations.
pub struct TicketRepository;

impl TicketRepository {
    /// Create a new ticket.
    ///
    /// The friendly_code is generated atomically using the database function.
    pub async fn create(pool: &PgPool, input: CreateTicket) -> Result<Ticket, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            INSERT INTO tickets (
                friendly_code,
                customer_id,
                item_type,
                item_description,
                condition_notes,
                requested_work,
                is_rush,
                promise_date,
                storage_location_id,
                quote_amount,
                taken_in_by
            )
            VALUES (
                generate_friendly_code(),
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10
            )
            RETURNING *
            "#,
        )
        .bind(input.customer_id)
        .bind(&input.item_type)
        .bind(&input.item_description)
        .bind(&input.condition_notes)
        .bind(&input.requested_work)
        .bind(input.is_rush)
        .bind(input.promise_date)
        .bind(input.storage_location_id)
        .bind(input.quote_amount)
        .bind(input.taken_in_by)
        .fetch_one(pool)
        .await?;

        Ok(ticket)
    }

    /// Find a ticket by ID.
    pub async fn find_by_id(pool: &PgPool, ticket_id: Uuid) -> Result<Option<Ticket>, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT * FROM tickets WHERE ticket_id = $1
            "#,
        )
        .bind(ticket_id)
        .fetch_optional(pool)
        .await?;

        Ok(ticket)
    }

    /// Find a ticket by friendly code.
    pub async fn find_by_code(
        pool: &PgPool,
        friendly_code: &str,
    ) -> Result<Option<Ticket>, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            SELECT * FROM tickets WHERE friendly_code = $1
            "#,
        )
        .bind(friendly_code)
        .fetch_optional(pool)
        .await?;

        Ok(ticket)
    }

    /// Update just the status of a ticket.
    ///
    /// This is a focused update for status changes, separate from general ticket updates.
    /// Also updates the last_modified_by and updated_at fields.
    pub async fn update_status(
        pool: &PgPool,
        ticket_id: Uuid,
        new_status: TicketStatus,
        modified_by: Uuid,
    ) -> Result<Ticket, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            UPDATE tickets SET
                status = $2,
                last_modified_by = $3,
                updated_at = NOW()
            WHERE ticket_id = $1
            RETURNING *
            "#,
        )
        .bind(ticket_id)
        .bind(new_status)
        .bind(modified_by)
        .fetch_one(pool)
        .await?;

        Ok(ticket)
    }

    /// Update a ticket.
    ///
    /// Only non-None fields in the input will be updated.
    /// Uses COALESCE for simple optional fields and CASE WHEN for nullable fields
    /// that can be explicitly set to NULL.
    pub async fn update(
        pool: &PgPool,
        ticket_id: Uuid,
        input: UpdateTicket,
    ) -> Result<Ticket, AppError> {
        let ticket = sqlx::query_as::<_, Ticket>(
            r#"
            UPDATE tickets SET
                item_type = COALESCE($2, item_type),
                item_description = COALESCE($3, item_description),
                condition_notes = COALESCE($4, condition_notes),
                requested_work = COALESCE($5, requested_work),
                is_rush = COALESCE($6, is_rush),
                promise_date = CASE WHEN $7::boolean THEN $8 ELSE promise_date END,
                storage_location_id = COALESCE($9, storage_location_id),
                quote_amount = CASE WHEN $10::boolean THEN $11 ELSE quote_amount END,
                actual_amount = CASE WHEN $12::boolean THEN $13 ELSE actual_amount END,
                worked_by = CASE WHEN $14::boolean THEN $15 ELSE worked_by END,
                last_modified_by = COALESCE($16, last_modified_by),
                updated_at = NOW()
            WHERE ticket_id = $1
            RETURNING *
            "#,
        )
        .bind(ticket_id)
        .bind(&input.item_type)
        .bind(&input.item_description)
        .bind(&input.condition_notes)
        .bind(&input.requested_work)
        .bind(input.is_rush)
        // For Option<Option<T>> fields, we need to signal when to update vs skip
        .bind(input.promise_date.is_some()) // $7: flag indicating if we should update
        .bind(input.promise_date.flatten()) // $8: actual value (could be None)
        .bind(input.storage_location_id)
        .bind(input.quote_amount.is_some()) // $10: flag
        .bind(input.quote_amount.flatten()) // $11: actual value
        .bind(input.actual_amount.is_some()) // $12: flag
        .bind(input.actual_amount.flatten()) // $13: actual value
        .bind(input.worked_by.is_some()) // $14: flag
        .bind(input.worked_by.flatten()) // $15: actual value
        .bind(input.last_modified_by)
        .fetch_one(pool)
        .await?;

        Ok(ticket)
    }

    /// List tickets with optional filters.
    ///
    /// Returns summaries joined with customer name for display.
    /// Default ordering: rush tickets first, then by created_at ascending (FIFO).
    pub async fn list(
        pool: &PgPool,
        filters: TicketFilters,
    ) -> Result<Vec<TicketSummary>, AppError> {
        let tickets = sqlx::query_as::<_, TicketSummary>(
            r#"
            SELECT
                t.ticket_id,
                t.friendly_code,
                t.customer_id,
                c.name as customer_name,
                t.item_type,
                t.item_description,
                t.status,
                t.is_rush,
                t.promise_date,
                t.quote_amount,
                t.created_at
            FROM tickets t
            JOIN customers c ON t.customer_id = c.customer_id
            WHERE ($1::boolean IS NULL OR t.is_rush = $1)
              AND ($2::uuid IS NULL OR t.customer_id = $2)
              AND ($3::timestamptz IS NULL OR t.created_at >= $3)
              AND ($4::timestamptz IS NULL OR t.created_at <= $4)
            ORDER BY t.is_rush DESC, t.created_at ASC
            LIMIT $5
            OFFSET $6
            "#,
        )
        .bind(filters.is_rush)
        .bind(filters.customer_id)
        .bind(filters.created_after)
        .bind(filters.created_before)
        .bind(filters.limit.unwrap_or(100))
        .bind(filters.offset.unwrap_or(0))
        .fetch_all(pool)
        .await?;

        // Filter by status in application code if needed
        let tickets = if let Some(ref statuses) = filters.statuses {
            if !statuses.is_empty() {
                tickets
                    .into_iter()
                    .filter(|t| statuses.contains(&t.status))
                    .collect()
            } else {
                tickets
            }
        } else {
            tickets
        };

        Ok(tickets)
    }

    /// List tickets filtered by specific statuses.
    ///
    /// This is optimized for queue views where we want specific status lanes.
    pub async fn list_by_status(
        pool: &PgPool,
        statuses: &[crate::models::ticket::TicketStatus],
        limit: Option<i64>,
    ) -> Result<Vec<TicketSummary>, AppError> {
        if statuses.is_empty() {
            return Ok(Vec::new());
        }

        // Convert statuses to strings for the ANY clause
        let status_strings: Vec<String> = statuses
            .iter()
            .map(|s| match s {
                crate::models::ticket::TicketStatus::Intake => "intake".to_string(),
                crate::models::ticket::TicketStatus::InProgress => "in_progress".to_string(),
                crate::models::ticket::TicketStatus::WaitingOnParts => {
                    "waiting_on_parts".to_string()
                }
                crate::models::ticket::TicketStatus::ReadyForPickup => {
                    "ready_for_pickup".to_string()
                }
                crate::models::ticket::TicketStatus::Closed => "closed".to_string(),
                crate::models::ticket::TicketStatus::Archived => "archived".to_string(),
            })
            .collect();

        let tickets = sqlx::query_as::<_, TicketSummary>(
            r#"
            SELECT
                t.ticket_id,
                t.friendly_code,
                t.customer_id,
                c.name as customer_name,
                t.item_type,
                t.item_description,
                t.status,
                t.is_rush,
                t.promise_date,
                t.quote_amount,
                t.created_at
            FROM tickets t
            JOIN customers c ON t.customer_id = c.customer_id
            WHERE t.status::text = ANY($1)
            ORDER BY t.is_rush DESC, t.created_at ASC
            LIMIT $2
            "#,
        )
        .bind(&status_strings)
        .bind(limit.unwrap_or(100))
        .fetch_all(pool)
        .await?;

        Ok(tickets)
    }

    /// Count tickets matching the given filters.
    pub async fn count(pool: &PgPool, filters: TicketFilters) -> Result<i64, AppError> {
        let count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT COUNT(*)
            FROM tickets t
            WHERE ($1::boolean IS NULL OR t.is_rush = $1)
              AND ($2::uuid IS NULL OR t.customer_id = $2)
              AND ($3::timestamptz IS NULL OR t.created_at >= $3)
              AND ($4::timestamptz IS NULL OR t.created_at <= $4)
            "#,
        )
        .bind(filters.is_rush)
        .bind(filters.customer_id)
        .bind(filters.created_after)
        .bind(filters.created_before)
        .fetch_one(pool)
        .await?;

        Ok(count)
    }

    /// Search tickets with full-text search across ticket, customer, and notes.
    ///
    /// Searches across:
    /// - Ticket: friendly_code, item_type, item_description, condition_notes, requested_work
    /// - Customer: name, phone, email
    /// - Notes: content
    ///
    /// Returns tickets matching the search query, sorted by relevance then date.
    pub async fn search(
        pool: &PgPool,
        params: TicketSearchParams,
    ) -> Result<Vec<QueueTicket>, AppError> {
        // Build status filter array if provided
        let status_strings: Option<Vec<String>> = params
            .statuses
            .map(|statuses| statuses.iter().map(Self::status_to_string).collect());

        // Use ILIKE for case-insensitive partial matching
        // The % wildcards allow matching anywhere in the text
        let search_pattern = format!("%{}%", params.query);

        let tickets = sqlx::query_as::<_, QueueTicket>(
            r#"
            SELECT DISTINCT
                t.ticket_id,
                t.friendly_code,
                t.customer_id,
                c.name as customer_name,
                t.item_type,
                t.item_description,
                t.status,
                t.is_rush,
                t.promise_date,
                t.quote_amount,
                t.created_at,
                CASE
                    WHEN t.promise_date IS NOT NULL
                     AND t.promise_date < CURRENT_DATE
                     AND t.status NOT IN ('closed', 'archived')
                    THEN TRUE
                    ELSE FALSE
                END as is_overdue
            FROM tickets t
            JOIN customers c ON t.customer_id = c.customer_id
            LEFT JOIN ticket_notes n ON t.ticket_id = n.ticket_id
            WHERE (
                t.friendly_code ILIKE $1
                OR t.item_type ILIKE $1
                OR t.item_description ILIKE $1
                OR t.condition_notes ILIKE $1
                OR t.requested_work ILIKE $1
                OR c.name ILIKE $1
                OR c.phone ILIKE $1
                OR c.email ILIKE $1
                OR n.content ILIKE $1
            )
            AND ($2::text[] IS NULL OR t.status::text = ANY($2))
            ORDER BY
                -- Prioritize exact friendly_code matches
                CASE WHEN t.friendly_code ILIKE $1 THEN 0 ELSE 1 END,
                -- Then by rush status
                t.is_rush DESC,
                -- Then by creation date (FIFO)
                t.created_at ASC
            LIMIT $3
            OFFSET $4
            "#,
        )
        .bind(&search_pattern)
        .bind(&status_strings)
        .bind(params.limit.unwrap_or(100))
        .bind(params.offset.unwrap_or(0))
        .fetch_all(pool)
        .await?;

        Ok(tickets)
    }

    /// Get the workboard queue with tickets grouped by status lane.
    ///
    /// Returns only active tickets (not closed/archived), sorted within each lane
    /// by rush first, then FIFO (created_at ascending).
    pub async fn get_queue(
        pool: &PgPool,
        limit_per_lane: Option<i64>,
    ) -> Result<WorkboardQueue, AppError> {
        // Fetch all active tickets in a single query, then group in memory
        // This is efficient for typical workloads (~30 tickets/day)
        let tickets = sqlx::query_as::<_, QueueTicket>(
            r#"
            SELECT
                t.ticket_id,
                t.friendly_code,
                t.customer_id,
                c.name as customer_name,
                t.item_type,
                t.item_description,
                t.status,
                t.is_rush,
                t.promise_date,
                t.quote_amount,
                t.created_at,
                CASE
                    WHEN t.promise_date IS NOT NULL
                     AND t.promise_date < CURRENT_DATE
                     AND t.status NOT IN ('closed', 'archived')
                    THEN TRUE
                    ELSE FALSE
                END as is_overdue
            FROM tickets t
            JOIN customers c ON t.customer_id = c.customer_id
            WHERE t.status NOT IN ('closed', 'archived')
            ORDER BY t.is_rush DESC, t.created_at ASC
            "#,
        )
        .fetch_all(pool)
        .await?;

        // Group tickets by status
        let limit = limit_per_lane.unwrap_or(100) as usize;

        let mut intake = Vec::new();
        let mut in_progress = Vec::new();
        let mut waiting_on_parts = Vec::new();
        let mut ready_for_pickup = Vec::new();

        for ticket in tickets {
            match ticket.status {
                TicketStatus::Intake => {
                    if intake.len() < limit {
                        intake.push(ticket);
                    }
                }
                TicketStatus::InProgress => {
                    if in_progress.len() < limit {
                        in_progress.push(ticket);
                    }
                }
                TicketStatus::WaitingOnParts => {
                    if waiting_on_parts.len() < limit {
                        waiting_on_parts.push(ticket);
                    }
                }
                TicketStatus::ReadyForPickup => {
                    if ready_for_pickup.len() < limit {
                        ready_for_pickup.push(ticket);
                    }
                }
                // Closed and Archived are filtered out by the query
                TicketStatus::Closed | TicketStatus::Archived => {}
            }
        }

        Ok(WorkboardQueue {
            intake,
            in_progress,
            waiting_on_parts,
            ready_for_pickup,
        })
    }

    /// Get tickets by status for a single lane.
    ///
    /// Returns tickets for the specified status, sorted by rush first, then FIFO.
    /// Includes overdue calculation.
    pub async fn get_lane(
        pool: &PgPool,
        status: TicketStatus,
        limit: Option<i64>,
    ) -> Result<Vec<QueueTicket>, AppError> {
        let status_str = Self::status_to_string(&status);

        let tickets = sqlx::query_as::<_, QueueTicket>(
            r#"
            SELECT
                t.ticket_id,
                t.friendly_code,
                t.customer_id,
                c.name as customer_name,
                t.item_type,
                t.item_description,
                t.status,
                t.is_rush,
                t.promise_date,
                t.quote_amount,
                t.created_at,
                CASE
                    WHEN t.promise_date IS NOT NULL
                     AND t.promise_date < CURRENT_DATE
                     AND t.status NOT IN ('closed', 'archived')
                    THEN TRUE
                    ELSE FALSE
                END as is_overdue
            FROM tickets t
            JOIN customers c ON t.customer_id = c.customer_id
            WHERE t.status::text = $1
            ORDER BY t.is_rush DESC, t.created_at ASC
            LIMIT $2
            "#,
        )
        .bind(&status_str)
        .bind(limit.unwrap_or(100))
        .fetch_all(pool)
        .await?;

        Ok(tickets)
    }

    /// Helper to convert TicketStatus to database string.
    fn status_to_string(status: &TicketStatus) -> String {
        match status {
            TicketStatus::Intake => "intake".to_string(),
            TicketStatus::InProgress => "in_progress".to_string(),
            TicketStatus::WaitingOnParts => "waiting_on_parts".to_string(),
            TicketStatus::ReadyForPickup => "ready_for_pickup".to_string(),
            TicketStatus::Closed => "closed".to_string(),
            TicketStatus::Archived => "archived".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Integration tests requiring a database connection should be run
    // with the test database available. Unit tests here focus on logic that
    // doesn't require database access.

    #[test]
    fn test_ticket_filters_default() {
        let filters = TicketFilters::default();
        assert!(filters.statuses.is_none());
        assert!(filters.is_rush.is_none());
        assert!(filters.customer_id.is_none());
        assert!(filters.created_after.is_none());
        assert!(filters.created_before.is_none());
        assert!(filters.limit.is_none());
        assert!(filters.offset.is_none());
    }

    #[test]
    fn test_update_ticket_default() {
        let update = UpdateTicket::default();
        assert!(update.item_type.is_none());
        assert!(update.item_description.is_none());
        assert!(update.is_rush.is_none());
    }

    #[test]
    fn test_status_to_string() {
        assert_eq!(
            TicketRepository::status_to_string(&TicketStatus::Intake),
            "intake"
        );
        assert_eq!(
            TicketRepository::status_to_string(&TicketStatus::InProgress),
            "in_progress"
        );
        assert_eq!(
            TicketRepository::status_to_string(&TicketStatus::WaitingOnParts),
            "waiting_on_parts"
        );
        assert_eq!(
            TicketRepository::status_to_string(&TicketStatus::ReadyForPickup),
            "ready_for_pickup"
        );
        assert_eq!(
            TicketRepository::status_to_string(&TicketStatus::Closed),
            "closed"
        );
        assert_eq!(
            TicketRepository::status_to_string(&TicketStatus::Archived),
            "archived"
        );
    }

    #[test]
    fn test_ticket_search_params() {
        let params = TicketSearchParams {
            query: "test".to_string(),
            statuses: Some(vec![TicketStatus::Intake, TicketStatus::InProgress]),
            limit: Some(50),
            offset: Some(10),
        };
        assert_eq!(params.query, "test");
        assert_eq!(params.statuses.as_ref().unwrap().len(), 2);
        assert_eq!(params.limit, Some(50));
        assert_eq!(params.offset, Some(10));
    }

    #[test]
    fn test_ticket_search_params_no_filters() {
        let params = TicketSearchParams {
            query: "ring".to_string(),
            statuses: None,
            limit: None,
            offset: None,
        };
        assert_eq!(params.query, "ring");
        assert!(params.statuses.is_none());
        assert!(params.limit.is_none());
        assert!(params.offset.is_none());
    }
}

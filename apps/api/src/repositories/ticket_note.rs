//! Ticket note repository for database operations.
//!
//! Notes are append-only - no update or delete methods.

use crate::error::AppError;
use crate::models::ticket_note::{CreateTicketNote, TicketNote};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for ticket note database operations.
pub struct TicketNoteRepository;

impl TicketNoteRepository {
    /// Create a new ticket note.
    ///
    /// Notes are append-only, so there is no update or delete.
    pub async fn create(pool: &PgPool, input: CreateTicketNote) -> Result<TicketNote, AppError> {
        let note = sqlx::query_as::<_, TicketNote>(
            r#"
            INSERT INTO ticket_notes (ticket_id, content, created_by)
            VALUES ($1, $2, $3)
            RETURNING *
            "#,
        )
        .bind(input.ticket_id)
        .bind(input.content)
        .bind(input.created_by)
        .fetch_one(pool)
        .await?;

        Ok(note)
    }

    /// Find all notes for a ticket.
    ///
    /// Returns notes ordered by created_at descending (most recent first).
    pub async fn find_by_ticket_id(
        pool: &PgPool,
        ticket_id: Uuid,
    ) -> Result<Vec<TicketNote>, AppError> {
        let notes = sqlx::query_as::<_, TicketNote>(
            r#"
            SELECT note_id, ticket_id, content, created_by, created_at
            FROM ticket_notes
            WHERE ticket_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(ticket_id)
        .fetch_all(pool)
        .await?;

        Ok(notes)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ticket_note_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

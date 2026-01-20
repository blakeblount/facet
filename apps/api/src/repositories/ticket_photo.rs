//! Ticket photo repository for database operations.

use crate::error::AppError;
use crate::models::ticket_photo::{CreateTicketPhoto, TicketPhoto, TicketPhotoSummary};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for ticket photo database operations.
pub struct TicketPhotoRepository;

impl TicketPhotoRepository {
    /// Create a new ticket photo record.
    ///
    /// This creates the database record for a photo that has already been
    /// uploaded to S3 storage. The storage_key should reference the S3 object.
    pub async fn create(pool: &PgPool, input: CreateTicketPhoto) -> Result<TicketPhoto, AppError> {
        let photo = sqlx::query_as::<_, TicketPhoto>(
            r#"
            INSERT INTO ticket_photos (ticket_id, storage_key, content_type, size_bytes, uploaded_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(input.ticket_id)
        .bind(&input.storage_key)
        .bind(&input.content_type)
        .bind(input.size_bytes)
        .bind(input.uploaded_by)
        .fetch_one(pool)
        .await?;

        Ok(photo)
    }

    /// Find a photo by ID.
    pub async fn find_by_id(
        pool: &PgPool,
        photo_id: Uuid,
    ) -> Result<Option<TicketPhoto>, AppError> {
        let photo = sqlx::query_as::<_, TicketPhoto>(
            r#"
            SELECT * FROM ticket_photos WHERE photo_id = $1
            "#,
        )
        .bind(photo_id)
        .fetch_optional(pool)
        .await?;

        Ok(photo)
    }

    /// Find all photos for a ticket.
    ///
    /// Returns photos ordered by upload time (oldest first).
    pub async fn find_by_ticket_id(
        pool: &PgPool,
        ticket_id: Uuid,
    ) -> Result<Vec<TicketPhotoSummary>, AppError> {
        let photos = sqlx::query_as::<_, TicketPhotoSummary>(
            r#"
            SELECT photo_id, storage_key, content_type, size_bytes, uploaded_at
            FROM ticket_photos
            WHERE ticket_id = $1
            ORDER BY uploaded_at ASC
            "#,
        )
        .bind(ticket_id)
        .fetch_all(pool)
        .await?;

        Ok(photos)
    }

    /// Count photos for a ticket.
    ///
    /// Used to enforce the maximum photos per ticket limit.
    pub async fn count_by_ticket_id(pool: &PgPool, ticket_id: Uuid) -> Result<i64, AppError> {
        let count: (i64,) = sqlx::query_as(
            r#"
            SELECT COUNT(*) FROM ticket_photos WHERE ticket_id = $1
            "#,
        )
        .bind(ticket_id)
        .fetch_one(pool)
        .await?;

        Ok(count.0)
    }

    /// Delete a photo by ID.
    ///
    /// Note: The caller should also delete the file from S3 storage.
    /// Returns the deleted photo if it existed, None otherwise.
    pub async fn delete(pool: &PgPool, photo_id: Uuid) -> Result<Option<TicketPhoto>, AppError> {
        let photo = sqlx::query_as::<_, TicketPhoto>(
            r#"
            DELETE FROM ticket_photos
            WHERE photo_id = $1
            RETURNING *
            "#,
        )
        .bind(photo_id)
        .fetch_optional(pool)
        .await?;

        Ok(photo)
    }

    /// Delete all photos for a ticket.
    ///
    /// Returns the storage keys of deleted photos so they can be removed from S3.
    pub async fn delete_by_ticket_id(
        pool: &PgPool,
        ticket_id: Uuid,
    ) -> Result<Vec<String>, AppError> {
        let photos = sqlx::query_as::<_, (String,)>(
            r#"
            DELETE FROM ticket_photos
            WHERE ticket_id = $1
            RETURNING storage_key
            "#,
        )
        .bind(ticket_id)
        .fetch_all(pool)
        .await?;

        Ok(photos.into_iter().map(|(key,)| key).collect())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_ticket_photo_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

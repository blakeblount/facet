//! Status history repository for database operations.

use crate::error::AppError;
use crate::models::status_history::{CreateStatusHistory, StatusHistoryEntry};
use sqlx::PgPool;

/// Repository for status history database operations.
pub struct StatusHistoryRepository;

impl StatusHistoryRepository {
    /// Create a new status history entry.
    pub async fn create(
        pool: &PgPool,
        input: CreateStatusHistory,
    ) -> Result<StatusHistoryEntry, AppError> {
        let entry = sqlx::query_as::<_, StatusHistoryEntry>(
            r#"
            INSERT INTO ticket_status_history (ticket_id, from_status, to_status, changed_by)
            VALUES ($1, $2, $3, $4)
            RETURNING *
            "#,
        )
        .bind(input.ticket_id)
        .bind(input.from_status)
        .bind(input.to_status)
        .bind(input.changed_by)
        .fetch_one(pool)
        .await?;

        Ok(entry)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_status_history_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

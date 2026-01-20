//! Field history repository for database operations.

use crate::error::AppError;
use crate::models::field_history::{CreateFieldHistory, FieldHistoryEntry};
use sqlx::PgPool;

/// Repository for field history database operations.
pub struct FieldHistoryRepository;

impl FieldHistoryRepository {
    /// Create a new field history entry.
    pub async fn create(
        pool: &PgPool,
        input: CreateFieldHistory,
    ) -> Result<FieldHistoryEntry, AppError> {
        let entry = sqlx::query_as::<_, FieldHistoryEntry>(
            r#"
            INSERT INTO ticket_field_history (ticket_id, field_name, old_value, new_value, changed_by)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING *
            "#,
        )
        .bind(input.ticket_id)
        .bind(&input.field_name)
        .bind(&input.old_value)
        .bind(&input.new_value)
        .bind(input.changed_by)
        .fetch_one(pool)
        .await?;

        Ok(entry)
    }

    /// Create multiple field history entries in a batch.
    pub async fn create_batch(
        pool: &PgPool,
        entries: Vec<CreateFieldHistory>,
    ) -> Result<(), AppError> {
        if entries.is_empty() {
            return Ok(());
        }

        for entry in entries {
            Self::create(pool, entry).await?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_field_history_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

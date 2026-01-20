//! Storage location repository for database operations.

use crate::error::AppError;
use crate::models::storage_location::{
    CreateStorageLocation, StorageLocation, StorageLocationSummary, UpdateStorageLocation,
};
use sqlx::PgPool;
use uuid::Uuid;

/// Repository for storage location database operations.
pub struct StorageLocationRepository;

impl StorageLocationRepository {
    /// Create a new storage location.
    pub async fn create(
        pool: &PgPool,
        input: CreateStorageLocation,
    ) -> Result<StorageLocation, AppError> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            INSERT INTO storage_locations (name)
            VALUES ($1)
            RETURNING *
            "#,
        )
        .bind(&input.name)
        .fetch_one(pool)
        .await?;

        Ok(location)
    }

    /// Find a storage location by name.
    pub async fn find_by_name(
        pool: &PgPool,
        name: &str,
    ) -> Result<Option<StorageLocation>, AppError> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT * FROM storage_locations WHERE LOWER(name) = LOWER($1)
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(location)
    }

    /// Find a storage location by ID.
    pub async fn find_by_id(
        pool: &PgPool,
        location_id: Uuid,
    ) -> Result<Option<StorageLocation>, AppError> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT * FROM storage_locations WHERE location_id = $1
            "#,
        )
        .bind(location_id)
        .fetch_optional(pool)
        .await?;

        Ok(location)
    }

    /// Find an active storage location by ID.
    ///
    /// Returns None if not found or if location is inactive.
    pub async fn find_active_by_id(
        pool: &PgPool,
        location_id: Uuid,
    ) -> Result<Option<StorageLocation>, AppError> {
        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            SELECT * FROM storage_locations
            WHERE location_id = $1 AND is_active = TRUE
            "#,
        )
        .bind(location_id)
        .fetch_optional(pool)
        .await?;

        Ok(location)
    }

    /// List storage locations with optional filtering.
    ///
    /// If include_inactive is false (default), only active locations are returned.
    pub async fn list(
        pool: &PgPool,
        include_inactive: bool,
    ) -> Result<Vec<StorageLocationSummary>, AppError> {
        let locations = if include_inactive {
            sqlx::query_as::<_, StorageLocationSummary>(
                r#"
                SELECT location_id, name, is_active
                FROM storage_locations
                ORDER BY name ASC
                "#,
            )
            .fetch_all(pool)
            .await?
        } else {
            sqlx::query_as::<_, StorageLocationSummary>(
                r#"
                SELECT location_id, name, is_active
                FROM storage_locations
                WHERE is_active = TRUE
                ORDER BY name ASC
                "#,
            )
            .fetch_all(pool)
            .await?
        };

        Ok(locations)
    }

    /// Update a storage location.
    ///
    /// Only the provided fields are updated.
    pub async fn update(
        pool: &PgPool,
        location_id: Uuid,
        input: UpdateStorageLocation,
    ) -> Result<Option<StorageLocation>, AppError> {
        // First check if location exists
        let existing = Self::find_by_id(pool, location_id).await?;
        if existing.is_none() {
            return Ok(None);
        }
        let existing = existing.unwrap();

        // Build update with provided fields, keeping existing values for unspecified fields
        let name = input.name.unwrap_or(existing.name);
        let is_active = input.is_active.unwrap_or(existing.is_active);

        let location = sqlx::query_as::<_, StorageLocation>(
            r#"
            UPDATE storage_locations
            SET name = $1, is_active = $2
            WHERE location_id = $3
            RETURNING *
            "#,
        )
        .bind(&name)
        .bind(is_active)
        .bind(location_id)
        .fetch_one(pool)
        .await?;

        Ok(Some(location))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_storage_location_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

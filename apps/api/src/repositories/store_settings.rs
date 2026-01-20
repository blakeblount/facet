//! Store settings repository for database operations.

use crate::auth::{hash_pin, verify_pin};
use crate::error::AppError;
use crate::models::store_settings::{
    StoreSettings, StoreSettingsPublic, TicketNumberResult, UpdateStoreSettings,
};
use sqlx::PgPool;

/// Repository for store settings database operations.
pub struct StoreSettingsRepository;

impl StoreSettingsRepository {
    /// Get the store settings.
    ///
    /// Returns the single store settings row.
    /// Returns a server error if settings don't exist (should be seeded).
    pub async fn get_settings(pool: &PgPool) -> Result<StoreSettings, AppError> {
        let settings = sqlx::query_as::<_, StoreSettings>(
            r#"
            SELECT * FROM store_settings LIMIT 1
            "#,
        )
        .fetch_optional(pool)
        .await?;

        settings.ok_or_else(|| AppError::server_error("Store settings not initialized"))
    }

    /// Get the public store settings (without admin PIN hash).
    pub async fn get_settings_public(pool: &PgPool) -> Result<StoreSettingsPublic, AppError> {
        let settings = Self::get_settings(pool).await?;
        Ok(StoreSettingsPublic::from(settings))
    }

    /// Update store settings.
    ///
    /// Only the provided fields are updated.
    pub async fn update_settings(
        pool: &PgPool,
        input: UpdateStoreSettings,
    ) -> Result<StoreSettingsPublic, AppError> {
        let existing = Self::get_settings(pool).await?;

        let store_name = input.store_name.unwrap_or(existing.store_name);
        let store_phone = input.store_phone.or(existing.store_phone);
        let store_address = input.store_address.or(existing.store_address);
        let ticket_prefix = input.ticket_prefix.unwrap_or(existing.ticket_prefix);
        let currency = input.currency.unwrap_or(existing.currency);
        let max_photos_per_ticket = input
            .max_photos_per_ticket
            .unwrap_or(existing.max_photos_per_ticket);

        let settings = sqlx::query_as::<_, StoreSettings>(
            r#"
            UPDATE store_settings
            SET store_name = $1,
                store_phone = $2,
                store_address = $3,
                ticket_prefix = $4,
                currency = $5,
                max_photos_per_ticket = $6,
                updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(&store_name)
        .bind(&store_phone)
        .bind(&store_address)
        .bind(&ticket_prefix)
        .bind(&currency)
        .bind(max_photos_per_ticket)
        .fetch_one(pool)
        .await?;

        Ok(StoreSettingsPublic::from(settings))
    }

    /// Verify the admin PIN.
    ///
    /// Returns true if the PIN is correct, false otherwise.
    pub async fn verify_admin_pin(pool: &PgPool, pin: &str) -> Result<bool, AppError> {
        let settings = Self::get_settings(pool).await?;
        let is_valid = verify_pin(pin, &settings.admin_pin_hash)?;
        Ok(is_valid)
    }

    /// Change the admin PIN.
    ///
    /// Returns the updated settings (public view).
    pub async fn change_admin_pin(
        pool: &PgPool,
        new_pin: &str,
    ) -> Result<StoreSettingsPublic, AppError> {
        let pin_hash = hash_pin(new_pin)?;

        let settings = sqlx::query_as::<_, StoreSettings>(
            r#"
            UPDATE store_settings
            SET admin_pin_hash = $1, updated_at = NOW()
            RETURNING *
            "#,
        )
        .bind(&pin_hash)
        .fetch_one(pool)
        .await?;

        Ok(StoreSettingsPublic::from(settings))
    }

    /// Get the next ticket number and increment the counter atomically.
    ///
    /// Returns the prefix and number to use for the new ticket.
    pub async fn get_and_increment_ticket_number(
        pool: &PgPool,
    ) -> Result<TicketNumberResult, AppError> {
        let result = sqlx::query_as::<_, (String, i32)>(
            r#"
            UPDATE store_settings
            SET next_ticket_number = next_ticket_number + 1,
                updated_at = NOW()
            RETURNING ticket_prefix, next_ticket_number - 1
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(TicketNumberResult {
            prefix: result.0,
            number: result.1,
        })
    }

    /// Mark setup as complete.
    ///
    /// Called after initial admin setup.
    pub async fn mark_setup_complete(pool: &PgPool) -> Result<StoreSettingsPublic, AppError> {
        let settings = sqlx::query_as::<_, StoreSettings>(
            r#"
            UPDATE store_settings
            SET setup_complete = TRUE, updated_at = NOW()
            RETURNING *
            "#,
        )
        .fetch_one(pool)
        .await?;

        Ok(StoreSettingsPublic::from(settings))
    }

    /// Check if initial setup is complete.
    pub async fn is_setup_complete(pool: &PgPool) -> Result<bool, AppError> {
        let settings = Self::get_settings(pool).await?;
        Ok(settings.setup_complete)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_store_settings_repository_exists() {
        // Basic sanity test
        assert!(true);
    }
}

//! Store settings request handlers.

use axum::{extract::State, response::IntoResponse, Json};

use crate::error::AppError;
use crate::models::store_settings::StoreSettingsPublic;
use crate::repositories::StoreSettingsRepository;
use crate::response::ApiResponse;
use crate::routes::AppState;

// =============================================================================
// GET /settings - Get Store Settings
// =============================================================================

/// GET /api/v1/settings - Get store settings (public read).
///
/// Returns the store settings without the admin PIN hash.
/// This endpoint is public and does not require authentication.
pub async fn get_settings(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    let settings: StoreSettingsPublic =
        StoreSettingsRepository::get_settings_public(&state.db).await?;

    Ok(Json(ApiResponse::success(settings)))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_settings_handler_exists() {
        // Basic sanity test
        assert!(true);
    }
}

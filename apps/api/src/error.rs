//! Application error types and HTTP response conversion.
//!
//! This module defines the error types used throughout the API
//! and implements conversion to HTTP responses with consistent JSON format.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Error codes matching the API specification.
pub mod codes {
    pub const VALIDATION_ERROR: &str = "VALIDATION_ERROR";
    pub const INVALID_PIN: &str = "INVALID_PIN";
    pub const FORBIDDEN: &str = "FORBIDDEN";
    pub const NOT_FOUND: &str = "NOT_FOUND";
    pub const CONFLICT: &str = "CONFLICT";
    pub const PHOTO_LIMIT: &str = "PHOTO_LIMIT";
    pub const PRINT_REQUIRED: &str = "PRINT_REQUIRED";
    pub const SERVER_ERROR: &str = "SERVER_ERROR";
}

/// Error detail in API response.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
}

/// Application errors that can be returned from handlers.
#[derive(Debug)]
pub enum AppError {
    /// Invalid request data (400).
    ValidationError(String),
    /// Employee or admin PIN incorrect (401).
    InvalidPin(String),
    /// Action not allowed (403).
    Forbidden(String),
    /// Resource not found (404).
    NotFound(String),
    /// Conflict (409).
    Conflict(String),
    /// Max photos per ticket reached (422).
    PhotoLimit(String),
    /// Cannot complete action until print succeeds (422).
    PrintRequired(String),
    /// Internal server error (500).
    ServerError(String),
}

impl AppError {
    /// Get the error code string for this error.
    pub fn code(&self) -> &'static str {
        match self {
            AppError::ValidationError(_) => codes::VALIDATION_ERROR,
            AppError::InvalidPin(_) => codes::INVALID_PIN,
            AppError::Forbidden(_) => codes::FORBIDDEN,
            AppError::NotFound(_) => codes::NOT_FOUND,
            AppError::Conflict(_) => codes::CONFLICT,
            AppError::PhotoLimit(_) => codes::PHOTO_LIMIT,
            AppError::PrintRequired(_) => codes::PRINT_REQUIRED,
            AppError::ServerError(_) => codes::SERVER_ERROR,
        }
    }

    /// Get the HTTP status code for this error.
    pub fn status_code(&self) -> StatusCode {
        match self {
            AppError::ValidationError(_) => StatusCode::BAD_REQUEST,
            AppError::InvalidPin(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Conflict(_) => StatusCode::CONFLICT,
            AppError::PhotoLimit(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::PrintRequired(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::ServerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Get the error message.
    pub fn message(&self) -> &str {
        match self {
            AppError::ValidationError(msg)
            | AppError::InvalidPin(msg)
            | AppError::Forbidden(msg)
            | AppError::NotFound(msg)
            | AppError::Conflict(msg)
            | AppError::PhotoLimit(msg)
            | AppError::PrintRequired(msg)
            | AppError::ServerError(msg) => msg,
        }
    }

    /// Create a validation error.
    pub fn validation(message: impl Into<String>) -> Self {
        AppError::ValidationError(message.into())
    }

    /// Create an invalid PIN error.
    pub fn invalid_pin(message: impl Into<String>) -> Self {
        AppError::InvalidPin(message.into())
    }

    /// Create a forbidden error.
    pub fn forbidden(message: impl Into<String>) -> Self {
        AppError::Forbidden(message.into())
    }

    /// Create a not found error.
    pub fn not_found(message: impl Into<String>) -> Self {
        AppError::NotFound(message.into())
    }

    /// Create a conflict error.
    pub fn conflict(message: impl Into<String>) -> Self {
        AppError::Conflict(message.into())
    }

    /// Create a photo limit error.
    pub fn photo_limit(message: impl Into<String>) -> Self {
        AppError::PhotoLimit(message.into())
    }

    /// Create a print required error.
    pub fn print_required(message: impl Into<String>) -> Self {
        AppError::PrintRequired(message.into())
    }

    /// Create a server error.
    pub fn server_error(message: impl Into<String>) -> Self {
        AppError::ServerError(message.into())
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code(), self.message())
    }
}

impl std::error::Error for AppError {}

/// Error response format matching the API specification.
#[derive(Serialize)]
struct ErrorResponse {
    data: Option<()>,
    error: ErrorDetail,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let error_response = ErrorResponse {
            data: None,
            error: ErrorDetail {
                code: self.code(),
                message: self.message().to_string(),
            },
        };

        (self.status_code(), Json(error_response)).into_response()
    }
}

/// Convert from sqlx errors to AppError.
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        match err {
            sqlx::Error::RowNotFound => AppError::not_found("Resource not found"),
            _ => AppError::server_error("Database error"),
        }
    }
}

/// Convert from HashError to AppError.
impl From<crate::auth::HashError> for AppError {
    fn from(err: crate::auth::HashError) -> Self {
        tracing::error!("Hash error: {:?}", err);
        AppError::server_error("Password hashing error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct TestErrorDetail {
        code: String,
        message: String,
    }

    #[derive(Deserialize)]
    struct TestErrorResponse {
        data: Option<()>,
        error: TestErrorDetail,
    }

    async fn extract_error_response(response: Response) -> (StatusCode, TestErrorResponse) {
        let status = response.status();
        let body = response.into_body().collect().await.unwrap().to_bytes();
        let parsed: TestErrorResponse = serde_json::from_slice(&body).unwrap();
        (status, parsed)
    }

    #[tokio::test]
    async fn test_validation_error_response() {
        let err = AppError::validation("Invalid email format");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::VALIDATION_ERROR);
        assert_eq!(body.error.message, "Invalid email format");
    }

    #[tokio::test]
    async fn test_not_found_error_response() {
        let err = AppError::not_found("Ticket not found");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::NOT_FOUND);
        assert_eq!(body.error.message, "Ticket not found");
    }

    #[tokio::test]
    async fn test_invalid_pin_error_response() {
        let err = AppError::invalid_pin("Invalid employee PIN");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::UNAUTHORIZED);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::INVALID_PIN);
        assert_eq!(body.error.message, "Invalid employee PIN");
    }

    #[tokio::test]
    async fn test_forbidden_error_response() {
        let err = AppError::forbidden("Cannot edit closed ticket");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::FORBIDDEN);
        assert_eq!(body.error.message, "Cannot edit closed ticket");
    }

    #[tokio::test]
    async fn test_conflict_error_response() {
        let err = AppError::conflict("Duplicate friendly code");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::CONFLICT);
        assert_eq!(body.error.message, "Duplicate friendly code");
    }

    #[tokio::test]
    async fn test_photo_limit_error_response() {
        let err = AppError::photo_limit("Maximum 10 photos per ticket");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::PHOTO_LIMIT);
        assert_eq!(body.error.message, "Maximum 10 photos per ticket");
    }

    #[tokio::test]
    async fn test_print_required_error_response() {
        let err = AppError::print_required("Print receipt before completing intake");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::PRINT_REQUIRED);
        assert_eq!(body.error.message, "Print receipt before completing intake");
    }

    #[tokio::test]
    async fn test_server_error_response() {
        let err = AppError::server_error("Internal server error");
        let response = err.into_response();
        let (status, body) = extract_error_response(response).await;

        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(body.data.is_none());
        assert_eq!(body.error.code, codes::SERVER_ERROR);
        assert_eq!(body.error.message, "Internal server error");
    }

    #[test]
    fn test_error_display() {
        let err = AppError::validation("Test message");
        assert_eq!(err.to_string(), "[VALIDATION_ERROR] Test message");
    }

    #[test]
    fn test_error_codes() {
        assert_eq!(AppError::validation("").code(), codes::VALIDATION_ERROR);
        assert_eq!(AppError::invalid_pin("").code(), codes::INVALID_PIN);
        assert_eq!(AppError::forbidden("").code(), codes::FORBIDDEN);
        assert_eq!(AppError::not_found("").code(), codes::NOT_FOUND);
        assert_eq!(AppError::conflict("").code(), codes::CONFLICT);
        assert_eq!(AppError::photo_limit("").code(), codes::PHOTO_LIMIT);
        assert_eq!(AppError::print_required("").code(), codes::PRINT_REQUIRED);
        assert_eq!(AppError::server_error("").code(), codes::SERVER_ERROR);
    }

    #[test]
    fn test_status_codes() {
        assert_eq!(
            AppError::validation("").status_code(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            AppError::invalid_pin("").status_code(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(AppError::forbidden("").status_code(), StatusCode::FORBIDDEN);
        assert_eq!(AppError::not_found("").status_code(), StatusCode::NOT_FOUND);
        assert_eq!(AppError::conflict("").status_code(), StatusCode::CONFLICT);
        assert_eq!(
            AppError::photo_limit("").status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            AppError::print_required("").status_code(),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            AppError::server_error("").status_code(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }
}

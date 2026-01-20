//! API response types and helper functions.
//!
//! This module provides the standard API response wrapper that all endpoints use.
//! The format follows the API specification:
//!
//! Success: `{ "data": { ... }, "error": null }`
//! Error: `{ "data": null, "error": { "code": "...", "message": "..." } }`

use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::error::{AppError, ErrorDetail};

/// Standard API response wrapper.
///
/// All API endpoints return this format for consistency.
/// The `T` type parameter is the data payload type.
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub data: Option<T>,
    pub error: Option<ErrorDetail>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a success response with data.
    pub fn success(data: T) -> Self {
        ApiResponse {
            data: Some(data),
            error: None,
        }
    }
}

impl ApiResponse<()> {
    /// Create an error response.
    ///
    /// This is defined separately on `ApiResponse<()>` since error responses
    /// never have data, which makes type inference easier.
    pub fn error(code: &'static str, message: impl Into<String>) -> Self {
        ApiResponse {
            data: None,
            error: Some(ErrorDetail {
                code,
                message: message.into(),
            }),
        }
    }
}

/// Type alias for handler results that return data on success.
pub type ApiResult<T> = Result<Json<ApiResponse<T>>, AppError>;

/// Helper function to return a successful response with data.
pub fn ok<T: Serialize>(data: T) -> ApiResult<T> {
    Ok(Json(ApiResponse::success(data)))
}

/// Helper function to return a successful response with HTTP 201 Created.
pub fn created<T: Serialize>(data: T) -> impl IntoResponse {
    (StatusCode::CREATED, Json(ApiResponse::success(data)))
}

/// Helper function to return an empty success response.
pub fn empty() -> ApiResult<()> {
    Ok(Json(ApiResponse::success(())))
}

/// Helper function to return an empty success response with HTTP 204 No Content.
pub fn no_content() -> impl IntoResponse {
    StatusCode::NO_CONTENT
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestData {
        id: String,
        name: String,
    }

    #[test]
    fn test_success_response() {
        let data = TestData {
            id: "123".to_string(),
            name: "Test".to_string(),
        };

        let response = ApiResponse::success(data);

        assert!(response.data.is_some());
        assert!(response.error.is_none());

        let data = response.data.unwrap();
        assert_eq!(data.id, "123");
        assert_eq!(data.name, "Test");
    }

    #[test]
    fn test_error_response() {
        let response = ApiResponse::error(crate::error::codes::NOT_FOUND, "Ticket not found");

        assert!(response.data.is_none());
        assert!(response.error.is_some());

        let error = response.error.unwrap();
        assert_eq!(error.code, "NOT_FOUND");
        assert_eq!(error.message, "Ticket not found");
    }

    #[test]
    fn test_response_serialization() {
        let data = TestData {
            id: "456".to_string(),
            name: "Test Item".to_string(),
        };

        let response = ApiResponse::success(data);
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""data":"#));
        assert!(json.contains(r#""error":null"#));
        assert!(json.contains(r#""id":"456""#));
        assert!(json.contains(r#""name":"Test Item""#));
    }

    #[test]
    fn test_error_response_serialization() {
        let response = ApiResponse::error(crate::error::codes::VALIDATION_ERROR, "Invalid input");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains(r#""data":null"#));
        assert!(json.contains(r#""error":"#));
        assert!(json.contains(r#""code":"VALIDATION_ERROR""#));
        assert!(json.contains(r#""message":"Invalid input""#));
    }

    #[test]
    fn test_empty_success() {
        let result = empty();
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_ok_helper() {
        let data = TestData {
            id: "789".to_string(),
            name: "Helper Test".to_string(),
        };

        let result = ok(data);
        assert!(result.is_ok());

        let response = result.unwrap().0;
        assert!(response.data.is_some());
        assert!(response.error.is_none());

        let data = response.data.unwrap();
        assert_eq!(data.id, "789");
    }
}

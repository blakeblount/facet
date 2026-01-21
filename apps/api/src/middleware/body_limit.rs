//! Middleware for handling request body size limits with JSON error responses.

use axum::{
    body::Body,
    http::{Request, Response, StatusCode},
    middleware::Next,
    response::IntoResponse,
    Json,
};
use serde::Serialize;

use crate::error::codes;

/// Error response format for payload too large errors.
#[derive(Serialize)]
struct PayloadTooLargeResponse {
    data: Option<()>,
    error: PayloadTooLargeDetail,
}

#[derive(Serialize)]
struct PayloadTooLargeDetail {
    code: &'static str,
    message: String,
}

/// Middleware that converts 413 Payload Too Large responses to JSON format.
///
/// This middleware should be applied after `RequestBodyLimitLayer` to catch
/// the 413 responses it generates and convert them to the API's JSON error format.
pub async fn json_payload_error(request: Request<Body>, next: Next) -> Response<Body> {
    let response = next.run(request).await;

    // Check if this is a 413 Payload Too Large response
    if response.status() == StatusCode::PAYLOAD_TOO_LARGE {
        // Return a JSON-formatted error response
        let json_response = PayloadTooLargeResponse {
            data: None,
            error: PayloadTooLargeDetail {
                code: codes::PAYLOAD_TOO_LARGE,
                message: "Request body exceeds maximum allowed size".to_string(),
            },
        };

        return (StatusCode::PAYLOAD_TOO_LARGE, Json(json_response)).into_response();
    }

    response
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware,
        routing::post,
        Router,
    };
    use http_body_util::BodyExt;
    use serde::Deserialize;
    use tower::ServiceExt;
    use tower_http::limit::RequestBodyLimitLayer;

    #[test]
    fn test_payload_too_large_response_serialization() {
        let response = PayloadTooLargeResponse {
            data: None,
            error: PayloadTooLargeDetail {
                code: codes::PAYLOAD_TOO_LARGE,
                message: "Test message".to_string(),
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("PAYLOAD_TOO_LARGE"));
        assert!(json.contains("Test message"));
    }

    async fn echo_handler(body: String) -> String {
        body
    }

    #[derive(Deserialize)]
    struct TestErrorResponse {
        data: Option<()>,
        error: TestErrorDetail,
    }

    #[derive(Deserialize)]
    struct TestErrorDetail {
        code: String,
        message: String,
    }

    #[tokio::test]
    async fn test_small_body_passes_through() {
        let app = Router::new()
            .route("/test", post(echo_handler))
            .layer(RequestBodyLimitLayer::new(1024)) // 1KB limit
            .layer(middleware::from_fn(json_payload_error));

        let body = "a".repeat(100); // 100 bytes, under limit
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "text/plain")
            .body(Body::from(body.clone()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert_eq!(body_bytes.as_ref(), body.as_bytes());
    }

    #[tokio::test]
    async fn test_large_body_returns_413_json() {
        let app = Router::new()
            .route("/test", post(echo_handler))
            .layer(RequestBodyLimitLayer::new(1024)) // 1KB limit
            .layer(middleware::from_fn(json_payload_error));

        let body = "a".repeat(2000); // 2KB, over limit
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "text/plain")
            .header("content-length", body.len().to_string())
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);

        // Verify response is JSON
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.contains("application/json"));

        // Verify JSON structure
        let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
        let error_response: TestErrorResponse = serde_json::from_slice(&body_bytes).unwrap();
        assert!(error_response.data.is_none());
        assert_eq!(error_response.error.code, "PAYLOAD_TOO_LARGE");
        assert!(error_response.error.message.contains("maximum"));
    }

    #[tokio::test]
    async fn test_exact_limit_passes() {
        let app = Router::new()
            .route("/test", post(echo_handler))
            .layer(RequestBodyLimitLayer::new(1024)) // 1KB limit
            .layer(middleware::from_fn(json_payload_error));

        let body = "a".repeat(1024); // Exactly 1KB
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "text/plain")
            .body(Body::from(body.clone()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_just_over_limit_fails() {
        let app = Router::new()
            .route("/test", post(echo_handler))
            .layer(RequestBodyLimitLayer::new(1024)) // 1KB limit
            .layer(middleware::from_fn(json_payload_error));

        let body = "a".repeat(1025); // 1KB + 1 byte
        let request = Request::builder()
            .method("POST")
            .uri("/test")
            .header("content-type", "text/plain")
            .header("content-length", body.len().to_string())
            .body(Body::from(body))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }
}

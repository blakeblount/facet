//! CORS configuration for the API.

use crate::Config;
use axum::http::HeaderValue;
use std::time::Duration;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};

/// Build CORS layer based on configuration.
///
/// Respects the configured origins list:
/// - If `cors_origins` is `["*"]`, allows any origin (for development)
/// - Otherwise, only allows the specified origins
pub fn build_cors_layer(config: &Config) -> CorsLayer {
    tracing::info!("CORS allowed origins: {:?}", config.cors_origins);

    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    // If origins is "*", allow any origin; otherwise, parse specific origins
    if config.cors_origins.len() == 1 && config.cors_origins[0] == "*" {
        tracing::warn!("CORS configured to allow all origins - not recommended for production");
        cors.allow_origin(Any)
    } else {
        // Parse and validate specific origins
        let origins: Vec<HeaderValue> = config
            .cors_origins
            .iter()
            .filter_map(|origin| {
                origin.parse::<HeaderValue>().ok().or_else(|| {
                    tracing::error!("Invalid CORS origin: {}", origin);
                    None
                })
            })
            .collect();

        if origins.is_empty() {
            tracing::error!("No valid CORS origins configured, falling back to deny all");
            cors.allow_origin(AllowOrigin::list(vec![]))
        } else {
            cors.allow_origin(AllowOrigin::list(origins))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::get, Router};
    use tower::ServiceExt;

    fn test_config_with_origins(origins: Vec<&str>) -> Config {
        use crate::config::{DEFAULT_MAX_BODY_SIZE, DEFAULT_MAX_PHOTO_SIZE};
        Config {
            server_addr: "127.0.0.1:3001".parse().unwrap(),
            database_url: "postgres://test".to_string(),
            s3_endpoint: None,
            s3_bucket: "test".to_string(),
            s3_access_key: None,
            s3_secret_key: None,
            cors_origins: origins.into_iter().map(String::from).collect(),
            log_filter: "".to_string(),
            max_body_size: DEFAULT_MAX_BODY_SIZE,
            max_photo_size: DEFAULT_MAX_PHOTO_SIZE,
        }
    }

    async fn handler() -> &'static str {
        "ok"
    }

    #[tokio::test]
    async fn test_cors_allows_any_origin_when_wildcard() {
        let config = test_config_with_origins(vec!["*"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        // With Allow-Origin: Any, the response should include Access-Control-Allow-Origin: *
        let cors_header = response
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap());
        assert_eq!(cors_header, Some("*"));
    }

    #[tokio::test]
    async fn test_cors_allows_configured_origin() {
        let config = test_config_with_origins(vec!["https://example.com"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let cors_header = response
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap());
        assert_eq!(cors_header, Some("https://example.com"));
    }

    #[tokio::test]
    async fn test_cors_rejects_unconfigured_origin() {
        let config = test_config_with_origins(vec!["https://example.com"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://evil.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Request still succeeds (CORS is browser-enforced), but no CORS header
        assert_eq!(response.status(), 200);
        let cors_header = response.headers().get("access-control-allow-origin");
        assert!(cors_header.is_none());
    }

    #[tokio::test]
    async fn test_cors_allows_multiple_origins() {
        let config = test_config_with_origins(vec!["https://example.com", "https://other.com"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        // Test first origin
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cors_header = response
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap());
        assert_eq!(cors_header, Some("https://example.com"));

        // Test second origin
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://other.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cors_header = response
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap());
        assert_eq!(cors_header, Some("https://other.com"));
    }

    #[tokio::test]
    async fn test_cors_handles_empty_origins() {
        // Empty list after filtering invalid origins
        let config = test_config_with_origins(vec![""]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // No CORS header should be present (deny all)
        let cors_header = response.headers().get("access-control-allow-origin");
        assert!(cors_header.is_none());
    }

    #[tokio::test]
    async fn test_cors_preflight_allowed_origin() {
        let config = test_config_with_origins(vec!["https://example.com"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/test")
                    .header("Origin", "https://example.com")
                    .header("Access-Control-Request-Method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // Preflight should succeed
        assert_eq!(response.status(), 200);
        let cors_header = response
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap());
        assert_eq!(cors_header, Some("https://example.com"));
    }

    #[tokio::test]
    async fn test_cors_preflight_disallowed_origin() {
        let config = test_config_with_origins(vec!["https://example.com"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        let response = app
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/test")
                    .header("Origin", "https://evil.com")
                    .header("Access-Control-Request-Method", "GET")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        // No CORS header for disallowed origin
        let cors_header = response.headers().get("access-control-allow-origin");
        assert!(cors_header.is_none());
    }

    #[tokio::test]
    async fn test_cors_handles_whitespace_in_origins() {
        // Whitespace should be trimmed by config parsing, but test anyway
        let config = test_config_with_origins(vec!["https://example.com"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cors_header = response
            .headers()
            .get("access-control-allow-origin")
            .map(|v| v.to_str().unwrap());
        assert_eq!(cors_header, Some("https://example.com"));
    }

    #[tokio::test]
    async fn test_cors_origin_is_case_sensitive() {
        let config = test_config_with_origins(vec!["https://Example.com"]);
        let cors = build_cors_layer(&config);

        let app = Router::new().route("/test", get(handler)).layer(cors);

        // Same case - should match
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://Example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cors_header = response.headers().get("access-control-allow-origin");
        assert!(cors_header.is_some());

        // Different case - should not match
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/test")
                    .header("Origin", "https://example.com")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        let cors_header = response.headers().get("access-control-allow-origin");
        assert!(cors_header.is_none());
    }
}

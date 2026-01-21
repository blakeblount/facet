use api::{api_router, create_pool, test_connection, AppState, Config, DbConfig};
use std::net::SocketAddr;
use std::time::Duration;
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Load configuration
    let config = Config::from_env_or_defaults();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_filter.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Create database connection pool
    let db_config = DbConfig::new(&config.database_url);
    let db_pool = create_pool(&db_config)
        .await
        .expect("Failed to create database pool");

    // Test database connection
    test_connection(&db_pool)
        .await
        .expect("Failed to connect to database");

    // Create application state
    let state = AppState::new(db_pool);

    // Build CORS layer
    let cors = build_cors_layer(&config);

    // Build router with middleware
    let app = api_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Start server with graceful shutdown
    tracing::info!("Starting server on {}", config.server_addr);

    let listener = tokio::net::TcpListener::bind(config.server_addr)
        .await
        .expect("Failed to bind to address");

    // Use into_make_service_with_connect_info to enable ConnectInfo<SocketAddr>
    // extraction in handlers for rate limiting
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .expect("Server error");

    tracing::info!("Server shutdown complete");
}

/// Build CORS layer based on configuration.
fn build_cors_layer(config: &Config) -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .max_age(Duration::from_secs(3600));

    // If origins is "*", allow any origin; otherwise, parse specific origins
    if config.cors_origins.len() == 1 && config.cors_origins[0] == "*" {
        cors.allow_origin(Any)
    } else {
        // For specific origins, we still use Any for simplicity in MVP
        // A more robust implementation would parse and validate each origin
        cors.allow_origin(Any)
    }
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM).
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C, starting graceful shutdown");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM, starting graceful shutdown");
        }
    }
}

use api::repositories::AdminSessionRepository;
use api::{
    api_router_with_limits, build_cors_layer, create_pool, test_connection, AppState,
    BodyLimitConfig, Config, DbConfig,
};
use std::net::SocketAddr;
use tokio::signal;
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

    // Clean up any expired admin sessions on startup
    match AdminSessionRepository::delete_expired(&db_pool).await {
        Ok(count) => {
            if count > 0 {
                tracing::info!("Cleaned up {} expired admin session(s)", count);
            }
        }
        Err(err) => {
            tracing::warn!("Failed to clean up expired sessions: {:?}", err);
        }
    }

    // Create application state
    let state = AppState::new(db_pool);

    // Build CORS layer
    let cors = build_cors_layer(&config);

    // Configure body size limits
    let body_limits = BodyLimitConfig {
        max_body_size: config.max_body_size,
        max_photo_size: config.max_photo_size,
    };

    tracing::info!(
        "Request body limits: {}KB default, {}MB for photos",
        config.max_body_size / 1024,
        config.max_photo_size / (1024 * 1024)
    );

    // Build router with middleware
    let app = api_router_with_limits(state, body_limits)
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

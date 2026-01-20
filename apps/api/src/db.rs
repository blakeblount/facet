//! Database connection pool and utilities.
//!
//! This module provides the PostgreSQL connection pool setup using sqlx.

use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// Database pool configuration options.
#[derive(Debug, Clone)]
pub struct DbConfig {
    /// PostgreSQL connection URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Minimum number of connections to keep idle
    pub min_connections: u32,
    /// Maximum time to wait for a connection
    pub acquire_timeout: Duration,
    /// Maximum idle time for a connection
    pub idle_timeout: Duration,
}

impl Default for DbConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        }
    }
}

impl DbConfig {
    /// Create a new DbConfig with the given database URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set the maximum number of connections.
    pub fn max_connections(mut self, max: u32) -> Self {
        self.max_connections = max;
        self
    }

    /// Set the minimum number of idle connections.
    pub fn min_connections(mut self, min: u32) -> Self {
        self.min_connections = min;
        self
    }

    /// Set the connection acquire timeout.
    pub fn acquire_timeout(mut self, timeout: Duration) -> Self {
        self.acquire_timeout = timeout;
        self
    }

    /// Set the idle timeout for connections.
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.idle_timeout = timeout;
        self
    }
}

/// Create a PostgreSQL connection pool with the given configuration.
///
/// # Errors
///
/// Returns an error if the pool cannot be created or initial connections fail.
pub async fn create_pool(config: &DbConfig) -> Result<PgPool, sqlx::Error> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(&config.url)
        .await?;

    tracing::info!(
        "Database pool created (max_connections={}, min_connections={})",
        config.max_connections,
        config.min_connections
    );

    Ok(pool)
}

/// Test the database connection by executing a simple query.
///
/// # Errors
///
/// Returns an error if the connection test fails.
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    tracing::info!("Database connection verified");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_config_default() {
        let config = DbConfig::default();
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.acquire_timeout, Duration::from_secs(30));
        assert_eq!(config.idle_timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_db_config_new() {
        let config = DbConfig::new("postgres://localhost/test");
        assert_eq!(config.url, "postgres://localhost/test");
    }

    #[test]
    fn test_db_config_builder() {
        let config = DbConfig::new("postgres://localhost/test")
            .max_connections(20)
            .min_connections(5)
            .acquire_timeout(Duration::from_secs(60))
            .idle_timeout(Duration::from_secs(300));

        assert_eq!(config.max_connections, 20);
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.acquire_timeout, Duration::from_secs(60));
        assert_eq!(config.idle_timeout, Duration::from_secs(300));
    }
}

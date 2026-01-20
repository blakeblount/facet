//! Application configuration from environment variables.

use crate::storage::StorageConfig;
use std::env;
use std::net::SocketAddr;

/// Application configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct Config {
    /// Server host and port
    pub server_addr: SocketAddr,

    /// Database connection URL
    pub database_url: String,

    /// S3-compatible storage configuration
    pub s3_endpoint: Option<String>,
    pub s3_bucket: String,
    pub s3_access_key: Option<String>,
    pub s3_secret_key: Option<String>,

    /// CORS allowed origins (comma-separated)
    pub cors_origins: Vec<String>,

    /// Log level filter
    pub log_filter: String,
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Required variables:
    /// - `DATABASE_URL`: PostgreSQL connection string
    /// - `S3_BUCKET`: S3 bucket name for photo storage
    ///
    /// Optional variables:
    /// - `HOST`: Server host (default: 0.0.0.0)
    /// - `PORT`: Server port (default: 3001)
    /// - `S3_ENDPOINT`: S3 endpoint URL (default: AWS S3)
    /// - `S3_ACCESS_KEY`: S3 access key
    /// - `S3_SECRET_KEY`: S3 secret key
    /// - `CORS_ORIGINS`: Comma-separated allowed origins (default: *)
    /// - `RUST_LOG`: Log level filter (default: api=debug,tower_http=debug)
    pub fn from_env() -> Result<Self, ConfigError> {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .map_err(|_| ConfigError::InvalidPort)?;

        let server_addr = format!("{}:{}", host, port)
            .parse()
            .map_err(|_| ConfigError::InvalidAddress)?;

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| ConfigError::Missing("DATABASE_URL".to_string()))?;

        let s3_bucket =
            env::var("S3_BUCKET").map_err(|_| ConfigError::Missing("S3_BUCKET".to_string()))?;

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let log_filter =
            env::var("RUST_LOG").unwrap_or_else(|_| "api=debug,tower_http=debug".to_string());

        Ok(Config {
            server_addr,
            database_url,
            s3_endpoint: env::var("S3_ENDPOINT").ok(),
            s3_bucket,
            s3_access_key: env::var("S3_ACCESS_KEY").ok(),
            s3_secret_key: env::var("S3_SECRET_KEY").ok(),
            cors_origins,
            log_filter,
        })
    }

    /// Load configuration, allowing missing optional values for development.
    /// This is useful when running without a full environment set up.
    pub fn from_env_or_defaults() -> Self {
        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<u16>()
            .unwrap_or(3001);

        let server_addr = format!("{}:{}", host, port)
            .parse()
            .unwrap_or_else(|_| SocketAddr::from(([0, 0, 0, 0], 3001)));

        let cors_origins = env::var("CORS_ORIGINS")
            .unwrap_or_else(|_| "*".to_string())
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();

        let log_filter =
            env::var("RUST_LOG").unwrap_or_else(|_| "api=debug,tower_http=debug".to_string());

        Config {
            server_addr,
            database_url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/facet_dev".to_string()),
            s3_endpoint: env::var("S3_ENDPOINT").ok(),
            s3_bucket: env::var("S3_BUCKET").unwrap_or_else(|_| "facet-dev".to_string()),
            s3_access_key: env::var("S3_ACCESS_KEY").ok(),
            s3_secret_key: env::var("S3_SECRET_KEY").ok(),
            cors_origins,
            log_filter,
        }
    }

    /// Create a StorageConfig from this Config.
    ///
    /// Uses the S3 configuration values (endpoint, bucket, credentials)
    /// to build a StorageConfig for initializing the storage client.
    pub fn storage_config(&self) -> StorageConfig {
        let mut config = StorageConfig::new(self.s3_bucket.clone());

        if let Some(endpoint) = &self.s3_endpoint {
            config = config.with_endpoint(endpoint);
            // Extract region from endpoint for S3-compatible services
            // e.g., "https://nyc3.digitaloceanspaces.com" -> "nyc3"
            if let Some(region) = extract_region_from_endpoint(endpoint) {
                config = config.with_region(region);
            }
        }

        if let (Some(access_key), Some(secret_key)) = (&self.s3_access_key, &self.s3_secret_key) {
            config = config.with_credentials(access_key, secret_key);
        }

        config
    }
}

/// Extract region from S3-compatible endpoint URL.
/// For DigitalOcean Spaces: "https://nyc3.digitaloceanspaces.com" -> "nyc3"
fn extract_region_from_endpoint(endpoint: &str) -> Option<String> {
    // Remove protocol
    let without_protocol = endpoint
        .strip_prefix("https://")
        .or_else(|| endpoint.strip_prefix("http://"))
        .unwrap_or(endpoint);

    // Get the first part before the dot
    without_protocol
        .split('.')
        .next()
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
}

/// Configuration loading errors.
#[derive(Debug)]
pub enum ConfigError {
    Missing(String),
    InvalidPort,
    InvalidAddress,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing(var) => {
                write!(f, "Missing required environment variable: {}", var)
            }
            ConfigError::InvalidPort => write!(f, "Invalid PORT value"),
            ConfigError::InvalidAddress => write!(f, "Invalid server address"),
        }
    }
}

impl std::error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_env_or_defaults() {
        // Should not panic even with no env vars set
        let config = Config::from_env_or_defaults();
        assert_eq!(config.server_addr.port(), 3001);
        assert!(!config.cors_origins.is_empty());
    }

    #[test]
    fn test_extract_region_from_endpoint() {
        // DigitalOcean Spaces
        assert_eq!(
            extract_region_from_endpoint("https://nyc3.digitaloceanspaces.com"),
            Some("nyc3".to_string())
        );
        assert_eq!(
            extract_region_from_endpoint("https://sfo2.digitaloceanspaces.com"),
            Some("sfo2".to_string())
        );

        // Without protocol
        assert_eq!(
            extract_region_from_endpoint("nyc3.digitaloceanspaces.com"),
            Some("nyc3".to_string())
        );

        // HTTP protocol
        assert_eq!(
            extract_region_from_endpoint("http://localhost:9000"),
            Some("localhost:9000".to_string())
        );

        // Empty or invalid
        assert_eq!(extract_region_from_endpoint(""), None);
    }

    #[test]
    fn test_storage_config_from_config() {
        let config = Config::from_env_or_defaults();
        let storage_config = config.storage_config();

        // Should use the bucket from config
        assert_eq!(storage_config.bucket, config.s3_bucket);
    }
}

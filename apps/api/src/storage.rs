//! S3-compatible storage client for photo storage.
//!
//! This module provides functionality for uploading, downloading, and managing
//! photos in an S3-compatible object storage service (like DigitalOcean Spaces).

use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{
    config::{Builder, Region},
    presigning::PresigningConfig,
    primitives::ByteStream,
    Client,
};
use std::time::Duration;
use thiserror::Error;

/// Default presigned URL expiration time (1 hour).
const DEFAULT_URL_EXPIRATION_SECS: u64 = 3600;

/// Storage-specific errors.
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("Failed to upload file: {0}")]
    UploadError(String),

    #[error("Failed to download file: {0}")]
    DownloadError(String),

    #[error("Failed to delete file: {0}")]
    DeleteError(String),

    #[error("Failed to generate signed URL: {0}")]
    SignedUrlError(String),

    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Invalid configuration: {0}")]
    ConfigError(String),
}

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

/// Configuration for the storage client.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// S3 endpoint URL (for S3-compatible services like DigitalOcean Spaces).
    pub endpoint: Option<String>,
    /// S3 bucket name.
    pub bucket: String,
    /// AWS region (or region for S3-compatible services).
    pub region: String,
    /// Access key ID.
    pub access_key: Option<String>,
    /// Secret access key.
    pub secret_key: Option<String>,
}

impl StorageConfig {
    /// Create a new storage configuration.
    pub fn new(bucket: String) -> Self {
        Self {
            endpoint: None,
            bucket,
            region: "us-east-1".to_string(),
            access_key: None,
            secret_key: None,
        }
    }

    /// Set the S3 endpoint (for S3-compatible services).
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set the region.
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = region.into();
        self
    }

    /// Set the credentials.
    pub fn with_credentials(
        mut self,
        access_key: impl Into<String>,
        secret_key: impl Into<String>,
    ) -> Self {
        self.access_key = Some(access_key.into());
        self.secret_key = Some(secret_key.into());
        self
    }
}

/// S3-compatible storage client.
#[derive(Debug, Clone)]
pub struct StorageClient {
    client: Client,
    bucket: String,
}

impl StorageClient {
    /// Create a new storage client from configuration.
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let mut builder = Builder::new()
            .behavior_version(BehaviorVersion::latest())
            .region(Region::new(config.region));

        // Set endpoint for S3-compatible services
        if let Some(endpoint) = &config.endpoint {
            builder = builder.endpoint_url(endpoint);
            // Force path-style addressing for S3-compatible services
            builder = builder.force_path_style(true);
        }

        // Set credentials if provided
        if let (Some(access_key), Some(secret_key)) = (&config.access_key, &config.secret_key) {
            let credentials = Credentials::new(
                access_key.clone(),
                secret_key.clone(),
                None, // session token
                None, // expiry
                "facet-storage",
            );
            builder = builder.credentials_provider(credentials);
        }

        let s3_config = builder.build();
        let client = Client::from_conf(s3_config);

        Ok(Self {
            client,
            bucket: config.bucket,
        })
    }

    /// Upload a file to storage.
    ///
    /// # Arguments
    /// * `key` - The object key (path) in the bucket
    /// * `data` - The file content as bytes
    /// * `content_type` - The MIME type of the file (e.g., "image/jpeg")
    ///
    /// # Returns
    /// The object key on success.
    pub async fn upload(
        &self,
        key: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> StorageResult<String> {
        let body = ByteStream::from(data);

        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| StorageError::UploadError(e.to_string()))?;

        Ok(key.to_string())
    }

    /// Download a file from storage.
    ///
    /// # Arguments
    /// * `key` - The object key (path) in the bucket
    ///
    /// # Returns
    /// The file content as bytes.
    pub async fn download(&self, key: &str) -> StorageResult<Vec<u8>> {
        let response = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| {
                let err_str = e.to_string();
                if err_str.contains("NoSuchKey") || err_str.contains("not found") {
                    StorageError::NotFound(key.to_string())
                } else {
                    StorageError::DownloadError(err_str)
                }
            })?;

        let data = response
            .body
            .collect()
            .await
            .map_err(|e| StorageError::DownloadError(e.to_string()))?
            .into_bytes()
            .to_vec();

        Ok(data)
    }

    /// Delete a file from storage.
    ///
    /// # Arguments
    /// * `key` - The object key (path) in the bucket
    pub async fn delete(&self, key: &str) -> StorageResult<()> {
        self.client
            .delete_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .map_err(|e| StorageError::DeleteError(e.to_string()))?;

        Ok(())
    }

    /// Generate a presigned URL for downloading a file.
    ///
    /// # Arguments
    /// * `key` - The object key (path) in the bucket
    /// * `expires_in` - How long the URL should be valid (optional, defaults to 1 hour)
    ///
    /// # Returns
    /// A presigned URL that allows temporary access to the file.
    pub async fn get_signed_url(
        &self,
        key: &str,
        expires_in: Option<Duration>,
    ) -> StorageResult<String> {
        let expiration = expires_in.unwrap_or(Duration::from_secs(DEFAULT_URL_EXPIRATION_SECS));

        let presigning_config = PresigningConfig::expires_in(expiration)
            .map_err(|e| StorageError::SignedUrlError(e.to_string()))?;

        let presigned_request = self
            .client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .presigned(presigning_config)
            .await
            .map_err(|e| StorageError::SignedUrlError(e.to_string()))?;

        Ok(presigned_request.uri().to_string())
    }

    /// Generate a presigned URL for uploading a file.
    ///
    /// # Arguments
    /// * `key` - The object key (path) where the file will be stored
    /// * `content_type` - The expected MIME type of the file
    /// * `expires_in` - How long the URL should be valid (optional, defaults to 1 hour)
    ///
    /// # Returns
    /// A presigned URL that allows temporary upload access.
    pub async fn get_upload_signed_url(
        &self,
        key: &str,
        content_type: &str,
        expires_in: Option<Duration>,
    ) -> StorageResult<String> {
        let expiration = expires_in.unwrap_or(Duration::from_secs(DEFAULT_URL_EXPIRATION_SECS));

        let presigning_config = PresigningConfig::expires_in(expiration)
            .map_err(|e| StorageError::SignedUrlError(e.to_string()))?;

        let presigned_request = self
            .client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .content_type(content_type)
            .presigned(presigning_config)
            .await
            .map_err(|e| StorageError::SignedUrlError(e.to_string()))?;

        Ok(presigned_request.uri().to_string())
    }

    /// Check if a file exists in storage.
    ///
    /// # Arguments
    /// * `key` - The object key (path) in the bucket
    ///
    /// # Returns
    /// `true` if the file exists, `false` otherwise.
    pub async fn exists(&self, key: &str) -> StorageResult<bool> {
        match self
            .client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("NotFound") || err_str.contains("not found") {
                    Ok(false)
                } else {
                    Err(StorageError::DownloadError(err_str))
                }
            }
        }
    }

    /// Get the bucket name.
    pub fn bucket(&self) -> &str {
        &self.bucket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_config_builder() {
        let config = StorageConfig::new("my-bucket".to_string())
            .with_endpoint("https://nyc3.digitaloceanspaces.com")
            .with_region("nyc3")
            .with_credentials("access-key", "secret-key");

        assert_eq!(config.bucket, "my-bucket");
        assert_eq!(
            config.endpoint,
            Some("https://nyc3.digitaloceanspaces.com".to_string())
        );
        assert_eq!(config.region, "nyc3");
        assert_eq!(config.access_key, Some("access-key".to_string()));
        assert_eq!(config.secret_key, Some("secret-key".to_string()));
    }

    #[test]
    fn test_storage_config_defaults() {
        let config = StorageConfig::new("test-bucket".to_string());

        assert_eq!(config.bucket, "test-bucket");
        assert!(config.endpoint.is_none());
        assert_eq!(config.region, "us-east-1");
        assert!(config.access_key.is_none());
        assert!(config.secret_key.is_none());
    }

    #[test]
    fn test_storage_error_display() {
        let err = StorageError::UploadError("connection failed".to_string());
        assert_eq!(err.to_string(), "Failed to upload file: connection failed");

        let err = StorageError::NotFound("photos/123.jpg".to_string());
        assert_eq!(err.to_string(), "File not found: photos/123.jpg");
    }
}

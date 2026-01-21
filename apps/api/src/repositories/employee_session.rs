//! Employee session repository for database operations.
//!
//! Similar to AdminSessionRepository but includes employee_id binding
//! to prevent header-based employee impersonation.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{Duration, Utc};
use rand::RngCore;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::employee_session::{EmployeeSession, EmployeeSessionResponse};

/// Default session duration in minutes (8 hours for a work shift).
const DEFAULT_SESSION_DURATION_MINUTES: i64 = 8 * 60;

/// Repository for employee session database operations.
pub struct EmployeeSessionRepository;

impl EmployeeSessionRepository {
    /// Generate a cryptographically secure session token.
    ///
    /// Creates a 256-bit random token encoded as base64url (no padding).
    pub fn generate_token() -> String {
        let mut token_bytes = [0u8; 32]; // 256 bits
        rand::thread_rng().fill_bytes(&mut token_bytes);
        URL_SAFE_NO_PAD.encode(token_bytes)
    }

    /// Create a new employee session.
    ///
    /// Generates a secure token and stores the session in the database,
    /// bound to the specified employee.
    pub async fn create(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<EmployeeSessionResponse, AppError> {
        Self::create_with_duration(pool, employee_id, DEFAULT_SESSION_DURATION_MINUTES).await
    }

    /// Create a new employee session with a custom duration.
    pub async fn create_with_duration(
        pool: &PgPool,
        employee_id: Uuid,
        duration_minutes: i64,
    ) -> Result<EmployeeSessionResponse, AppError> {
        let token = Self::generate_token();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(duration_minutes);

        let session = sqlx::query_as::<_, EmployeeSession>(
            r#"
            INSERT INTO employee_sessions (employee_id, session_token, expires_at)
            VALUES ($1, $2, $3)
            RETURNING session_id, employee_id, session_token, created_at, expires_at, last_activity_at
            "#,
        )
        .bind(employee_id)
        .bind(&token)
        .bind(expires_at)
        .fetch_one(pool)
        .await?;

        Ok(EmployeeSessionResponse {
            employee_id: session.employee_id,
            session_token: session.session_token,
            expires_at: session.expires_at,
        })
    }

    /// Find a session by its token.
    ///
    /// Returns None if the session doesn't exist.
    pub async fn find_by_token(
        pool: &PgPool,
        token: &str,
    ) -> Result<Option<EmployeeSession>, AppError> {
        let session = sqlx::query_as::<_, EmployeeSession>(
            r#"
            SELECT session_id, employee_id, session_token, created_at, expires_at, last_activity_at
            FROM employee_sessions
            WHERE session_token = $1
            "#,
        )
        .bind(token)
        .fetch_optional(pool)
        .await?;

        Ok(session)
    }

    /// Verify a session token and update last activity (sliding expiration).
    ///
    /// Returns the session if valid and not expired, None otherwise.
    /// If valid, updates last_activity_at and extends expires_at.
    pub async fn verify_and_touch(
        pool: &PgPool,
        token: &str,
    ) -> Result<Option<EmployeeSession>, AppError> {
        let session = Self::find_by_token(pool, token).await?;

        match session {
            Some(s) if !s.is_expired() => {
                // Update last activity and extend expiration
                let new_expires_at =
                    Utc::now() + Duration::minutes(DEFAULT_SESSION_DURATION_MINUTES);
                sqlx::query(
                    r#"
                    UPDATE employee_sessions
                    SET last_activity_at = NOW(), expires_at = $1
                    WHERE session_id = $2
                    "#,
                )
                .bind(new_expires_at)
                .bind(s.session_id)
                .execute(pool)
                .await?;

                // Return the session with updated values
                Ok(Some(EmployeeSession {
                    last_activity_at: Utc::now(),
                    expires_at: new_expires_at,
                    ..s
                }))
            }
            Some(_) => {
                // Session exists but is expired
                Ok(None)
            }
            None => Ok(None),
        }
    }

    /// Delete a session by its token (logout).
    pub async fn delete_by_token(pool: &PgPool, token: &str) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM employee_sessions
            WHERE session_token = $1
            "#,
        )
        .bind(token)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete a session by its ID.
    pub async fn delete(pool: &PgPool, session_id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM employee_sessions
            WHERE session_id = $1
            "#,
        )
        .bind(session_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete all sessions for an employee (e.g., when deactivating).
    ///
    /// Returns the number of sessions deleted.
    pub async fn delete_all_for_employee(
        pool: &PgPool,
        employee_id: Uuid,
    ) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM employee_sessions
            WHERE employee_id = $1
            "#,
        )
        .bind(employee_id)
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// Delete all expired sessions (cleanup).
    ///
    /// Returns the number of sessions deleted.
    pub async fn delete_expired(pool: &PgPool) -> Result<u64, AppError> {
        let result = sqlx::query(
            r#"
            DELETE FROM employee_sessions
            WHERE expires_at < NOW()
            "#,
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_length() {
        let token = EmployeeSessionRepository::generate_token();
        // 32 bytes -> ~43 chars in base64url without padding
        assert!(token.len() >= 40);
        assert!(token.len() <= 50);
    }

    #[test]
    fn test_generate_token_uniqueness() {
        let token1 = EmployeeSessionRepository::generate_token();
        let token2 = EmployeeSessionRepository::generate_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_token_is_base64url() {
        let token = EmployeeSessionRepository::generate_token();
        // Should only contain base64url characters
        assert!(token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }
}

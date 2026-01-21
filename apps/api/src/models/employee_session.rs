//! Employee session model for secure session-based authentication.
//!
//! Similar to admin sessions but includes an employee_id to bind the session
//! to a specific employee, preventing header-based impersonation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An employee session stored in the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EmployeeSession {
    /// Unique session identifier
    pub session_id: Uuid,
    /// The employee this session belongs to
    pub employee_id: Uuid,
    /// Cryptographically random session token (256-bit, base64url encoded)
    pub session_token: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session expires
    pub expires_at: DateTime<Utc>,
    /// Last activity timestamp (for sliding expiration)
    pub last_activity_at: DateTime<Utc>,
}

impl EmployeeSession {
    /// Check if this session has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Data needed to create a new employee session.
pub struct CreateEmployeeSession {
    /// The employee ID to bind to this session
    pub employee_id: Uuid,
    /// The session token
    pub session_token: String,
    /// Session duration in minutes
    pub duration_minutes: i64,
}

/// Response returned when a session is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeSessionResponse {
    /// The employee's unique identifier
    pub employee_id: Uuid,
    /// The session token to use for subsequent requests
    pub session_token: String,
    /// When the session expires (ISO 8601 format)
    pub expires_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_not_expired() {
        let session = EmployeeSession {
            session_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            session_token: "test_token".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
            last_activity_at: Utc::now(),
        };
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_expired() {
        let session = EmployeeSession {
            session_id: Uuid::new_v4(),
            employee_id: Uuid::new_v4(),
            session_token: "test_token".to_string(),
            created_at: Utc::now() - chrono::Duration::hours(9),
            expires_at: Utc::now() - chrono::Duration::hours(1),
            last_activity_at: Utc::now() - chrono::Duration::hours(9),
        };
        assert!(session.is_expired());
    }

    #[test]
    fn test_employee_session_response_serialization() {
        let response = EmployeeSessionResponse {
            employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            session_token: "test_token_abc123".to_string(),
            expires_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"employee_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"session_token\":\"test_token_abc123\""));
        assert!(json.contains("\"expires_at\":"));
    }
}

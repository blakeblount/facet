//! Admin session model for secure session-based authentication.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An admin session stored in the database.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AdminSession {
    /// Unique session identifier
    pub session_id: Uuid,
    /// Cryptographically random session token (256-bit, base64url encoded)
    pub session_token: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session expires
    pub expires_at: DateTime<Utc>,
    /// Last activity timestamp (for sliding expiration)
    pub last_activity_at: DateTime<Utc>,
}

impl AdminSession {
    /// Check if this session has expired.
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }
}

/// Data needed to create a new admin session.
pub struct CreateAdminSession {
    /// The session token
    pub session_token: String,
    /// Session duration in minutes
    pub duration_minutes: i64,
}

/// Response returned when a session is created.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminSessionResponse {
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
        let session = AdminSession {
            session_id: Uuid::new_v4(),
            session_token: "test_token".to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::minutes(30),
            last_activity_at: Utc::now(),
        };
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_expired() {
        let session = AdminSession {
            session_id: Uuid::new_v4(),
            session_token: "test_token".to_string(),
            created_at: Utc::now() - chrono::Duration::hours(1),
            expires_at: Utc::now() - chrono::Duration::minutes(30),
            last_activity_at: Utc::now() - chrono::Duration::hours(1),
        };
        assert!(session.is_expired());
    }

    #[test]
    fn test_admin_session_response_serialization() {
        let response = AdminSessionResponse {
            session_token: "test_token_abc123".to_string(),
            expires_at: Utc::now(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"session_token\":\"test_token_abc123\""));
        assert!(json.contains("\"expires_at\":"));
    }
}

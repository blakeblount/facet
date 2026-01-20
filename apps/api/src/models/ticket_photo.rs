//! Ticket photo model and related types.
//!
//! Photos are attached to tickets and stored in S3-compatible storage.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Full ticket photo entity with all fields.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TicketPhoto {
    pub photo_id: Uuid,
    pub ticket_id: Uuid,
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i32,
    pub uploaded_by: Uuid,
    pub uploaded_at: DateTime<Utc>,
}

/// Summary view of a ticket photo (for listing).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TicketPhotoSummary {
    pub photo_id: Uuid,
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i32,
    pub uploaded_at: DateTime<Utc>,
}

/// Input for creating a new ticket photo record.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTicketPhoto {
    pub ticket_id: Uuid,
    pub storage_key: String,
    pub content_type: String,
    pub size_bytes: i32,
    pub uploaded_by: Uuid,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ticket_photo_deserialize() {
        let json = r#"{
            "ticket_id": "00000000-0000-0000-0000-000000000001",
            "storage_key": "photos/ticket-123/image.jpg",
            "content_type": "image/jpeg",
            "size_bytes": 102400,
            "uploaded_by": "00000000-0000-0000-0000-000000000002"
        }"#;
        let input: CreateTicketPhoto = serde_json::from_str(json).unwrap();
        assert_eq!(input.storage_key, "photos/ticket-123/image.jpg");
        assert_eq!(input.content_type, "image/jpeg");
        assert_eq!(input.size_bytes, 102400);
    }

    #[test]
    fn test_ticket_photo_serialize() {
        let photo = TicketPhoto {
            photo_id: Uuid::nil(),
            ticket_id: Uuid::nil(),
            storage_key: "photos/test.jpg".to_string(),
            content_type: "image/jpeg".to_string(),
            size_bytes: 1024,
            uploaded_by: Uuid::nil(),
            uploaded_at: DateTime::from_timestamp(0, 0).unwrap(),
        };

        let json = serde_json::to_string(&photo).unwrap();
        assert!(json.contains("photos/test.jpg"));
        assert!(json.contains("image/jpeg"));
    }
}

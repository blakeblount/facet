//! Ticket request handlers.

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    CreateCustomer, CreateFieldHistory, CreateStatusHistory, CreateTicket, Ticket, TicketStatus,
    UpdateTicket,
};
use crate::repositories::{
    CustomerRepository, EmployeeRepository, FieldHistoryRepository, StatusHistoryRepository,
    TicketRepository,
};
use crate::response::ApiResponse;
use crate::routes::AppState;
use crate::services::pdf::{generate_receipt_pdf, ReceiptData};

/// Customer info for inline creation during ticket intake.
#[derive(Debug, Clone, Deserialize)]
pub struct InlineCustomer {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// Request body for creating a new ticket.
///
/// Either `customer_id` (for existing customer) or `customer` (for inline creation)
/// must be provided, but not both.
#[derive(Debug, Clone, Deserialize)]
pub struct CreateTicketRequest {
    /// Existing customer ID (mutually exclusive with `customer`)
    pub customer_id: Option<Uuid>,

    /// Inline customer creation (mutually exclusive with `customer_id`)
    pub customer: Option<InlineCustomer>,

    /// Item type (e.g., "ring", "necklace")
    pub item_type: Option<String>,

    /// Description of the item (required)
    pub item_description: String,

    /// Notes about the item's condition (required)
    pub condition_notes: String,

    /// Description of requested work (required)
    pub requested_work: String,

    /// Whether this is a rush job
    #[serde(default)]
    pub is_rush: bool,

    /// Promised completion date
    pub promise_date: Option<NaiveDate>,

    /// Storage location ID (required)
    pub storage_location_id: Uuid,

    /// Quoted amount for the work
    pub quote_amount: Option<Decimal>,
}

/// Response for a created ticket.
#[derive(Debug, Clone, Serialize)]
pub struct CreateTicketResponse {
    /// The created ticket
    #[serde(flatten)]
    pub ticket: Ticket,

    /// URL to download the receipt PDF
    pub receipt_url: String,

    /// URL to download the label PDF
    pub label_url: String,
}

/// Extract employee ID from X-Employee-ID header.
fn extract_employee_id(headers: &HeaderMap) -> Result<Uuid, AppError> {
    let header_value = headers
        .get("X-Employee-ID")
        .ok_or_else(|| AppError::validation("X-Employee-ID header is required"))?;

    let header_str = header_value
        .to_str()
        .map_err(|_| AppError::validation("Invalid X-Employee-ID header value"))?;

    Uuid::parse_str(header_str)
        .map_err(|_| AppError::validation("X-Employee-ID must be a valid UUID"))
}

/// POST /api/v1/tickets - Create a new ticket.
pub async fn create_ticket(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<CreateTicketRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Extract and validate employee ID from header
    let employee_id = extract_employee_id(&headers)?;

    // Verify employee exists and is active
    let employee = EmployeeRepository::find_active_by_id(&state.db, employee_id)
        .await?
        .ok_or_else(|| AppError::validation("Employee not found or inactive"))?;

    // 2. Validate request - must have either customer_id OR customer, not both
    let customer_id = match (&body.customer_id, &body.customer) {
        (Some(id), None) => {
            // Verify existing customer exists
            CustomerRepository::find_by_id(&state.db, *id)
                .await?
                .ok_or_else(|| AppError::not_found("Customer not found"))?;
            *id
        }
        (None, Some(inline)) => {
            // Create new customer inline
            let new_customer = CustomerRepository::create(
                &state.db,
                CreateCustomer {
                    name: inline.name.clone(),
                    phone: inline.phone.clone(),
                    email: inline.email.clone(),
                },
            )
            .await?;
            new_customer.customer_id
        }
        (Some(_), Some(_)) => {
            return Err(AppError::validation(
                "Provide either customer_id or customer, not both",
            ));
        }
        (None, None) => {
            return Err(AppError::validation(
                "Either customer_id or customer is required",
            ));
        }
    };

    // 3. Validate required fields
    if body.item_description.trim().is_empty() {
        return Err(AppError::validation("item_description is required"));
    }
    if body.condition_notes.trim().is_empty() {
        return Err(AppError::validation("condition_notes is required"));
    }
    if body.requested_work.trim().is_empty() {
        return Err(AppError::validation("requested_work is required"));
    }

    // 4. Create the ticket
    let create_ticket = CreateTicket {
        customer_id,
        item_type: body.item_type,
        item_description: body.item_description,
        condition_notes: body.condition_notes,
        requested_work: body.requested_work,
        is_rush: body.is_rush,
        promise_date: body.promise_date,
        storage_location_id: body.storage_location_id,
        quote_amount: body.quote_amount,
        taken_in_by: employee.employee_id,
    };

    let ticket = TicketRepository::create(&state.db, create_ticket).await?;

    // 5. Create initial status history entry (null -> intake)
    StatusHistoryRepository::create(
        &state.db,
        CreateStatusHistory {
            ticket_id: ticket.ticket_id,
            from_status: None,
            to_status: TicketStatus::Intake,
            changed_by: employee.employee_id,
        },
    )
    .await?;

    // 6. Build response with print URLs
    let response = CreateTicketResponse {
        receipt_url: format!("/api/v1/tickets/{}/receipt.pdf", ticket.ticket_id),
        label_url: format!("/api/v1/tickets/{}/label.pdf", ticket.ticket_id),
        ticket,
    };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

/// Store settings data for PDF generation.
/// Fetched from database or uses defaults if not configured.
#[derive(Debug, Clone, sqlx::FromRow)]
struct StoreSettings {
    store_name: String,
    store_phone: Option<String>,
    store_address: Option<String>,
}

/// GET /api/v1/tickets/:ticket_id/receipt.pdf - Generate receipt PDF for a ticket.
pub async fn get_receipt_pdf(
    State(state): State<AppState>,
    Path(ticket_id): Path<Uuid>,
) -> Result<Response, AppError> {
    // 1. Find the ticket
    let ticket = TicketRepository::find_by_id(&state.db, ticket_id)
        .await?
        .ok_or_else(|| AppError::not_found("Ticket not found"))?;

    // 2. Find the customer
    let customer = CustomerRepository::find_by_id(&state.db, ticket.customer_id)
        .await?
        .ok_or_else(|| AppError::not_found("Customer not found"))?;

    // 3. Get store settings (or use defaults)
    let store_settings = sqlx::query_as::<_, StoreSettings>(
        r#"
        SELECT store_name, store_phone, store_address
        FROM store_settings
        LIMIT 1
        "#,
    )
    .fetch_optional(&state.db)
    .await?
    .unwrap_or(StoreSettings {
        store_name: "Jewelry Store".to_string(),
        store_phone: None,
        store_address: None,
    });

    // 4. Generate PDF
    let receipt_data = ReceiptData {
        ticket,
        customer,
        store_name: store_settings.store_name,
        store_phone: store_settings.store_phone,
        store_address: store_settings.store_address,
    };

    let pdf_bytes = generate_receipt_pdf(&receipt_data)?;

    // 5. Return PDF response
    let filename = format!("receipt-{}.pdf", receipt_data.ticket.friendly_code);
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/pdf")
        .header(
            header::CONTENT_DISPOSITION,
            format!("inline; filename=\"{}\"", filename),
        )
        .body(Body::from(pdf_bytes))
        .map_err(|e| AppError::server_error(format!("Failed to build response: {}", e)))?;

    Ok(response)
}

/// Helper to deserialize Option<Option<T>> where explicit null means Some(None).
///
/// This is used for fields that can be:
/// - Absent: The field is not in the JSON (outer Option is None)
/// - Null: The field is explicitly set to null (outer Option is Some(None))
/// - Present: The field has a value (outer Option is Some(Some(value)))
fn deserialize_optional_nullable<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    // This gets called only when the field is present in JSON
    // If it's null, we get Some(None); if it's a value, we get Some(Some(value))
    Option::<T>::deserialize(deserializer).map(Some)
}

/// Request body for updating a ticket.
///
/// All fields are optional - only provided fields will be updated.
/// For nullable fields (promise_date, quote_amount, actual_amount, worked_by_employee_id),
/// the outer Option indicates if the field was provided, and the inner Option indicates
/// if it should be set to null.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTicketRequest {
    /// Item type (e.g., "ring", "necklace")
    pub item_type: Option<String>,

    /// Description of the item
    pub item_description: Option<String>,

    /// Notes about the item's condition
    pub condition_notes: Option<String>,

    /// Description of requested work
    pub requested_work: Option<String>,

    /// Whether this is a rush job
    pub is_rush: Option<bool>,

    /// Promised completion date (null to clear)
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub promise_date: Option<Option<NaiveDate>>,

    /// Storage location ID
    pub storage_location_id: Option<Uuid>,

    /// Quoted amount for the work (null to clear)
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub quote_amount: Option<Option<Decimal>>,

    /// Actual amount charged (null to clear)
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub actual_amount: Option<Option<Decimal>>,

    /// Employee who worked on the ticket (null to clear)
    #[serde(default, deserialize_with = "deserialize_optional_nullable")]
    pub worked_by_employee_id: Option<Option<Uuid>>,
}

/// Check if admin PIN is valid.
///
/// For now this is a placeholder that always returns false.
/// Will be implemented properly when password hashing utilities are available.
async fn verify_admin_pin(pool: &sqlx::PgPool, pin: &str) -> Result<bool, AppError> {
    // TODO: Implement actual PIN verification when password hashing is available
    // For now, check if there's a store_settings row with a matching admin_pin_hash
    // This is a temporary implementation that won't work until argon2 hashing is added

    // Get the admin_pin_hash from store_settings
    let result =
        sqlx::query_scalar::<_, String>("SELECT admin_pin_hash FROM store_settings LIMIT 1")
            .fetch_optional(pool)
            .await?;

    // If no store_settings row, admin PIN verification fails
    if result.is_none() {
        return Ok(false);
    }

    // TODO: When argon2 is available, verify the PIN against the hash
    // For now, this always returns false - admin override won't work until
    // facet-052 (password hashing utilities) is implemented
    let _ = pin; // Suppress unused warning
    Ok(false)
}

/// PUT /api/v1/tickets/:ticket_id - Update a ticket.
pub async fn update_ticket(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
    Json(body): Json<UpdateTicketRequest>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Extract and validate employee ID from header
    let employee_id = extract_employee_id(&headers)?;

    // Verify employee exists and is active
    let employee = EmployeeRepository::find_active_by_id(&state.db, employee_id)
        .await?
        .ok_or_else(|| AppError::validation("Employee not found or inactive"))?;

    // 2. Find the ticket
    let existing_ticket = TicketRepository::find_by_id(&state.db, ticket_id)
        .await?
        .ok_or_else(|| AppError::not_found("Ticket not found"))?;

    // 3. Check if ticket is closed/archived
    if !existing_ticket.status.is_open() {
        // Check for admin override via X-Admin-PIN header
        let has_admin_override = if let Some(pin_header) = headers.get("X-Admin-PIN") {
            if let Ok(pin_str) = pin_header.to_str() {
                verify_admin_pin(&state.db, pin_str).await?
            } else {
                false
            }
        } else {
            false
        };

        if !has_admin_override {
            return Err(AppError::forbidden(
                "Cannot edit closed or archived ticket without admin override",
            ));
        }
    }

    // 4. Track field changes for audit trail
    let mut field_changes: Vec<CreateFieldHistory> = Vec::new();

    // Helper to record a field change
    macro_rules! track_change {
        ($field_name:expr, $old_val:expr, $new_val:expr) => {
            if let Some(ref new_value) = $new_val {
                let old_str = $old_val.as_ref().map(|v| v.to_string());
                let new_str = Some(new_value.to_string());
                if old_str != new_str {
                    field_changes.push(CreateFieldHistory {
                        ticket_id,
                        field_name: $field_name.to_string(),
                        old_value: old_str,
                        new_value: new_str,
                        changed_by: employee.employee_id,
                    });
                }
            }
        };
    }

    // Helper for nullable fields (Option<Option<T>>)
    macro_rules! track_nullable_change {
        ($field_name:expr, $old_val:expr, $new_val:expr) => {
            if let Some(ref new_outer) = $new_val {
                let old_str = $old_val.as_ref().map(|v| v.to_string());
                let new_str = new_outer.as_ref().map(|v| v.to_string());
                if old_str != new_str {
                    field_changes.push(CreateFieldHistory {
                        ticket_id,
                        field_name: $field_name.to_string(),
                        old_value: old_str,
                        new_value: new_str,
                        changed_by: employee.employee_id,
                    });
                }
            }
        };
    }

    // Track changes for each field
    track_change!("item_type", existing_ticket.item_type, body.item_type);
    track_change!(
        "item_description",
        Some(existing_ticket.item_description.clone()),
        body.item_description
    );
    track_change!(
        "condition_notes",
        Some(existing_ticket.condition_notes.clone()),
        body.condition_notes
    );
    track_change!(
        "requested_work",
        Some(existing_ticket.requested_work.clone()),
        body.requested_work
    );
    track_change!("is_rush", Some(existing_ticket.is_rush), body.is_rush);
    track_nullable_change!(
        "promise_date",
        existing_ticket.promise_date,
        body.promise_date
    );
    track_change!(
        "storage_location_id",
        Some(existing_ticket.storage_location_id),
        body.storage_location_id
    );
    track_nullable_change!(
        "quote_amount",
        existing_ticket.quote_amount,
        body.quote_amount
    );
    track_nullable_change!(
        "actual_amount",
        existing_ticket.actual_amount,
        body.actual_amount
    );
    track_nullable_change!(
        "worked_by",
        existing_ticket.worked_by,
        body.worked_by_employee_id
    );

    // 5. Build update struct
    let update = UpdateTicket {
        item_type: body.item_type,
        item_description: body.item_description,
        condition_notes: body.condition_notes,
        requested_work: body.requested_work,
        is_rush: body.is_rush,
        promise_date: body.promise_date,
        storage_location_id: body.storage_location_id,
        quote_amount: body.quote_amount,
        actual_amount: body.actual_amount,
        worked_by: body.worked_by_employee_id,
        last_modified_by: Some(employee.employee_id),
    };

    // 6. Update the ticket
    let updated_ticket = TicketRepository::update(&state.db, ticket_id, update).await?;

    // 7. Record field changes in history
    FieldHistoryRepository::create_batch(&state.db, field_changes).await?;

    // 8. Return updated ticket
    Ok(Json(ApiResponse::success(updated_ticket)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_ticket_request_deserialize() {
        let json = r#"{
            "customer_id": "550e8400-e29b-41d4-a716-446655440000",
            "item_description": "Gold ring",
            "condition_notes": "Minor scratches",
            "requested_work": "Resize and polish",
            "is_rush": false,
            "storage_location_id": "660e8400-e29b-41d4-a716-446655440000"
        }"#;

        let request: CreateTicketRequest = serde_json::from_str(json).unwrap();
        assert!(request.customer_id.is_some());
        assert!(request.customer.is_none());
        assert_eq!(request.item_description, "Gold ring");
        assert!(!request.is_rush);
    }

    #[test]
    fn test_create_ticket_request_with_inline_customer() {
        let json = r#"{
            "customer": {
                "name": "John Doe",
                "phone": "555-1234"
            },
            "item_description": "Silver necklace",
            "condition_notes": "Broken clasp",
            "requested_work": "Replace clasp",
            "storage_location_id": "660e8400-e29b-41d4-a716-446655440000"
        }"#;

        let request: CreateTicketRequest = serde_json::from_str(json).unwrap();
        assert!(request.customer_id.is_none());
        assert!(request.customer.is_some());

        let customer = request.customer.unwrap();
        assert_eq!(customer.name, "John Doe");
        assert_eq!(customer.phone, Some("555-1234".to_string()));
        assert!(customer.email.is_none());
    }

    #[test]
    fn test_is_rush_defaults_to_false() {
        let json = r#"{
            "customer_id": "550e8400-e29b-41d4-a716-446655440000",
            "item_description": "Test",
            "condition_notes": "Test",
            "requested_work": "Test",
            "storage_location_id": "660e8400-e29b-41d4-a716-446655440000"
        }"#;

        let request: CreateTicketRequest = serde_json::from_str(json).unwrap();
        assert!(!request.is_rush);
    }

    #[test]
    fn test_update_ticket_request_partial() {
        let json = r#"{
            "item_description": "Updated description",
            "quote_amount": 175.00
        }"#;

        let request: UpdateTicketRequest = serde_json::from_str(json).unwrap();
        assert_eq!(
            request.item_description,
            Some("Updated description".to_string())
        );
        assert!(request.quote_amount.is_some());
        assert!(request.item_type.is_none());
        assert!(request.is_rush.is_none());
    }

    #[test]
    fn test_update_ticket_request_nullable_fields() {
        // Test setting a nullable field to null explicitly
        let json = r#"{
            "promise_date": null,
            "quote_amount": null
        }"#;

        let request: UpdateTicketRequest = serde_json::from_str(json).unwrap();
        // When deserializing null, it becomes Some(None)
        assert_eq!(request.promise_date, Some(None));
        assert_eq!(request.quote_amount, Some(None));
    }

    #[test]
    fn test_update_ticket_request_with_values() {
        let json = r#"{
            "item_description": "Gold ring",
            "is_rush": true,
            "storage_location_id": "660e8400-e29b-41d4-a716-446655440000",
            "worked_by_employee_id": "770e8400-e29b-41d4-a716-446655440000"
        }"#;

        let request: UpdateTicketRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.item_description, Some("Gold ring".to_string()));
        assert_eq!(request.is_rush, Some(true));
        assert!(request.storage_location_id.is_some());
        assert!(request.worked_by_employee_id.is_some());
    }

    #[test]
    fn test_update_ticket_request_empty() {
        let json = r#"{}"#;

        let request: UpdateTicketRequest = serde_json::from_str(json).unwrap();
        assert!(request.item_type.is_none());
        assert!(request.item_description.is_none());
        assert!(request.condition_notes.is_none());
        assert!(request.requested_work.is_none());
        assert!(request.is_rush.is_none());
        assert!(request.promise_date.is_none());
        assert!(request.storage_location_id.is_none());
        assert!(request.quote_amount.is_none());
        assert!(request.actual_amount.is_none());
        assert!(request.worked_by_employee_id.is_none());
    }
}

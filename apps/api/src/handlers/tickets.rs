//! Ticket request handlers.

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{CreateCustomer, CreateStatusHistory, CreateTicket, Ticket, TicketStatus};
use crate::repositories::{
    CustomerRepository, EmployeeRepository, StatusHistoryRepository, TicketRepository,
};
use crate::response::ApiResponse;
use crate::routes::AppState;

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
}

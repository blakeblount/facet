//! Ticket request handlers.

use axum::{
    body::Body,
    extract::{Multipart, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{
    CreateCustomer, CreateFieldHistory, CreateStatusHistory, CreateTicket, CreateTicketNote,
    CreateTicketPhoto, Customer, Employee, EmployeeRole, QueueTicket, Ticket, TicketFilters,
    TicketNote as TicketNoteModel, TicketPhoto as TicketPhotoModel, TicketSearchParams,
    TicketStatus, UpdateTicket,
};
use crate::repositories::{
    CustomerRepository, EmployeeRepository, FieldHistoryRepository, StatusHistoryRepository,
    TicketNoteRepository, TicketPhotoRepository, TicketRepository,
};
use crate::response::ApiResponse;
use crate::routes::AppState;
use crate::services::pdf::{generate_label_pdf, generate_receipt_pdf, LabelData, ReceiptData};

/// Query parameters for listing tickets.
#[derive(Debug, Clone, Deserialize)]
pub struct ListTicketsQuery {
    /// Filter by status (comma-separated values like "intake,in_progress")
    pub status: Option<String>,
    /// Filter by rush flag
    pub is_rush: Option<bool>,
    /// Full-text search across ticket, customer, and notes
    pub search: Option<String>,
    /// Filter by customer ID
    pub customer_id: Option<Uuid>,
    /// Filter by created date range (start)
    pub from_date: Option<DateTime<Utc>>,
    /// Filter by created date range (end)
    pub to_date: Option<DateTime<Utc>>,
    /// Include archived tickets (default: false)
    #[serde(default)]
    pub include_archived: bool,
    /// Limit results (default: 100)
    pub limit: Option<i64>,
    /// Offset for pagination (default: 0)
    pub offset: Option<i64>,
}

impl ListTicketsQuery {
    /// Parse comma-separated status string into TicketStatus values.
    fn parse_statuses(&self) -> Option<Vec<TicketStatus>> {
        self.status.as_ref().map(|s| {
            s.split(',')
                .filter_map(|status_str| match status_str.trim() {
                    "intake" => Some(TicketStatus::Intake),
                    "in_progress" => Some(TicketStatus::InProgress),
                    "waiting_on_parts" => Some(TicketStatus::WaitingOnParts),
                    "ready_for_pickup" => Some(TicketStatus::ReadyForPickup),
                    "closed" => Some(TicketStatus::Closed),
                    "archived" => Some(TicketStatus::Archived),
                    _ => None,
                })
                .collect()
        })
    }
}

/// Paginated response for listing tickets.
#[derive(Debug, Clone, Serialize)]
pub struct ListTicketsResponse {
    /// List of tickets
    pub tickets: Vec<QueueTicket>,
    /// Pagination info
    pub pagination: PaginationInfo,
}

/// Pagination metadata.
#[derive(Debug, Clone, Serialize)]
pub struct PaginationInfo {
    /// Number of items returned
    pub count: usize,
    /// Limit used in the query
    pub limit: i64,
    /// Offset used in the query
    pub offset: i64,
    /// Whether there may be more results
    pub has_more: bool,
}

// =============================================================================
// GET /tickets/:ticket_id - Ticket Detail
// =============================================================================

/// Employee attribution summary for display.
#[derive(Debug, Clone, Serialize)]
pub struct EmployeeAttribution {
    pub employee_id: Uuid,
    pub name: String,
}

/// Customer info in ticket detail response.
#[derive(Debug, Clone, Serialize)]
pub struct TicketCustomer {
    pub customer_id: Uuid,
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

impl From<Customer> for TicketCustomer {
    fn from(c: Customer) -> Self {
        Self {
            customer_id: c.customer_id,
            name: c.name,
            phone: c.phone,
            email: c.email,
        }
    }
}

/// Storage location info in ticket detail response.
#[derive(Debug, Clone, Serialize)]
pub struct TicketStorageLocation {
    pub location_id: Uuid,
    pub name: String,
}

/// Photo record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
#[allow(dead_code)] // storage_key reserved for future signed URL generation
struct PhotoRecord {
    photo_id: Uuid,
    storage_key: String,
    uploaded_at: DateTime<Utc>,
    uploaded_by: Uuid,
    employee_name: String,
}

/// Photo info in ticket detail response.
#[derive(Debug, Clone, Serialize)]
pub struct TicketPhoto {
    pub photo_id: Uuid,
    /// Signed URL for accessing the photo (placeholder until storage is integrated in AppState)
    pub url: String,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by: EmployeeAttribution,
}

/// Note record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
struct NoteRecord {
    note_id: Uuid,
    content: String,
    created_at: DateTime<Utc>,
    created_by: Uuid,
    employee_name: String,
}

/// Note info in ticket detail response.
#[derive(Debug, Clone, Serialize)]
pub struct TicketNote {
    pub note_id: Uuid,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub created_by: EmployeeAttribution,
}

/// Status history record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
struct StatusHistoryRecord {
    from_status: Option<TicketStatus>,
    to_status: TicketStatus,
    changed_at: DateTime<Utc>,
    changed_by: Uuid,
    employee_name: String,
}

/// Status history entry in ticket detail response.
#[derive(Debug, Clone, Serialize)]
pub struct TicketStatusHistoryEntry {
    pub from_status: Option<TicketStatus>,
    pub to_status: TicketStatus,
    pub changed_at: DateTime<Utc>,
    pub changed_by: EmployeeAttribution,
}

/// Full ticket detail response.
#[derive(Debug, Clone, Serialize)]
pub struct TicketDetailResponse {
    pub ticket_id: Uuid,
    pub friendly_code: String,
    pub status: TicketStatus,
    pub is_rush: bool,

    pub customer: TicketCustomer,

    pub item_type: Option<String>,
    pub item_description: String,
    pub condition_notes: String,
    pub requested_work: String,

    pub promise_date: Option<NaiveDate>,
    pub storage_location: TicketStorageLocation,

    pub quote_amount: Option<Decimal>,
    pub actual_amount: Option<Decimal>,

    pub photos: Vec<TicketPhoto>,
    pub notes: Vec<TicketNote>,
    pub status_history: Vec<TicketStatusHistoryEntry>,

    pub taken_in_by: EmployeeAttribution,
    pub worked_by: Option<EmployeeAttribution>,
    pub closed_by: Option<EmployeeAttribution>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
}

/// Storage location record from the database.
#[derive(Debug, Clone, sqlx::FromRow)]
struct StorageLocationRecord {
    location_id: Uuid,
    name: String,
}

/// GET /api/v1/tickets/:ticket_id - Get full ticket details.
pub async fn get_ticket(
    State(state): State<AppState>,
    Path(ticket_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Find the ticket
    let ticket = TicketRepository::find_by_id(&state.db, ticket_id)
        .await?
        .ok_or_else(|| AppError::not_found("Ticket not found"))?;

    // 2. Find the customer
    let customer = CustomerRepository::find_by_id(&state.db, ticket.customer_id)
        .await?
        .ok_or_else(|| AppError::not_found("Customer not found"))?;

    // 3. Get storage location
    let storage_location = sqlx::query_as::<_, StorageLocationRecord>(
        "SELECT location_id, name FROM storage_locations WHERE location_id = $1",
    )
    .bind(ticket.storage_location_id)
    .fetch_optional(&state.db)
    .await?
    .ok_or_else(|| AppError::not_found("Storage location not found"))?;

    // 4. Get employee who took in the ticket
    let taken_in_by = EmployeeRepository::find_by_id(&state.db, ticket.taken_in_by)
        .await?
        .ok_or_else(|| AppError::server_error("Taken in by employee not found"))?;

    // 5. Get worked_by employee if set
    let worked_by = if let Some(worked_by_id) = ticket.worked_by {
        EmployeeRepository::find_by_id(&state.db, worked_by_id)
            .await?
            .map(|e| EmployeeAttribution {
                employee_id: e.employee_id,
                name: e.name,
            })
    } else {
        None
    };

    // 6. Get closed_by employee if set
    let closed_by = if let Some(closed_by_id) = ticket.closed_by {
        EmployeeRepository::find_by_id(&state.db, closed_by_id)
            .await?
            .map(|e| EmployeeAttribution {
                employee_id: e.employee_id,
                name: e.name,
            })
    } else {
        None
    };

    // 7. Get photos with employee names
    let photo_records = sqlx::query_as::<_, PhotoRecord>(
        r#"
        SELECT
            p.photo_id,
            p.storage_key,
            p.uploaded_at,
            p.uploaded_by,
            e.name as employee_name
        FROM ticket_photos p
        JOIN employees e ON p.uploaded_by = e.employee_id
        WHERE p.ticket_id = $1
        ORDER BY p.uploaded_at ASC
        "#,
    )
    .bind(ticket_id)
    .fetch_all(&state.db)
    .await?;

    // Convert to response format
    // Note: For now, we use the storage_key as a placeholder URL.
    // When StorageClient is integrated into AppState, this should generate signed URLs.
    let photos: Vec<TicketPhoto> = photo_records
        .into_iter()
        .map(|p| TicketPhoto {
            photo_id: p.photo_id,
            // TODO: Generate signed URL when storage client is in AppState
            // For now, return a placeholder API path
            url: format!("/api/v1/tickets/{}/photos/{}", ticket_id, p.photo_id),
            uploaded_at: p.uploaded_at,
            uploaded_by: EmployeeAttribution {
                employee_id: p.uploaded_by,
                name: p.employee_name,
            },
        })
        .collect();

    // 8. Get notes with employee names
    let note_records = sqlx::query_as::<_, NoteRecord>(
        r#"
        SELECT
            n.note_id,
            n.content,
            n.created_at,
            n.created_by,
            e.name as employee_name
        FROM ticket_notes n
        JOIN employees e ON n.created_by = e.employee_id
        WHERE n.ticket_id = $1
        ORDER BY n.created_at ASC
        "#,
    )
    .bind(ticket_id)
    .fetch_all(&state.db)
    .await?;

    let notes: Vec<TicketNote> = note_records
        .into_iter()
        .map(|n| TicketNote {
            note_id: n.note_id,
            content: n.content,
            created_at: n.created_at,
            created_by: EmployeeAttribution {
                employee_id: n.created_by,
                name: n.employee_name,
            },
        })
        .collect();

    // 9. Get status history with employee names
    let status_history_records = sqlx::query_as::<_, StatusHistoryRecord>(
        r#"
        SELECT
            h.from_status,
            h.to_status,
            h.changed_at,
            h.changed_by,
            e.name as employee_name
        FROM ticket_status_history h
        JOIN employees e ON h.changed_by = e.employee_id
        WHERE h.ticket_id = $1
        ORDER BY h.changed_at ASC
        "#,
    )
    .bind(ticket_id)
    .fetch_all(&state.db)
    .await?;

    let status_history: Vec<TicketStatusHistoryEntry> = status_history_records
        .into_iter()
        .map(|h| TicketStatusHistoryEntry {
            from_status: h.from_status,
            to_status: h.to_status,
            changed_at: h.changed_at,
            changed_by: EmployeeAttribution {
                employee_id: h.changed_by,
                name: h.employee_name,
            },
        })
        .collect();

    // 10. Build the response
    let response = TicketDetailResponse {
        ticket_id: ticket.ticket_id,
        friendly_code: ticket.friendly_code,
        status: ticket.status,
        is_rush: ticket.is_rush,
        customer: customer.into(),
        item_type: ticket.item_type,
        item_description: ticket.item_description,
        condition_notes: ticket.condition_notes,
        requested_work: ticket.requested_work,
        promise_date: ticket.promise_date,
        storage_location: TicketStorageLocation {
            location_id: storage_location.location_id,
            name: storage_location.name,
        },
        quote_amount: ticket.quote_amount,
        actual_amount: ticket.actual_amount,
        photos,
        notes,
        status_history,
        taken_in_by: EmployeeAttribution {
            employee_id: taken_in_by.employee_id,
            name: taken_in_by.name,
        },
        worked_by,
        closed_by,
        created_at: ticket.created_at,
        updated_at: ticket.updated_at,
        closed_at: ticket.closed_at,
    };

    Ok(Json(ApiResponse::success(response)))
}

/// GET /api/v1/tickets - List tickets with filters.
pub async fn list_tickets(
    State(state): State<AppState>,
    Query(query): Query<ListTicketsQuery>,
) -> Result<impl IntoResponse, AppError> {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    // If search is provided, use the search method
    let tickets = if let Some(ref search_query) = query.search {
        // Determine which statuses to search
        let statuses = if let Some(parsed) = query.parse_statuses() {
            Some(parsed)
        } else if query.include_archived {
            None // Search all statuses including archived
        } else {
            // Default: all active statuses (not closed/archived)
            Some(vec![
                TicketStatus::Intake,
                TicketStatus::InProgress,
                TicketStatus::WaitingOnParts,
                TicketStatus::ReadyForPickup,
            ])
        };

        let params = TicketSearchParams {
            query: search_query.clone(),
            statuses,
            limit: Some(limit + 1), // Fetch one extra to determine has_more
            offset: Some(offset),
        };

        TicketRepository::search(&state.db, params).await?
    } else {
        // Use the list method with filters
        let mut statuses = query.parse_statuses();

        // If no status filter and not including archived, default to active statuses
        if statuses.is_none() && !query.include_archived {
            statuses = Some(vec![
                TicketStatus::Intake,
                TicketStatus::InProgress,
                TicketStatus::WaitingOnParts,
                TicketStatus::ReadyForPickup,
            ]);
        }

        let filters = TicketFilters {
            statuses,
            is_rush: query.is_rush,
            customer_id: query.customer_id,
            created_after: query.from_date,
            created_before: query.to_date,
            limit: Some(limit + 1), // Fetch one extra to determine has_more
            offset: Some(offset),
        };

        // Convert TicketSummary to QueueTicket for consistent response format
        let summaries = TicketRepository::list(&state.db, filters).await?;

        // Convert to QueueTicket format (add is_overdue calculation)
        let today = Utc::now().date_naive();
        summaries
            .into_iter()
            .map(|s| QueueTicket {
                ticket_id: s.ticket_id,
                friendly_code: s.friendly_code,
                customer_id: s.customer_id,
                customer_name: s.customer_name,
                item_type: s.item_type,
                item_description: s.item_description,
                status: s.status,
                is_rush: s.is_rush,
                promise_date: s.promise_date,
                quote_amount: s.quote_amount,
                created_at: s.created_at,
                is_overdue: s
                    .promise_date
                    .map(|d| d < today && s.status.is_open())
                    .unwrap_or(false),
            })
            .collect()
    };

    // Determine if there are more results
    let has_more = tickets.len() as i64 > limit;
    let tickets: Vec<QueueTicket> = tickets.into_iter().take(limit as usize).collect();

    let response = ListTicketsResponse {
        pagination: PaginationInfo {
            count: tickets.len(),
            limit,
            offset,
            has_more,
        },
        tickets,
    };

    Ok(Json(ApiResponse::success(response)))
}

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

/// Check if an employee is authorized to modify a ticket.
///
/// An employee can modify a ticket if:
/// - They have the Admin role, OR
/// - They took in the ticket (taken_in_by matches), OR
/// - They are assigned to work on the ticket (worked_by matches)
///
/// Returns an error if not authorized. The error message intentionally
/// does not reveal whether the ticket exists to prevent enumeration.
pub fn is_authorized_for_ticket(employee: &Employee, ticket: &Ticket) -> bool {
    employee.role == EmployeeRole::Admin
        || ticket.taken_in_by == employee.employee_id
        || ticket.worked_by == Some(employee.employee_id)
}

/// Returns an authorization error for ticket operations.
///
/// Uses a generic message to avoid leaking information about ticket existence.
fn forbidden_ticket_error() -> AppError {
    AppError::forbidden("You do not have permission to modify this ticket")
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

/// GET /api/v1/tickets/:ticket_id/label.pdf - Generate label PDF for a physical tag.
pub async fn get_label_pdf(
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

    // 3. Generate label PDF
    let label_data = LabelData {
        ticket,
        customer_name: customer.name,
    };

    let pdf_bytes = generate_label_pdf(&label_data)?;

    // 3. Return PDF response
    let filename = format!("label-{}.pdf", label_data.ticket.friendly_code);
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
        .ok_or_else(forbidden_ticket_error)?;

    // 3. Authorization check: employee must be owner, assigned worker, or admin
    if !is_authorized_for_ticket(&employee, &existing_ticket) {
        return Err(forbidden_ticket_error());
    }

    // 4. Check if ticket is closed/archived
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

    // 5. Track field changes for audit trail
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

    // 6. Build update struct
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

    // 7. Update the ticket
    let updated_ticket = TicketRepository::update(&state.db, ticket_id, update).await?;

    // 8. Record field changes in history
    FieldHistoryRepository::create_batch(&state.db, field_changes).await?;

    // 9. Return updated ticket
    Ok(Json(ApiResponse::success(updated_ticket)))
}

// =============================================================================
// GET /queue - Workboard Queue
// =============================================================================

/// A single lane in the workboard queue.
#[derive(Debug, Clone, Serialize)]
pub struct QueueLane {
    /// Number of tickets in this lane.
    pub count: usize,
    /// Tickets in this lane, sorted by rush first then FIFO.
    pub tickets: Vec<QueueTicket>,
}

/// All lanes in the workboard queue.
#[derive(Debug, Clone, Serialize)]
pub struct QueueLanes {
    pub intake: QueueLane,
    pub in_progress: QueueLane,
    pub waiting_on_parts: QueueLane,
    pub ready_for_pickup: QueueLane,
}

/// Response for the GET /queue endpoint.
#[derive(Debug, Clone, Serialize)]
pub struct GetQueueResponse {
    pub lanes: QueueLanes,
}

/// GET /api/v1/queue - Get workboard queue with tickets grouped by status lane.
///
/// Returns tickets grouped by status for workboard display.
/// Each lane is sorted by rush first, then FIFO (oldest first).
/// Excludes closed and archived tickets.
/// Includes `is_overdue` flag for visual indicator.
pub async fn get_queue(State(state): State<AppState>) -> Result<impl IntoResponse, AppError> {
    // Use the repository method which handles grouping and sorting
    let queue = TicketRepository::get_queue(&state.db, None).await?;

    // Build response with count for each lane
    let response = GetQueueResponse {
        lanes: QueueLanes {
            intake: QueueLane {
                count: queue.intake.len(),
                tickets: queue.intake,
            },
            in_progress: QueueLane {
                count: queue.in_progress.len(),
                tickets: queue.in_progress,
            },
            waiting_on_parts: QueueLane {
                count: queue.waiting_on_parts.len(),
                tickets: queue.waiting_on_parts,
            },
            ready_for_pickup: QueueLane {
                count: queue.ready_for_pickup.len(),
                tickets: queue.ready_for_pickup,
            },
        },
    };

    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /tickets/:ticket_id/close - Close Ticket
// =============================================================================

/// Request body for closing a ticket.
#[derive(Debug, Clone, Deserialize)]
pub struct CloseTicketRequest {
    /// The actual amount charged for the repair work (required)
    pub actual_amount: Decimal,
}

/// Response for a closed ticket.
#[derive(Debug, Clone, Serialize)]
pub struct CloseTicketResponse {
    /// The closed ticket
    #[serde(flatten)]
    pub ticket: Ticket,
    /// The previous status before closing
    pub previous_status: TicketStatus,
}

/// POST /api/v1/tickets/:ticket_id/close - Close a ticket.
///
/// Closes the ticket with the actual amount charged.
/// Requires X-Employee-ID header for attribution.
/// Only tickets with status ReadyForPickup can be closed.
pub async fn close_ticket(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
    Json(body): Json<CloseTicketRequest>,
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
        .ok_or_else(forbidden_ticket_error)?;

    // 3. Authorization check: employee must be owner, assigned worker, or admin
    if !is_authorized_for_ticket(&employee, &existing_ticket) {
        return Err(forbidden_ticket_error());
    }

    let previous_status = existing_ticket.status;

    // 4. Validate ticket is in ReadyForPickup status
    if previous_status != TicketStatus::ReadyForPickup {
        return Err(AppError::validation(format!(
            "Only tickets with status 'ready_for_pickup' can be closed, current status is '{}'",
            serde_json::to_string(&previous_status).unwrap_or_else(|_| "unknown".to_string())
        )));
    }

    // 5. Close the ticket
    let closed_ticket = TicketRepository::close(
        &state.db,
        ticket_id,
        body.actual_amount,
        employee.employee_id,
    )
    .await?;

    // 6. Create status history entry
    StatusHistoryRepository::create(
        &state.db,
        CreateStatusHistory {
            ticket_id,
            from_status: Some(previous_status),
            to_status: TicketStatus::Closed,
            changed_by: employee.employee_id,
        },
    )
    .await?;

    // 7. Return closed ticket with previous status
    let response = CloseTicketResponse {
        ticket: closed_ticket,
        previous_status,
    };

    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /tickets/:ticket_id/status - Change Ticket Status
// =============================================================================

/// Request body for changing ticket status.
#[derive(Debug, Clone, Deserialize)]
pub struct ChangeStatusRequest {
    /// The new status to set
    pub status: TicketStatus,
}

/// Response for a status change.
#[derive(Debug, Clone, Serialize)]
pub struct ChangeStatusResponse {
    /// The updated ticket
    #[serde(flatten)]
    pub ticket: Ticket,
    /// The previous status
    pub previous_status: TicketStatus,
}

/// POST /api/v1/tickets/:ticket_id/status - Change ticket status.
///
/// Validates the status transition and records it in the status history.
/// Requires X-Employee-ID header for attribution.
pub async fn change_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
    Json(body): Json<ChangeStatusRequest>,
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
        .ok_or_else(forbidden_ticket_error)?;

    // 3. Authorization check: employee must be owner, assigned worker, or admin
    if !is_authorized_for_ticket(&employee, &existing_ticket) {
        return Err(forbidden_ticket_error());
    }

    let previous_status = existing_ticket.status;

    // 4. Validate the status transition
    if !previous_status.can_transition_to(body.status) {
        return Err(AppError::validation(format!(
            "Cannot transition from {} to {}",
            serde_json::to_string(&previous_status).unwrap_or_else(|_| "unknown".to_string()),
            serde_json::to_string(&body.status).unwrap_or_else(|_| "unknown".to_string())
        )));
    }

    // 5. Update the ticket status
    let updated_ticket =
        TicketRepository::update_status(&state.db, ticket_id, body.status, employee.employee_id)
            .await?;

    // 6. Create status history entry
    StatusHistoryRepository::create(
        &state.db,
        CreateStatusHistory {
            ticket_id,
            from_status: Some(previous_status),
            to_status: body.status,
            changed_by: employee.employee_id,
        },
    )
    .await?;

    // 7. Return updated ticket with previous status
    let response = ChangeStatusResponse {
        ticket: updated_ticket,
        previous_status,
    };

    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /tickets/:ticket_id/rush - Toggle Rush Flag
// =============================================================================

/// Request body for toggling rush flag.
#[derive(Debug, Clone, Deserialize)]
pub struct ToggleRushRequest {
    /// The new rush status to set
    pub is_rush: bool,
}

/// Response for a rush flag toggle.
#[derive(Debug, Clone, Serialize)]
pub struct ToggleRushResponse {
    /// The updated ticket
    #[serde(flatten)]
    pub ticket: Ticket,
    /// The previous rush status
    pub previous_is_rush: bool,
}

/// POST /api/v1/tickets/:ticket_id/rush - Toggle rush flag on a ticket.
///
/// Sets the is_rush flag and records the change in field history.
/// Requires X-Employee-ID header for attribution.
pub async fn toggle_rush(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
    Json(body): Json<ToggleRushRequest>,
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
        .ok_or_else(forbidden_ticket_error)?;

    // 3. Authorization check: employee must be owner, assigned worker, or admin
    if !is_authorized_for_ticket(&employee, &existing_ticket) {
        return Err(forbidden_ticket_error());
    }

    let previous_is_rush = existing_ticket.is_rush;

    // 4. Check if ticket is closed/archived
    if !existing_ticket.status.is_open() {
        return Err(AppError::forbidden(
            "Cannot modify rush flag on closed or archived ticket",
        ));
    }

    // 5. Skip update if value is the same
    if previous_is_rush == body.is_rush {
        let response = ToggleRushResponse {
            ticket: existing_ticket,
            previous_is_rush,
        };
        return Ok(Json(ApiResponse::success(response)));
    }

    // 6. Update the rush flag
    let updated_ticket =
        TicketRepository::set_rush(&state.db, ticket_id, body.is_rush, employee.employee_id)
            .await?;

    // 7. Record field change in history
    FieldHistoryRepository::create(
        &state.db,
        CreateFieldHistory {
            ticket_id,
            field_name: "is_rush".to_string(),
            old_value: Some(previous_is_rush.to_string()),
            new_value: Some(body.is_rush.to_string()),
            changed_by: employee.employee_id,
        },
    )
    .await?;

    // 8. Return updated ticket with previous status
    let response = ToggleRushResponse {
        ticket: updated_ticket,
        previous_is_rush,
    };

    Ok(Json(ApiResponse::success(response)))
}

// =============================================================================
// POST /tickets/:ticket_id/notes - Add Note
// =============================================================================

/// Request body for adding a note to a ticket.
#[derive(Debug, Clone, Deserialize)]
pub struct AddNoteRequest {
    /// The note content (required)
    pub content: String,
}

/// Response for a successfully created note.
#[derive(Debug, Clone, Serialize)]
pub struct AddNoteResponse {
    /// The created note
    #[serde(flatten)]
    pub note: TicketNoteModel,
}

/// POST /api/v1/tickets/:ticket_id/notes - Add an internal note to a ticket.
///
/// Notes are append-only - no edit or delete available.
/// Requires X-Employee-ID header for attribution.
pub async fn add_note(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
    Json(body): Json<AddNoteRequest>,
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
        .ok_or_else(forbidden_ticket_error)?;

    // 3. Authorization check: employee must be owner, assigned worker, or admin
    if !is_authorized_for_ticket(&employee, &existing_ticket) {
        return Err(forbidden_ticket_error());
    }

    // 4. Validate content is not empty
    if body.content.trim().is_empty() {
        return Err(AppError::validation("Note content cannot be empty"));
    }

    // 5. Create the note
    let note = TicketNoteRepository::create(
        &state.db,
        CreateTicketNote {
            ticket_id,
            content: body.content,
            created_by: employee.employee_id,
        },
    )
    .await?;

    // 6. Return created note
    let response = AddNoteResponse { note };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

// =============================================================================
// POST /tickets/:ticket_id/photos - Upload Photo
// =============================================================================

/// Maximum number of photos allowed per ticket.
const MAX_PHOTOS_PER_TICKET: i64 = 10;

/// Maximum file size in bytes (10MB).
const MAX_FILE_SIZE: usize = 10 * 1024 * 1024;

/// Allowed content types for photo uploads.
const ALLOWED_CONTENT_TYPES: &[&str] = &["image/jpeg", "image/png", "image/webp"];

/// Response for a successfully uploaded photo.
#[derive(Debug, Clone, Serialize)]
pub struct UploadPhotoResponse {
    /// The created photo record
    #[serde(flatten)]
    pub photo: TicketPhotoModel,
    /// Signed URL for accessing the photo
    pub url: String,
}

/// POST /api/v1/tickets/:ticket_id/photos - Upload a photo to a ticket.
///
/// Accepts multipart/form-data with a single file field named "photo".
/// Validates file type (jpeg, png, webp) and size (max 10MB).
/// Requires X-Employee-ID header for attribution.
pub async fn upload_photo(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(ticket_id): Path<Uuid>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    // 1. Extract and validate employee ID from header
    let employee_id = extract_employee_id(&headers)?;

    // Verify employee exists and is active
    let employee = EmployeeRepository::find_active_by_id(&state.db, employee_id)
        .await?
        .ok_or_else(|| AppError::validation("Employee not found or inactive"))?;

    // 2. Find the ticket
    let ticket = TicketRepository::find_by_id(&state.db, ticket_id)
        .await?
        .ok_or_else(forbidden_ticket_error)?;

    // 3. Authorization check: employee must be owner, assigned worker, or admin
    if !is_authorized_for_ticket(&employee, &ticket) {
        return Err(forbidden_ticket_error());
    }

    // 4. Check photo limit
    let current_count = TicketPhotoRepository::count_by_ticket_id(&state.db, ticket_id).await?;
    if current_count >= MAX_PHOTOS_PER_TICKET {
        return Err(AppError::photo_limit(format!(
            "Maximum {} photos per ticket reached",
            MAX_PHOTOS_PER_TICKET
        )));
    }

    // 5. Check if storage is available (S3 or local fallback)
    let use_local_storage = state.storage.is_none();

    // 6. Extract file from multipart form
    let mut file_data: Option<(String, Vec<u8>)> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::validation(format!("Failed to read multipart field: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "photo" {
            // Get content type from field
            let content_type = field
                .content_type()
                .map(|ct| ct.to_string())
                .unwrap_or_else(|| "application/octet-stream".to_string());

            // Validate content type
            if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
                return Err(AppError::validation(format!(
                    "Invalid file type '{}'. Allowed types: jpeg, png, webp",
                    content_type
                )));
            }

            // Read file data
            let data = field
                .bytes()
                .await
                .map_err(|e| AppError::validation(format!("Failed to read file data: {}", e)))?;

            // Validate file size
            if data.len() > MAX_FILE_SIZE {
                return Err(AppError::validation(format!(
                    "File too large. Maximum size is {}MB",
                    MAX_FILE_SIZE / (1024 * 1024)
                )));
            }

            if data.is_empty() {
                return Err(AppError::validation("Empty file provided"));
            }

            file_data = Some((content_type, data.to_vec()));
            break;
        }
    }

    let (content_type, data) =
        file_data.ok_or_else(|| AppError::validation("No 'photo' field in request"))?;

    // 7. Generate unique storage key
    let photo_id = Uuid::new_v4();
    let extension = match content_type.as_str() {
        "image/jpeg" => "jpg",
        "image/png" => "png",
        "image/webp" => "webp",
        _ => "bin",
    };
    let storage_key = format!("tickets/{}/{}.{}", ticket.ticket_id, photo_id, extension);

    // 8. Upload to storage (S3 or local fallback)
    let file_size = data.len() as i32;
    let url: String;

    if use_local_storage {
        // Local file storage fallback for development
        let local_dir = std::path::Path::new("uploads")
            .join("tickets")
            .join(ticket.ticket_id.to_string());
        std::fs::create_dir_all(&local_dir).map_err(|e| {
            AppError::server_error(format!("Failed to create upload directory: {}", e))
        })?;

        let file_path = local_dir.join(format!("{}.{}", photo_id, extension));
        std::fs::write(&file_path, &data)
            .map_err(|e| AppError::server_error(format!("Failed to save photo: {}", e)))?;

        // Return a local file path as the URL (for dev purposes)
        url = format!(
            "/uploads/tickets/{}/{}.{}",
            ticket.ticket_id, photo_id, extension
        );
    } else {
        // S3 storage
        let storage = state.storage.as_ref().unwrap();
        storage
            .upload(&storage_key, data, &content_type)
            .await
            .map_err(|e| AppError::server_error(format!("Failed to upload photo: {}", e)))?;

        url = storage
            .get_signed_url(&storage_key, None)
            .await
            .map_err(|e| AppError::server_error(format!("Failed to generate signed URL: {}", e)))?;
    }

    // 9. Create database record
    let photo = TicketPhotoRepository::create(
        &state.db,
        CreateTicketPhoto {
            ticket_id,
            storage_key: storage_key.clone(),
            content_type,
            size_bytes: file_size,
            uploaded_by: employee.employee_id,
        },
    )
    .await?;

    // 10. Return response
    let response = UploadPhotoResponse { photo, url };

    Ok((StatusCode::CREATED, Json(ApiResponse::success(response))))
}

// =============================================================================
// DELETE /tickets/:ticket_id/photos/:photo_id - Delete Photo (Admin Only)
// =============================================================================

/// Path parameters for photo deletion.
#[derive(Debug, Clone, Deserialize)]
pub struct DeletePhotoPath {
    pub ticket_id: Uuid,
    pub photo_id: Uuid,
}

/// Response for a deleted photo.
#[derive(Debug, Clone, Serialize)]
pub struct DeletePhotoResponse {
    /// The ID of the deleted photo
    pub photo_id: Uuid,
    /// The ticket ID the photo belonged to
    pub ticket_id: Uuid,
}

/// Extract admin PIN from X-Admin-PIN header.
fn extract_admin_pin(headers: &HeaderMap) -> Result<String, AppError> {
    let header_value = headers
        .get("X-Admin-PIN")
        .ok_or_else(|| AppError::validation("X-Admin-PIN header is required"))?;

    header_value
        .to_str()
        .map(|s| s.to_string())
        .map_err(|_| AppError::validation("Invalid X-Admin-PIN header value"))
}

/// Verify the admin PIN against an admin employee.
///
/// Returns the admin employee if PIN is valid.
async fn verify_admin_employee(
    pool: &sqlx::PgPool,
    pin: &str,
) -> Result<crate::models::Employee, AppError> {
    use crate::auth::verify_pin;
    use crate::models::EmployeeRole;

    // Get all active employees for PIN verification
    let employees = EmployeeRepository::find_active_for_pin_verification(pool).await?;

    // Find an admin employee whose PIN matches
    for employee in employees {
        if employee.role == EmployeeRole::Admin && verify_pin(pin, &employee.pin_hash)? {
            return Ok(employee);
        }
    }

    Err(AppError::invalid_pin("Invalid admin PIN"))
}

/// DELETE /api/v1/tickets/:ticket_id/photos/:photo_id - Delete a photo (admin only).
///
/// Requires X-Admin-PIN header for authorization.
/// Deletes the photo from S3 storage and the database.
pub async fn delete_photo(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(path): Path<DeletePhotoPath>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Extract and verify admin PIN
    let admin_pin = extract_admin_pin(&headers)?;
    let _admin = verify_admin_employee(&state.db, &admin_pin).await?;

    // 2. Verify ticket exists
    let _ticket = TicketRepository::find_by_id(&state.db, path.ticket_id)
        .await?
        .ok_or_else(|| AppError::not_found("Ticket not found"))?;

    // 3. Find the photo
    let photo = TicketPhotoRepository::find_by_id(&state.db, path.photo_id)
        .await?
        .ok_or_else(|| AppError::not_found("Photo not found"))?;

    // 4. Verify photo belongs to the ticket
    if photo.ticket_id != path.ticket_id {
        return Err(AppError::not_found("Photo not found on this ticket"));
    }

    // 5. Delete from S3 storage (if storage is configured)
    if let Some(storage) = &state.storage {
        storage.delete(&photo.storage_key).await.map_err(|e| {
            AppError::server_error(format!("Failed to delete photo from storage: {}", e))
        })?;
    }

    // 6. Delete database record
    TicketPhotoRepository::delete(&state.db, path.photo_id).await?;

    // 7. Return success response
    let response = DeletePhotoResponse {
        photo_id: path.photo_id,
        ticket_id: path.ticket_id,
    };

    Ok(Json(ApiResponse::success(response)))
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

    #[test]
    fn test_list_tickets_query_deserialize_empty() {
        let query: ListTicketsQuery = serde_urlencoded::from_str("").unwrap();
        assert!(query.status.is_none());
        assert!(query.is_rush.is_none());
        assert!(query.search.is_none());
        assert!(query.customer_id.is_none());
        assert!(query.from_date.is_none());
        assert!(query.to_date.is_none());
        assert!(!query.include_archived);
        assert!(query.limit.is_none());
        assert!(query.offset.is_none());
    }

    #[test]
    fn test_list_tickets_query_with_status_filter() {
        let query: ListTicketsQuery =
            serde_urlencoded::from_str("status=intake,in_progress").unwrap();
        assert_eq!(query.status, Some("intake,in_progress".to_string()));

        let statuses = query.parse_statuses().unwrap();
        assert_eq!(statuses.len(), 2);
        assert!(statuses.contains(&TicketStatus::Intake));
        assert!(statuses.contains(&TicketStatus::InProgress));
    }

    #[test]
    fn test_list_tickets_query_with_single_status() {
        let query: ListTicketsQuery =
            serde_urlencoded::from_str("status=ready_for_pickup").unwrap();
        let statuses = query.parse_statuses().unwrap();
        assert_eq!(statuses.len(), 1);
        assert!(statuses.contains(&TicketStatus::ReadyForPickup));
    }

    #[test]
    fn test_list_tickets_query_ignores_invalid_status() {
        let query: ListTicketsQuery =
            serde_urlencoded::from_str("status=intake,invalid,in_progress").unwrap();
        let statuses = query.parse_statuses().unwrap();
        assert_eq!(statuses.len(), 2);
        assert!(statuses.contains(&TicketStatus::Intake));
        assert!(statuses.contains(&TicketStatus::InProgress));
    }

    #[test]
    fn test_list_tickets_query_with_rush_filter() {
        let query: ListTicketsQuery = serde_urlencoded::from_str("is_rush=true").unwrap();
        assert_eq!(query.is_rush, Some(true));

        let query: ListTicketsQuery = serde_urlencoded::from_str("is_rush=false").unwrap();
        assert_eq!(query.is_rush, Some(false));
    }

    #[test]
    fn test_list_tickets_query_with_search() {
        let query: ListTicketsQuery = serde_urlencoded::from_str("search=gold+ring").unwrap();
        assert_eq!(query.search, Some("gold ring".to_string()));
    }

    #[test]
    fn test_list_tickets_query_with_pagination() {
        let query: ListTicketsQuery = serde_urlencoded::from_str("limit=50&offset=100").unwrap();
        assert_eq!(query.limit, Some(50));
        assert_eq!(query.offset, Some(100));
    }

    #[test]
    fn test_list_tickets_query_with_customer_id() {
        let query: ListTicketsQuery =
            serde_urlencoded::from_str("customer_id=550e8400-e29b-41d4-a716-446655440000").unwrap();
        assert_eq!(
            query.customer_id,
            Some(Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap())
        );
    }

    #[test]
    fn test_list_tickets_query_include_archived() {
        let query: ListTicketsQuery = serde_urlencoded::from_str("include_archived=true").unwrap();
        assert!(query.include_archived);

        let query: ListTicketsQuery = serde_urlencoded::from_str("").unwrap();
        assert!(!query.include_archived);
    }

    #[test]
    fn test_pagination_info_serialization() {
        let info = PaginationInfo {
            count: 10,
            limit: 50,
            offset: 0,
            has_more: false,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("\"count\":10"));
        assert!(json.contains("\"limit\":50"));
        assert!(json.contains("\"offset\":0"));
        assert!(json.contains("\"has_more\":false"));
    }

    #[test]
    fn test_employee_attribution_serialization() {
        let attribution = EmployeeAttribution {
            employee_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Alice".to_string(),
        };
        let json = serde_json::to_string(&attribution).unwrap();
        assert!(json.contains("\"employee_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"name\":\"Alice\""));
    }

    #[test]
    fn test_ticket_customer_serialization() {
        let customer = TicketCustomer {
            customer_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Jane Doe".to_string(),
            phone: Some("555-1234".to_string()),
            email: None,
        };
        let json = serde_json::to_string(&customer).unwrap();
        assert!(json.contains("\"customer_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"name\":\"Jane Doe\""));
        assert!(json.contains("\"phone\":\"555-1234\""));
        assert!(json.contains("\"email\":null"));
    }

    #[test]
    fn test_ticket_customer_from_customer() {
        let customer = Customer {
            customer_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Jane Doe".to_string(),
            phone: Some("555-1234".to_string()),
            email: Some("jane@example.com".to_string()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let ticket_customer: TicketCustomer = customer.into();
        assert_eq!(ticket_customer.name, "Jane Doe");
        assert_eq!(ticket_customer.phone, Some("555-1234".to_string()));
        assert_eq!(ticket_customer.email, Some("jane@example.com".to_string()));
    }

    #[test]
    fn test_ticket_storage_location_serialization() {
        let location = TicketStorageLocation {
            location_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            name: "Safe Drawer 1".to_string(),
        };
        let json = serde_json::to_string(&location).unwrap();
        assert!(json.contains("\"location_id\":\"660e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"name\":\"Safe Drawer 1\""));
    }

    #[test]
    fn test_ticket_note_serialization() {
        let note = TicketNote {
            note_id: Uuid::parse_str("770e8400-e29b-41d4-a716-446655440000").unwrap(),
            content: "Customer mentioned ring has sentimental value".to_string(),
            created_at: Utc::now(),
            created_by: EmployeeAttribution {
                employee_id: Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
                name: "Alice".to_string(),
            },
        };
        let json = serde_json::to_string(&note).unwrap();
        assert!(json.contains("\"note_id\":\"770e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"content\":\"Customer mentioned ring has sentimental value\""));
        assert!(json.contains("\"created_by\""));
    }

    #[test]
    fn test_ticket_status_history_entry_serialization() {
        let entry = TicketStatusHistoryEntry {
            from_status: Some(TicketStatus::Intake),
            to_status: TicketStatus::InProgress,
            changed_at: Utc::now(),
            changed_by: EmployeeAttribution {
                employee_id: Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
                name: "Bob".to_string(),
            },
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"from_status\":\"intake\""));
        assert!(json.contains("\"to_status\":\"in_progress\""));
        assert!(json.contains("\"changed_by\""));
    }

    #[test]
    fn test_ticket_status_history_entry_null_from_status() {
        let entry = TicketStatusHistoryEntry {
            from_status: None,
            to_status: TicketStatus::Intake,
            changed_at: Utc::now(),
            changed_by: EmployeeAttribution {
                employee_id: Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
                name: "Alice".to_string(),
            },
        };
        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("\"from_status\":null"));
        assert!(json.contains("\"to_status\":\"intake\""));
    }

    #[test]
    fn test_ticket_photo_serialization() {
        let photo = TicketPhoto {
            photo_id: Uuid::parse_str("990e8400-e29b-41d4-a716-446655440000").unwrap(),
            url: "/api/v1/tickets/abc/photos/990e8400-e29b-41d4-a716-446655440000".to_string(),
            uploaded_at: Utc::now(),
            uploaded_by: EmployeeAttribution {
                employee_id: Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
                name: "Alice".to_string(),
            },
        };
        let json = serde_json::to_string(&photo).unwrap();
        assert!(json.contains("\"photo_id\":\"990e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains(
            "\"url\":\"/api/v1/tickets/abc/photos/990e8400-e29b-41d4-a716-446655440000\""
        ));
        assert!(json.contains("\"uploaded_by\""));
    }

    #[test]
    fn test_queue_lane_serialization() {
        let lane = QueueLane {
            count: 2,
            tickets: vec![],
        };
        let json = serde_json::to_string(&lane).unwrap();
        assert!(json.contains("\"count\":2"));
        assert!(json.contains("\"tickets\":[]"));
    }

    #[test]
    fn test_queue_lanes_serialization() {
        let lanes = QueueLanes {
            intake: QueueLane {
                count: 1,
                tickets: vec![],
            },
            in_progress: QueueLane {
                count: 2,
                tickets: vec![],
            },
            waiting_on_parts: QueueLane {
                count: 0,
                tickets: vec![],
            },
            ready_for_pickup: QueueLane {
                count: 3,
                tickets: vec![],
            },
        };
        let json = serde_json::to_string(&lanes).unwrap();
        assert!(json.contains("\"intake\":{\"count\":1"));
        assert!(json.contains("\"in_progress\":{\"count\":2"));
        assert!(json.contains("\"waiting_on_parts\":{\"count\":0"));
        assert!(json.contains("\"ready_for_pickup\":{\"count\":3"));
    }

    #[test]
    fn test_get_queue_response_serialization() {
        let response = GetQueueResponse {
            lanes: QueueLanes {
                intake: QueueLane {
                    count: 0,
                    tickets: vec![],
                },
                in_progress: QueueLane {
                    count: 0,
                    tickets: vec![],
                },
                waiting_on_parts: QueueLane {
                    count: 0,
                    tickets: vec![],
                },
                ready_for_pickup: QueueLane {
                    count: 0,
                    tickets: vec![],
                },
            },
        };
        let json = serde_json::to_string(&response).unwrap();
        // Verify top-level "lanes" key exists
        assert!(json.contains("\"lanes\":"));
        // Verify all status lanes are present
        assert!(json.contains("\"intake\""));
        assert!(json.contains("\"in_progress\""));
        assert!(json.contains("\"waiting_on_parts\""));
        assert!(json.contains("\"ready_for_pickup\""));
    }

    #[test]
    fn test_change_status_request_deserialize() {
        let json = r#"{"status": "in_progress"}"#;
        let request: ChangeStatusRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.status, TicketStatus::InProgress);
    }

    #[test]
    fn test_change_status_request_deserialize_waiting_on_parts() {
        let json = r#"{"status": "waiting_on_parts"}"#;
        let request: ChangeStatusRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.status, TicketStatus::WaitingOnParts);
    }

    #[test]
    fn test_change_status_request_deserialize_ready_for_pickup() {
        let json = r#"{"status": "ready_for_pickup"}"#;
        let request: ChangeStatusRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.status, TicketStatus::ReadyForPickup);
    }

    #[test]
    fn test_change_status_request_invalid_status() {
        let json = r#"{"status": "invalid"}"#;
        let result: Result<ChangeStatusRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_close_ticket_request_deserialize() {
        let json = r#"{"actual_amount": 145.00}"#;
        let request: CloseTicketRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.actual_amount, Decimal::new(14500, 2));
    }

    #[test]
    fn test_close_ticket_request_deserialize_zero() {
        let json = r#"{"actual_amount": 0}"#;
        let request: CloseTicketRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.actual_amount, Decimal::ZERO);
    }

    #[test]
    fn test_close_ticket_request_deserialize_decimal() {
        let json = r#"{"actual_amount": 99.99}"#;
        let request: CloseTicketRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.actual_amount, Decimal::new(9999, 2));
    }

    #[test]
    fn test_close_ticket_request_missing_amount() {
        let json = r#"{}"#;
        let result: Result<CloseTicketRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_close_ticket_response_serialization() {
        use chrono::TimeZone;

        let ticket = Ticket {
            ticket_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            friendly_code: "JR-TEST1".to_string(),
            customer_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            item_type: Some("ring".to_string()),
            item_description: "Gold ring".to_string(),
            condition_notes: "Good condition".to_string(),
            requested_work: "Resize".to_string(),
            status: TicketStatus::Closed,
            is_rush: false,
            promise_date: None,
            storage_location_id: Uuid::parse_str("770e8400-e29b-41d4-a716-446655440000").unwrap(),
            quote_amount: Some(Decimal::new(10000, 2)),
            actual_amount: Some(Decimal::new(14500, 2)),
            taken_in_by: Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
            worked_by: None,
            closed_by: Some(Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap()),
            last_modified_by: Some(
                Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
            ),
            created_at: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2025, 1, 2, 12, 0, 0).unwrap(),
            closed_at: Some(Utc.with_ymd_and_hms(2025, 1, 2, 12, 0, 0).unwrap()),
            queue_position: None,
        };

        let response = CloseTicketResponse {
            ticket,
            previous_status: TicketStatus::ReadyForPickup,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ticket_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"status\":\"closed\""));
        assert!(json.contains("\"actual_amount\":\"145.00\""));
        assert!(json.contains("\"previous_status\":\"ready_for_pickup\""));
    }

    #[test]
    fn test_delete_photo_path_deserialize() {
        let ticket_id = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
        let photo_id = Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap();

        let path = DeletePhotoPath {
            ticket_id,
            photo_id,
        };

        assert_eq!(
            path.ticket_id,
            Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap()
        );
        assert_eq!(
            path.photo_id,
            Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap()
        );
    }

    #[test]
    fn test_delete_photo_response_serialization() {
        let response = DeletePhotoResponse {
            photo_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            ticket_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"photo_id\":\"660e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"ticket_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
    }

    #[test]
    fn test_toggle_rush_request_deserialize_true() {
        let json = r#"{"is_rush": true}"#;
        let request: ToggleRushRequest = serde_json::from_str(json).unwrap();
        assert!(request.is_rush);
    }

    #[test]
    fn test_toggle_rush_request_deserialize_false() {
        let json = r#"{"is_rush": false}"#;
        let request: ToggleRushRequest = serde_json::from_str(json).unwrap();
        assert!(!request.is_rush);
    }

    #[test]
    fn test_toggle_rush_request_missing_field() {
        let json = r#"{}"#;
        let result: Result<ToggleRushRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_toggle_rush_response_serialization() {
        use chrono::TimeZone;

        let ticket = Ticket {
            ticket_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            friendly_code: "JR-TEST1".to_string(),
            customer_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            item_type: Some("ring".to_string()),
            item_description: "Gold ring".to_string(),
            condition_notes: "Good condition".to_string(),
            requested_work: "Resize".to_string(),
            status: TicketStatus::Intake,
            is_rush: true,
            promise_date: None,
            storage_location_id: Uuid::parse_str("770e8400-e29b-41d4-a716-446655440000").unwrap(),
            quote_amount: Some(Decimal::new(10000, 2)),
            actual_amount: None,
            taken_in_by: Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
            worked_by: None,
            closed_by: None,
            last_modified_by: Some(
                Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap(),
            ),
            created_at: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
            updated_at: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
            closed_at: None,
            queue_position: None,
        };

        let response = ToggleRushResponse {
            ticket,
            previous_is_rush: false,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"ticket_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"is_rush\":true"));
        assert!(json.contains("\"previous_is_rush\":false"));
    }

    #[test]
    fn test_add_note_request_deserialize() {
        let json = r#"{"content": "Customer called about the repair"}"#;
        let request: AddNoteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.content, "Customer called about the repair");
    }

    #[test]
    fn test_add_note_request_empty_content() {
        let json = r#"{"content": ""}"#;
        let request: AddNoteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.content, "");
    }

    #[test]
    fn test_add_note_request_missing_content() {
        let json = r#"{}"#;
        let result: Result<AddNoteRequest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_note_request_multiline_content() {
        let json = r#"{"content": "Line 1\nLine 2\nLine 3"}"#;
        let request: AddNoteRequest = serde_json::from_str(json).unwrap();
        assert!(request.content.contains('\n'));
    }

    #[test]
    fn test_add_note_response_serialization() {
        use chrono::TimeZone;

        let note = TicketNoteModel {
            note_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            ticket_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            content: "Customer mentioned ring has sentimental value".to_string(),
            created_by: Uuid::parse_str("770e8400-e29b-41d4-a716-446655440000").unwrap(),
            created_at: Utc.with_ymd_and_hms(2025, 1, 1, 12, 0, 0).unwrap(),
        };

        let response = AddNoteResponse { note };
        let json = serde_json::to_string(&response).unwrap();

        // Due to #[serde(flatten)], fields are at the top level
        assert!(json.contains("\"note_id\":\"550e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"ticket_id\":\"660e8400-e29b-41d4-a716-446655440000\""));
        assert!(json.contains("\"content\":\"Customer mentioned ring has sentimental value\""));
        assert!(json.contains("\"created_by\":\"770e8400-e29b-41d4-a716-446655440000\""));
    }

    // =============================================================================
    // Authorization function tests
    // =============================================================================

    fn create_test_employee(role: EmployeeRole, employee_id: Uuid) -> Employee {
        Employee {
            employee_id,
            name: "Test Employee".to_string(),
            pin_hash: "hash".to_string(),
            role,
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn create_test_ticket(taken_in_by: Uuid, worked_by: Option<Uuid>) -> Ticket {
        Ticket {
            ticket_id: Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap(),
            friendly_code: "JR-TEST1".to_string(),
            customer_id: Uuid::parse_str("660e8400-e29b-41d4-a716-446655440000").unwrap(),
            item_type: Some("ring".to_string()),
            item_description: "Gold ring".to_string(),
            condition_notes: "Good condition".to_string(),
            requested_work: "Resize".to_string(),
            status: TicketStatus::Intake,
            is_rush: false,
            promise_date: None,
            storage_location_id: Uuid::parse_str("770e8400-e29b-41d4-a716-446655440000").unwrap(),
            quote_amount: None,
            actual_amount: None,
            taken_in_by,
            worked_by,
            closed_by: None,
            last_modified_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            closed_at: None,
            queue_position: None,
        }
    }

    #[test]
    fn test_is_authorized_for_ticket_owner() {
        let employee_id = Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap();
        let employee = create_test_employee(EmployeeRole::Staff, employee_id);
        let ticket = create_test_ticket(employee_id, None);

        assert!(is_authorized_for_ticket(&employee, &ticket));
    }

    #[test]
    fn test_is_authorized_for_ticket_assigned_worker() {
        let owner_id = Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap();
        let worker_id = Uuid::parse_str("990e8400-e29b-41d4-a716-446655440000").unwrap();
        let employee = create_test_employee(EmployeeRole::Staff, worker_id);
        let ticket = create_test_ticket(owner_id, Some(worker_id));

        assert!(is_authorized_for_ticket(&employee, &ticket));
    }

    #[test]
    fn test_is_authorized_for_ticket_admin() {
        let owner_id = Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap();
        let admin_id = Uuid::parse_str("aa0e8400-e29b-41d4-a716-446655440000").unwrap();
        let employee = create_test_employee(EmployeeRole::Admin, admin_id);
        let ticket = create_test_ticket(owner_id, None);

        // Admin can modify any ticket regardless of ownership
        assert!(is_authorized_for_ticket(&employee, &ticket));
    }

    #[test]
    fn test_is_authorized_for_ticket_unrelated_staff() {
        let owner_id = Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap();
        let worker_id = Uuid::parse_str("990e8400-e29b-41d4-a716-446655440000").unwrap();
        let unrelated_id = Uuid::parse_str("bb0e8400-e29b-41d4-a716-446655440000").unwrap();
        let employee = create_test_employee(EmployeeRole::Staff, unrelated_id);
        let ticket = create_test_ticket(owner_id, Some(worker_id));

        // Staff member who is neither owner nor assigned worker cannot modify
        assert!(!is_authorized_for_ticket(&employee, &ticket));
    }

    #[test]
    fn test_is_authorized_for_ticket_staff_no_worker_assigned() {
        let owner_id = Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap();
        let unrelated_id = Uuid::parse_str("bb0e8400-e29b-41d4-a716-446655440000").unwrap();
        let employee = create_test_employee(EmployeeRole::Staff, unrelated_id);
        let ticket = create_test_ticket(owner_id, None);

        // Staff member who is not the owner cannot modify when no worker assigned
        assert!(!is_authorized_for_ticket(&employee, &ticket));
    }

    #[test]
    fn test_is_authorized_for_ticket_owner_and_admin() {
        let owner_id = Uuid::parse_str("880e8400-e29b-41d4-a716-446655440000").unwrap();
        let employee = create_test_employee(EmployeeRole::Admin, owner_id);
        let ticket = create_test_ticket(owner_id, None);

        // Admin who is also the owner should be authorized (tests short-circuit)
        assert!(is_authorized_for_ticket(&employee, &ticket));
    }
}

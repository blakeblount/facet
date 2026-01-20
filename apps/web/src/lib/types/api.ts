/**
 * API TypeScript types matching the Rust backend.
 *
 * These types mirror the API response structures from apps/api.
 */

// =============================================================================
// Common Types
// =============================================================================

/**
 * Standard API response wrapper.
 * All API endpoints return this format for consistency.
 */
export interface ApiResponse<T> {
	data: T | null;
	error: ApiError | null;
}

/**
 * Error detail in API response.
 */
export interface ApiError {
	code: string;
	message: string;
}

/**
 * Error codes matching the API specification.
 */
export const ErrorCodes = {
	VALIDATION_ERROR: 'VALIDATION_ERROR',
	INVALID_PIN: 'INVALID_PIN',
	FORBIDDEN: 'FORBIDDEN',
	NOT_FOUND: 'NOT_FOUND',
	CONFLICT: 'CONFLICT',
	PHOTO_LIMIT: 'PHOTO_LIMIT',
	PRINT_REQUIRED: 'PRINT_REQUIRED',
	SERVER_ERROR: 'SERVER_ERROR'
} as const;

export type ErrorCode = (typeof ErrorCodes)[keyof typeof ErrorCodes];

// =============================================================================
// Ticket Types
// =============================================================================

/**
 * Ticket status enum values.
 */
export type TicketStatus =
	| 'intake'
	| 'in_progress'
	| 'waiting_on_parts'
	| 'ready_for_pickup'
	| 'closed'
	| 'archived';

/**
 * Full ticket entity with all fields.
 */
export interface Ticket {
	ticket_id: string;
	friendly_code: string;
	customer_id: string;
	item_type: string | null;
	item_description: string;
	condition_notes: string;
	requested_work: string;
	status: TicketStatus;
	is_rush: boolean;
	promise_date: string | null; // ISO date string (YYYY-MM-DD)
	storage_location_id: string;
	quote_amount: string | null; // Decimal as string for precision
	actual_amount: string | null;
	taken_in_by: string;
	worked_by: string | null;
	closed_by: string | null;
	last_modified_by: string | null;
	created_at: string; // ISO datetime string
	updated_at: string;
	closed_at: string | null;
	queue_position: number | null;
}

/**
 * Extended ticket summary for queue/workboard views.
 */
export interface QueueTicket {
	ticket_id: string;
	friendly_code: string;
	customer_id: string;
	customer_name: string;
	item_type: string | null;
	item_description: string;
	status: TicketStatus;
	is_rush: boolean;
	promise_date: string | null;
	quote_amount: string | null;
	created_at: string;
	is_overdue: boolean;
}

/**
 * Pagination metadata.
 */
export interface PaginationInfo {
	count: number;
	limit: number;
	offset: number;
	has_more: boolean;
}

/**
 * Response for listing tickets.
 */
export interface ListTicketsResponse {
	tickets: QueueTicket[];
	pagination: PaginationInfo;
}

/**
 * Query parameters for listing tickets.
 */
export interface ListTicketsParams {
	status?: string; // Comma-separated values like "intake,in_progress"
	is_rush?: boolean;
	search?: string;
	customer_id?: string;
	from_date?: string; // ISO datetime
	to_date?: string;
	include_archived?: boolean;
	limit?: number;
	offset?: number;
}

// =============================================================================
// Ticket Detail Types
// =============================================================================

/**
 * Employee attribution summary for display.
 */
export interface EmployeeAttribution {
	employee_id: string;
	name: string;
}

/**
 * Customer info in ticket detail response.
 */
export interface TicketCustomer {
	customer_id: string;
	name: string;
	phone: string | null;
	email: string | null;
}

/**
 * Storage location info in ticket detail response.
 */
export interface TicketStorageLocation {
	location_id: string;
	name: string;
}

/**
 * Photo info in ticket detail response.
 */
export interface TicketPhoto {
	photo_id: string;
	url: string;
	uploaded_at: string;
	uploaded_by: EmployeeAttribution;
}

/**
 * Note info in ticket detail response.
 */
export interface TicketNote {
	note_id: string;
	content: string;
	created_at: string;
	created_by: EmployeeAttribution;
}

/**
 * Status history entry in ticket detail response.
 */
export interface TicketStatusHistoryEntry {
	from_status: TicketStatus | null;
	to_status: TicketStatus;
	changed_at: string;
	changed_by: EmployeeAttribution;
}

/**
 * Full ticket detail response.
 */
export interface TicketDetailResponse {
	ticket_id: string;
	friendly_code: string;
	status: TicketStatus;
	is_rush: boolean;
	customer: TicketCustomer;
	item_type: string | null;
	item_description: string;
	condition_notes: string;
	requested_work: string;
	promise_date: string | null;
	storage_location: TicketStorageLocation;
	quote_amount: string | null;
	actual_amount: string | null;
	photos: TicketPhoto[];
	notes: TicketNote[];
	status_history: TicketStatusHistoryEntry[];
	taken_in_by: EmployeeAttribution;
	worked_by: EmployeeAttribution | null;
	closed_by: EmployeeAttribution | null;
	created_at: string;
	updated_at: string;
	closed_at: string | null;
}

// =============================================================================
// Create/Update Ticket Types
// =============================================================================

/**
 * Customer info for inline creation during ticket intake.
 */
export interface InlineCustomer {
	name: string;
	phone?: string | null;
	email?: string | null;
}

/**
 * Request body for creating a new ticket.
 */
export interface CreateTicketRequest {
	customer_id?: string;
	customer?: InlineCustomer;
	item_type?: string | null;
	item_description: string;
	condition_notes: string;
	requested_work: string;
	is_rush?: boolean;
	promise_date?: string | null;
	storage_location_id: string;
	quote_amount?: string | null;
}

/**
 * Response for a created ticket.
 */
export interface CreateTicketResponse extends Ticket {
	receipt_url: string;
	label_url: string;
}

/**
 * Request body for updating a ticket.
 */
export interface UpdateTicketRequest {
	item_type?: string | null;
	item_description?: string;
	condition_notes?: string;
	requested_work?: string;
	is_rush?: boolean;
	promise_date?: string | null;
	storage_location_id?: string;
	quote_amount?: string | null;
	actual_amount?: string | null;
	worked_by_employee_id?: string | null;
}

// =============================================================================
// Queue Types
// =============================================================================

/**
 * A single lane in the workboard queue.
 */
export interface QueueLane {
	count: number;
	tickets: QueueTicket[];
}

/**
 * All lanes in the workboard queue.
 */
export interface QueueLanes {
	intake: QueueLane;
	in_progress: QueueLane;
	waiting_on_parts: QueueLane;
	ready_for_pickup: QueueLane;
}

/**
 * Response for the GET /queue endpoint.
 */
export interface GetQueueResponse {
	lanes: QueueLanes;
}

// =============================================================================
// Status Change Types
// =============================================================================

/**
 * Request body for changing ticket status.
 */
export interface ChangeStatusRequest {
	status: TicketStatus;
}

/**
 * Response for a status change.
 */
export interface ChangeStatusResponse extends Ticket {
	previous_status: TicketStatus;
}

// =============================================================================
// Close Ticket Types
// =============================================================================

/**
 * Request body for closing a ticket.
 */
export interface CloseTicketRequest {
	actual_amount: string; // Decimal as string
}

/**
 * Response for a closed ticket.
 */
export interface CloseTicketResponse extends Ticket {
	previous_status: TicketStatus;
}

// =============================================================================
// Customer Types
// =============================================================================

/**
 * Full customer entity.
 */
export interface Customer {
	customer_id: string;
	name: string;
	phone: string | null;
	email: string | null;
	created_at: string;
	updated_at: string;
}

/**
 * Request for creating a customer.
 */
export interface CreateCustomerRequest {
	name: string;
	phone?: string | null;
	email?: string | null;
}

// =============================================================================
// Employee Types
// =============================================================================

/**
 * Employee role enum values.
 */
export type EmployeeRole = 'staff' | 'admin';

/**
 * Summary view of an employee (without PIN hash).
 */
export interface EmployeeSummary {
	employee_id: string;
	name: string;
	role: EmployeeRole;
	is_active: boolean;
}

/**
 * Response from PIN verification.
 */
export interface VerifyPinResponse {
	employee_id: string;
	name: string;
	role: EmployeeRole;
}

/**
 * Request body for creating an employee.
 */
export interface CreateEmployeeRequest {
	name: string;
	pin: string;
	role?: EmployeeRole;
}

/**
 * Request body for updating an employee.
 */
export interface UpdateEmployeeRequest {
	name?: string;
	pin?: string;
	role?: EmployeeRole;
	is_active?: boolean;
}

/**
 * Response for listing employees.
 */
export interface ListEmployeesResponse {
	employees: EmployeeSummary[];
	count: number;
}

/**
 * Response for deleting an employee.
 */
export interface DeleteEmployeeResponse {
	deleted: boolean;
	warning?: string;
}

// =============================================================================
// Storage Location Types
// =============================================================================

/**
 * Storage location entity.
 */
export interface StorageLocation {
	location_id: string;
	name: string;
	is_active: boolean;
	created_at: string;
	updated_at: string;
}

// =============================================================================
// Store Settings Types
// =============================================================================

/**
 * Store settings entity.
 */
export interface StoreSettings {
	store_name: string;
	store_phone: string | null;
	store_address: string | null;
}

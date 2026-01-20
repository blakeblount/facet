/**
 * Centralized API client for the Facet frontend.
 *
 * Provides type-safe access to all API endpoints with:
 * - Automatic X-Employee-ID header injection
 * - X-Admin-PIN header for admin operations
 * - Consistent error handling
 * - Response parsing and type safety
 */

import type {
	ApiResponse,
	Ticket,
	ListTicketsResponse,
	ListTicketsParams,
	TicketDetailResponse,
	CreateTicketRequest,
	CreateTicketResponse,
	UpdateTicketRequest,
	GetQueueResponse,
	ChangeStatusRequest,
	ChangeStatusResponse,
	CloseTicketRequest,
	CloseTicketResponse,
	TicketStatus,
	Customer,
	CreateCustomerRequest,
	EmployeeSummary,
	StorageLocation,
	StoreSettings
} from '$lib/types/api';

// =============================================================================
// Configuration
// =============================================================================

/**
 * API configuration.
 * Base URL defaults to /api/v1 for same-origin requests in production.
 * Can be overridden for development or testing.
 */
export interface ApiConfig {
	baseUrl: string;
}

const defaultConfig: ApiConfig = {
	baseUrl: '/api/v1'
};

let config: ApiConfig = { ...defaultConfig };

/**
 * Configure the API client.
 * Call this early in your app initialization if you need a custom base URL.
 */
export function configureApi(newConfig: Partial<ApiConfig>): void {
	config = { ...config, ...newConfig };
}

/**
 * Get the current API configuration.
 */
export function getApiConfig(): ApiConfig {
	return { ...config };
}

// =============================================================================
// Employee Context
// =============================================================================

/**
 * The currently authenticated employee ID.
 * Set this after employee login/PIN validation.
 */
let currentEmployeeId: string | null = null;

/**
 * Set the current employee ID for API requests.
 * This will be sent as the X-Employee-ID header on all requests.
 */
export function setCurrentEmployee(employeeId: string | null): void {
	currentEmployeeId = employeeId;
}

/**
 * Get the current employee ID.
 */
export function getCurrentEmployee(): string | null {
	return currentEmployeeId;
}

/**
 * Check if an employee is currently set.
 */
export function hasCurrentEmployee(): boolean {
	return currentEmployeeId !== null;
}

// =============================================================================
// Error Handling
// =============================================================================

/**
 * API error thrown when a request fails.
 */
export class ApiClientError extends Error {
	constructor(
		public readonly code: string,
		message: string,
		public readonly status: number
	) {
		super(message);
		this.name = 'ApiClientError';
	}

	/**
	 * Check if this is a specific error type.
	 */
	isCode(code: string): boolean {
		return this.code === code;
	}

	/**
	 * Check if this is a validation error.
	 */
	isValidationError(): boolean {
		return this.code === 'VALIDATION_ERROR';
	}

	/**
	 * Check if this is a not found error.
	 */
	isNotFound(): boolean {
		return this.code === 'NOT_FOUND';
	}

	/**
	 * Check if this is a forbidden error.
	 */
	isForbidden(): boolean {
		return this.code === 'FORBIDDEN';
	}
}

// =============================================================================
// Core Request Functions
// =============================================================================

/**
 * Build headers for an API request.
 */
function buildHeaders(adminPin?: string): HeadersInit {
	const headers: HeadersInit = {
		'Content-Type': 'application/json'
	};

	if (currentEmployeeId) {
		headers['X-Employee-ID'] = currentEmployeeId;
	}

	if (adminPin) {
		headers['X-Admin-PIN'] = adminPin;
	}

	return headers;
}

/**
 * Build a URL with query parameters.
 */
function buildUrl(path: string, params?: Record<string, unknown>): string {
	const url = new URL(`${config.baseUrl}${path}`, window.location.origin);

	if (params) {
		for (const [key, value] of Object.entries(params)) {
			if (value !== undefined && value !== null) {
				url.searchParams.set(key, String(value));
			}
		}
	}

	return url.toString();
}

/**
 * Parse an API response and handle errors.
 */
async function parseResponse<T>(response: Response): Promise<T> {
	// Handle non-JSON responses (like PDFs)
	const contentType = response.headers.get('content-type');
	if (contentType && !contentType.includes('application/json')) {
		if (!response.ok) {
			throw new ApiClientError(
				'SERVER_ERROR',
				`Request failed with status ${response.status}`,
				response.status
			);
		}
		// Return the response body as-is for non-JSON responses
		return response as unknown as T;
	}

	const json: ApiResponse<T> = await response.json();

	if (json.error) {
		throw new ApiClientError(json.error.code, json.error.message, response.status);
	}

	if (json.data === null) {
		throw new ApiClientError('SERVER_ERROR', 'Unexpected null data in response', response.status);
	}

	return json.data;
}

/**
 * Make a GET request to the API.
 */
async function get<T>(path: string, params?: Record<string, unknown>): Promise<T> {
	const url = buildUrl(path, params);
	const response = await fetch(url, {
		method: 'GET',
		headers: buildHeaders()
	});
	return parseResponse<T>(response);
}

/**
 * Make a POST request to the API.
 */
async function post<T>(path: string, body?: unknown, adminPin?: string): Promise<T> {
	const url = buildUrl(path);
	const response = await fetch(url, {
		method: 'POST',
		headers: buildHeaders(adminPin),
		body: body ? JSON.stringify(body) : undefined
	});
	return parseResponse<T>(response);
}

/**
 * Make a PUT request to the API.
 */
async function put<T>(path: string, body?: unknown, adminPin?: string): Promise<T> {
	const url = buildUrl(path);
	const response = await fetch(url, {
		method: 'PUT',
		headers: buildHeaders(adminPin),
		body: body ? JSON.stringify(body) : undefined
	});
	return parseResponse<T>(response);
}

// =============================================================================
// Health Check
// =============================================================================

/**
 * Health check response.
 */
export interface HealthResponse {
	status: string;
}

/**
 * Check API health.
 */
export async function checkHealth(): Promise<HealthResponse> {
	const response = await fetch(`${config.baseUrl.replace('/api/v1', '')}/health`);
	return response.json();
}

// =============================================================================
// Ticket Endpoints
// =============================================================================

/**
 * List tickets with optional filters.
 */
export async function listTickets(params?: ListTicketsParams): Promise<ListTicketsResponse> {
	return get<ListTicketsResponse>('/tickets', params as Record<string, unknown>);
}

/**
 * Get full ticket details by ID.
 */
export async function getTicket(ticketId: string): Promise<TicketDetailResponse> {
	return get<TicketDetailResponse>(`/tickets/${ticketId}`);
}

/**
 * Create a new ticket.
 * Requires X-Employee-ID header (set via setCurrentEmployee).
 */
export async function createTicket(request: CreateTicketRequest): Promise<CreateTicketResponse> {
	return post<CreateTicketResponse>('/tickets', request);
}

/**
 * Update an existing ticket.
 * Requires X-Employee-ID header (set via setCurrentEmployee).
 * For closed/archived tickets, requires adminPin.
 */
export async function updateTicket(
	ticketId: string,
	request: UpdateTicketRequest,
	adminPin?: string
): Promise<Ticket> {
	return put<Ticket>(`/tickets/${ticketId}`, request, adminPin);
}

/**
 * Change ticket status.
 * Requires X-Employee-ID header (set via setCurrentEmployee).
 */
export async function changeTicketStatus(
	ticketId: string,
	status: TicketStatus
): Promise<ChangeStatusResponse> {
	const request: ChangeStatusRequest = { status };
	return post<ChangeStatusResponse>(`/tickets/${ticketId}/status`, request);
}

/**
 * Close a ticket.
 * Requires X-Employee-ID header (set via setCurrentEmployee).
 * Only tickets with status 'ready_for_pickup' can be closed.
 */
export async function closeTicket(
	ticketId: string,
	actualAmount: string
): Promise<CloseTicketResponse> {
	const request: CloseTicketRequest = { actual_amount: actualAmount };
	return post<CloseTicketResponse>(`/tickets/${ticketId}/close`, request);
}

/**
 * Get the receipt PDF URL for a ticket.
 */
export function getReceiptPdfUrl(ticketId: string): string {
	return `${config.baseUrl}/tickets/${ticketId}/receipt.pdf`;
}

/**
 * Get the label PDF URL for a ticket.
 */
export function getLabelPdfUrl(ticketId: string): string {
	return `${config.baseUrl}/tickets/${ticketId}/label.pdf`;
}

/**
 * Fetch the receipt PDF as a blob.
 */
export async function fetchReceiptPdf(ticketId: string): Promise<Blob> {
	const url = getReceiptPdfUrl(ticketId);
	const response = await fetch(url, {
		method: 'GET',
		headers: buildHeaders()
	});

	if (!response.ok) {
		throw new ApiClientError(
			'SERVER_ERROR',
			`Failed to fetch receipt PDF: ${response.status}`,
			response.status
		);
	}

	return response.blob();
}

/**
 * Fetch the label PDF as a blob.
 */
export async function fetchLabelPdf(ticketId: string): Promise<Blob> {
	const url = getLabelPdfUrl(ticketId);
	const response = await fetch(url, {
		method: 'GET',
		headers: buildHeaders()
	});

	if (!response.ok) {
		throw new ApiClientError(
			'SERVER_ERROR',
			`Failed to fetch label PDF: ${response.status}`,
			response.status
		);
	}

	return response.blob();
}

// =============================================================================
// Queue Endpoints
// =============================================================================

/**
 * Get the workboard queue with tickets grouped by status lane.
 */
export async function getQueue(): Promise<GetQueueResponse> {
	return get<GetQueueResponse>('/queue');
}

// =============================================================================
// Customer Endpoints (Placeholder - not yet implemented in backend)
// =============================================================================

/**
 * List customers with optional search.
 * Note: This endpoint is not yet implemented in the backend.
 */
export async function listCustomers(search?: string): Promise<Customer[]> {
	const params = search ? { search } : undefined;
	return get<Customer[]>('/customers', params);
}

/**
 * Get customer by ID.
 * Note: This endpoint is not yet implemented in the backend.
 */
export async function getCustomer(customerId: string): Promise<Customer> {
	return get<Customer>(`/customers/${customerId}`);
}

/**
 * Create a new customer.
 * Note: This endpoint is not yet implemented in the backend.
 */
export async function createCustomer(request: CreateCustomerRequest): Promise<Customer> {
	return post<Customer>('/customers', request);
}

// =============================================================================
// Employee Endpoints (Placeholder - not yet implemented in backend)
// =============================================================================

/**
 * List all active employees.
 * Note: This endpoint is not yet implemented in the backend.
 */
export async function listEmployees(): Promise<EmployeeSummary[]> {
	return get<EmployeeSummary[]>('/employees');
}

/**
 * Verify employee PIN and get employee info.
 * Note: This endpoint is not yet implemented in the backend.
 */
export async function verifyEmployeePin(employeeId: string, pin: string): Promise<EmployeeSummary> {
	return post<EmployeeSummary>(`/employees/${employeeId}/verify`, { pin });
}

// =============================================================================
// Storage Location Endpoints (Placeholder - not yet implemented in backend)
// =============================================================================

/**
 * List all storage locations.
 * Note: This endpoint is not yet implemented in the backend.
 */
export async function listStorageLocations(): Promise<StorageLocation[]> {
	return get<StorageLocation[]>('/locations');
}

// =============================================================================
// Store Settings Endpoints (Placeholder - not yet implemented in backend)
// =============================================================================

/**
 * Get store settings.
 * Note: This endpoint is not yet implemented in the backend.
 */
export async function getStoreSettings(): Promise<StoreSettings> {
	return get<StoreSettings>('/settings');
}

// =============================================================================
// Re-exports for convenience
// =============================================================================

export type {
	ApiResponse,
	ApiError,
	Ticket,
	QueueTicket,
	ListTicketsResponse,
	ListTicketsParams,
	TicketDetailResponse,
	CreateTicketRequest,
	CreateTicketResponse,
	UpdateTicketRequest,
	GetQueueResponse,
	ChangeStatusRequest,
	ChangeStatusResponse,
	CloseTicketRequest,
	CloseTicketResponse,
	TicketStatus,
	Customer,
	CreateCustomerRequest,
	EmployeeSummary,
	StorageLocation,
	StoreSettings,
	EmployeeAttribution,
	TicketCustomer,
	TicketStorageLocation,
	TicketPhoto,
	TicketNote,
	TicketStatusHistoryEntry,
	QueueLane,
	QueueLanes,
	PaginationInfo,
	InlineCustomer,
	EmployeeRole
} from '$lib/types/api';

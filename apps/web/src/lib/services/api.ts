/**
 * Centralized API client for the Facet frontend.
 *
 * Provides type-safe access to all API endpoints with:
 * - Automatic X-Employee-ID header injection
 * - X-Admin-Session header for admin operations (session-based auth)
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
	VerifyPinResponse,
	StorageLocationSummary,
	ListLocationsResponse,
	CreateStorageLocationRequest,
	UpdateStorageLocationRequest,
	StoreSettings,
	UpdateStoreSettingsRequest,
	CreateEmployeeRequest,
	UpdateEmployeeRequest,
	ListEmployeesResponse,
	DeleteEmployeeResponse
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
// Employee Session Context
// =============================================================================

/**
 * The currently authenticated employee ID.
 * Set this after employee login/PIN validation.
 */
let currentEmployeeId: string | null = null;

/**
 * The current employee session token.
 * Set after successful employee PIN verification.
 */
let employeeSessionToken: string | null = null;

/**
 * Expiration timestamp for the current employee session.
 */
let employeeSessionExpiresAt: Date | null = null;

/**
 * Set the employee session and ID.
 * Called internally after successful employee PIN verification.
 */
function setEmployeeSession(employeeId: string, token: string, expiresAt: string): void {
	currentEmployeeId = employeeId;
	employeeSessionToken = token;
	employeeSessionExpiresAt = new Date(expiresAt);
}

/**
 * Set the current employee ID for API requests (deprecated).
 * @deprecated Use setEmployeeSession instead. This is kept for backward compatibility.
 */
export function setCurrentEmployee(employeeId: string | null): void {
	currentEmployeeId = employeeId;
	if (employeeId === null) {
		employeeSessionToken = null;
		employeeSessionExpiresAt = null;
	}
}

/**
 * Get the current employee ID.
 */
export function getCurrentEmployee(): string | null {
	return currentEmployeeId;
}

/**
 * Check if there's an active employee session.
 */
export function hasEmployeeSession(): boolean {
	if (!employeeSessionToken || !employeeSessionExpiresAt) {
		// Fall back to legacy check
		return currentEmployeeId !== null;
	}
	// Check if session has expired
	if (new Date() > employeeSessionExpiresAt) {
		clearEmployeeSession();
		return false;
	}
	return true;
}

/**
 * Check if an employee is currently set.
 * @deprecated Use hasEmployeeSession instead.
 */
export function hasCurrentEmployee(): boolean {
	return hasEmployeeSession();
}

/**
 * Get the current employee session token.
 * Returns null if no session or session has expired.
 */
export function getEmployeeSessionToken(): string | null {
	if (!employeeSessionToken || !employeeSessionExpiresAt) {
		return null;
	}
	if (new Date() > employeeSessionExpiresAt) {
		clearEmployeeSession();
		return null;
	}
	return employeeSessionToken;
}

/**
 * Clear the employee session.
 * Call this after logout or when the session expires.
 */
export function clearEmployeeSession(): void {
	currentEmployeeId = null;
	employeeSessionToken = null;
	employeeSessionExpiresAt = null;
}

// =============================================================================
// Admin Session Context
// =============================================================================

/**
 * The current admin session token.
 * Set after successful admin PIN verification.
 */
let adminSessionToken: string | null = null;

/**
 * Expiration timestamp for the current admin session.
 */
let adminSessionExpiresAt: Date | null = null;

/**
 * Set the admin session token.
 * Called internally after successful admin PIN verification.
 */
function setAdminSession(token: string, expiresAt: string): void {
	adminSessionToken = token;
	adminSessionExpiresAt = new Date(expiresAt);
}

/**
 * Clear the admin session.
 * Call this after logout or when the session expires.
 */
export function clearAdminSession(): void {
	adminSessionToken = null;
	adminSessionExpiresAt = null;
}

/**
 * Check if there's an active admin session.
 */
export function hasAdminSession(): boolean {
	if (!adminSessionToken || !adminSessionExpiresAt) {
		return false;
	}
	// Check if session has expired
	if (new Date() > adminSessionExpiresAt) {
		clearAdminSession();
		return false;
	}
	return true;
}

/**
 * Get the current admin session token.
 * Returns null if no session or session has expired.
 */
export function getAdminSessionToken(): string | null {
	if (!hasAdminSession()) {
		return null;
	}
	return adminSessionToken;
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
 * @param useAdminSession - If true, includes the X-Admin-Session header if a session is active
 */
function buildHeaders(useAdminSession?: boolean): HeadersInit {
	const headers: HeadersInit = {
		'Content-Type': 'application/json'
	};

	// Use session token if available (preferred), fall back to employee ID (deprecated)
	if (employeeSessionToken) {
		headers['X-Employee-Session'] = employeeSessionToken;
	} else if (currentEmployeeId) {
		// Deprecated: X-Employee-ID header for backward compatibility
		headers['X-Employee-ID'] = currentEmployeeId;
	}

	if (useAdminSession && adminSessionToken) {
		headers['X-Admin-Session'] = adminSessionToken;
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
 * @param useAdminSession - If true, includes admin session authentication
 */
async function post<T>(path: string, body?: unknown, useAdminSession?: boolean): Promise<T> {
	const url = buildUrl(path);
	const response = await fetch(url, {
		method: 'POST',
		headers: buildHeaders(useAdminSession),
		body: body ? JSON.stringify(body) : undefined
	});
	return parseResponse<T>(response);
}

/**
 * Make a PUT request to the API.
 * @param useAdminSession - If true, includes admin session authentication
 */
async function put<T>(path: string, body?: unknown, useAdminSession?: boolean): Promise<T> {
	const url = buildUrl(path);
	const response = await fetch(url, {
		method: 'PUT',
		headers: buildHeaders(useAdminSession),
		body: body ? JSON.stringify(body) : undefined
	});
	return parseResponse<T>(response);
}

/**
 * Make a DELETE request to the API.
 * @param useAdminSession - If true, includes admin session authentication
 */
async function del<T>(path: string, useAdminSession?: boolean): Promise<T> {
	const url = buildUrl(path);
	const response = await fetch(url, {
		method: 'DELETE',
		headers: buildHeaders(useAdminSession)
	});
	return parseResponse<T>(response);
}

/**
 * Make a GET request to the API with admin session authentication.
 */
async function getWithAdmin<T>(path: string, params?: Record<string, unknown>): Promise<T> {
	const url = buildUrl(path, params);
	const response = await fetch(url, {
		method: 'GET',
		headers: buildHeaders(true)
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
 * For closed/archived tickets, requires admin session.
 */
export async function updateTicket(
	ticketId: string,
	request: UpdateTicketRequest,
	useAdminSession?: boolean
): Promise<Ticket> {
	return put<Ticket>(`/tickets/${ticketId}`, request, useAdminSession);
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
 * Request body for toggling rush flag.
 */
export interface ToggleRushRequest {
	is_rush: boolean;
}

/**
 * Response for a rush flag toggle.
 */
export interface ToggleRushResponse extends Ticket {
	previous_is_rush: boolean;
}

/**
 * Toggle rush flag on a ticket.
 * Requires X-Employee-ID header (set via setCurrentEmployee).
 */
export async function toggleRush(ticketId: string, isRush: boolean): Promise<ToggleRushResponse> {
	const request: ToggleRushRequest = { is_rush: isRush };
	return post<ToggleRushResponse>(`/tickets/${ticketId}/rush`, request);
}

/**
 * Request body for adding a note.
 */
export interface AddNoteRequest {
	content: string;
}

/**
 * Response for adding a note.
 */
export interface AddNoteResponse {
	note_id: string;
	ticket_id: string;
	content: string;
	created_by: string;
	created_at: string;
}

/**
 * Add a note to a ticket.
 * Requires X-Employee-ID header (set via setCurrentEmployee).
 */
export async function addTicketNote(ticketId: string, content: string): Promise<AddNoteResponse> {
	const request: AddNoteRequest = { content };
	return post<AddNoteResponse>(`/tickets/${ticketId}/notes`, request);
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
// Employee Endpoints
// =============================================================================

/**
 * List employees (admin only).
 * Requires active admin session.
 * By default returns only active employees.
 * Use includeInactive=true to include inactive employees.
 */
export async function listEmployees(includeInactive?: boolean): Promise<ListEmployeesResponse> {
	const params = includeInactive ? { include_inactive: true } : undefined;
	return getWithAdmin<ListEmployeesResponse>('/employees', params);
}

/**
 * Create a new employee (admin only).
 * Requires active admin session.
 * Returns the created employee summary.
 */
export async function createEmployee(request: CreateEmployeeRequest): Promise<EmployeeSummary> {
	return post<EmployeeSummary>('/employees', request, true);
}

/**
 * Update an employee (admin only).
 * Requires active admin session.
 * Returns the updated employee summary.
 */
export async function updateEmployee(
	employeeId: string,
	request: UpdateEmployeeRequest
): Promise<EmployeeSummary> {
	return put<EmployeeSummary>(`/employees/${employeeId}`, request, true);
}

/**
 * Delete an employee (admin only).
 * Requires active admin session.
 * Returns deletion status with optional warning about history loss.
 */
export async function deleteEmployee(employeeId: string): Promise<DeleteEmployeeResponse> {
	return del<DeleteEmployeeResponse>(`/employees/${employeeId}`, true);
}

/**
 * Verify employee PIN and create an employee session.
 * On success, stores the session token for subsequent employee requests.
 * Returns employee_id, name, role, session_token, and expires_at if the PIN is valid.
 * Throws ApiClientError with code 'INVALID_PIN' if the PIN is invalid.
 */
export async function verifyEmployeePin(pin: string): Promise<VerifyPinResponse> {
	const response = await post<VerifyPinResponse>('/employees/verify', { pin });
	// Store the session token for subsequent requests
	if (response.session_token) {
		setEmployeeSession(response.employee_id, response.session_token, response.expires_at);
	}
	return response;
}

/**
 * Log out the current employee session.
 * Invalidates the session on the server and clears local session state.
 */
export async function employeeLogout(): Promise<void> {
	if (employeeSessionToken) {
		try {
			// Tell the server to invalidate the session
			await post<{ success: boolean }>('/employees/logout', undefined, false);
		} catch {
			// Ignore errors - we'll clear the local session anyway
		}
	}
	clearEmployeeSession();
}

/**
 * Response from admin PIN verification.
 * Includes a session token for subsequent authenticated requests.
 */
export interface AdminVerifyResponse {
	valid: boolean;
	session_token: string;
	expires_at: string;
}

/**
 * Verify the admin PIN and create an admin session.
 * On success, stores the session token for subsequent admin requests.
 * Returns { valid: true, session_token: "...", expires_at: "..." } if the PIN is correct.
 * Throws ApiClientError with code 'INVALID_PIN' if incorrect.
 */
export async function verifyAdminPin(pin: string): Promise<AdminVerifyResponse> {
	const response = await post<AdminVerifyResponse>('/admin/verify', { pin });
	// Store the session token for subsequent requests
	if (response.valid && response.session_token) {
		setAdminSession(response.session_token, response.expires_at);
	}
	return response;
}

/**
 * Log out the current admin session.
 * Invalidates the session on the server and clears local session state.
 */
export async function adminLogout(): Promise<void> {
	if (adminSessionToken) {
		try {
			// Tell the server to invalidate the session
			await post<{ success: boolean }>('/admin/logout', undefined, true);
		} catch {
			// Ignore errors - we'll clear the local session anyway
		}
	}
	clearAdminSession();
}

// =============================================================================
// Storage Location Endpoints
// =============================================================================

/**
 * List all storage locations.
 * Public endpoint - does not require authentication.
 * By default returns only active locations.
 * Use includeInactive=true to include inactive locations.
 */
export async function listStorageLocations(
	includeInactive?: boolean
): Promise<ListLocationsResponse> {
	const params = includeInactive ? { include_inactive: true } : undefined;
	return get<ListLocationsResponse>('/locations', params);
}

/**
 * Create a new storage location (admin only).
 * Requires active admin session.
 * Returns the created location summary.
 */
export async function createStorageLocation(
	request: CreateStorageLocationRequest
): Promise<StorageLocationSummary> {
	return post<StorageLocationSummary>('/locations', request, true);
}

/**
 * Update a storage location (admin only).
 * Requires active admin session.
 * Returns the updated location summary.
 */
export async function updateStorageLocation(
	locationId: string,
	request: UpdateStorageLocationRequest
): Promise<StorageLocationSummary> {
	return put<StorageLocationSummary>(`/locations/${locationId}`, request, true);
}

// =============================================================================
// Store Settings Endpoints
// =============================================================================

/**
 * Get store settings.
 * Public endpoint - does not require authentication.
 */
export async function getStoreSettings(): Promise<StoreSettings> {
	return get<StoreSettings>('/settings');
}

/**
 * Update store settings.
 * Requires active admin session.
 *
 * @param updates - Partial store settings to update
 * @returns Updated store settings
 */
export async function updateStoreSettings(
	updates: UpdateStoreSettingsRequest
): Promise<StoreSettings> {
	return put<StoreSettings>('/settings', updates, true);
}

// =============================================================================
// Photo Upload
// =============================================================================

/**
 * Response from uploading a photo to a ticket.
 */
export interface UploadPhotoResponse {
	photo_id: string;
	ticket_id: string;
	storage_key: string;
	content_type: string;
	size_bytes: number;
	uploaded_at: string;
	uploaded_by: string;
	url: string;
}

/**
 * Upload a photo to a ticket.
 * Uses multipart/form-data with a field named "photo".
 * Returns progress via the onProgress callback.
 */
export async function uploadTicketPhoto(
	ticketId: string,
	file: File,
	onProgress?: (progress: number) => void
): Promise<UploadPhotoResponse> {
	const url = buildUrl(`/tickets/${ticketId}/photos`);

	const formData = new FormData();
	formData.append('photo', file);

	return new Promise((resolve, reject) => {
		const xhr = new XMLHttpRequest();

		xhr.upload.addEventListener('progress', (event) => {
			if (event.lengthComputable && onProgress) {
				const progress = Math.round((event.loaded / event.total) * 100);
				onProgress(progress);
			}
		});

		xhr.addEventListener('load', () => {
			if (xhr.status >= 200 && xhr.status < 300) {
				try {
					const response = JSON.parse(xhr.responseText);
					if (response.error) {
						reject(new ApiClientError(response.error.code, response.error.message, xhr.status));
					} else {
						resolve(response.data);
					}
				} catch {
					reject(new ApiClientError('SERVER_ERROR', 'Failed to parse response', xhr.status));
				}
			} else {
				try {
					const response = JSON.parse(xhr.responseText);
					if (response.error) {
						reject(new ApiClientError(response.error.code, response.error.message, xhr.status));
					} else {
						reject(
							new ApiClientError(
								'SERVER_ERROR',
								`Request failed with status ${xhr.status}`,
								xhr.status
							)
						);
					}
				} catch {
					reject(
						new ApiClientError(
							'SERVER_ERROR',
							`Request failed with status ${xhr.status}`,
							xhr.status
						)
					);
				}
			}
		});

		xhr.addEventListener('error', () => {
			reject(new ApiClientError('SERVER_ERROR', 'Network error', 0));
		});

		xhr.open('POST', url);

		// Add headers - use session token if available (preferred), fall back to employee ID (deprecated)
		if (employeeSessionToken) {
			xhr.setRequestHeader('X-Employee-Session', employeeSessionToken);
		} else if (currentEmployeeId) {
			// Deprecated: X-Employee-ID header for backward compatibility
			xhr.setRequestHeader('X-Employee-ID', currentEmployeeId);
		}

		xhr.send(formData);
	});
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
	EmployeeInfo,
	VerifyPinResponse,
	StorageLocation,
	StorageLocationSummary,
	ListLocationsResponse,
	CreateStorageLocationRequest,
	UpdateStorageLocationRequest,
	StoreSettings,
	UpdateStoreSettingsRequest,
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
	EmployeeRole,
	CreateEmployeeRequest,
	UpdateEmployeeRequest,
	ListEmployeesResponse,
	DeleteEmployeeResponse
} from '$lib/types/api';

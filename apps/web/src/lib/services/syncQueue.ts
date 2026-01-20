/**
 * Sync Queue Service for offline ticket creation
 *
 * Stores offline tickets in IndexedDB and syncs them when back online.
 * Uses client-side UUIDs to identify tickets.
 */

import {
	createTicket,
	uploadTicketPhoto,
	setCurrentEmployee,
	type CreateTicketRequest
} from './api';

// =============================================================================
// Types
// =============================================================================

/**
 * Status of a queued ticket
 */
export type QueuedTicketStatus = 'pending' | 'syncing' | 'synced' | 'failed';

/**
 * A photo stored for offline upload
 */
export interface QueuedPhoto {
	/** Client-side unique ID */
	id: string;
	/** File name */
	name: string;
	/** MIME type */
	type: string;
	/** File data as ArrayBuffer (blobs don't persist well in IndexedDB) */
	data: ArrayBuffer;
	/** File size in bytes */
	size: number;
}

/**
 * A ticket queued for sync
 */
export interface QueuedTicket {
	/** Client-side UUID (will be replaced by server ID after sync) */
	clientId: string;
	/** The ticket creation request data */
	request: CreateTicketRequest;
	/** Photos to upload after ticket creation */
	photos: QueuedPhoto[];
	/** Employee ID who created the ticket */
	employeeId: string;
	/** Employee name for display */
	employeeName: string;
	/** When the ticket was queued */
	queuedAt: string;
	/** Current sync status */
	status: QueuedTicketStatus;
	/** Error message if sync failed */
	errorMessage?: string;
	/** Server ticket ID after successful sync */
	serverTicketId?: string;
	/** Server friendly code after successful sync */
	serverFriendlyCode?: string;
	/** Number of sync attempts */
	syncAttempts: number;
}

/**
 * Result of a sync operation
 */
export interface SyncResult {
	success: boolean;
	ticketId?: string;
	friendlyCode?: string;
	error?: string;
}

/**
 * Callback for sync progress updates
 */
export type SyncProgressCallback = (
	clientId: string,
	status: QueuedTicketStatus,
	result?: SyncResult
) => void;

// =============================================================================
// IndexedDB Setup
// =============================================================================

const DB_NAME = 'facet-offline';
const DB_VERSION = 1;
const STORE_NAME = 'queued-tickets';

let db: IDBDatabase | null = null;

/**
 * Open or create the IndexedDB database
 */
async function openDatabase(): Promise<IDBDatabase> {
	if (db) {
		return db;
	}

	return new Promise((resolve, reject) => {
		const request = indexedDB.open(DB_NAME, DB_VERSION);

		request.onerror = () => {
			reject(new Error('Failed to open IndexedDB'));
		};

		request.onsuccess = () => {
			db = request.result;
			resolve(db);
		};

		request.onupgradeneeded = (event) => {
			const database = (event.target as IDBOpenDBRequest).result;

			// Create object store for queued tickets
			if (!database.objectStoreNames.contains(STORE_NAME)) {
				const store = database.createObjectStore(STORE_NAME, { keyPath: 'clientId' });
				store.createIndex('status', 'status', { unique: false });
				store.createIndex('queuedAt', 'queuedAt', { unique: false });
			}
		};
	});
}

/**
 * Get an object store for a transaction
 */
async function getStore(mode: IDBTransactionMode): Promise<IDBObjectStore> {
	const database = await openDatabase();
	const transaction = database.transaction(STORE_NAME, mode);
	return transaction.objectStore(STORE_NAME);
}

// =============================================================================
// Queue Operations
// =============================================================================

/**
 * Generate a client-side UUID
 */
function generateClientId(): string {
	return crypto.randomUUID();
}

/**
 * Convert a File to a QueuedPhoto with ArrayBuffer data
 */
async function fileToQueuedPhoto(file: File): Promise<QueuedPhoto> {
	const data = await file.arrayBuffer();
	return {
		id: generateClientId(),
		name: file.name,
		type: file.type,
		data,
		size: file.size
	};
}

/**
 * Convert a QueuedPhoto back to a File
 */
function queuedPhotoToFile(photo: QueuedPhoto): File {
	const blob = new Blob([photo.data], { type: photo.type });
	return new File([blob], photo.name, { type: photo.type });
}

/**
 * Add a ticket to the sync queue
 */
export async function queueTicket(
	request: CreateTicketRequest,
	photos: File[],
	employeeId: string,
	employeeName: string
): Promise<QueuedTicket> {
	const store = await getStore('readwrite');

	// Convert photos to storable format
	const queuedPhotos = await Promise.all(photos.map(fileToQueuedPhoto));

	const ticket: QueuedTicket = {
		clientId: generateClientId(),
		request,
		photos: queuedPhotos,
		employeeId,
		employeeName,
		queuedAt: new Date().toISOString(),
		status: 'pending',
		syncAttempts: 0
	};

	return new Promise((resolve, reject) => {
		const addRequest = store.add(ticket);

		addRequest.onsuccess = () => {
			resolve(ticket);
		};

		addRequest.onerror = () => {
			reject(new Error('Failed to queue ticket'));
		};
	});
}

/**
 * Get all queued tickets
 */
export async function getQueuedTickets(): Promise<QueuedTicket[]> {
	const store = await getStore('readonly');

	return new Promise((resolve, reject) => {
		const request = store.getAll();

		request.onsuccess = () => {
			resolve(request.result);
		};

		request.onerror = () => {
			reject(new Error('Failed to get queued tickets'));
		};
	});
}

/**
 * Get pending tickets (not yet synced or previously failed)
 */
export async function getPendingTickets(): Promise<QueuedTicket[]> {
	const tickets = await getQueuedTickets();
	return tickets.filter((t) => t.status === 'pending' || t.status === 'failed');
}

/**
 * Get count of pending tickets
 */
export async function getPendingCount(): Promise<number> {
	const pending = await getPendingTickets();
	return pending.length;
}

/**
 * Update a queued ticket
 */
async function updateTicket(ticket: QueuedTicket): Promise<void> {
	const store = await getStore('readwrite');

	return new Promise((resolve, reject) => {
		const request = store.put(ticket);

		request.onsuccess = () => {
			resolve();
		};

		request.onerror = () => {
			reject(new Error('Failed to update ticket'));
		};
	});
}

/**
 * Remove a synced ticket from the queue
 */
export async function removeTicket(clientId: string): Promise<void> {
	const store = await getStore('readwrite');

	return new Promise((resolve, reject) => {
		const request = store.delete(clientId);

		request.onsuccess = () => {
			resolve();
		};

		request.onerror = () => {
			reject(new Error('Failed to remove ticket'));
		};
	});
}

/**
 * Clear all synced tickets from the queue
 */
export async function clearSyncedTickets(): Promise<void> {
	const tickets = await getQueuedTickets();
	const synced = tickets.filter((t) => t.status === 'synced');

	for (const ticket of synced) {
		await removeTicket(ticket.clientId);
	}
}

// =============================================================================
// Sync Operations
// =============================================================================

/**
 * Sync a single ticket to the server
 */
async function syncTicket(ticket: QueuedTicket): Promise<SyncResult> {
	try {
		// Set the employee context for API requests
		setCurrentEmployee(ticket.employeeId);

		// Create the ticket
		const response = await createTicket(ticket.request);

		// Upload photos
		for (const photo of ticket.photos) {
			const file = queuedPhotoToFile(photo);
			await uploadTicketPhoto(response.ticket_id, file);
		}

		return {
			success: true,
			ticketId: response.ticket_id,
			friendlyCode: response.friendly_code
		};
	} catch (err) {
		return {
			success: false,
			error: err instanceof Error ? err.message : 'Unknown error'
		};
	}
}

/**
 * Sync all pending tickets
 * Returns the results for each ticket
 */
export async function syncAllPending(
	onProgress?: SyncProgressCallback
): Promise<Map<string, SyncResult>> {
	const results = new Map<string, SyncResult>();
	const pending = await getPendingTickets();

	for (const ticket of pending) {
		// Update status to syncing
		ticket.status = 'syncing';
		ticket.syncAttempts++;
		await updateTicket(ticket);
		onProgress?.(ticket.clientId, 'syncing');

		// Attempt sync
		const result = await syncTicket(ticket);
		results.set(ticket.clientId, result);

		if (result.success) {
			// Update ticket with server info
			ticket.status = 'synced';
			ticket.serverTicketId = result.ticketId;
			ticket.serverFriendlyCode = result.friendlyCode;
			ticket.errorMessage = undefined;
		} else {
			// Mark as failed
			ticket.status = 'failed';
			ticket.errorMessage = result.error;
		}

		await updateTicket(ticket);
		onProgress?.(ticket.clientId, ticket.status, result);
	}

	return results;
}

/**
 * Check if there are any pending tickets
 */
export async function hasPendingTickets(): Promise<boolean> {
	const count = await getPendingCount();
	return count > 0;
}

// =============================================================================
// Store Integration
// =============================================================================

interface SyncQueueState {
	pendingCount: number;
	isSyncing: boolean;
	lastSyncAt: Date | null;
	lastSyncError: string | null;
}

function createSyncQueueStore() {
	const state = $state<SyncQueueState>({
		pendingCount: 0,
		isSyncing: false,
		lastSyncAt: null,
		lastSyncError: null
	});

	/**
	 * Refresh the pending count from IndexedDB
	 */
	async function refreshCount(): Promise<void> {
		try {
			state.pendingCount = await getPendingCount();
		} catch {
			// IndexedDB not available or error
			state.pendingCount = 0;
		}
	}

	/**
	 * Initialize the store
	 */
	async function init(): Promise<void> {
		await refreshCount();
	}

	/**
	 * Queue a ticket and update the pending count
	 */
	async function queue(
		request: CreateTicketRequest,
		photos: File[],
		employeeId: string,
		employeeName: string
	): Promise<QueuedTicket> {
		const ticket = await queueTicket(request, photos, employeeId, employeeName);
		await refreshCount();
		return ticket;
	}

	/**
	 * Sync all pending tickets
	 */
	async function syncAll(onProgress?: SyncProgressCallback): Promise<Map<string, SyncResult>> {
		if (state.isSyncing) {
			return new Map();
		}

		state.isSyncing = true;
		state.lastSyncError = null;

		try {
			const results = await syncAllPending(onProgress);
			state.lastSyncAt = new Date();

			// Check for any failures
			const failures = [...results.values()].filter((r) => !r.success);
			if (failures.length > 0) {
				state.lastSyncError = `${failures.length} ticket(s) failed to sync`;
			}

			await refreshCount();
			return results;
		} catch (err) {
			state.lastSyncError = err instanceof Error ? err.message : 'Sync failed';
			throw err;
		} finally {
			state.isSyncing = false;
		}
	}

	/**
	 * Clear synced tickets and refresh count
	 */
	async function clearSynced(): Promise<void> {
		await clearSyncedTickets();
		await refreshCount();
	}

	return {
		/** Number of pending tickets (reactive) */
		get pendingCount() {
			return state.pendingCount;
		},

		/** Whether sync is in progress (reactive) */
		get isSyncing() {
			return state.isSyncing;
		},

		/** When last sync completed (reactive) */
		get lastSyncAt() {
			return state.lastSyncAt;
		},

		/** Last sync error message (reactive) */
		get lastSyncError() {
			return state.lastSyncError;
		},

		/** Whether there are pending tickets (reactive) */
		get hasPending() {
			return state.pendingCount > 0;
		},

		/** Initialize the store */
		init,

		/** Refresh pending count from IndexedDB */
		refreshCount,

		/** Queue a ticket for offline sync */
		queue,

		/** Sync all pending tickets */
		syncAll,

		/** Clear synced tickets */
		clearSynced,

		/** Get all queued tickets */
		getAll: getQueuedTickets,

		/** Get pending tickets */
		getPending: getPendingTickets
	};
}

/**
 * Global sync queue store instance
 *
 * Usage:
 * ```ts
 * import { syncQueueStore } from '$lib/services/syncQueue';
 *
 * // Initialize on app load
 * await syncQueueStore.init();
 *
 * // Queue a ticket when offline
 * const ticket = await syncQueueStore.queue(request, photos, employeeId, employeeName);
 *
 * // Sync when back online
 * const results = await syncQueueStore.syncAll((clientId, status, result) => {
 *   console.log(`Ticket ${clientId}: ${status}`);
 * });
 *
 * // Check pending count (reactive)
 * if (syncQueueStore.hasPending) {
 *   console.log(`${syncQueueStore.pendingCount} tickets waiting to sync`);
 * }
 * ```
 */
export const syncQueueStore = createSyncQueueStore();

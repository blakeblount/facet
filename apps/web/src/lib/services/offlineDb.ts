/**
 * Shared IndexedDB setup for offline functionality
 *
 * This module manages the shared database connection and schema
 * for all offline features (sync queue, employee cache, etc.)
 */

// =============================================================================
// Configuration
// =============================================================================

export const DB_NAME = 'facet-offline';
export const DB_VERSION = 2;

// Store names
export const QUEUED_TICKETS_STORE = 'queued-tickets';
export const EMPLOYEE_CACHE_STORE = 'employee-cache';

// =============================================================================
// Database Connection
// =============================================================================

let db: IDBDatabase | null = null;

/**
 * Open or get the shared IndexedDB database
 */
export async function getDatabase(): Promise<IDBDatabase> {
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

			// Create store for queued tickets (version 1)
			if (!database.objectStoreNames.contains(QUEUED_TICKETS_STORE)) {
				const ticketStore = database.createObjectStore(QUEUED_TICKETS_STORE, {
					keyPath: 'clientId'
				});
				ticketStore.createIndex('status', 'status', { unique: false });
				ticketStore.createIndex('queuedAt', 'queuedAt', { unique: false });
			}

			// Create store for cached employees (version 2)
			if (!database.objectStoreNames.contains(EMPLOYEE_CACHE_STORE)) {
				database.createObjectStore(EMPLOYEE_CACHE_STORE, { keyPath: 'employeeId' });
			}
		};
	});
}

/**
 * Get an object store for a transaction
 */
export async function getStore(
	storeName: string,
	mode: IDBTransactionMode
): Promise<IDBObjectStore> {
	const database = await getDatabase();
	const transaction = database.transaction(storeName, mode);
	return transaction.objectStore(storeName);
}

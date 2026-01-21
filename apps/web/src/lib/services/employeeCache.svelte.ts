/**
 * Employee Cache Service for offline PIN verification
 *
 * Caches employee credentials after successful online PIN verification,
 * allowing offline ticket creation with local PIN verification.
 *
 * Security considerations:
 * - PIN is hashed using SHA-256 before storage
 * - Cached credentials expire after a configurable time (default: 7 days)
 * - Only employees who have successfully verified online can work offline
 */

import type { VerifyPinResponse, EmployeeInfo } from '$lib/types/api';
import { getStore, EMPLOYEE_CACHE_STORE } from './offlineDb';

// =============================================================================
// Types
// =============================================================================

/**
 * Cached employee credentials stored in IndexedDB
 */
export interface CachedEmployee {
	/** Employee ID */
	employeeId: string;
	/** Employee name */
	name: string;
	/** Employee role */
	role: string;
	/** SHA-256 hash of the PIN */
	pinHash: string;
	/** When the credentials were cached */
	cachedAt: string;
	/** When the credentials expire */
	expiresAt: string;
}

// =============================================================================
// Configuration
// =============================================================================

const STORE_NAME = EMPLOYEE_CACHE_STORE;

/** Default cache duration: 7 days */
const DEFAULT_CACHE_DURATION_MS = 7 * 24 * 60 * 60 * 1000;

// =============================================================================
// Hashing
// =============================================================================

/**
 * Hash a PIN using SHA-256
 * Uses the Web Crypto API for secure hashing
 */
async function hashPin(pin: string): Promise<string> {
	const encoder = new TextEncoder();
	const data = encoder.encode(pin);
	const hashBuffer = await crypto.subtle.digest('SHA-256', data);
	const hashArray = Array.from(new Uint8Array(hashBuffer));
	return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}

// =============================================================================
// IndexedDB Setup (uses shared offlineDb)
// =============================================================================

/**
 * Get the employee cache object store
 */
async function getEmployeeStore(mode: IDBTransactionMode): Promise<IDBObjectStore> {
	return getStore(STORE_NAME, mode);
}

// =============================================================================
// Cache Operations
// =============================================================================

/**
 * Cache employee credentials after successful online verification
 */
export async function cacheEmployee(
	employee: VerifyPinResponse,
	pin: string,
	cacheDurationMs: number = DEFAULT_CACHE_DURATION_MS
): Promise<void> {
	const store = await getEmployeeStore('readwrite');

	const pinHash = await hashPin(pin);
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
	const now = new Date();
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
	const expiresAt = new Date(now.getTime() + cacheDurationMs);

	const cached: CachedEmployee = {
		employeeId: employee.employee_id,
		name: employee.name,
		role: employee.role,
		pinHash,
		cachedAt: now.toISOString(),
		expiresAt: expiresAt.toISOString()
	};

	return new Promise((resolve, reject) => {
		const request = store.put(cached);

		request.onsuccess = () => {
			resolve();
		};

		request.onerror = () => {
			reject(new Error('Failed to cache employee credentials'));
		};
	});
}

/**
 * Get all cached employees
 */
async function getAllCachedEmployees(): Promise<CachedEmployee[]> {
	const store = await getEmployeeStore('readonly');

	return new Promise((resolve, reject) => {
		const request = store.getAll();

		request.onsuccess = () => {
			resolve(request.result);
		};

		request.onerror = () => {
			reject(new Error('Failed to get cached employees'));
		};
	});
}

/**
 * Verify PIN against cached credentials
 * Returns the employee data if verification succeeds, null otherwise
 */
export async function verifyCachedPin(pin: string): Promise<EmployeeInfo | null> {
	const pinHash = await hashPin(pin);
	const cached = await getAllCachedEmployees();
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
	const now = new Date();

	// Find employee with matching PIN hash that hasn't expired
	for (const employee of cached) {
		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
		const expiresAt = new Date(employee.expiresAt);

		// Skip expired credentials
		if (expiresAt <= now) {
			continue;
		}

		// Check PIN hash
		if (employee.pinHash === pinHash) {
			return {
				employee_id: employee.employeeId,
				name: employee.name,
				role: employee.role as 'staff' | 'admin'
			};
		}
	}

	return null;
}

/**
 * Check if there are any cached credentials (not expired)
 */
export async function hasCachedCredentials(): Promise<boolean> {
	const cached = await getAllCachedEmployees();
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
	const now = new Date();

	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
	return cached.some((e) => new Date(e.expiresAt) > now);
}

/**
 * Clear all cached credentials
 */
export async function clearCachedCredentials(): Promise<void> {
	const store = await getEmployeeStore('readwrite');

	return new Promise((resolve, reject) => {
		const request = store.clear();

		request.onsuccess = () => {
			resolve();
		};

		request.onerror = () => {
			reject(new Error('Failed to clear cached credentials'));
		};
	});
}

/**
 * Remove expired credentials
 */
export async function cleanupExpiredCredentials(): Promise<number> {
	const cached = await getAllCachedEmployees();
	const store = await getEmployeeStore('readwrite');
	// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
	const now = new Date();
	let removed = 0;

	for (const employee of cached) {
		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- not reactive state
		const expiresAt = new Date(employee.expiresAt);
		if (expiresAt <= now) {
			await new Promise<void>((resolve, reject) => {
				const request = store.delete(employee.employeeId);
				request.onsuccess = () => {
					removed++;
					resolve();
				};
				request.onerror = () => reject(new Error('Failed to delete expired credential'));
			});
		}
	}

	return removed;
}

// =============================================================================
// Store Integration
// =============================================================================

interface EmployeeCacheState {
	hasCachedCredentials: boolean;
}

function createEmployeeCacheStore() {
	const state = $state<EmployeeCacheState>({
		hasCachedCredentials: false
	});

	/**
	 * Refresh the cached credentials state
	 */
	async function refresh(): Promise<void> {
		try {
			state.hasCachedCredentials = await hasCachedCredentials();
		} catch {
			state.hasCachedCredentials = false;
		}
	}

	/**
	 * Initialize the store
	 */
	async function init(): Promise<void> {
		await refresh();
		// Clean up expired credentials on init
		await cleanupExpiredCredentials();
	}

	/**
	 * Cache an employee after successful verification
	 */
	async function cache(employee: VerifyPinResponse, pin: string): Promise<void> {
		await cacheEmployee(employee, pin);
		await refresh();
	}

	/**
	 * Verify PIN offline
	 */
	async function verifyOffline(pin: string): Promise<EmployeeInfo | null> {
		return verifyCachedPin(pin);
	}

	/**
	 * Clear all cached credentials
	 */
	async function clear(): Promise<void> {
		await clearCachedCredentials();
		await refresh();
	}

	return {
		/** Whether there are cached credentials available (reactive) */
		get hasCachedCredentials() {
			return state.hasCachedCredentials;
		},

		/** Initialize the store */
		init,

		/** Refresh the cached credentials state */
		refresh,

		/** Cache employee credentials after successful verification */
		cache,

		/** Verify PIN against cached credentials */
		verifyOffline,

		/** Clear all cached credentials */
		clear
	};
}

/**
 * Global employee cache store instance
 *
 * Usage:
 * ```ts
 * import { employeeCacheStore } from '$lib/services/employeeCache.svelte';
 *
 * // Initialize on app load
 * await employeeCacheStore.init();
 *
 * // After successful online verification, cache the credentials
 * await employeeCacheStore.cache(employee, pin);
 *
 * // When offline, verify against cache
 * const employee = await employeeCacheStore.verifyOffline(pin);
 * if (employee) {
 *   // Proceed with offline operation
 * } else {
 *   // Show "Invalid PIN" error
 * }
 * ```
 */
export const employeeCacheStore = createEmployeeCacheStore();

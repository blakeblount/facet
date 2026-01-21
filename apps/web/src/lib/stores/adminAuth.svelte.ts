/**
 * Admin authentication store for managing admin PIN session state.
 *
 * Provides a reactive store that:
 * - Tracks admin authentication status
 * - Stores the verified admin PIN for API calls
 * - Persists session in sessionStorage (cleared on browser close)
 */

import { verifyAdminPin, ApiClientError } from '$lib/services/api';

const STORAGE_KEY = 'facet_admin_pin';

interface AdminAuthState {
	/** Whether the admin is currently authenticated */
	isAuthenticated: boolean;
	/** The verified admin PIN (for API calls) */
	pin: string | null;
	/** Whether authentication is in progress */
	isVerifying: boolean;
	/** Last verification error message */
	error: string | null;
}

function createAdminAuthStore() {
	const state = $state<AdminAuthState>({
		isAuthenticated: false,
		pin: null,
		isVerifying: false,
		error: null
	});

	/**
	 * Initialize the store from sessionStorage
	 */
	function init() {
		if (typeof window === 'undefined') {
			return;
		}

		const stored = sessionStorage.getItem(STORAGE_KEY);
		if (stored) {
			// We have a stored PIN - assume it's still valid
			// The API will reject it if it's been changed
			state.pin = stored;
			state.isAuthenticated = true;
		}
	}

	/**
	 * Verify the admin PIN and establish a session
	 */
	async function verify(pin: string): Promise<boolean> {
		state.isVerifying = true;
		state.error = null;

		try {
			await verifyAdminPin(pin);

			// PIN is valid - store it
			state.pin = pin;
			state.isAuthenticated = true;

			// Persist in sessionStorage
			if (typeof window !== 'undefined') {
				sessionStorage.setItem(STORAGE_KEY, pin);
			}

			return true;
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isCode('INVALID_PIN')) {
					state.error = 'Invalid admin PIN. Please try again.';
				} else {
					state.error = err.message || 'Verification failed. Please try again.';
				}
			} else {
				state.error = 'An error occurred. Please try again.';
			}
			return false;
		} finally {
			state.isVerifying = false;
		}
	}

	/**
	 * Clear the admin session
	 */
	function logout() {
		state.isAuthenticated = false;
		state.pin = null;
		state.error = null;

		if (typeof window !== 'undefined') {
			sessionStorage.removeItem(STORAGE_KEY);
		}
	}

	/**
	 * Clear any error message
	 */
	function clearError() {
		state.error = null;
	}

	return {
		/** Whether admin is authenticated (reactive) */
		get isAuthenticated() {
			return state.isAuthenticated;
		},

		/** The verified admin PIN for API calls (reactive) */
		get pin() {
			return state.pin;
		},

		/** Whether verification is in progress (reactive) */
		get isVerifying() {
			return state.isVerifying;
		},

		/** Last error message (reactive) */
		get error() {
			return state.error;
		},

		/** Initialize from sessionStorage */
		init,

		/** Verify admin PIN */
		verify,

		/** Clear session */
		logout,

		/** Clear error */
		clearError
	};
}

/**
 * Global admin auth store instance
 *
 * Usage:
 * ```ts
 * import { adminAuthStore } from '$lib/stores/adminAuth.svelte';
 *
 * // Initialize on admin page load
 * adminAuthStore.init();
 *
 * // Check if authenticated (reactive)
 * if (adminAuthStore.isAuthenticated) {
 *   // Make admin API calls with adminAuthStore.pin
 * }
 *
 * // Verify PIN
 * const success = await adminAuthStore.verify(pin);
 *
 * // Logout
 * adminAuthStore.logout();
 * ```
 */
export const adminAuthStore = createAdminAuthStore();

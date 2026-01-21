/**
 * Admin authentication store for managing admin session state.
 *
 * Provides a reactive store that:
 * - Tracks admin authentication status
 * - Uses session-based auth (session token stored in api.ts)
 * - Persists auth state in sessionStorage (cleared on browser close)
 */

import { verifyAdminPin, adminLogout, hasAdminSession, ApiClientError } from '$lib/services/api';

const STORAGE_KEY = 'facet_admin_auth';

interface AdminAuthState {
	/** Whether the admin is currently authenticated */
	isAuthenticated: boolean;
	/** Whether authentication is in progress */
	isVerifying: boolean;
	/** Last verification error message */
	error: string | null;
}

function createAdminAuthStore() {
	const state = $state<AdminAuthState>({
		isAuthenticated: false,
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

		// Check if we have an active session
		const stored = sessionStorage.getItem(STORAGE_KEY);
		if (stored === 'true' && hasAdminSession()) {
			state.isAuthenticated = true;
		} else {
			// Clear any stale auth state
			sessionStorage.removeItem(STORAGE_KEY);
			state.isAuthenticated = false;
		}
	}

	/**
	 * Verify the admin PIN and establish a session
	 */
	async function verify(pin: string): Promise<boolean> {
		state.isVerifying = true;
		state.error = null;

		try {
			// verifyAdminPin now automatically stores the session token
			await verifyAdminPin(pin);

			// Session is now active
			state.isAuthenticated = true;

			// Persist auth state in sessionStorage
			if (typeof window !== 'undefined') {
				sessionStorage.setItem(STORAGE_KEY, 'true');
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
	async function logout() {
		// Log out on the server (invalidates session token)
		await adminLogout();

		state.isAuthenticated = false;
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

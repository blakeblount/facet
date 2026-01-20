/**
 * Offline status store for managing network connectivity
 *
 * Provides a reactive store that tracks online/offline status using:
 * - navigator.onLine property
 * - online/offline event listeners
 */

interface OfflineState {
	isOnline: boolean;
	lastOnlineAt: Date | null;
	lastOfflineAt: Date | null;
}

function createOfflineStore() {
	const state = $state<OfflineState>({
		isOnline: typeof navigator !== 'undefined' ? navigator.onLine : true,
		lastOnlineAt: null,
		lastOfflineAt: null
	});

	/**
	 * Handle online event
	 */
	function handleOnline() {
		state.isOnline = true;
		state.lastOnlineAt = new Date();
	}

	/**
	 * Handle offline event
	 */
	function handleOffline() {
		state.isOnline = false;
		state.lastOfflineAt = new Date();
	}

	/**
	 * Initialize event listeners for online/offline events
	 * Should be called once when app loads
	 */
	function init() {
		if (typeof window === 'undefined') {
			return;
		}

		// Set initial state
		state.isOnline = navigator.onLine;

		// Add event listeners
		window.addEventListener('online', handleOnline);
		window.addEventListener('offline', handleOffline);
	}

	/**
	 * Clean up event listeners
	 * Should be called when app unmounts (rarely needed in SvelteKit)
	 */
	function destroy() {
		if (typeof window === 'undefined') {
			return;
		}

		window.removeEventListener('online', handleOnline);
		window.removeEventListener('offline', handleOffline);
	}

	return {
		/** Whether the app is currently online (reactive) */
		get isOnline() {
			return state.isOnline;
		},

		/** Whether the app is currently offline (reactive) */
		get isOffline() {
			return !state.isOnline;
		},

		/** When the app last came online (reactive) */
		get lastOnlineAt() {
			return state.lastOnlineAt;
		},

		/** When the app last went offline (reactive) */
		get lastOfflineAt() {
			return state.lastOfflineAt;
		},

		/** Initialize event listeners */
		init,

		/** Clean up event listeners */
		destroy
	};
}

/**
 * Global offline store instance
 *
 * Usage:
 * ```ts
 * import { offlineStore } from '$lib/stores/offline.svelte';
 *
 * // Initialize on app load (e.g., in +layout.svelte)
 * offlineStore.init();
 *
 * // Check online status (reactive)
 * if (offlineStore.isOffline) {
 *   // Handle offline state
 * }
 * ```
 */
export const offlineStore = createOfflineStore();

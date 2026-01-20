/**
 * Theme store for managing application themes
 *
 * Provides a reactive store for theme switching with localStorage persistence.
 * Applies theme by setting data-theme attribute on the html element.
 */

/** Available themes */
export type Theme = 'imperial' | 'arcane';

/** All available themes */
export const THEMES: Theme[] = ['imperial', 'arcane'];

/** Theme display names for UI */
export const THEME_NAMES: Record<Theme, string> = {
	imperial: 'Imperial',
	arcane: 'Arcane'
};

/** Theme descriptions for UI */
export const THEME_DESCRIPTIONS: Record<Theme, string> = {
	imperial: 'Clean, sophisticated American luxury aesthetic',
	arcane: 'Fantasy-inspired pixel art with steampunk undertones'
};

/** Default theme */
export const DEFAULT_THEME: Theme = 'imperial';

/** LocalStorage key for theme preference */
const STORAGE_KEY = 'facet-theme';

interface ThemeState {
	current: Theme;
}

function createThemeStore() {
	const state = $state<ThemeState>({ current: DEFAULT_THEME });

	/**
	 * Load theme from localStorage
	 * Should be called on app initialization
	 */
	function loadFromStorage(): Theme {
		if (typeof window === 'undefined') {
			return DEFAULT_THEME;
		}

		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored && THEMES.includes(stored as Theme)) {
				return stored as Theme;
			}
		} catch {
			// localStorage not available or error reading
		}

		return DEFAULT_THEME;
	}

	/**
	 * Save theme to localStorage
	 */
	function saveToStorage(theme: Theme): void {
		if (typeof window === 'undefined') {
			return;
		}

		try {
			localStorage.setItem(STORAGE_KEY, theme);
		} catch {
			// localStorage not available or quota exceeded
		}
	}

	/**
	 * Apply theme to DOM by setting data-theme attribute on html element
	 */
	function applyToDOM(theme: Theme): void {
		if (typeof document === 'undefined') {
			return;
		}

		document.documentElement.setAttribute('data-theme', theme);
	}

	return {
		/** Get current theme (reactive) */
		get current() {
			return state.current;
		},

		/**
		 * Initialize theme from localStorage and apply to DOM
		 * Should be called once when app loads
		 */
		init() {
			const theme = loadFromStorage();
			state.current = theme;
			applyToDOM(theme);
		},

		/**
		 * Set theme, persist to localStorage, and apply to DOM
		 * @param theme Theme to set
		 */
		set(theme: Theme) {
			if (!THEMES.includes(theme)) {
				console.warn(`Invalid theme: ${theme}`);
				return;
			}

			state.current = theme;
			saveToStorage(theme);
			applyToDOM(theme);
		},

		/**
		 * Toggle to next theme in the list
		 */
		toggle() {
			const currentIndex = THEMES.indexOf(state.current);
			const nextIndex = (currentIndex + 1) % THEMES.length;
			this.set(THEMES[nextIndex]);
		},

		/**
		 * Check if a theme is currently active
		 * @param theme Theme to check
		 */
		isActive(theme: Theme) {
			return state.current === theme;
		}
	};
}

/**
 * Global theme store instance
 *
 * Usage:
 * ```ts
 * import { themeStore, THEME_NAMES } from '$lib/stores/theme.svelte';
 *
 * // Initialize on app load (e.g., in +layout.svelte)
 * themeStore.init();
 *
 * // Get current theme (reactive)
 * const current = themeStore.current;
 *
 * // Set a specific theme
 * themeStore.set('arcane');
 *
 * // Toggle to next theme
 * themeStore.toggle();
 * ```
 */
export const themeStore = createThemeStore();

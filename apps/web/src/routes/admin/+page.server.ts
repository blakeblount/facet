import type { PageServerLoad } from './$types';
import type { StoreSettings, StorageLocation } from '$lib/types/api';

/**
 * Server-side load function for the admin page.
 * Fetches store settings and storage locations (public endpoints).
 * Employees require admin authentication and are fetched client-side.
 */
export const load: PageServerLoad = async ({ fetch }) => {
	try {
		// Fetch public admin data in parallel
		// Note: Employees require X-Admin-PIN header, so they're fetched client-side
		const [settingsRes, locationsRes] = await Promise.all([
			fetch('/api/v1/settings').catch(() => null),
			fetch('/api/v1/locations').catch(() => null)
		]);

		// Parse responses, handling failures gracefully
		let settings: StoreSettings | null = null;
		let locations: StorageLocation[] = [];

		if (settingsRes?.ok) {
			const json = await settingsRes.json();
			if (json.data) {
				settings = json.data;
			}
		}

		if (locationsRes?.ok) {
			const json = await locationsRes.json();
			if (json.data) {
				// API returns { locations: [...], count: N }
				locations = json.data.locations || [];
			}
		}

		return {
			settings,
			locations,
			error: null
		};
	} catch {
		// During development without the API running, return empty data
		return {
			settings: null,
			locations: [],
			error: null
		};
	}
};

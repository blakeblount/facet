import type { PageServerLoad } from './$types';
import type { StoreSettings, EmployeeSummary, StorageLocation } from '$lib/types/api';

/**
 * Server-side load function for the admin page.
 * Fetches store settings, employees, and storage locations.
 */
export const load: PageServerLoad = async ({ fetch }) => {
	try {
		// Fetch all admin data in parallel
		const [settingsRes, employeesRes, locationsRes] = await Promise.all([
			fetch('/api/v1/settings').catch(() => null),
			fetch('/api/v1/employees').catch(() => null),
			fetch('/api/v1/locations').catch(() => null)
		]);

		// Parse responses, handling failures gracefully
		let settings: StoreSettings | null = null;
		let employees: EmployeeSummary[] = [];
		let locations: StorageLocation[] = [];

		if (settingsRes?.ok) {
			const json = await settingsRes.json();
			if (json.data) {
				settings = json.data;
			}
		}

		if (employeesRes?.ok) {
			const json = await employeesRes.json();
			if (json.data) {
				employees = json.data;
			}
		}

		if (locationsRes?.ok) {
			const json = await locationsRes.json();
			if (json.data) {
				locations = json.data;
			}
		}

		return {
			settings,
			employees,
			locations,
			error: null
		};
	} catch {
		// During development without the API running, return empty data
		return {
			settings: null,
			employees: [],
			locations: [],
			error: null
		};
	}
};

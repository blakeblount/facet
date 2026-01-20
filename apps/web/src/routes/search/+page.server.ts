import type { PageServerLoad } from './$types';
import type { ListTicketsResponse } from '$lib/types/api';

/**
 * Server-side load function for the search page.
 * Searches tickets using the API when a query is provided.
 * Supports filtering by status and date range.
 */
export const load: PageServerLoad = async ({ url, fetch }) => {
	const query = url.searchParams.get('q')?.trim() || '';
	const status = url.searchParams.get('status')?.trim() || '';
	const fromDate = url.searchParams.get('from_date')?.trim() || '';
	const toDate = url.searchParams.get('to_date')?.trim() || '';

	// If no query and no filters, return empty state
	if (!query && !status && !fromDate && !toDate) {
		return {
			query: '',
			status: '',
			fromDate: '',
			toDate: '',
			results: null,
			error: null
		};
	}

	try {
		const searchParams = new URLSearchParams({
			include_archived: 'true',
			limit: '50'
		});

		// Add search query if present
		if (query) {
			searchParams.set('search', query);
		}

		// Add status filter if present
		if (status) {
			searchParams.set('status', status);
		}

		// Add date range filters if present
		if (fromDate) {
			// Convert date to start of day ISO timestamp
			searchParams.set('from_date', `${fromDate}T00:00:00Z`);
		}
		if (toDate) {
			// Convert date to end of day ISO timestamp
			searchParams.set('to_date', `${toDate}T23:59:59Z`);
		}

		const response = await fetch(`/api/v1/tickets?${searchParams.toString()}`);

		if (!response.ok) {
			return {
				query,
				status,
				fromDate,
				toDate,
				results: null,
				error: `Search failed: ${response.status}`
			};
		}

		const json = await response.json();

		if (json.error) {
			return {
				query,
				status,
				fromDate,
				toDate,
				results: null,
				error: json.error.message
			};
		}

		return {
			query,
			status,
			fromDate,
			toDate,
			results: json.data as ListTicketsResponse,
			error: null
		};
	} catch {
		// During development without the API running, return empty results
		const emptyResults: ListTicketsResponse = {
			tickets: [],
			pagination: {
				count: 0,
				limit: 50,
				offset: 0,
				has_more: false
			}
		};

		return {
			query,
			status,
			fromDate,
			toDate,
			results: emptyResults,
			error: null
		};
	}
};

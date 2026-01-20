import type { PageServerLoad } from './$types';
import type { ListTicketsResponse } from '$lib/types/api';

/**
 * Server-side load function for the search page.
 * Searches tickets using the API when a query is provided.
 */
export const load: PageServerLoad = async ({ url, fetch }) => {
	const query = url.searchParams.get('q')?.trim() || '';

	// If no query, return empty state
	if (!query) {
		return {
			query: '',
			results: null,
			error: null
		};
	}

	try {
		const searchParams = new URLSearchParams({
			search: query,
			include_archived: 'false',
			limit: '50'
		});

		const response = await fetch(`/api/v1/tickets?${searchParams.toString()}`);

		if (!response.ok) {
			return {
				query,
				results: null,
				error: `Search failed: ${response.status}`
			};
		}

		const json = await response.json();

		if (json.error) {
			return {
				query,
				results: null,
				error: json.error.message
			};
		}

		return {
			query,
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
			results: emptyResults,
			error: null
		};
	}
};

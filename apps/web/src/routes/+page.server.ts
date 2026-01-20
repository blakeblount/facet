import type { PageServerLoad } from './$types';
import type { GetQueueResponse } from '$lib/types/api';

/**
 * Server-side load function for the workboard page.
 * Fetches the queue data from the API.
 */
export const load: PageServerLoad = async ({ fetch }) => {
	try {
		// In production, this would call the actual API
		// For now, we return a placeholder structure that matches the API types
		const response = await fetch('/api/v1/queue');

		if (!response.ok) {
			return {
				queue: null,
				error: `Failed to load queue: ${response.status}`
			};
		}

		const json = await response.json();

		if (json.error) {
			return {
				queue: null,
				error: json.error.message
			};
		}

		return {
			queue: json.data as GetQueueResponse,
			error: null
		};
	} catch {
		// During development without the API running, return empty queue
		const emptyQueue: GetQueueResponse = {
			lanes: {
				intake: { count: 0, tickets: [] },
				in_progress: { count: 0, tickets: [] },
				waiting_on_parts: { count: 0, tickets: [] },
				ready_for_pickup: { count: 0, tickets: [] }
			}
		};

		return {
			queue: emptyQueue,
			error: null
		};
	}
};

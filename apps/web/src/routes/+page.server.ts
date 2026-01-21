import type { PageServerLoad } from './$types';
import type { GetQueueResponse } from '$lib/types/api';

/**
 * Server-side load function for the workboard page.
 *
 * The queue endpoint requires employee authentication, which is stored client-side.
 * Server-side rendering cannot access the auth token, so we return null here
 * and let the client fetch the queue data after authentication.
 *
 * This enables:
 * - Fast initial page load (no blocking API call)
 * - Proper auth flow (client fetches after employee login)
 * - Better UX (loading state shown while fetching)
 */
export const load: PageServerLoad = async () => {
	// Queue data requires authentication - fetched client-side
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
};

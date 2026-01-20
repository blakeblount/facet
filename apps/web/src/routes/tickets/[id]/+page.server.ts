import type { PageServerLoad } from './$types';

/**
 * Server-side load function for the ticket detail page.
 * Extracts the ticket ID from the URL and will fetch ticket data.
 */
export const load: PageServerLoad = async ({ params }) => {
	return {
		id: params.id
	};
};

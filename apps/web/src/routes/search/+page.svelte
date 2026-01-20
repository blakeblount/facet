<script lang="ts">
	import type { PageData } from './$types';
	import { resolve } from '$app/paths';

	let { data }: { data: PageData } = $props();
	let searchQuery = $state(data.query || '');
</script>

<div class="search-page">
	<div class="search-header">
		<h1 class="page-title">Search Tickets</h1>
		<p class="page-subtitle">
			Search by ticket code, customer name, phone number, or item description.
		</p>
	</div>

	<form class="search-form" method="GET" action="/search">
		<div class="search-input-wrapper">
			<input
				type="search"
				name="q"
				class="search-input"
				placeholder="Search tickets..."
				bind:value={searchQuery}
				autocomplete="off"
			/>
			<button type="submit" class="search-button">Search</button>
		</div>
	</form>

	{#if data.error}
		<div class="error-message">
			<p>Search failed: {data.error}</p>
		</div>
	{:else if data.query && data.results}
		<div class="search-results">
			<p class="results-count">
				Found {data.results.pagination.count} ticket{data.results.pagination.count === 1 ? '' : 's'}
			</p>

			{#if data.results.tickets.length > 0}
				<div class="results-list">
					{#each data.results.tickets as ticket (ticket.ticket_id)}
						<a
							href={resolve('/tickets/[id]', { id: ticket.ticket_id })}
							class="result-card"
							class:is-rush={ticket.is_rush}
						>
							<div class="result-header">
								<span class="ticket-code">{ticket.friendly_code}</span>
								<span class="ticket-status status-{ticket.status.replace('_', '-')}">
									{ticket.status.replace(/_/g, ' ')}
								</span>
							</div>
							<div class="result-body">
								<div class="ticket-customer">{ticket.customer_name}</div>
								<div class="ticket-description">{ticket.item_description}</div>
							</div>
							{#if ticket.is_rush}
								<span class="rush-badge">RUSH</span>
							{/if}
						</a>
					{/each}
				</div>
			{:else}
				<p class="no-results">No tickets found matching "{data.query}"</p>
			{/if}
		</div>
	{:else if !data.query}
		<p class="search-hint">Enter a search term to find tickets.</p>
	{/if}
</div>

<style>
	.search-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		max-width: 900px;
	}

	.search-header {
		margin-bottom: var(--space-sm);
	}

	.page-title {
		font-size: 1.75rem;
		margin-bottom: var(--space-xs);
	}

	.page-subtitle {
		color: var(--color-text-muted);
	}

	.search-form {
		width: 100%;
	}

	.search-input-wrapper {
		display: flex;
		gap: var(--space-sm);
	}

	.search-input {
		flex: 1;
		padding: var(--space-md);
		font-size: 1rem;
		border: 2px solid var(--color-border);
		border-radius: var(--radius-md);
		transition: border-color var(--transition-fast);
	}

	.search-input:focus {
		outline: none;
		border-color: var(--color-primary);
	}

	.search-button {
		padding: var(--space-md) var(--space-xl);
		background-color: var(--color-primary);
		color: white;
		font-size: 1rem;
		font-weight: 500;
		border: none;
		border-radius: var(--radius-md);
		cursor: pointer;
		transition: background-color var(--transition-fast);
	}

	.search-button:hover {
		background-color: var(--color-primary-dark);
	}

	.error-message {
		padding: var(--space-lg);
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: var(--radius-md);
		color: #991b1b;
	}

	.search-results {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
	}

	.results-count {
		color: var(--color-text-muted);
		font-size: 0.875rem;
	}

	.results-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.result-card {
		position: relative;
		display: block;
		padding: var(--space-md);
		background-color: var(--color-bg-card);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		color: var(--color-text);
		text-decoration: none;
		transition:
			box-shadow var(--transition-fast),
			transform var(--transition-fast);
	}

	.result-card:hover {
		box-shadow: var(--shadow-md);
		transform: translateY(-1px);
		text-decoration: none;
	}

	.result-card.is-rush {
		border-left: 4px solid var(--color-rush);
	}

	.result-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--space-sm);
	}

	.ticket-code {
		font-family: var(--font-mono);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-primary);
	}

	.ticket-status {
		padding: var(--space-xs) var(--space-sm);
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: capitalize;
		border-radius: var(--radius-sm);
	}

	.status-intake {
		background-color: var(--color-intake);
		color: white;
	}

	.status-in-progress {
		background-color: var(--color-in-progress);
		color: white;
	}

	.status-waiting-on-parts {
		background-color: var(--color-waiting);
		color: white;
	}

	.status-ready-for-pickup {
		background-color: var(--color-ready);
		color: white;
	}

	.status-closed,
	.status-archived {
		background-color: var(--color-closed);
		color: white;
	}

	.result-body {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
	}

	.ticket-customer {
		font-weight: 500;
	}

	.ticket-description {
		font-size: 0.875rem;
		color: var(--color-text-muted);
	}

	.rush-badge {
		position: absolute;
		top: var(--space-sm);
		right: var(--space-sm);
		padding: var(--space-xs) var(--space-sm);
		background-color: var(--color-rush);
		color: white;
		font-size: 0.625rem;
		font-weight: 700;
		border-radius: var(--radius-sm);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.no-results,
	.search-hint {
		padding: var(--space-xl);
		text-align: center;
		color: var(--color-text-muted);
	}
</style>

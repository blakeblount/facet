<script lang="ts">
	import type { PageData } from './$types';
	import TicketDetailModal from '$lib/components/TicketDetailModal.svelte';
	import Select from '$lib/components/Select.svelte';
	import DatePicker from '$lib/components/DatePicker.svelte';

	let { data }: { data: PageData } = $props();
	let searchQuery = $state(data.query || '');
	let statusFilter = $state(data.status || '');
	let fromDate = $state(data.fromDate || '');
	let toDate = $state(data.toDate || '');

	// Status options for filter dropdown
	const statusOptions: Array<{ value: string; label: string }> = [
		{ value: '', label: 'All Statuses' },
		{ value: 'intake', label: 'Intake' },
		{ value: 'in_progress', label: 'In Progress' },
		{ value: 'waiting_on_parts', label: 'Waiting on Parts' },
		{ value: 'ready_for_pickup', label: 'Ready for Pickup' },
		{ value: 'closed', label: 'Closed' },
		{ value: 'archived', label: 'Archived' }
	];

	// Check if any filters are active
	const hasActiveFilters = $derived(!!statusFilter || !!fromDate || !!toDate);

	// Modal state
	let selectedTicketId: string | null = $state(null);
	let isModalOpen = $state(false);

	function openTicketModal(ticketId: string) {
		selectedTicketId = ticketId;
		isModalOpen = true;
	}

	function closeModal() {
		isModalOpen = false;
		selectedTicketId = null;
	}

	function clearFilters() {
		statusFilter = '';
		fromDate = '';
		toDate = '';
	}
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

		<div class="filters-section">
			<div class="filters-row">
				<div class="filter-item">
					<Select
						label="Status"
						options={statusOptions}
						bind:value={statusFilter}
						name="status"
						placeholder="All Statuses"
					/>
				</div>
				<div class="filter-item">
					<DatePicker
						label="From Date"
						bind:value={fromDate}
						name="from_date"
						placeholder="Start date"
						maxDate={toDate || undefined}
					/>
				</div>
				<div class="filter-item">
					<DatePicker
						label="To Date"
						bind:value={toDate}
						name="to_date"
						placeholder="End date"
						minDate={fromDate || undefined}
					/>
				</div>
			</div>
			{#if hasActiveFilters}
				<button type="button" class="clear-filters-btn" onclick={clearFilters}>
					Clear Filters
				</button>
			{/if}
		</div>
	</form>

	{#if data.error}
		<div class="error-message">
			<p>Search failed: {data.error}</p>
		</div>
	{:else if data.results}
		<div class="search-results">
			<p class="results-count">
				Found {data.results.pagination.count} ticket{data.results.pagination.count === 1 ? '' : 's'}
				{#if data.query}
					matching "{data.query}"
				{/if}
			</p>

			{#if data.results.tickets.length > 0}
				<div class="results-list">
					{#each data.results.tickets as ticket (ticket.ticket_id)}
						<button
							type="button"
							class="result-card"
							class:is-rush={ticket.is_rush}
							class:is-closed={ticket.status === 'closed' || ticket.status === 'archived'}
							onclick={() => openTicketModal(ticket.ticket_id)}
						>
							<div class="result-header">
								<span class="ticket-code">{ticket.friendly_code}</span>
								<span class="ticket-status status-{ticket.status.replace(/_/g, '-')}">
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
						</button>
					{/each}
				</div>
			{:else}
				<p class="no-results">
					No tickets found{data.query ? ` matching "${data.query}"` : ''} with the current filters.
				</p>
			{/if}
		</div>
	{:else}
		<p class="search-hint">Enter a search term or apply filters to find tickets.</p>
	{/if}
</div>

<TicketDetailModal ticketId={selectedTicketId} open={isModalOpen} onClose={closeModal} />

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
		width: 100%;
		padding: var(--space-md);
		background-color: var(--color-bg-card);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		color: var(--color-text);
		text-align: left;
		font: inherit;
		cursor: pointer;
		transition:
			box-shadow var(--transition-fast),
			transform var(--transition-fast);
	}

	.result-card:hover {
		box-shadow: var(--shadow-md);
		transform: translateY(-1px);
	}

	.result-card:focus-visible {
		outline: 2px solid var(--color-primary);
		outline-offset: 2px;
	}

	.result-card.is-rush {
		border-left: 4px solid var(--color-rush);
	}

	.result-card.is-closed {
		opacity: 0.7;
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

	.filters-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		margin-top: var(--space-md);
		padding-top: var(--space-md);
		border-top: 1px solid var(--color-border);
	}

	.filters-row {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-md);
	}

	.filter-item {
		min-width: 0;
	}

	.clear-filters-btn {
		align-self: flex-start;
		padding: var(--space-xs) var(--space-md);
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		font-size: 0.875rem;
		color: var(--color-text-muted);
		cursor: pointer;
		transition:
			color var(--transition-fast),
			border-color var(--transition-fast);
	}

	.clear-filters-btn:hover {
		color: var(--color-text);
		border-color: var(--color-text-muted);
	}

	@media (max-width: 640px) {
		.filters-row {
			grid-template-columns: 1fr;
		}
	}
</style>

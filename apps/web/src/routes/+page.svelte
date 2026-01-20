<script lang="ts">
	import type { PageData } from './$types';
	import { resolve } from '$app/paths';
	import Button from '$lib/components/Button.svelte';

	let { data }: { data: PageData } = $props();

	function handleNewTicket() {
		// TODO: Open intake form modal when implemented
		console.log('New ticket clicked');
	}
</script>

<div class="workboard">
	<div class="workboard-header">
		<h1 class="page-title">Workboard</h1>
		<p class="page-subtitle">
			Manage repair tickets across status lanes. Rush tickets appear first in each lane.
		</p>
	</div>

	{#if data.error}
		<div class="error-message">
			<p>Failed to load queue: {data.error}</p>
		</div>
	{:else if data.queue}
		<div class="lanes-container">
			<div class="lane">
				<div class="lane-header lane-header-intake">
					<div class="lane-header-left">
						<h2 class="lane-title">Intake</h2>
						<span class="lane-count">{data.queue.lanes.intake.count}</span>
					</div>
					<Button variant="secondary" size="sm" onclick={handleNewTicket} class="new-ticket-btn">
						+ New
					</Button>
				</div>
				<div class="lane-content">
					{#each data.queue.lanes.intake.tickets as ticket (ticket.ticket_id)}
						<a
							href={resolve('/tickets/[id]', { id: ticket.ticket_id })}
							class="ticket-card"
							class:is-rush={ticket.is_rush}
						>
							<div class="ticket-code">{ticket.friendly_code}</div>
							<div class="ticket-customer">{ticket.customer_name}</div>
							<div class="ticket-description">{ticket.item_description}</div>
							{#if ticket.is_rush}
								<span class="rush-badge">RUSH</span>
							{/if}
						</a>
					{:else}
						<p class="lane-empty">No tickets</p>
					{/each}
				</div>
			</div>

			<div class="lane">
				<div class="lane-header lane-header-in-progress">
					<h2 class="lane-title">In Progress</h2>
					<span class="lane-count">{data.queue.lanes.in_progress.count}</span>
				</div>
				<div class="lane-content">
					{#each data.queue.lanes.in_progress.tickets as ticket (ticket.ticket_id)}
						<a
							href={resolve('/tickets/[id]', { id: ticket.ticket_id })}
							class="ticket-card"
							class:is-rush={ticket.is_rush}
						>
							<div class="ticket-code">{ticket.friendly_code}</div>
							<div class="ticket-customer">{ticket.customer_name}</div>
							<div class="ticket-description">{ticket.item_description}</div>
							{#if ticket.is_rush}
								<span class="rush-badge">RUSH</span>
							{/if}
						</a>
					{:else}
						<p class="lane-empty">No tickets</p>
					{/each}
				</div>
			</div>

			<div class="lane">
				<div class="lane-header lane-header-waiting">
					<h2 class="lane-title">Waiting on Parts</h2>
					<span class="lane-count">{data.queue.lanes.waiting_on_parts.count}</span>
				</div>
				<div class="lane-content">
					{#each data.queue.lanes.waiting_on_parts.tickets as ticket (ticket.ticket_id)}
						<a
							href={resolve('/tickets/[id]', { id: ticket.ticket_id })}
							class="ticket-card"
							class:is-rush={ticket.is_rush}
						>
							<div class="ticket-code">{ticket.friendly_code}</div>
							<div class="ticket-customer">{ticket.customer_name}</div>
							<div class="ticket-description">{ticket.item_description}</div>
							{#if ticket.is_rush}
								<span class="rush-badge">RUSH</span>
							{/if}
						</a>
					{:else}
						<p class="lane-empty">No tickets</p>
					{/each}
				</div>
			</div>

			<div class="lane">
				<div class="lane-header lane-header-ready">
					<h2 class="lane-title">Ready for Pickup</h2>
					<span class="lane-count">{data.queue.lanes.ready_for_pickup.count}</span>
				</div>
				<div class="lane-content">
					{#each data.queue.lanes.ready_for_pickup.tickets as ticket (ticket.ticket_id)}
						<a
							href={resolve('/tickets/[id]', { id: ticket.ticket_id })}
							class="ticket-card"
							class:is-rush={ticket.is_rush}
						>
							<div class="ticket-code">{ticket.friendly_code}</div>
							<div class="ticket-customer">{ticket.customer_name}</div>
							<div class="ticket-description">{ticket.item_description}</div>
							{#if ticket.is_rush}
								<span class="rush-badge">RUSH</span>
							{/if}
						</a>
					{:else}
						<p class="lane-empty">No tickets</p>
					{/each}
				</div>
			</div>
		</div>
	{:else}
		<div class="loading">Loading workboard...</div>
	{/if}
</div>

<style>
	.workboard {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		height: calc(100vh - var(--header-height) - var(--space-lg) * 2);
	}

	.workboard-header {
		flex-shrink: 0;
	}

	.page-title {
		font-size: 1.75rem;
		margin-bottom: var(--space-xs);
	}

	.page-subtitle {
		color: var(--color-text-muted);
	}

	.error-message {
		padding: var(--space-lg);
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: var(--radius-md);
		color: #991b1b;
	}

	.loading {
		padding: var(--space-xl);
		text-align: center;
		color: var(--color-text-muted);
	}

	.lanes-container {
		display: flex;
		gap: var(--space-md);
		flex: 1;
		min-height: 0;
		overflow-x: auto;
		padding-bottom: var(--space-sm);
	}

	/* Custom scrollbar for horizontal scroll */
	.lanes-container::-webkit-scrollbar {
		height: 8px;
	}

	.lanes-container::-webkit-scrollbar-track {
		background: var(--color-border);
		border-radius: 4px;
	}

	.lanes-container::-webkit-scrollbar-thumb {
		background-color: var(--color-text-muted);
		border-radius: 4px;
	}

	.lane {
		display: flex;
		flex-direction: column;
		flex: 1 0 280px;
		min-width: 280px;
		max-width: 360px;
		background-color: var(--color-bg-card);
		border-radius: var(--radius-lg);
		box-shadow: var(--shadow-sm);
		overflow: hidden;
	}

	.lane-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-sm) var(--space-md);
		color: white;
		gap: var(--space-sm);
		min-height: 3rem;
	}

	.lane-header-left {
		display: flex;
		align-items: center;
		gap: var(--space-sm);
	}

	/* New ticket button styling for contrast on colored header */
	.lane-header :global(.new-ticket-btn) {
		background-color: rgba(255, 255, 255, 0.95);
		color: var(--color-intake);
		border-color: transparent;
		font-weight: 600;
	}

	.lane-header :global(.new-ticket-btn:hover) {
		background-color: white;
	}

	.lane-header-intake {
		background-color: var(--color-intake);
	}

	.lane-header-in-progress {
		background-color: var(--color-in-progress);
	}

	.lane-header-waiting {
		background-color: var(--color-waiting);
	}

	.lane-header-ready {
		background-color: var(--color-ready);
	}

	.lane-title {
		font-size: 1rem;
		font-weight: 600;
	}

	.lane-count {
		background-color: rgba(255, 255, 255, 0.2);
		padding: var(--space-xs) var(--space-sm);
		border-radius: var(--radius-sm);
		font-size: 0.875rem;
		font-weight: 600;
	}

	.lane-content {
		flex: 1;
		padding: var(--space-sm);
		overflow-y: auto;
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.lane-empty {
		padding: var(--space-md);
		text-align: center;
		color: var(--color-text-muted);
		font-size: 0.875rem;
	}

	.ticket-card {
		position: relative;
		display: block;
		padding: var(--space-md);
		background-color: var(--color-bg);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		color: var(--color-text);
		text-decoration: none;
		transition:
			box-shadow var(--transition-fast),
			transform var(--transition-fast);
	}

	.ticket-card:hover {
		box-shadow: var(--shadow-md);
		transform: translateY(-1px);
		text-decoration: none;
	}

	.ticket-card.is-rush {
		border-left: 4px solid var(--color-rush);
	}

	.ticket-code {
		font-family: var(--font-mono);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-primary);
		margin-bottom: var(--space-xs);
	}

	.ticket-customer {
		font-weight: 500;
		margin-bottom: var(--space-xs);
	}

	.ticket-description {
		font-size: 0.875rem;
		color: var(--color-text-muted);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
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
</style>

<script lang="ts">
	import Card from './Card.svelte';
	import Badge from './Badge.svelte';
	import type { QueueTicket } from '$lib/types/api';

	interface Props {
		/** The ticket data to display */
		ticket: QueueTicket;
		/** Optional thumbnail URL for the first photo */
		thumbnailUrl?: string | null;
		/** Click handler when card is clicked */
		onclick?: () => void;
		/** Whether the card is currently being dragged */
		isDragging?: boolean;
		/** Additional CSS class for the card */
		class?: string;
	}

	let {
		ticket,
		thumbnailUrl = null,
		onclick,
		isDragging = false,
		class: className = ''
	}: Props = $props();

	/** Format promise date for display */
	function formatPromiseDate(dateString: string | null): string {
		if (!dateString) return '';
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric'
		});
	}

	/** Truncate item description for display */
	function truncateDescription(text: string, maxLength: number = 60): string {
		if (text.length <= maxLength) return text;
		return text.slice(0, maxLength).trim() + '...';
	}
</script>

<div
	class="ticket-card-wrapper {className}"
	class:is-overdue={ticket.is_overdue}
	class:is-dragging={isDragging}
	draggable="true"
	data-ticket-id={ticket.ticket_id}
>
	<Card clickable padding="sm" shadow="sm" {onclick}>
		<div class="ticket-card-content">
			<!-- Top row: Ticket code and badges -->
			<div class="ticket-header">
				<span class="ticket-code">{ticket.friendly_code}</span>
				<div class="ticket-badges">
					{#if ticket.is_rush}
						<Badge variant="rush" text="Rush" size="sm" />
					{/if}
					{#if ticket.is_overdue}
						<Badge variant="overdue" text="Overdue" size="sm" />
					{/if}
				</div>
			</div>

			<!-- Main content row: thumbnail + info -->
			<div class="ticket-body">
				{#if thumbnailUrl}
					<div class="ticket-thumbnail">
						<img src={thumbnailUrl} alt="Item thumbnail" />
					</div>
				{/if}
				<div class="ticket-info">
					<span class="customer-name">{ticket.customer_name}</span>
					<span class="item-description">{truncateDescription(ticket.item_description)}</span>
				</div>
			</div>

			<!-- Footer: Promise date -->
			{#if ticket.promise_date}
				<div class="ticket-footer">
					<span class="promise-date" class:overdue={ticket.is_overdue}>
						<svg class="calendar-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
							<path
								d="M4.75 0a.75.75 0 0 1 .75.75V2h5V.75a.75.75 0 0 1 1.5 0V2H14a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1h1.75V.75a.75.75 0 0 1 .75-.75ZM2.5 6v8.5h11V6h-11Zm10.5-2.5h-10V5h10V3.5Z"
							/>
						</svg>
						{formatPromiseDate(ticket.promise_date)}
					</span>
				</div>
			{/if}
		</div>
	</Card>
</div>

<style>
	.ticket-card-wrapper {
		position: relative;
		border-radius: var(--radius-lg, 0.75rem);
		transition:
			transform var(--transition-fast, 150ms ease),
			opacity var(--transition-fast, 150ms ease);
	}

	/* Overdue indicator - red left border */
	.ticket-card-wrapper.is-overdue {
		position: relative;
	}

	.ticket-card-wrapper.is-overdue::before {
		content: '';
		position: absolute;
		left: 0;
		top: 0;
		bottom: 0;
		width: 4px;
		background-color: var(--color-rush, #ef4444);
		border-radius: var(--radius-lg, 0.75rem) 0 0 var(--radius-lg, 0.75rem);
		z-index: 1;
	}

	/* Dragging state */
	.ticket-card-wrapper.is-dragging {
		opacity: 0.5;
		transform: scale(0.98);
	}

	/* Card content layout */
	.ticket-card-content {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm, 0.5rem);
	}

	/* Header with code and badges */
	.ticket-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-sm, 0.5rem);
	}

	.ticket-code {
		font-family: var(--font-mono, monospace);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text, #1e293b);
	}

	.ticket-badges {
		display: flex;
		gap: var(--space-xs, 0.25rem);
		flex-shrink: 0;
	}

	/* Body with thumbnail and info */
	.ticket-body {
		display: flex;
		gap: var(--space-sm, 0.5rem);
		align-items: flex-start;
	}

	.ticket-thumbnail {
		flex-shrink: 0;
		width: 48px;
		height: 48px;
		border-radius: var(--radius-md, 0.375rem);
		overflow: hidden;
		background-color: var(--color-bg, #f8fafc);
	}

	.ticket-thumbnail img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.ticket-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0; /* Allow text truncation */
		flex: 1;
	}

	.customer-name {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.item-description {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
		line-height: 1.4;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	/* Footer with promise date */
	.ticket-footer {
		display: flex;
		align-items: center;
		padding-top: var(--space-xs, 0.25rem);
		border-top: 1px solid var(--color-border, #e2e8f0);
	}

	.promise-date {
		display: inline-flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	.promise-date.overdue {
		color: var(--color-rush, #ef4444);
		font-weight: 500;
	}

	.calendar-icon {
		width: 12px;
		height: 12px;
	}

	/* Draggable cursor */
	.ticket-card-wrapper[draggable='true'] {
		cursor: grab;
	}

	.ticket-card-wrapper[draggable='true']:active {
		cursor: grabbing;
	}
</style>

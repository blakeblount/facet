<script lang="ts">
	import Modal from './Modal.svelte';
	import { getTicket, type TicketDetailResponse, type TicketStatus } from '$lib/services/api';

	interface Props {
		/** The ticket ID to display */
		ticketId: string | null;
		/** Whether the modal is open */
		open: boolean;
		/** Callback when modal closes */
		onClose: () => void;
	}

	let { ticketId, open, onClose }: Props = $props();

	let ticket: TicketDetailResponse | null = $state(null);
	let loading: boolean = $state(false);
	let error: string | null = $state(null);

	// Fetch ticket when ticketId changes and modal is open
	$effect(() => {
		if (open && ticketId) {
			fetchTicket(ticketId);
		} else if (!open) {
			// Reset state when modal closes
			ticket = null;
			error = null;
		}
	});

	async function fetchTicket(id: string) {
		loading = true;
		error = null;
		try {
			ticket = await getTicket(id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load ticket';
			ticket = null;
		} finally {
			loading = false;
		}
	}

	function formatDate(dateString: string | null): string {
		if (!dateString) return '—';
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatDateTime(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}

	function formatCurrency(amount: string | null): string {
		if (!amount) return '—';
		const num = parseFloat(amount);
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD'
		}).format(num);
	}

	function getStatusLabel(status: TicketStatus): string {
		const labels: Record<TicketStatus, string> = {
			intake: 'Intake',
			in_progress: 'In Progress',
			waiting_on_parts: 'Waiting on Parts',
			ready_for_pickup: 'Ready for Pickup',
			closed: 'Closed',
			archived: 'Archived'
		};
		return labels[status];
	}

	function getStatusClass(status: TicketStatus): string {
		const classes: Record<TicketStatus, string> = {
			intake: 'status-intake',
			in_progress: 'status-in-progress',
			waiting_on_parts: 'status-waiting',
			ready_for_pickup: 'status-ready',
			closed: 'status-closed',
			archived: 'status-closed'
		};
		return classes[status];
	}
</script>

<Modal {open} title={ticket?.friendly_code ?? 'Ticket Details'} {onClose}>
	<div class="ticket-detail-content">
		{#if loading}
			<div class="loading-state">
				<div class="spinner"></div>
				<p>Loading ticket details...</p>
			</div>
		{:else if error}
			<div class="error-state">
				<p class="error-message">{error}</p>
				<button class="retry-button" onclick={() => ticketId && fetchTicket(ticketId)}>
					Retry
				</button>
			</div>
		{:else if ticket}
			<!-- Header with status and rush indicator -->
			<div class="ticket-header">
				<div class="ticket-code-row">
					<span class="ticket-code">{ticket.friendly_code}</span>
					{#if ticket.is_rush}
						<span class="rush-badge">RUSH</span>
					{/if}
				</div>
				<span class="status-badge {getStatusClass(ticket.status)}">
					{getStatusLabel(ticket.status)}
				</span>
			</div>

			<!-- Customer Section -->
			<section class="detail-section">
				<h3 class="section-title">Customer</h3>
				<div class="section-content">
					<div class="info-row">
						<span class="info-label">Name</span>
						<span class="info-value">{ticket.customer.name}</span>
					</div>
					{#if ticket.customer.phone}
						<div class="info-row">
							<span class="info-label">Phone</span>
							<span class="info-value">{ticket.customer.phone}</span>
						</div>
					{/if}
					{#if ticket.customer.email}
						<div class="info-row">
							<span class="info-label">Email</span>
							<span class="info-value">{ticket.customer.email}</span>
						</div>
					{/if}
				</div>
			</section>

			<!-- Item Section -->
			<section class="detail-section">
				<h3 class="section-title">Item Details</h3>
				<div class="section-content">
					{#if ticket.item_type}
						<div class="info-row">
							<span class="info-label">Type</span>
							<span class="info-value">{ticket.item_type}</span>
						</div>
					{/if}
					<div class="info-row">
						<span class="info-label">Description</span>
						<span class="info-value">{ticket.item_description}</span>
					</div>
					<div class="info-row">
						<span class="info-label">Condition</span>
						<span class="info-value text-block">{ticket.condition_notes}</span>
					</div>
					<div class="info-row">
						<span class="info-label">Requested Work</span>
						<span class="info-value text-block">{ticket.requested_work}</span>
					</div>
				</div>
			</section>

			<!-- Photos Section -->
			<section class="detail-section">
				<h3 class="section-title">Photos ({ticket.photos.length})</h3>
				<div class="section-content">
					{#if ticket.photos.length > 0}
						<div class="photos-grid">
							{#each ticket.photos as photo (photo.photo_id)}
								<div class="photo-item">
									<img src={photo.url} alt="Ticket item" class="photo-thumbnail" />
									<span class="photo-meta">
										{formatDateTime(photo.uploaded_at)} by {photo.uploaded_by.name}
									</span>
								</div>
							{/each}
						</div>
					{:else}
						<p class="empty-message">No photos attached</p>
					{/if}
				</div>
			</section>

			<!-- Pricing Section -->
			<section class="detail-section">
				<h3 class="section-title">Pricing</h3>
				<div class="section-content">
					<div class="info-row">
						<span class="info-label">Quote</span>
						<span class="info-value">{formatCurrency(ticket.quote_amount)}</span>
					</div>
					<div class="info-row">
						<span class="info-label">Actual Charged</span>
						<span class="info-value">{formatCurrency(ticket.actual_amount)}</span>
					</div>
				</div>
			</section>

			<!-- Status & Location Section -->
			<section class="detail-section">
				<h3 class="section-title">Status & Location</h3>
				<div class="section-content">
					<div class="info-row">
						<span class="info-label">Promise Date</span>
						<span class="info-value">{formatDate(ticket.promise_date)}</span>
					</div>
					<div class="info-row">
						<span class="info-label">Storage Location</span>
						<span class="info-value">{ticket.storage_location.name}</span>
					</div>
				</div>
			</section>

			<!-- Notes Section -->
			<section class="detail-section">
				<h3 class="section-title">Notes ({ticket.notes.length})</h3>
				<div class="section-content">
					{#if ticket.notes.length > 0}
						<ul class="notes-list">
							{#each ticket.notes as note (note.note_id)}
								<li class="note-item">
									<p class="note-content">{note.content}</p>
									<span class="note-meta">
										{formatDateTime(note.created_at)} by {note.created_by.name}
									</span>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="empty-message">No notes yet</p>
					{/if}
				</div>
			</section>

			<!-- Activity / Attribution Section -->
			<section class="detail-section">
				<h3 class="section-title">Activity</h3>
				<div class="section-content">
					<div class="info-row">
						<span class="info-label">Taken in by</span>
						<span class="info-value">{ticket.taken_in_by.name}</span>
					</div>
					{#if ticket.worked_by}
						<div class="info-row">
							<span class="info-label">Worked by</span>
							<span class="info-value">{ticket.worked_by.name}</span>
						</div>
					{/if}
					{#if ticket.closed_by}
						<div class="info-row">
							<span class="info-label">Closed by</span>
							<span class="info-value">{ticket.closed_by.name}</span>
						</div>
					{/if}
					<div class="info-row">
						<span class="info-label">Created</span>
						<span class="info-value">{formatDateTime(ticket.created_at)}</span>
					</div>
					{#if ticket.closed_at}
						<div class="info-row">
							<span class="info-label">Closed</span>
							<span class="info-value">{formatDateTime(ticket.closed_at)}</span>
						</div>
					{/if}
				</div>
			</section>

			<!-- Actions placeholder (to be implemented in facet-5w6) -->
			<section class="detail-section actions-section">
				<div class="actions-placeholder">
					<!-- Action buttons will be added by facet-5w6 -->
				</div>
			</section>
		{/if}
	</div>
</Modal>

<style>
	.ticket-detail-content {
		width: 600px;
		max-width: 90vw;
	}

	/* Loading state */
	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl, 2rem);
		gap: var(--space-md, 1rem);
	}

	.spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--color-border, #e2e8f0);
		border-top-color: var(--color-primary, #1e40af);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* Error state */
	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl, 2rem);
		gap: var(--space-md, 1rem);
	}

	.error-message {
		color: var(--color-rush, #ef4444);
		text-align: center;
	}

	.retry-button {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		background-color: var(--color-primary, #1e40af);
		color: white;
		border: none;
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		font-weight: 500;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.retry-button:hover {
		background-color: var(--color-primary-dark, #1e3a8a);
	}

	/* Ticket header */
	.ticket-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: var(--space-md, 1rem);
		margin-bottom: var(--space-md, 1rem);
		border-bottom: 1px solid var(--color-border, #e2e8f0);
	}

	.ticket-code-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
	}

	.ticket-code {
		font-size: 1.25rem;
		font-weight: 600;
		font-family: var(--font-mono, monospace);
		color: var(--color-text, #1e293b);
	}

	.rush-badge {
		padding: 0.125rem 0.5rem;
		background-color: var(--color-rush, #ef4444);
		color: white;
		font-size: 0.75rem;
		font-weight: 600;
		border-radius: var(--radius-sm, 0.25rem);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.status-badge {
		padding: 0.25rem 0.75rem;
		font-size: 0.875rem;
		font-weight: 500;
		border-radius: var(--radius-md, 0.5rem);
	}

	/* Sections */
	.detail-section {
		margin-bottom: var(--space-lg, 1.5rem);
	}

	.detail-section:last-child {
		margin-bottom: 0;
	}

	.section-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text-muted, #64748b);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.section-content {
		background-color: var(--color-bg, #f8fafc);
		border-radius: var(--radius-md, 0.5rem);
		padding: var(--space-md, 1rem);
	}

	/* Info rows */
	.info-row {
		display: flex;
		padding: var(--space-xs, 0.25rem) 0;
	}

	.info-row:not(:last-child) {
		border-bottom: 1px solid var(--color-border, #e2e8f0);
		padding-bottom: var(--space-sm, 0.5rem);
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.info-label {
		flex-shrink: 0;
		width: 140px;
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
	}

	.info-value {
		flex: 1;
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
	}

	.info-value.text-block {
		white-space: pre-wrap;
		line-height: 1.5;
	}

	/* Photos */
	.photos-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
		gap: var(--space-md, 1rem);
	}

	.photo-item {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.photo-thumbnail {
		width: 100%;
		aspect-ratio: 1;
		object-fit: cover;
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		transition: transform var(--transition-fast, 150ms ease);
	}

	.photo-thumbnail:hover {
		transform: scale(1.02);
	}

	.photo-meta {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
		line-height: 1.3;
	}

	/* Notes */
	.notes-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.note-item {
		padding: var(--space-sm, 0.5rem) 0;
	}

	.note-item:not(:last-child) {
		border-bottom: 1px solid var(--color-border, #e2e8f0);
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.note-content {
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
		line-height: 1.5;
		margin-bottom: var(--space-xs, 0.25rem);
		white-space: pre-wrap;
	}

	.note-meta {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	/* Empty state */
	.empty-message {
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
		font-style: italic;
	}

	/* Actions placeholder */
	.actions-section {
		margin-top: var(--space-lg, 1.5rem);
		padding-top: var(--space-md, 1rem);
		border-top: 1px solid var(--color-border, #e2e8f0);
	}

	.actions-placeholder {
		min-height: 0;
	}
</style>

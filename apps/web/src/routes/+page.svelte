<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';
	import type { PageData } from './$types';
	import { invalidateAll } from '$app/navigation';
	import Button from '$lib/components/Button.svelte';
	import StatusLane from '$lib/components/StatusLane.svelte';
	import TicketCard from '$lib/components/TicketCard.svelte';
	import EmployeeIdModal from '$lib/components/EmployeeIdModal.svelte';
	import IntakeFormModal from '$lib/components/IntakeFormModal.svelte';
	import TicketDetailModal from '$lib/components/TicketDetailModal.svelte';
	import {
		changeTicketStatus,
		setCurrentEmployee,
		type TicketStatus,
		type QueueTicket,
		type VerifyPinResponse
	} from '$lib/services/api';

	let { data }: { data: PageData } = $props();

	// Drag state
	let draggingTicketId = $state<string | null>(null);
	let dragOverLane = $state<TicketStatus | null>(null);

	// Modal state for intake form
	let showIntakeModal = $state(false);

	// Modal state for ticket detail view
	let showTicketDetailModal = $state(false);
	let selectedTicketId = $state<string | null>(null);

	// Modal state for PIN verification before status change
	let showPinModal = $state(false);
	let pendingStatusChange = $state<{
		ticketId: string;
		newStatus: TicketStatus;
		sourceStatus: TicketStatus;
	} | null>(null);
	let statusUpdateError = $state<string | null>(null);

	// Local optimistic state - tracks tickets moved before API confirms
	const optimisticMoves = new SvelteMap<
		string,
		{ fromStatus: TicketStatus; toStatus: TicketStatus }
	>();

	// Computed lanes that apply optimistic moves
	const lanes = $derived.by(() => {
		if (!data.queue) return null;

		// Deep clone the lanes
		const result = {
			intake: { count: 0, tickets: [...data.queue.lanes.intake.tickets] },
			in_progress: { count: 0, tickets: [...data.queue.lanes.in_progress.tickets] },
			waiting_on_parts: { count: 0, tickets: [...data.queue.lanes.waiting_on_parts.tickets] },
			ready_for_pickup: { count: 0, tickets: [...data.queue.lanes.ready_for_pickup.tickets] }
		};

		// Apply optimistic moves
		for (const [ticketId, move] of optimisticMoves) {
			// Remove from source
			const sourceLane = result[move.fromStatus as keyof typeof result];
			if (sourceLane) {
				const idx = sourceLane.tickets.findIndex((t) => t.ticket_id === ticketId);
				if (idx !== -1) {
					const [ticket] = sourceLane.tickets.splice(idx, 1);
					// Add to destination
					const destLane = result[move.toStatus as keyof typeof result];
					if (destLane && ticket) {
						// Update ticket status
						const movedTicket = { ...ticket, status: move.toStatus };
						destLane.tickets.unshift(movedTicket);
					}
				}
			}
		}

		// Update counts
		result.intake.count = result.intake.tickets.length;
		result.in_progress.count = result.in_progress.tickets.length;
		result.waiting_on_parts.count = result.waiting_on_parts.tickets.length;
		result.ready_for_pickup.count = result.ready_for_pickup.tickets.length;

		return result;
	});

	function handleNewTicket() {
		showIntakeModal = true;
	}

	function handleIntakeClose() {
		showIntakeModal = false;
	}

	async function handleIntakeSuccess(ticketId: string, friendlyCode: string) {
		showIntakeModal = false;
		// Refresh the workboard to show the new ticket
		await invalidateAll();
	}

	function handleDragStart(ticketId: string) {
		draggingTicketId = ticketId;
	}

	function handleDragEnd() {
		draggingTicketId = null;
		dragOverLane = null;
	}

	function handleDragEnter(status: TicketStatus) {
		dragOverLane = status;
	}

	function handleDragLeave(status: TicketStatus) {
		if (dragOverLane === status) {
			dragOverLane = null;
		}
	}

	function findTicketStatus(ticketId: string): TicketStatus | null {
		if (!data.queue) return null;
		const laneEntries = Object.entries(data.queue.lanes) as [
			TicketStatus,
			{ tickets: QueueTicket[] }
		][];
		for (const [status, lane] of laneEntries) {
			if (lane.tickets.some((t) => t.ticket_id === ticketId)) {
				return status;
			}
		}
		return null;
	}

	function handleDrop(ticketId: string, targetStatus: TicketStatus) {
		const sourceStatus = findTicketStatus(ticketId);
		if (!sourceStatus || sourceStatus === targetStatus) {
			handleDragEnd();
			return;
		}

		// Store the pending change and show PIN modal
		pendingStatusChange = {
			ticketId,
			newStatus: targetStatus,
			sourceStatus
		};
		statusUpdateError = null;
		showPinModal = true;
		handleDragEnd();
	}

	async function handlePinSuccess(employee: VerifyPinResponse) {
		if (!pendingStatusChange) return;

		const { ticketId, newStatus, sourceStatus } = pendingStatusChange;

		// Set the current employee for the API call
		setCurrentEmployee(employee.employee_id);

		// Apply optimistic update
		optimisticMoves.set(ticketId, { fromStatus: sourceStatus, toStatus: newStatus });

		// Close modal immediately for better UX
		showPinModal = false;
		const savedPendingChange = pendingStatusChange;
		pendingStatusChange = null;

		try {
			await changeTicketStatus(ticketId, newStatus);
			// Success - refresh data from server
			await invalidateAll();
		} catch (err) {
			// Revert optimistic update
			console.error('Failed to update ticket status:', err);
			statusUpdateError =
				err instanceof Error ? err.message : 'Failed to update status. Please try again.';
			// Re-show the modal with error? Or show toast?
			// For now, just revert
		} finally {
			// Clear optimistic move
			optimisticMoves.delete(savedPendingChange.ticketId);
		}
	}

	function handlePinModalClose() {
		showPinModal = false;
		pendingStatusChange = null;
	}

	function openTicketModal(ticketId: string) {
		selectedTicketId = ticketId;
		showTicketDetailModal = true;
	}

	function closeTicketModal() {
		showTicketDetailModal = false;
		selectedTicketId = null;
	}

	async function handleTicketClosed() {
		// Refresh the workboard to reflect the closed ticket
		await invalidateAll();
	}

	// Helper to determine if lane is a valid drop target
	function isValidDropTarget(status: TicketStatus): boolean {
		return draggingTicketId !== null && findTicketStatus(draggingTicketId) !== status;
	}
</script>

<div class="workboard">
	<div class="workboard-header">
		<div class="workboard-header-content">
			<div class="workboard-header-text">
				<h1 class="page-title">Workboard</h1>
				<p class="page-subtitle">
					Manage repair tickets across status lanes. Rush tickets appear first in each lane.
				</p>
			</div>
			<Button variant="primary" onclick={handleNewTicket}>+ New Ticket</Button>
		</div>
	</div>

	{#if data.error}
		<div class="error-message">
			<p>Failed to load queue: {data.error}</p>
		</div>
	{:else if lanes}
		{#if statusUpdateError}
			<div class="error-message error-toast">
				<p>{statusUpdateError}</p>
				<button onclick={() => (statusUpdateError = null)}>Dismiss</button>
			</div>
		{/if}

		<div class="lanes-container">
			<!-- Intake Lane -->
			<StatusLane
				status="intake"
				count={lanes.intake.count}
				isDropTarget={isValidDropTarget('intake')}
				isDragOver={dragOverLane === 'intake'}
				ondragenter={() => handleDragEnter('intake')}
				ondragleave={() => handleDragLeave('intake')}
				ondrop={(ticketId) => handleDrop(ticketId, 'intake')}
			>
				{#each lanes.intake.tickets as ticket (ticket.ticket_id)}
					<TicketCard
						{ticket}
						isDragging={draggingTicketId === ticket.ticket_id}
						ondragstart={handleDragStart}
						ondragend={handleDragEnd}
						onclick={() => openTicketModal(ticket.ticket_id)}
					/>
				{:else}
					<p class="lane-empty-message">No tickets</p>
				{/each}
			</StatusLane>

			<!-- In Progress Lane -->
			<StatusLane
				status="in_progress"
				count={lanes.in_progress.count}
				isDropTarget={isValidDropTarget('in_progress')}
				isDragOver={dragOverLane === 'in_progress'}
				ondragenter={() => handleDragEnter('in_progress')}
				ondragleave={() => handleDragLeave('in_progress')}
				ondrop={(ticketId) => handleDrop(ticketId, 'in_progress')}
			>
				{#each lanes.in_progress.tickets as ticket (ticket.ticket_id)}
					<TicketCard
						{ticket}
						isDragging={draggingTicketId === ticket.ticket_id}
						ondragstart={handleDragStart}
						ondragend={handleDragEnd}
						onclick={() => openTicketModal(ticket.ticket_id)}
					/>
				{:else}
					<p class="lane-empty-message">No tickets</p>
				{/each}
			</StatusLane>

			<!-- Waiting on Parts Lane -->
			<StatusLane
				status="waiting_on_parts"
				count={lanes.waiting_on_parts.count}
				isDropTarget={isValidDropTarget('waiting_on_parts')}
				isDragOver={dragOverLane === 'waiting_on_parts'}
				ondragenter={() => handleDragEnter('waiting_on_parts')}
				ondragleave={() => handleDragLeave('waiting_on_parts')}
				ondrop={(ticketId) => handleDrop(ticketId, 'waiting_on_parts')}
			>
				{#each lanes.waiting_on_parts.tickets as ticket (ticket.ticket_id)}
					<TicketCard
						{ticket}
						isDragging={draggingTicketId === ticket.ticket_id}
						ondragstart={handleDragStart}
						ondragend={handleDragEnd}
						onclick={() => openTicketModal(ticket.ticket_id)}
					/>
				{:else}
					<p class="lane-empty-message">No tickets</p>
				{/each}
			</StatusLane>

			<!-- Ready for Pickup Lane -->
			<StatusLane
				status="ready_for_pickup"
				count={lanes.ready_for_pickup.count}
				isDropTarget={isValidDropTarget('ready_for_pickup')}
				isDragOver={dragOverLane === 'ready_for_pickup'}
				ondragenter={() => handleDragEnter('ready_for_pickup')}
				ondragleave={() => handleDragLeave('ready_for_pickup')}
				ondrop={(ticketId) => handleDrop(ticketId, 'ready_for_pickup')}
			>
				{#each lanes.ready_for_pickup.tickets as ticket (ticket.ticket_id)}
					<TicketCard
						{ticket}
						isDragging={draggingTicketId === ticket.ticket_id}
						ondragstart={handleDragStart}
						ondragend={handleDragEnd}
						onclick={() => openTicketModal(ticket.ticket_id)}
					/>
				{:else}
					<p class="lane-empty-message">No tickets</p>
				{/each}
			</StatusLane>
		</div>
	{:else}
		<div class="loading">Loading workboard...</div>
	{/if}
</div>

<!-- Employee PIN Modal for status changes -->
<EmployeeIdModal
	open={showPinModal}
	title="Verify Employee PIN"
	onClose={handlePinModalClose}
	onSuccess={handlePinSuccess}
/>

<!-- Intake Form Modal for creating new tickets -->
<IntakeFormModal
	open={showIntakeModal}
	onClose={handleIntakeClose}
	onSuccess={handleIntakeSuccess}
/>

<!-- Ticket Detail Modal for viewing ticket details -->
<TicketDetailModal
	ticketId={selectedTicketId}
	open={showTicketDetailModal}
	onClose={closeTicketModal}
	onTicketClosed={handleTicketClosed}
/>

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

	.workboard-header-content {
		display: flex;
		justify-content: space-between;
		align-items: flex-start;
		gap: var(--space-lg);
	}

	.workboard-header-text {
		flex: 1;
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

	.error-toast {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md);
	}

	.error-toast button {
		background: none;
		border: 1px solid currentColor;
		border-radius: var(--radius-sm);
		padding: var(--space-xs) var(--space-sm);
		color: inherit;
		cursor: pointer;
		font-size: 0.875rem;
	}

	.error-toast button:hover {
		background-color: rgba(153, 27, 27, 0.1);
	}

	.loading {
		padding: var(--space-xl);
		text-align: center;
		color: var(--color-text-muted);
	}

	.lanes-container {
		display: flex;
		justify-content: center;
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

	.lane-empty-message {
		padding: var(--space-md);
		text-align: center;
		color: var(--color-text-muted);
		font-size: 0.875rem;
		font-style: italic;
	}
</style>

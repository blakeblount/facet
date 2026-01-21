<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { TicketStatus } from '$lib/types/api';

	interface Props {
		/** The status this lane represents */
		status: TicketStatus;
		/** Display label for the status */
		label?: string;
		/** Number of tickets in this lane */
		count: number;
		/** Whether the lane is currently a drop target */
		isDropTarget?: boolean;
		/** Whether an item is being dragged over this lane */
		isDragOver?: boolean;
		/** Callback when dragenter event fires */
		ondragenter?: () => void;
		/** Callback when dragleave event fires */
		ondragleave?: () => void;
		/** Callback when drop event fires with ticket ID */
		ondrop?: (ticketId: string) => void;
		/** Additional CSS class for the lane */
		class?: string;
		/** Lane card content (slot for ticket cards) */
		children?: Snippet;
	}

	let {
		status,
		label,
		count,
		isDropTarget = false,
		isDragOver = false,
		ondragenter,
		ondragleave,
		ondrop,
		class: className = '',
		children
	}: Props = $props();

	function handleDragOver(event: DragEvent) {
		// Prevent default to allow drop
		event.preventDefault();
		if (event.dataTransfer) {
			event.dataTransfer.dropEffect = 'move';
		}
	}

	function handleDragEnter(event: DragEvent) {
		event.preventDefault();
		ondragenter?.();
	}

	function handleDragLeave(event: DragEvent) {
		// Only trigger if leaving the lane itself, not a child element
		const relatedTarget = event.relatedTarget as HTMLElement | null;
		const currentTarget = event.currentTarget as HTMLElement;
		if (!relatedTarget || !currentTarget.contains(relatedTarget)) {
			ondragleave?.();
		}
	}

	function handleDrop(event: DragEvent) {
		event.preventDefault();
		const ticketId = event.dataTransfer?.getData('text/plain');
		if (ticketId) {
			ondrop?.(ticketId);
		}
	}

	/** Map status to display label */
	const statusLabels: Record<TicketStatus, string> = {
		intake: 'Intake',
		in_progress: 'In Progress',
		waiting_on_parts: 'Waiting on Parts',
		ready_for_pickup: 'Ready for Pickup',
		closed: 'Closed',
		archived: 'Archived'
	};

	/** Map status to CSS class */
	const statusClasses: Record<TicketStatus, string> = {
		intake: 'lane-status-intake',
		in_progress: 'lane-status-in-progress',
		waiting_on_parts: 'lane-status-waiting',
		ready_for_pickup: 'lane-status-ready',
		closed: 'lane-status-closed',
		archived: 'lane-status-archived'
	};

	const displayLabel = $derived(label || statusLabels[status]);
	const statusClass = $derived(statusClasses[status]);
</script>

<div
	class="lane {statusClass} {className}"
	class:is-drop-target={isDropTarget}
	class:is-drag-over={isDragOver}
	data-status={status}
	ondragover={handleDragOver}
	ondragenter={handleDragEnter}
	ondragleave={handleDragLeave}
	ondrop={handleDrop}
	role="list"
	aria-label="{displayLabel} lane"
>
	<header class="lane-header">
		<h3 class="lane-title">{displayLabel}</h3>
		<span class="lane-count">{count}</span>
	</header>
	<div class="lane-content">
		{#if children}
			{@render children()}
		{:else}
			<div class="lane-empty">
				<span class="lane-empty-text">No tickets</span>
			</div>
		{/if}
	</div>
</div>

<style>
	.lane {
		display: flex;
		flex-direction: column;
		min-width: 280px;
		max-width: 320px;
		height: 100%;
		background-color: var(--color-background, #fafaf8);
		border-radius: var(--radius-lg, 0.5rem);
		border: 1px solid var(--color-border, #e2e8f0);
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	/* Drop target styling */
	.is-drop-target {
		border-style: dashed;
		border-width: 2px;
	}

	.is-drag-over {
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgb(30 64 175 / 0.15);
		background-color: var(--color-surface, #ffffff);
	}

	/* Lane header */
	.lane-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-md, 1rem);
		border-bottom: 2px solid var(--color-border, #e2e8f0);
		flex-shrink: 0;
	}

	.lane-title {
		margin: 0;
		font-family: var(--font-heading, inherit);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text, #1e293b);
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	.lane-count {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 1.75rem;
		height: 1.75rem;
		padding: 0 var(--space-sm, 0.5rem);
		font-size: 0.875rem;
		font-weight: 700;
		color: white;
		background-color: var(--color-text-muted, #6b6b6b);
		border-radius: var(--radius-md, 0.375rem);
		text-shadow: 0 1px 2px rgb(0 0 0 / 0.3);
	}

	/* Status-specific count colors */
	.lane-status-intake .lane-count {
		background-color: var(--color-status-intake, #6b5b95);
	}

	.lane-status-in-progress .lane-count {
		background-color: var(--color-status-in-progress, #1e3a5f);
	}

	.lane-status-waiting .lane-count {
		background-color: var(--color-status-waiting, #d4a017);
	}

	.lane-status-ready .lane-count {
		background-color: var(--color-status-ready, #2d5a3d);
	}

	.lane-status-closed .lane-count {
		background-color: var(--color-status-closed, #6b6b6b);
	}

	.lane-status-archived .lane-count {
		background-color: var(--color-status-closed, #6b6b6b);
	}

	/* Scrollable content area */
	.lane-content {
		flex: 1;
		overflow-y: auto;
		padding: var(--space-sm, 0.5rem);
		display: flex;
		flex-direction: column;
		gap: var(--space-sm, 0.5rem);
	}

	/* Custom scrollbar styling */
	.lane-content::-webkit-scrollbar {
		width: 6px;
	}

	.lane-content::-webkit-scrollbar-track {
		background: transparent;
	}

	.lane-content::-webkit-scrollbar-thumb {
		background-color: var(--color-border, #e2e8f0);
		border-radius: 3px;
	}

	.lane-content::-webkit-scrollbar-thumb:hover {
		background-color: var(--color-text-muted, #6b6b6b);
	}

	/* Empty state */
	.lane-empty {
		display: flex;
		align-items: center;
		justify-content: center;
		flex: 1;
		min-height: 100px;
		border: 2px dashed var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.375rem);
		margin: var(--space-xs, 0.25rem);
	}

	.lane-empty-text {
		color: var(--color-text-muted, #6b6b6b);
		font-size: 0.875rem;
		font-style: italic;
	}

	/* Status-specific header accent colors */
	.lane-status-intake .lane-header {
		border-bottom-color: var(--color-status-intake, #6b5b95);
	}

	.lane-status-in-progress .lane-header {
		border-bottom-color: var(--color-status-in-progress, #1e3a5f);
	}

	.lane-status-waiting .lane-header {
		border-bottom-color: var(--color-status-waiting, #d4a017);
	}

	.lane-status-ready .lane-header {
		border-bottom-color: var(--color-status-ready, #2d5a3d);
	}

	.lane-status-closed .lane-header {
		border-bottom-color: var(--color-status-closed, #6b6b6b);
	}

	.lane-status-archived .lane-header {
		border-bottom-color: var(--color-status-closed, #6b6b6b);
	}
</style>

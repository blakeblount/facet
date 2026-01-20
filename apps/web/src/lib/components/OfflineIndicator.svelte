<script lang="ts">
	import { offlineStore } from '$lib/stores/offline.svelte';
	import { syncQueueStore } from '$lib/services/syncQueue';

	interface Props {
		/** Show pending count badge */
		showPendingCount?: boolean;
	}

	let { showPendingCount = true }: Props = $props();

	// Computed display text
	const statusText = $derived(() => {
		if (offlineStore.isOffline) {
			return 'Offline';
		}
		if (syncQueueStore.isSyncing) {
			return 'Syncing...';
		}
		if (syncQueueStore.hasPending) {
			return `${syncQueueStore.pendingCount} pending`;
		}
		return null;
	});

	// Computed visibility
	const isVisible = $derived(
		offlineStore.isOffline || syncQueueStore.hasPending || syncQueueStore.isSyncing
	);
</script>

{#if isVisible}
	<div
		class="offline-indicator"
		class:offline={offlineStore.isOffline}
		class:syncing={syncQueueStore.isSyncing}
		class:pending={!offlineStore.isOffline &&
			syncQueueStore.hasPending &&
			!syncQueueStore.isSyncing}
	>
		{#if offlineStore.isOffline}
			<!-- Offline icon -->
			<svg
				class="status-icon"
				xmlns="http://www.w3.org/2000/svg"
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<line x1="2" x2="22" y1="2" y2="22" />
				<path d="M8.5 16.5a5 5 0 0 1 7 0" />
				<path d="M2 8.82a15 15 0 0 1 4.17-2.65" />
				<path d="M10.66 5c4.01-.36 8.14.9 11.34 3.76" />
				<path d="M16.85 11.25a10 10 0 0 1 2.22 1.68" />
				<path d="M5 13a10 10 0 0 1 5.24-2.76" />
				<line x1="12" x2="12.01" y1="20" y2="20" />
			</svg>
		{:else if syncQueueStore.isSyncing}
			<!-- Syncing spinner -->
			<span class="sync-spinner" aria-label="Syncing"></span>
		{:else}
			<!-- Pending cloud icon -->
			<svg
				class="status-icon"
				xmlns="http://www.w3.org/2000/svg"
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M12 13v8" />
				<path d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242" />
				<path d="m8 17 4-4 4 4" />
			</svg>
		{/if}

		{#if statusText()}
			<span class="status-text">{statusText()}</span>
		{/if}

		{#if showPendingCount && syncQueueStore.hasPending && !offlineStore.isOffline && !syncQueueStore.isSyncing}
			<span class="pending-badge">{syncQueueStore.pendingCount}</span>
		{/if}
	</div>
{/if}

<style>
	.offline-indicator {
		display: flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
		padding: var(--space-xs, 0.25rem) var(--space-sm, 0.5rem);
		font-size: 0.75rem;
		font-weight: 500;
		border-radius: var(--radius-md, 0.5rem);
		transition: all var(--transition-fast, 150ms ease);
	}

	.offline-indicator.offline {
		background-color: rgba(239, 68, 68, 0.2);
		color: #fecaca;
	}

	.offline-indicator.syncing {
		background-color: rgba(59, 130, 246, 0.2);
		color: #bfdbfe;
	}

	.offline-indicator.pending {
		background-color: rgba(251, 191, 36, 0.2);
		color: #fef08a;
	}

	.status-icon {
		flex-shrink: 0;
	}

	.status-text {
		white-space: nowrap;
	}

	.sync-spinner {
		width: 14px;
		height: 14px;
		border: 2px solid currentColor;
		border-top-color: transparent;
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.pending-badge {
		display: flex;
		align-items: center;
		justify-content: center;
		min-width: 1.25rem;
		height: 1.25rem;
		padding: 0 0.25rem;
		background-color: currentColor;
		color: var(--color-primary, #1e40af);
		border-radius: 9999px;
		font-size: 0.625rem;
		font-weight: 700;
	}
</style>

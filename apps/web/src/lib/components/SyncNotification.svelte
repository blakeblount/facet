<script lang="ts">
	import { offlineStore } from '$lib/stores/offline.svelte';
	import { syncQueueStore } from '$lib/services/syncQueue.svelte';

	// Track sync results for notifications
	let notifications = $state<
		Array<{
			id: string;
			type: 'success' | 'error' | 'info';
			message: string;
			timestamp: number;
		}>
	>([]);

	let isAutoSyncing = $state(false);

	// Watch for online status changes and auto-sync
	$effect(() => {
		if (offlineStore.isOnline && syncQueueStore.hasPending && !syncQueueStore.isSyncing) {
			handleAutoSync();
		}
	});

	async function handleAutoSync() {
		if (isAutoSyncing || syncQueueStore.isSyncing) {
			return;
		}

		isAutoSyncing = true;

		// Add info notification
		addNotification('info', `Syncing ${syncQueueStore.pendingCount} offline ticket(s)...`);

		try {
			const results = await syncQueueStore.syncAll((clientId, status, result) => {
				if (status === 'synced' && result?.friendlyCode) {
					addNotification('success', `Ticket ${result.friendlyCode} synced successfully`);
				} else if (status === 'failed' && result?.error) {
					addNotification('error', `Failed to sync ticket: ${result.error}`);
				}
			});

			// Summary notification
			const successful = [...results.values()].filter((r) => r.success).length;
			const failed = [...results.values()].filter((r) => !r.success).length;

			if (failed > 0) {
				addNotification('error', `Sync complete: ${successful} synced, ${failed} failed`);
			} else if (successful > 0) {
				// Remove the "syncing" notification and show completion
				removeNotificationsOfType('info');
			}

			// Clean up synced tickets after a delay
			setTimeout(() => {
				syncQueueStore.clearSynced();
			}, 5000);
		} catch {
			addNotification('error', 'Sync failed. Will retry when online.');
		} finally {
			isAutoSyncing = false;
		}
	}

	function addNotification(type: 'success' | 'error' | 'info', message: string) {
		const id = crypto.randomUUID();
		notifications = [
			...notifications,
			{
				id,
				type,
				message,
				timestamp: Date.now()
			}
		];

		// Auto-dismiss success and info notifications after 5 seconds
		if (type !== 'error') {
			setTimeout(() => {
				dismissNotification(id);
			}, 5000);
		}
	}

	function dismissNotification(id: string) {
		notifications = notifications.filter((n) => n.id !== id);
	}

	function removeNotificationsOfType(type: 'success' | 'error' | 'info') {
		notifications = notifications.filter((n) => n.type !== type);
	}
</script>

{#if notifications.length > 0}
	<div class="sync-notifications" role="status" aria-live="polite">
		{#each notifications as notification (notification.id)}
			<div class="notification notification-{notification.type}">
				{#if notification.type === 'success'}
					<svg
						class="notification-icon"
						xmlns="http://www.w3.org/2000/svg"
						width="18"
						height="18"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<polyline points="20 6 9 17 4 12" />
					</svg>
				{:else if notification.type === 'error'}
					<svg
						class="notification-icon"
						xmlns="http://www.w3.org/2000/svg"
						width="18"
						height="18"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<circle cx="12" cy="12" r="10" />
						<line x1="12" x2="12" y1="8" y2="12" />
						<line x1="12" x2="12.01" y1="16" y2="16" />
					</svg>
				{:else}
					<span class="notification-spinner" aria-hidden="true"></span>
				{/if}
				<span class="notification-message">{notification.message}</span>
				<button
					class="notification-dismiss"
					onclick={() => dismissNotification(notification.id)}
					aria-label="Dismiss notification"
				>
					<svg
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
						<line x1="18" x2="6" y1="6" y2="18" />
						<line x1="6" x2="18" y1="6" y2="18" />
					</svg>
				</button>
			</div>
		{/each}
	</div>
{/if}

<style>
	.sync-notifications {
		position: fixed;
		bottom: var(--space-lg, 1.5rem);
		right: var(--space-lg, 1.5rem);
		z-index: 1000;
		display: flex;
		flex-direction: column;
		gap: var(--space-sm, 0.5rem);
		max-width: 400px;
	}

	.notification {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		background-color: var(--color-surface, #ffffff);
		border-radius: var(--radius-md, 0.5rem);
		box-shadow:
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 2px 4px -1px rgba(0, 0, 0, 0.06);
		animation: slideIn 0.2s ease-out;
	}

	@keyframes slideIn {
		from {
			opacity: 0;
			transform: translateX(20px);
		}
		to {
			opacity: 1;
			transform: translateX(0);
		}
	}

	.notification-success {
		border-left: 4px solid var(--color-success, #22c55e);
	}

	.notification-error {
		border-left: 4px solid var(--color-rush, #ef4444);
	}

	.notification-info {
		border-left: 4px solid var(--color-primary, #1e40af);
	}

	.notification-icon {
		flex-shrink: 0;
	}

	.notification-success .notification-icon {
		color: var(--color-success, #22c55e);
	}

	.notification-error .notification-icon {
		color: var(--color-rush, #ef4444);
	}

	.notification-spinner {
		width: 16px;
		height: 16px;
		border: 2px solid var(--color-border, #e2e8f0);
		border-top-color: var(--color-primary, #1e40af);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
		flex-shrink: 0;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.notification-message {
		flex: 1;
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
	}

	.notification-dismiss {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		padding: 0;
		background: none;
		border: none;
		border-radius: var(--radius-sm, 0.25rem);
		color: var(--color-text-muted, #64748b);
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.notification-dismiss:hover {
		background-color: var(--color-bg, #f8fafc);
	}

	/* Mobile adjustments */
	@media (max-width: 480px) {
		.sync-notifications {
			left: var(--space-md, 1rem);
			right: var(--space-md, 1rem);
			max-width: none;
		}
	}
</style>

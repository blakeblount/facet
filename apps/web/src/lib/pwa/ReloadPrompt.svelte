<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	let offlineReady = $state(false);
	let needRefresh = $state(false);
	let swRegistration: ServiceWorkerRegistration | null = $state(null);

	async function updateServiceWorker() {
		if (swRegistration?.waiting) {
			swRegistration.waiting.postMessage({ type: 'SKIP_WAITING' });
		}
	}

	function close() {
		offlineReady = false;
		needRefresh = false;
	}

	onMount(async () => {
		if (!browser) return;

		const { registerSW } = await import('virtual:pwa-register');

		registerSW({
			immediate: true,
			onRegisteredSW(swUrl: string, registration: ServiceWorkerRegistration | undefined) {
				swRegistration = registration ?? null;
				console.log(`Service Worker registered: ${swUrl}`);
			},
			onOfflineReady() {
				offlineReady = true;
				console.log('App ready to work offline');
			},
			onNeedRefresh() {
				needRefresh = true;
				console.log('New content available, please refresh');
			},
			onRegisterError(error: Error) {
				console.error('Service Worker registration error:', error);
			}
		});
	});
</script>

{#if offlineReady || needRefresh}
	<div class="pwa-toast" role="alert" aria-labelledby="toast-message">
		<div class="message" id="toast-message">
			{#if offlineReady}
				<span>App ready to work offline</span>
			{:else}
				<span>New content available, click reload to update</span>
			{/if}
		</div>
		<div class="buttons">
			{#if needRefresh}
				<button onclick={updateServiceWorker}>Reload</button>
			{/if}
			<button onclick={close}>Close</button>
		</div>
	</div>
{/if}

<style>
	.pwa-toast {
		position: fixed;
		right: var(--space-lg, 1.5rem);
		bottom: var(--space-lg, 1.5rem);
		margin: var(--space-md, 1rem);
		padding: var(--space-md, 1rem);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		background-color: var(--color-surface, #ffffff);
		box-shadow:
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 2px 4px -1px rgba(0, 0, 0, 0.06);
		z-index: 9999;
		display: flex;
		flex-direction: column;
		gap: var(--space-sm, 0.5rem);
	}

	.message {
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
	}

	.buttons {
		display: flex;
		gap: var(--space-sm, 0.5rem);
	}

	button {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		border-radius: var(--radius-sm, 0.25rem);
		border: 1px solid var(--color-border, #e2e8f0);
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text, #1e293b);
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	button:hover {
		background-color: var(--color-border, #e2e8f0);
	}

	button:first-child {
		background-color: var(--color-primary, #1e40af);
		color: white;
		border-color: var(--color-primary, #1e40af);
	}

	button:first-child:hover {
		background-color: var(--color-primary-dark, #1e3a8a);
	}
</style>

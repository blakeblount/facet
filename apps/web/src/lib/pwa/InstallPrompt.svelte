<script lang="ts">
	import { onMount } from 'svelte';
	import { browser } from '$app/environment';

	interface BeforeInstallPromptEvent extends Event {
		prompt(): Promise<void>;
		userChoice: Promise<{ outcome: 'accepted' | 'dismissed' }>;
	}

	let deferredPrompt: BeforeInstallPromptEvent | null = $state(null);
	let showInstallPrompt = $state(false);
	let isInstalled = $state(false);

	async function handleInstall() {
		if (!deferredPrompt) return;

		deferredPrompt.prompt();
		const { outcome } = await deferredPrompt.userChoice;

		if (outcome === 'accepted') {
			console.log('User accepted the install prompt');
		} else {
			console.log('User dismissed the install prompt');
		}

		deferredPrompt = null;
		showInstallPrompt = false;
	}

	function dismissPrompt() {
		showInstallPrompt = false;
		// Store dismissal in localStorage to not bother user again for a while
		if (browser) {
			localStorage.setItem('pwa-install-dismissed', Date.now().toString());
		}
	}

	onMount(() => {
		if (!browser) return;

		// Check if already installed
		if (window.matchMedia('(display-mode: standalone)').matches) {
			isInstalled = true;
			return;
		}

		// Check if recently dismissed
		const dismissed = localStorage.getItem('pwa-install-dismissed');
		if (dismissed) {
			const dismissedTime = parseInt(dismissed, 10);
			const daysSinceDismissed = (Date.now() - dismissedTime) / (1000 * 60 * 60 * 24);
			if (daysSinceDismissed < 7) {
				return; // Don't show again for 7 days
			}
		}

		// Listen for the beforeinstallprompt event
		const handler = (e: Event) => {
			e.preventDefault();
			deferredPrompt = e as BeforeInstallPromptEvent;
			showInstallPrompt = true;
		};

		window.addEventListener('beforeinstallprompt', handler);

		// Listen for successful installation
		window.addEventListener('appinstalled', () => {
			isInstalled = true;
			showInstallPrompt = false;
			deferredPrompt = null;
			console.log('App was installed');
		});

		return () => {
			window.removeEventListener('beforeinstallprompt', handler);
		};
	});
</script>

{#if showInstallPrompt && !isInstalled}
	<div class="install-prompt" role="dialog" aria-labelledby="install-title">
		<div class="content">
			<h3 id="install-title">Install Facet</h3>
			<p>Install this app on your device for quick access and offline capability.</p>
		</div>
		<div class="buttons">
			<button class="install-btn" onclick={handleInstall}>Install</button>
			<button class="dismiss-btn" onclick={dismissPrompt}>Not now</button>
		</div>
	</div>
{/if}

<style>
	.install-prompt {
		position: fixed;
		left: 1rem;
		bottom: 1rem;
		max-width: 320px;
		padding: 1rem;
		border: 1px solid #e5e7eb;
		border-radius: 0.75rem;
		background-color: white;
		box-shadow:
			0 10px 15px -3px rgb(0 0 0 / 0.1),
			0 4px 6px -4px rgb(0 0 0 / 0.1);
		z-index: 9998;
	}

	.content {
		margin-bottom: 1rem;
	}

	h3 {
		margin: 0 0 0.5rem;
		font-size: 1rem;
		font-weight: 600;
		color: #111827;
	}

	p {
		margin: 0;
		font-size: 0.875rem;
		color: #6b7280;
		line-height: 1.5;
	}

	.buttons {
		display: flex;
		gap: 0.5rem;
	}

	button {
		padding: 0.5rem 1rem;
		font-size: 0.875rem;
		font-weight: 500;
		border-radius: 0.375rem;
		cursor: pointer;
		transition: background-color 0.15s;
	}

	.install-btn {
		background-color: #1e40af;
		color: white;
		border: none;
	}

	.install-btn:hover {
		background-color: #1e3a8a;
	}

	.dismiss-btn {
		background-color: transparent;
		color: #6b7280;
		border: 1px solid #d1d5db;
	}

	.dismiss-btn:hover {
		background-color: #f3f4f6;
	}
</style>

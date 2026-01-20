<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { ReloadPrompt, InstallPrompt } from '$lib/pwa';
	import Header from '$lib/components/Header.svelte';
	import SyncNotification from '$lib/components/SyncNotification.svelte';
	import { themeStore } from '$lib/stores/theme.svelte';
	import { offlineStore } from '$lib/stores/offline.svelte';
	import { syncQueueStore } from '$lib/services/syncQueue';
	import '../app.css';
	import '$lib/themes/imperial.css';
	import '$lib/themes/arcane.css';

	// Initialize theme on app load
	$effect(() => {
		themeStore.init();
	});

	// Initialize offline detection and sync queue
	$effect(() => {
		offlineStore.init();
		syncQueueStore.init();
	});

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Facet - Jewelry Repair Tracking</title>
</svelte:head>

<div class="app-layout">
	<Header />

	<!-- Main content area -->
	<main class="app-main">
		{@render children()}
	</main>
</div>

<ReloadPrompt />
<InstallPrompt />
<SyncNotification />

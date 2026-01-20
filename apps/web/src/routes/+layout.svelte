<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { ReloadPrompt, InstallPrompt } from '$lib/pwa';
	import Header from '$lib/components/Header.svelte';
	import '../app.css';

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

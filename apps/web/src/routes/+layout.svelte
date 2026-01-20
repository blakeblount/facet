<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { ReloadPrompt, InstallPrompt } from '$lib/pwa';
	import { resolve } from '$app/paths';
	import '../app.css';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
	<title>Facet - Jewelry Repair Tracking</title>
</svelte:head>

<div class="app-layout">
	<!-- Header slot area - Header component will be added here -->
	<header class="app-header">
		<div class="header-content">
			<a href={resolve('/')} class="logo">
				<span class="logo-text">Facet</span>
			</a>
			<nav class="nav-links">
				<a href={resolve('/')} class="nav-link">Workboard</a>
				<a href={resolve('/search')} class="nav-link">Search</a>
				<a href={resolve('/admin')} class="nav-link">Admin</a>
			</nav>
		</div>
	</header>

	<!-- Main content area -->
	<main class="app-main">
		{@render children()}
	</main>
</div>

<ReloadPrompt />
<InstallPrompt />

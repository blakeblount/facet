<script lang="ts">
	import favicon from '$lib/assets/favicon.svg';
	import { ReloadPrompt, InstallPrompt } from '$lib/pwa';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{@render children()}

<ReloadPrompt />
<InstallPrompt />

<script lang="ts">
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<div class="ticket-detail">
	<div class="ticket-header">
		<h1 class="page-title">Ticket Detail</h1>
		<p class="page-subtitle">Ticket ID: {data.id}</p>
	</div>

	<div class="placeholder-content">
		<p>Ticket detail page coming soon.</p>
		<p>This route serves as a placeholder for individual ticket views.</p>
	</div>
</div>

<style>
	.ticket-detail {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		max-width: 900px;
		margin: 0 auto;
	}

	.page-title {
		font-size: 1.75rem;
		margin-bottom: var(--space-xs);
	}

	.page-subtitle {
		color: var(--color-text-muted);
		font-family: var(--font-mono);
	}

	.placeholder-content {
		padding: var(--space-xl);
		background-color: var(--color-bg-card);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		text-align: center;
		color: var(--color-text-muted);
	}

	.placeholder-content p {
		margin: var(--space-sm) 0;
	}
</style>

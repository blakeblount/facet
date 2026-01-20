<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Padding size for card content */
		padding?: 'none' | 'sm' | 'md' | 'lg';
		/** Shadow depth for the card */
		shadow?: 'none' | 'sm' | 'md' | 'lg';
		/** Whether the card is clickable (adds hover effects) */
		clickable?: boolean;
		/** Additional CSS class for the card */
		class?: string;
		/** Click handler (only applicable when clickable is true) */
		onclick?: (e: MouseEvent) => void;
		/** Card content */
		children: Snippet;
	}

	let {
		padding = 'md',
		shadow = 'sm',
		clickable = false,
		class: className = '',
		onclick,
		children
	}: Props = $props();
</script>

{#if clickable}
	<button
		type="button"
		class="card card-padding-{padding} card-shadow-{shadow} is-clickable {className}"
		{onclick}
	>
		{@render children()}
	</button>
{:else}
	<div class="card card-padding-{padding} card-shadow-{shadow} {className}">
		{@render children()}
	</div>
{/if}

<style>
	.card {
		background-color: var(--color-bg-card, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-lg, 0.75rem);
		transition:
			box-shadow var(--transition-fast, 150ms ease),
			border-color var(--transition-fast, 150ms ease),
			transform var(--transition-fast, 150ms ease);
	}

	/* Padding variants */
	.card-padding-none {
		padding: 0;
	}

	.card-padding-sm {
		padding: var(--space-sm, 0.5rem);
	}

	.card-padding-md {
		padding: var(--space-md, 1rem);
	}

	.card-padding-lg {
		padding: var(--space-lg, 1.5rem);
	}

	/* Shadow variants */
	.card-shadow-none {
		box-shadow: none;
	}

	.card-shadow-sm {
		box-shadow: var(--shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
	}

	.card-shadow-md {
		box-shadow: var(--shadow-md, 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1));
	}

	.card-shadow-lg {
		box-shadow: var(
			--shadow-lg,
			0 10px 15px -3px rgb(0 0 0 / 0.1),
			0 4px 6px -4px rgb(0 0 0 / 0.1)
		);
	}

	/* Clickable state - button reset and hover effects */
	.is-clickable {
		display: block;
		width: 100%;
		text-align: inherit;
		font: inherit;
		color: inherit;
		cursor: pointer;
	}

	.is-clickable:hover {
		border-color: var(--color-primary-light, #3b82f6);
		transform: translateY(-1px);
	}

	.is-clickable:hover.card-shadow-none {
		box-shadow: var(--shadow-sm, 0 1px 2px 0 rgb(0 0 0 / 0.05));
	}

	.is-clickable:hover.card-shadow-sm {
		box-shadow: var(--shadow-md, 0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1));
	}

	.is-clickable:hover.card-shadow-md {
		box-shadow: var(
			--shadow-lg,
			0 10px 15px -3px rgb(0 0 0 / 0.1),
			0 4px 6px -4px rgb(0 0 0 / 0.1)
		);
	}

	.is-clickable:hover.card-shadow-lg {
		box-shadow:
			0 20px 25px -5px rgb(0 0 0 / 0.1),
			0 8px 10px -6px rgb(0 0 0 / 0.1);
	}

	.is-clickable:active {
		transform: translateY(0);
	}

	/* Focus state for accessibility */
	.is-clickable:focus-visible {
		outline: 2px solid var(--color-primary, #1e40af);
		outline-offset: 2px;
	}
</style>

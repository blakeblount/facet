<script lang="ts">
	import type { Snippet } from 'svelte';

	interface Props {
		/** Button variant - determines color scheme */
		variant?: 'primary' | 'secondary' | 'danger';
		/** Button size */
		size?: 'sm' | 'md' | 'lg';
		/** Whether the button is disabled */
		disabled?: boolean;
		/** Whether the button is in a loading state */
		loading?: boolean;
		/** Button type attribute */
		type?: 'button' | 'submit' | 'reset';
		/** Additional CSS class for the button */
		class?: string;
		/** Click handler */
		onclick?: (e: MouseEvent) => void;
		/** Button content */
		children: Snippet;
	}

	let {
		variant = 'primary',
		size = 'md',
		disabled = false,
		loading = false,
		type = 'button',
		class: className = '',
		onclick,
		children
	}: Props = $props();

	const isDisabled = $derived(disabled || loading);
</script>

<button
	{type}
	class="btn btn-{variant} btn-{size} {className}"
	class:is-loading={loading}
	disabled={isDisabled}
	aria-disabled={isDisabled}
	aria-busy={loading}
	{onclick}
>
	{#if loading}
		<span class="spinner" aria-hidden="true"></span>
	{/if}
	<span class="btn-content" class:visually-hidden={loading}>
		{@render children()}
	</span>
</button>

<style>
	.btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: var(--space-sm, 0.5rem);
		font-family: inherit;
		font-weight: 500;
		line-height: 1;
		text-align: center;
		white-space: nowrap;
		vertical-align: middle;
		cursor: pointer;
		user-select: none;
		border: 1px solid transparent;
		border-radius: var(--radius-md, 0.5rem);
		transition:
			background-color var(--transition-fast, 150ms ease),
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease),
			opacity var(--transition-fast, 150ms ease);
	}

	/* Focus state - visible for accessibility */
	.btn:focus-visible {
		outline: 2px solid var(--color-primary, #1e40af);
		outline-offset: 2px;
	}

	/* Size variants */
	.btn-sm {
		padding: var(--space-xs, 0.25rem) var(--space-sm, 0.5rem);
		font-size: 0.75rem;
		min-height: 1.75rem;
	}

	.btn-md {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		min-height: 2.25rem;
	}

	.btn-lg {
		padding: var(--space-sm, 0.5rem) var(--space-lg, 1.5rem);
		font-size: 1rem;
		min-height: 2.75rem;
	}

	/* Primary variant */
	.btn-primary {
		background-color: var(--color-primary, #1e40af);
		color: white;
		border-color: var(--color-primary, #1e40af);
	}

	.btn-primary:hover:not(:disabled) {
		background-color: var(--color-primary-dark, #1e3a8a);
		border-color: var(--color-primary-dark, #1e3a8a);
	}

	.btn-primary:active:not(:disabled) {
		background-color: var(--color-primary-dark, #1e3a8a);
		box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	/* Secondary variant */
	.btn-secondary {
		background-color: var(--color-surface, #ffffff);
		color: var(--color-text, #1e293b);
		border-color: var(--color-border, #e2e8f0);
	}

	.btn-secondary:hover:not(:disabled) {
		background-color: var(--color-bg, #f8fafc);
		border-color: var(--color-primary-light, #3b82f6);
	}

	.btn-secondary:active:not(:disabled) {
		background-color: var(--color-bg, #f8fafc);
		box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.05);
	}

	/* Danger variant */
	.btn-danger {
		background-color: var(--color-rush, #ef4444);
		color: white;
		border-color: var(--color-rush, #ef4444);
	}

	.btn-danger:hover:not(:disabled) {
		background-color: #dc2626;
		border-color: #dc2626;
	}

	.btn-danger:active:not(:disabled) {
		background-color: #b91c1c;
		border-color: #b91c1c;
		box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
	}

	/* Disabled state */
	.btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	/* Loading state */
	.is-loading {
		position: relative;
		cursor: wait;
	}

	.btn-content {
		display: inline-flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
	}

	.visually-hidden {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}

	/* Spinner animation */
	.spinner {
		width: 1em;
		height: 1em;
		border: 2px solid currentColor;
		border-right-color: transparent;
		border-radius: 50%;
		animation: spin 0.75s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}
</style>

<script lang="ts">
	interface Props {
		/** Placeholder text shown when input is empty */
		placeholder?: string;
		/** Current value of the search input (two-way bindable) */
		value?: string;
		/** Whether the input is disabled */
		disabled?: boolean;
		/** Debounce delay in milliseconds */
		debounce?: number;
		/** Additional CSS class for the wrapper */
		class?: string;
		/** Unique ID for the input (auto-generated if not provided) */
		id?: string;
		/** Called when the debounced value changes */
		onsearch?: (value: string) => void;
		/** Called when Enter key is pressed */
		onsubmit?: (value: string) => void;
	}

	let {
		placeholder = 'Search...',
		value = $bindable(''),
		disabled = false,
		debounce = 300,
		class: className = '',
		id,
		onsearch,
		onsubmit
	}: Props = $props();

	// Generate a stable unique ID if not provided
	const generatedId = `search-${Math.random().toString(36).substring(2, 9)}`;
	const inputId = $derived(id ?? generatedId);

	// Track debounce timer
	let debounceTimer: ReturnType<typeof setTimeout> | undefined;

	// Handle input changes with debounce
	function handleInput(event: Event) {
		const target = event.target as HTMLInputElement;
		value = target.value;

		// Clear previous timer
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		// Set new timer for debounced callback
		if (onsearch) {
			debounceTimer = setTimeout(() => {
				onsearch(value);
			}, debounce);
		}
	}

	// Handle Enter key
	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();

			// Cancel any pending debounce
			if (debounceTimer) {
				clearTimeout(debounceTimer);
			}

			// Trigger both callbacks immediately
			onsearch?.(value);
			onsubmit?.(value);
		}
	}

	// Clear the input
	function handleClear() {
		value = '';

		// Clear any pending debounce
		if (debounceTimer) {
			clearTimeout(debounceTimer);
		}

		// Notify that search was cleared
		onsearch?.('');
	}

	// Cleanup timer on unmount
	$effect(() => {
		return () => {
			if (debounceTimer) {
				clearTimeout(debounceTimer);
			}
		};
	});

	// Derived state for showing clear button
	const showClear = $derived(value.length > 0 && !disabled);
</script>

<div class="search-wrapper {className}" class:is-disabled={disabled}>
	<span class="search-icon" aria-hidden="true">
		<svg
			width="16"
			height="16"
			viewBox="0 0 16 16"
			fill="none"
			stroke="currentColor"
			stroke-width="2"
			stroke-linecap="round"
			stroke-linejoin="round"
		>
			<circle cx="6.5" cy="6.5" r="5" />
			<path d="M10.5 10.5L14.5 14.5" />
		</svg>
	</span>

	<input
		type="search"
		id={inputId}
		class="search-field"
		{placeholder}
		{disabled}
		{value}
		oninput={handleInput}
		onkeydown={handleKeydown}
		aria-label={placeholder}
	/>

	{#if showClear}
		<button type="button" class="clear-button" onclick={handleClear} aria-label="Clear search">
			<svg
				width="14"
				height="14"
				viewBox="0 0 14 14"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				stroke-linejoin="round"
			>
				<path d="M3 3L11 11M3 11L11 3" />
			</svg>
		</button>
	{/if}
</div>

<style>
	.search-wrapper {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: var(--space-md, 1rem);
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted, #64748b);
		pointer-events: none;
	}

	.search-field {
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		padding-left: calc(var(--space-md, 1rem) + 16px + var(--space-sm, 0.5rem));
		padding-right: calc(var(--space-md, 1rem) + 14px + var(--space-sm, 0.5rem));
		font-size: 0.875rem;
		font-family: inherit;
		line-height: 1.5;
		color: var(--color-text, #1e293b);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.search-field::placeholder {
		color: var(--color-text-muted, #64748b);
	}

	.search-field:hover:not(:disabled) {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.search-field:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.15);
	}

	/* Hide the default search cancel button in WebKit browsers */
	.search-field::-webkit-search-cancel-button {
		display: none;
	}

	.clear-button {
		position: absolute;
		right: var(--space-sm, 0.5rem);
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		padding: 0;
		color: var(--color-text-muted, #64748b);
		background: transparent;
		border: none;
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
		transition:
			color var(--transition-fast, 150ms ease),
			background-color var(--transition-fast, 150ms ease);
	}

	.clear-button:hover {
		color: var(--color-text, #1e293b);
		background-color: var(--color-bg, #f8fafc);
	}

	.clear-button:focus-visible {
		outline: 2px solid var(--color-primary, #1e40af);
		outline-offset: 2px;
	}

	/* Disabled state */
	.is-disabled .search-icon {
		color: var(--color-text-muted, #64748b);
		opacity: 0.5;
	}

	.search-field:disabled {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text-muted, #64748b);
		cursor: not-allowed;
		opacity: 0.7;
	}
</style>

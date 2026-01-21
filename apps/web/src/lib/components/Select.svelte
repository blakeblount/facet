<script lang="ts">
	interface Option {
		value: string;
		label: string;
	}

	interface Props {
		/** Label text displayed above the select */
		label?: string;
		/** Options to display in the dropdown */
		options: Option[];
		/** Currently selected value (two-way bindable) */
		value?: string;
		/** Placeholder text shown when no option is selected */
		placeholder?: string;
		/** Error message to display below the select */
		error?: string;
		/** Whether the select is disabled */
		disabled?: boolean;
		/** Whether the select is required */
		required?: boolean;
		/** Input name attribute for forms */
		name?: string;
		/** Additional CSS class for the wrapper */
		class?: string;
		/** Unique ID for the select (auto-generated if not provided) */
		id?: string;
		/** Callback when select loses focus */
		onblur?: () => void;
	}

	let {
		label,
		options,
		value = $bindable(''),
		placeholder = 'Select an option',
		error,
		disabled = false,
		required = false,
		name,
		class: className = '',
		id,
		onblur
	}: Props = $props();

	// Generate a stable unique ID if not provided
	const generatedId = `select-${Math.random().toString(36).substring(2, 9)}`;
	const selectId = $derived(id ?? generatedId);
	const errorId = $derived(`${selectId}-error`);
	const listboxId = $derived(`${selectId}-listbox`);

	// Component state
	let isOpen = $state(false);
	let highlightedIndex = $state(-1);
	let buttonRef: HTMLButtonElement | undefined = $state();
	let listboxRef: HTMLUListElement | undefined = $state();

	// Find the selected option label for display
	const selectedOption = $derived(options.find((opt) => opt.value === value));
	const displayText = $derived(selectedOption?.label ?? placeholder);

	function toggle() {
		if (disabled) return;
		isOpen = !isOpen;
		if (isOpen) {
			// Set highlighted index to currently selected option
			highlightedIndex = options.findIndex((opt) => opt.value === value);
			if (highlightedIndex === -1) highlightedIndex = 0;
		}
	}

	function close() {
		isOpen = false;
		highlightedIndex = -1;
	}

	function handleBlur(event: FocusEvent) {
		// Only call onblur if focus is leaving the select entirely
		const target = event.relatedTarget as Node | null;
		if (target && (buttonRef?.contains(target) || listboxRef?.contains(target))) {
			return;
		}
		onblur?.();
	}

	function selectOption(option: Option) {
		value = option.value;
		close();
		buttonRef?.focus();
	}

	function handleKeydown(event: KeyboardEvent) {
		if (disabled) return;

		switch (event.key) {
			case 'Enter':
			case ' ':
				event.preventDefault();
				if (isOpen && highlightedIndex >= 0) {
					selectOption(options[highlightedIndex]);
				} else {
					toggle();
				}
				break;

			case 'Escape':
				event.preventDefault();
				close();
				buttonRef?.focus();
				break;

			case 'ArrowDown':
				event.preventDefault();
				if (!isOpen) {
					isOpen = true;
					highlightedIndex = options.findIndex((opt) => opt.value === value);
					if (highlightedIndex === -1) highlightedIndex = 0;
				} else {
					highlightedIndex = Math.min(highlightedIndex + 1, options.length - 1);
				}
				scrollToHighlighted();
				break;

			case 'ArrowUp':
				event.preventDefault();
				if (!isOpen) {
					isOpen = true;
					highlightedIndex = options.findIndex((opt) => opt.value === value);
					if (highlightedIndex === -1) highlightedIndex = options.length - 1;
				} else {
					highlightedIndex = Math.max(highlightedIndex - 1, 0);
				}
				scrollToHighlighted();
				break;

			case 'Home':
				event.preventDefault();
				if (isOpen) {
					highlightedIndex = 0;
					scrollToHighlighted();
				}
				break;

			case 'End':
				event.preventDefault();
				if (isOpen) {
					highlightedIndex = options.length - 1;
					scrollToHighlighted();
				}
				break;

			case 'Tab':
				if (isOpen) {
					close();
				}
				break;
		}
	}

	function scrollToHighlighted() {
		// Allow DOM to update first
		setTimeout(() => {
			const highlightedEl = listboxRef?.querySelector('[data-highlighted="true"]');
			highlightedEl?.scrollIntoView({ block: 'nearest' });
		}, 0);
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as Node;
		if (buttonRef && !buttonRef.contains(target) && listboxRef && !listboxRef.contains(target)) {
			close();
		}
	}

	// Close on click outside
	$effect(() => {
		if (isOpen) {
			document.addEventListener('click', handleClickOutside, true);
			return () => {
				document.removeEventListener('click', handleClickOutside, true);
			};
		}
	});
</script>

<div class="select-wrapper {className}" class:has-error={!!error} class:is-disabled={disabled}>
	{#if label}
		<label for={selectId} class="select-label">
			{label}
			{#if required}
				<span class="required-indicator" aria-hidden="true">*</span>
			{/if}
		</label>
	{/if}

	<!-- Hidden native select for form submission -->
	{#if name}
		<select {name} {required} {disabled} tabindex="-1" class="visually-hidden" bind:value>
			<option value="">{placeholder}</option>
			{#each options as option (option.value)}
				<option value={option.value}>{option.label}</option>
			{/each}
		</select>
	{/if}

	<div class="select-container">
		<button
			bind:this={buttonRef}
			type="button"
			id={selectId}
			class="select-trigger"
			class:is-placeholder={!selectedOption}
			aria-haspopup="listbox"
			aria-expanded={isOpen}
			aria-controls={listboxId}
			aria-describedby={error ? errorId : undefined}
			{disabled}
			onclick={toggle}
			onkeydown={handleKeydown}
			onblur={handleBlur}
		>
			<span class="select-value">{displayText}</span>
			<span class="select-icon" aria-hidden="true">
				<svg
					width="12"
					height="12"
					viewBox="0 0 12 12"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path d="M3 4.5L6 7.5L9 4.5" />
				</svg>
			</span>
		</button>

		{#if isOpen}
			<ul
				bind:this={listboxRef}
				id={listboxId}
				class="select-listbox"
				role="listbox"
				aria-labelledby={selectId}
				tabindex="-1"
			>
				{#each options as option, index (option.value)}
					<!-- svelte-ignore a11y_click_events_have_key_events -->
					<li
						role="option"
						class="select-option"
						class:is-selected={option.value === value}
						class:is-highlighted={index === highlightedIndex}
						aria-selected={option.value === value}
						data-highlighted={index === highlightedIndex}
						onclick={() => selectOption(option)}
						onmouseenter={() => (highlightedIndex = index)}
					>
						{option.label}
						{#if option.value === value}
							<span class="check-icon" aria-hidden="true">
								<svg
									width="12"
									height="12"
									viewBox="0 0 12 12"
									fill="none"
									stroke="currentColor"
									stroke-width="2"
									stroke-linecap="round"
									stroke-linejoin="round"
								>
									<path d="M2 6L5 9L10 3" />
								</svg>
							</span>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</div>

	{#if error}
		<p id={errorId} class="select-error" role="alert">
			{error}
		</p>
	{/if}
</div>

<style>
	.select-wrapper {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.select-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.required-indicator {
		color: var(--color-rush, #ef4444);
		margin-left: 0.125rem;
	}

	.select-container {
		position: relative;
	}

	.select-trigger {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		font-family: inherit;
		line-height: 1.5;
		text-align: left;
		color: var(--color-text, #1e293b);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.select-trigger.is-placeholder {
		color: var(--color-text-muted, #64748b);
	}

	.select-trigger:hover:not(:disabled) {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.select-trigger:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.15);
	}

	.select-trigger:disabled {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text-muted, #64748b);
		cursor: not-allowed;
		opacity: 0.7;
	}

	.select-value {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.select-icon {
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted, #64748b);
		transition: transform var(--transition-fast, 150ms ease);
	}

	.select-trigger[aria-expanded='true'] .select-icon {
		transform: rotate(180deg);
	}

	.select-listbox {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		right: 0;
		z-index: 50;
		max-height: 240px;
		overflow-y: auto;
		margin: 0;
		padding: var(--space-xs, 0.25rem);
		list-style: none;
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		box-shadow: var(--shadow-lg, 0 10px 15px -3px rgb(0 0 0 / 0.1));
	}

	.select-option {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.select-option:hover,
	.select-option.is-highlighted {
		background-color: var(--color-bg, #f8fafc);
	}

	.select-option.is-selected {
		font-weight: 500;
		color: var(--color-primary, #1e40af);
	}

	.check-icon {
		flex-shrink: 0;
		display: flex;
		align-items: center;
		color: var(--color-primary, #1e40af);
	}

	/* Error state */
	.has-error .select-trigger {
		border-color: var(--color-rush, #ef4444);
	}

	.has-error .select-trigger:focus {
		border-color: var(--color-rush, #ef4444);
		box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15);
	}

	.select-error {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		line-height: 1.4;
	}

	/* Disabled state */
	.is-disabled .select-label {
		color: var(--color-text-muted, #64748b);
	}

	/* Visually hidden helper (for native select) */
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
</style>

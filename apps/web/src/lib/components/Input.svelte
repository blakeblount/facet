<script lang="ts">
	interface Props {
		/** Label text displayed above the input */
		label?: string;
		/** Placeholder text shown when input is empty */
		placeholder?: string;
		/** Current value of the input (two-way bindable) */
		value?: string;
		/** Error message to display below the input */
		error?: string;
		/** Whether the input is disabled */
		disabled?: boolean;
		/** Whether the input is required */
		required?: boolean;
		/** Input type (text, email, password, etc.) */
		type?: 'text' | 'email' | 'password' | 'tel' | 'url' | 'search' | 'number';
		/** Input name attribute for forms */
		name?: string;
		/** Additional CSS class for the wrapper */
		class?: string;
		/** Unique ID for the input (auto-generated if not provided) */
		id?: string;
	}

	let {
		label,
		placeholder = '',
		value = $bindable(''),
		error,
		disabled = false,
		required = false,
		type = 'text',
		name,
		class: className = '',
		id
	}: Props = $props();

	// Generate a stable unique ID if not provided (only once per component instance)
	const generatedId = `input-${Math.random().toString(36).substring(2, 9)}`;
	const inputId = $derived(id ?? generatedId);
	const errorId = $derived(`${inputId}-error`);
</script>

<div class="input-wrapper {className}" class:has-error={!!error} class:is-disabled={disabled}>
	{#if label}
		<label for={inputId} class="input-label">
			{label}
			{#if required}
				<span class="required-indicator" aria-hidden="true">*</span>
			{/if}
		</label>
	{/if}

	<input
		{type}
		{name}
		id={inputId}
		class="input-field"
		{placeholder}
		{disabled}
		{required}
		bind:value
		aria-invalid={error ? 'true' : undefined}
		aria-describedby={error ? errorId : undefined}
	/>

	{#if error}
		<p id={errorId} class="input-error" role="alert">
			{error}
		</p>
	{/if}
</div>

<style>
	.input-wrapper {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.input-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.required-indicator {
		color: var(--color-rush, #ef4444);
		margin-left: 0.125rem;
	}

	.input-field {
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--color-text, #1e293b);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.input-field::placeholder {
		color: var(--color-text-muted, #64748b);
	}

	.input-field:hover:not(:disabled) {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.input-field:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.15);
	}

	/* Error state */
	.has-error .input-field {
		border-color: var(--color-rush, #ef4444);
	}

	.has-error .input-field:focus {
		border-color: var(--color-rush, #ef4444);
		box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15);
	}

	.input-error {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		line-height: 1.4;
	}

	/* Disabled state */
	.is-disabled .input-label {
		color: var(--color-text-muted, #64748b);
	}

	.input-field:disabled {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text-muted, #64748b);
		cursor: not-allowed;
		opacity: 0.7;
	}
</style>

<script lang="ts">
	interface Props {
		/** Label text displayed above the textarea */
		label?: string;
		/** Placeholder text shown when textarea is empty */
		placeholder?: string;
		/** Current value of the textarea (two-way bindable) */
		value?: string;
		/** Number of visible text rows */
		rows?: number;
		/** Error message to display below the textarea */
		error?: string;
		/** Whether the textarea is disabled */
		disabled?: boolean;
		/** Whether the textarea is required */
		required?: boolean;
		/** Whether the textarea should auto-resize to fit content */
		autoResize?: boolean;
		/** Textarea name attribute for forms */
		name?: string;
		/** Additional CSS class for the wrapper */
		class?: string;
		/** Unique ID for the textarea (auto-generated if not provided) */
		id?: string;
	}

	let {
		label,
		placeholder = '',
		value = $bindable(''),
		rows = 3,
		error,
		disabled = false,
		required = false,
		autoResize = false,
		name,
		class: className = '',
		id
	}: Props = $props();

	// Generate a stable unique ID if not provided (only once per component instance)
	const generatedId = `textarea-${Math.random().toString(36).substring(2, 9)}`;
	const textareaId = $derived(id ?? generatedId);
	const errorId = $derived(`${textareaId}-error`);

	let textareaElement: HTMLTextAreaElement | undefined = $state();

	function handleInput() {
		if (autoResize && textareaElement) {
			// Reset height to auto to get the correct scrollHeight
			textareaElement.style.height = 'auto';
			// Set height to scrollHeight to fit content
			textareaElement.style.height = `${textareaElement.scrollHeight}px`;
		}
	}

	// Auto-resize on initial render if autoResize is enabled and value exists
	$effect(() => {
		if (autoResize && textareaElement && value) {
			textareaElement.style.height = 'auto';
			textareaElement.style.height = `${textareaElement.scrollHeight}px`;
		}
	});
</script>

<div class="textarea-wrapper {className}" class:has-error={!!error} class:is-disabled={disabled}>
	{#if label}
		<label for={textareaId} class="textarea-label">
			{label}
			{#if required}
				<span class="required-indicator" aria-hidden="true">*</span>
			{/if}
		</label>
	{/if}

	<textarea
		{name}
		id={textareaId}
		class="textarea-field"
		class:auto-resize={autoResize}
		{placeholder}
		{disabled}
		{required}
		{rows}
		bind:value
		bind:this={textareaElement}
		oninput={handleInput}
		aria-invalid={error ? 'true' : undefined}
		aria-describedby={error ? errorId : undefined}
	></textarea>

	{#if error}
		<p id={errorId} class="textarea-error" role="alert">
			{error}
		</p>
	{/if}
</div>

<style>
	.textarea-wrapper {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.textarea-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.required-indicator {
		color: var(--color-rush, #ef4444);
		margin-left: 0.125rem;
	}

	.textarea-field {
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		line-height: 1.5;
		font-family: inherit;
		color: var(--color-text, #1e293b);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		resize: vertical;
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.textarea-field.auto-resize {
		resize: none;
		overflow: hidden;
	}

	.textarea-field::placeholder {
		color: var(--color-text-muted, #64748b);
	}

	.textarea-field:hover:not(:disabled) {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.textarea-field:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.15);
	}

	/* Error state */
	.has-error .textarea-field {
		border-color: var(--color-rush, #ef4444);
	}

	.has-error .textarea-field:focus {
		border-color: var(--color-rush, #ef4444);
		box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15);
	}

	.textarea-error {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		line-height: 1.4;
	}

	/* Disabled state */
	.is-disabled .textarea-label {
		color: var(--color-text-muted, #64748b);
	}

	.textarea-field:disabled {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text-muted, #64748b);
		cursor: not-allowed;
		opacity: 0.7;
	}
</style>

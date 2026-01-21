<script lang="ts">
	import { onMount, type Snippet } from 'svelte';

	interface Props {
		/** Whether the modal is open (for standalone usage without store) */
		open?: boolean;
		/** Modal title displayed in header */
		title?: string;
		/** Callback when modal requests close */
		onClose?: () => void;
		/** Whether clicking backdrop closes modal (default: true) */
		closeOnBackdrop?: boolean;
		/** Whether ESC key closes modal (default: true) */
		closeOnEsc?: boolean;
		/** Content to render inside the modal */
		children?: Snippet;
	}

	let {
		open = false,
		title = '',
		onClose,
		closeOnBackdrop = true,
		closeOnEsc = true,
		children
	}: Props = $props();

	let dialogEl: HTMLDialogElement;
	let previousActiveElement: Element | null = null;

	// Track open state and sync with dialog element
	$effect(() => {
		if (!dialogEl) return;

		if (open) {
			// Store the previously focused element
			previousActiveElement = document.activeElement;
			// Show the dialog as a modal (enables backdrop and focus trap)
			dialogEl.showModal();
		} else {
			dialogEl.close();
			// Restore focus to previous element
			if (previousActiveElement instanceof HTMLElement) {
				previousActiveElement.focus();
			}
		}
	});

	function handleBackdropClick(event: MouseEvent) {
		// Only close if clicking the backdrop (dialog element itself, not content)
		if (event.target === dialogEl && closeOnBackdrop) {
			requestClose();
		}
	}

	function handleKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && closeOnEsc) {
			event.preventDefault();
			requestClose();
		}
	}

	function requestClose() {
		onClose?.();
	}

	// Prevent body scroll when modal is open
	$effect(() => {
		if (open) {
			const scrollbarWidth = window.innerWidth - document.documentElement.clientWidth;
			document.body.style.overflow = 'hidden';
			document.body.style.paddingRight = `${scrollbarWidth}px`;
		} else {
			document.body.style.overflow = '';
			document.body.style.paddingRight = '';
		}

		return () => {
			document.body.style.overflow = '';
			document.body.style.paddingRight = '';
		};
	});

	onMount(() => {
		return () => {
			// Cleanup on unmount
			document.body.style.overflow = '';
			document.body.style.paddingRight = '';
		};
	});
</script>

<dialog
	bind:this={dialogEl}
	class="modal-dialog"
	onclick={handleBackdropClick}
	onkeydown={handleKeydown}
	aria-labelledby={title ? 'modal-title' : undefined}
>
	<div class="modal-container" role="document">
		{#if title}
			<header class="modal-header">
				<h2 id="modal-title" class="modal-title">{title}</h2>
				<button
					type="button"
					class="modal-close-button"
					onclick={requestClose}
					aria-label="Close modal"
				>
					<svg
						xmlns="http://www.w3.org/2000/svg"
						width="20"
						height="20"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2"
						stroke-linecap="round"
						stroke-linejoin="round"
					>
						<path d="M18 6 6 18" />
						<path d="m6 6 12 12" />
					</svg>
				</button>
			</header>
		{/if}
		<div class="modal-content">
			{#if children}
				{@render children()}
			{/if}
		</div>
	</div>
</dialog>

<style>
	/* Use native dialog for built-in accessibility (focus trap, backdrop) */
	.modal-dialog {
		position: fixed;
		inset: 0;
		margin: auto;
		padding: 0;
		border: none;
		background: transparent;
		max-width: 90vw;
		max-height: 90vh;
		width: fit-content;
		height: fit-content;
		overflow: visible;
	}

	/* Backdrop styling via ::backdrop pseudo-element */
	.modal-dialog::backdrop {
		background-color: var(--color-modal-backdrop, rgb(0 0 0 / 0.4));
		animation: backdrop-fade-in var(--transition-normal, 200ms ease) forwards;
	}

	@keyframes backdrop-fade-in {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	/* Open state animation */
	.modal-dialog[open] {
		animation: modal-slide-in var(--transition-normal, 200ms ease) forwards;
	}

	@keyframes modal-slide-in {
		from {
			opacity: 0;
			transform: translateY(-10px) scale(0.98);
		}
		to {
			opacity: 1;
			transform: translateY(0) scale(1);
		}
	}

	.modal-container {
		display: flex;
		flex-direction: column;
		background-color: var(--color-surface, #ffffff);
		border-radius: var(--radius-lg, 0.5rem);
		box-shadow: var(--shadow-lg, 0 12px 24px -4px rgb(0 0 0 / 0.08));
		max-height: 90vh;
		overflow: hidden;
	}

	.modal-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md, 1rem);
		padding: var(--space-md, 1rem) var(--space-lg, 1.5rem);
		border-bottom: 1px solid var(--color-border, #e8e6e1);
		flex-shrink: 0;
	}

	.modal-title {
		margin: 0;
		font-size: 1.25rem;
		font-weight: 600;
		color: var(--color-text, #2d2d2d);
		font-family: var(--font-heading, inherit);
	}

	.modal-close-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		padding: 0;
		border: none;
		background: transparent;
		color: var(--color-text-muted, #6b6b6b);
		border-radius: var(--radius-md, 0.375rem);
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.modal-close-button:hover {
		background-color: var(--color-border, #e8e6e1);
		color: var(--color-text, #2d2d2d);
	}

	.modal-close-button:focus-visible {
		outline: 2px solid var(--color-primary, #1e3a5f);
		outline-offset: 2px;
	}

	.modal-content {
		padding: var(--space-lg, 1.5rem);
		overflow-y: auto;
	}

	/* Responsive sizing */
	@media (max-width: 768px) {
		.modal-dialog {
			max-width: 95vw;
		}

		.modal-container {
			max-height: 95vh;
		}
	}
</style>

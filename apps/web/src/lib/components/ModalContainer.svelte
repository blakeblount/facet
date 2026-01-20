<script lang="ts">
	import { modalStore } from '$lib/stores/modal';
	import Modal from './Modal.svelte';

	// Track which modals are animating out
	let closingModals = $state<Set<string>>(new Set());

	function handleClose(id: string) {
		// Add to closing set to trigger animation
		closingModals = new Set([...closingModals, id]);

		// Wait for animation, then actually close
		setTimeout(() => {
			modalStore.close(id);
			closingModals = new Set([...closingModals].filter((mId) => mId !== id));
		}, 200);
	}
</script>

{#each modalStore.stack as modal, index (modal.id)}
	{@const isClosing = closingModals.has(modal.id)}
	{@const zIndex = 1000 + index * 10}
	{@const ModalContent = modal.component}
	<div class="modal-wrapper" style:--modal-z-index={zIndex}>
		<Modal
			open={!isClosing}
			title={modal.title}
			onClose={() => handleClose(modal.id)}
			closeOnBackdrop={modal.closeOnBackdrop}
			closeOnEsc={modal.closeOnEsc}
		>
			<ModalContent {...modal.props} />
		</Modal>
	</div>
{/each}

<style>
	.modal-wrapper {
		/* z-index is set via CSS variable for stacking */
		--modal-z-index: 1000;
	}

	.modal-wrapper :global(.modal-dialog) {
		z-index: var(--modal-z-index);
	}

	.modal-wrapper :global(.modal-dialog::backdrop) {
		z-index: calc(var(--modal-z-index) - 1);
	}
</style>

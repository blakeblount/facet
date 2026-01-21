<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Button from './Button.svelte';
	import { adminAuthStore } from '$lib/stores/adminAuth.svelte';

	interface Props {
		/** Whether the modal is open */
		open?: boolean;
		/** Callback when modal requests close */
		onClose?: () => void;
		/** Callback when PIN is successfully verified */
		onSuccess?: () => void;
	}

	let { open = false, onClose, onSuccess }: Props = $props();

	let pin = $state('');
	let formEl: HTMLFormElement | undefined = $state();

	// Focus input when modal opens
	$effect(() => {
		if (open && formEl) {
			// Small delay to allow modal animation to start
			setTimeout(() => {
				const input = formEl?.querySelector('input[type="password"]') as HTMLInputElement | null;
				input?.focus();
			}, 50);
		}
	});

	// Reset state when modal opens
	$effect(() => {
		if (open) {
			pin = '';
			adminAuthStore.clearError();
		}
	});

	async function handleSubmit() {
		if (!pin.trim()) {
			return;
		}

		const success = await adminAuthStore.verify(pin);
		if (success) {
			onSuccess?.();
		}
	}

	function handleClose() {
		if (!adminAuthStore.isVerifying) {
			onClose?.();
		}
	}
</script>

<Modal {open} title="Admin Authentication" onClose={handleClose} closeOnBackdrop={!adminAuthStore.isVerifying} closeOnEsc={!adminAuthStore.isVerifying}>
	<form
		bind:this={formEl}
		class="admin-pin-form"
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
	>
		<p class="form-description">
			Enter the admin PIN to access employee management and other admin settings.
		</p>

		<div class="form-content">
			<Input
				label="Admin PIN"
				type="password"
				placeholder="Enter admin PIN"
				bind:value={pin}
				error={adminAuthStore.error ?? undefined}
				disabled={adminAuthStore.isVerifying}
				required
			/>
		</div>

		<div class="form-actions">
			<Button variant="secondary" onclick={handleClose} disabled={adminAuthStore.isVerifying}>Cancel</Button>
			<Button variant="primary" type="submit" loading={adminAuthStore.isVerifying} disabled={adminAuthStore.isVerifying || !pin.trim()}>
				Unlock
			</Button>
		</div>
	</form>
</Modal>

<style>
	.admin-pin-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg, 1.5rem);
		min-width: 320px;
	}

	.form-description {
		margin: 0;
		color: var(--color-text-muted);
		font-size: 0.875rem;
		line-height: 1.5;
	}

	.form-content {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
	}
</style>

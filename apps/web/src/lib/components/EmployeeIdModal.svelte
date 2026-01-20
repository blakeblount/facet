<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Button from './Button.svelte';
	import { verifyEmployeePin, ApiClientError, type VerifyPinResponse } from '$lib/services/api';

	interface Props {
		/** Whether the modal is open */
		open?: boolean;
		/** Modal title (default: "Enter Employee PIN") */
		title?: string;
		/** Callback when modal requests close */
		onClose?: () => void;
		/** Callback when PIN is successfully verified, returns the employee data */
		onSuccess?: (employee: VerifyPinResponse) => void;
	}

	let { open = false, title = 'Enter Employee PIN', onClose, onSuccess }: Props = $props();

	let pin = $state('');
	let error = $state('');
	let isLoading = $state(false);
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

	// Reset state when modal opens/closes
	$effect(() => {
		if (open) {
			pin = '';
			error = '';
			isLoading = false;
		}
	});

	async function handleSubmit() {
		if (!pin.trim()) {
			error = 'Please enter your PIN';
			return;
		}

		isLoading = true;
		error = '';

		try {
			const employee = await verifyEmployeePin(pin);
			onSuccess?.(employee);
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isCode('INVALID_PIN')) {
					error = 'Invalid PIN. Please try again.';
				} else {
					error = err.message || 'An error occurred. Please try again.';
				}
			} else {
				error = 'An error occurred. Please try again.';
			}
		} finally {
			isLoading = false;
		}
	}

	function handleClose() {
		if (!isLoading) {
			onClose?.();
		}
	}
</script>

<Modal {open} {title} onClose={handleClose} closeOnBackdrop={!isLoading} closeOnEsc={!isLoading}>
	<form
		bind:this={formEl}
		class="employee-id-form"
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
	>
		<div class="form-content">
			<Input
				label="Employee PIN"
				type="password"
				placeholder="Enter your PIN"
				bind:value={pin}
				{error}
				disabled={isLoading}
				required
			/>
		</div>

		<div class="form-actions">
			<Button variant="secondary" onclick={handleClose} disabled={isLoading}>Cancel</Button>
			<Button variant="primary" type="submit" loading={isLoading} disabled={isLoading}>
				Verify
			</Button>
		</div>
	</form>
</Modal>

<style>
	.employee-id-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg, 1.5rem);
		min-width: 300px;
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

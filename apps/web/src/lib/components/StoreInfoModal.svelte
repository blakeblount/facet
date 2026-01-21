<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Button from './Button.svelte';
	import { updateStoreSettings, ApiClientError } from '$lib/services/api';
	import type { StoreSettings } from '$lib/types/api';

	interface Props {
		/** Whether the modal is open */
		open?: boolean;
		/** Current settings to edit */
		settings?: StoreSettings | null;
		/** Callback when modal requests close */
		onClose?: () => void;
		/** Callback when settings are successfully saved */
		onSuccess?: (settings: StoreSettings) => void;
	}

	let { open = false, settings = null, onClose, onSuccess }: Props = $props();

	let storeName = $state('');
	let storePhone = $state('');
	let storeAddress = $state('');
	let error = $state('');
	let nameError = $state('');
	let phoneError = $state('');
	let addressError = $state('');
	let isLoading = $state(false);
	let formEl: HTMLFormElement | undefined = $state();

	// Reset state when modal opens
	$effect(() => {
		if (open) {
			if (settings) {
				storeName = settings.store_name || '';
				storePhone = settings.store_phone || '';
				storeAddress = settings.store_address || '';
			} else {
				storeName = '';
				storePhone = '';
				storeAddress = '';
			}
			error = '';
			nameError = '';
			phoneError = '';
			addressError = '';
			isLoading = false;
		}
	});

	// Focus input when modal opens
	$effect(() => {
		if (open && formEl) {
			setTimeout(() => {
				const input = formEl?.querySelector('input[type="text"]') as HTMLInputElement | null;
				input?.focus();
			}, 50);
		}
	});

	function validateForm(): boolean {
		nameError = '';
		phoneError = '';
		addressError = '';

		let isValid = true;

		if (!storeName.trim()) {
			nameError = 'Store name is required';
			isValid = false;
		} else if (storeName.trim().length > 255) {
			nameError = 'Store name must be 255 characters or less';
			isValid = false;
		}

		if (storePhone.trim().length > 50) {
			phoneError = 'Phone must be 50 characters or less';
			isValid = false;
		}

		if (storeAddress.trim().length > 255) {
			addressError = 'Address must be 255 characters or less';
			isValid = false;
		}

		return isValid;
	}

	async function handleSubmit() {
		if (!validateForm()) {
			return;
		}

		isLoading = true;
		error = '';

		try {
			const result = await updateStoreSettings({
				store_name: storeName.trim(),
				store_phone: storePhone.trim() || undefined,
				store_address: storeAddress.trim() || undefined
			});

			onSuccess?.(result);
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isValidationError()) {
					error = err.message || 'Please check your input and try again.';
				} else if (err.isCode('UNAUTHORIZED')) {
					error = 'Session expired. Please re-authenticate.';
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

<Modal
	{open}
	title="Edit Store Information"
	onClose={handleClose}
	closeOnBackdrop={!isLoading}
	closeOnEsc={!isLoading}
>
	<form
		bind:this={formEl}
		class="store-info-form"
		onsubmit={(e) => {
			e.preventDefault();
			handleSubmit();
		}}
	>
		{#if error}
			<div class="form-error" role="alert">
				{error}
			</div>
		{/if}

		<div class="form-content">
			<Input
				label="Store Name"
				placeholder="Your store name"
				bind:value={storeName}
				error={nameError}
				disabled={isLoading}
				required
			/>

			<Input
				label="Phone"
				type="tel"
				placeholder="(555) 123-4567"
				bind:value={storePhone}
				error={phoneError}
				disabled={isLoading}
			/>

			<Input
				label="Address"
				placeholder="123 Main St, City, State 12345"
				bind:value={storeAddress}
				error={addressError}
				disabled={isLoading}
			/>
		</div>

		<div class="form-actions">
			<Button variant="secondary" onclick={handleClose} disabled={isLoading}>Cancel</Button>
			<Button variant="primary" type="submit" loading={isLoading} disabled={isLoading}>
				Save Changes
			</Button>
		</div>
	</form>
</Modal>

<style>
	.store-info-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg, 1.5rem);
		min-width: 400px;
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

	.form-error {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: var(--radius-md, 0.5rem);
		color: #991b1b;
		font-size: 0.875rem;
	}
</style>

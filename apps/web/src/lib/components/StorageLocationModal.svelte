<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Button from './Button.svelte';
	import { createStorageLocation, updateStorageLocation, ApiClientError } from '$lib/services/api';
	import type { StorageLocationSummary } from '$lib/types/api';

	interface Props {
		/** Whether the modal is open */
		open?: boolean;
		/** Location to edit (null for create mode) */
		location?: StorageLocationSummary | null;
		/** Callback when modal requests close */
		onClose?: () => void;
		/** Callback when location is successfully saved */
		onSuccess?: (location: StorageLocationSummary) => void;
	}

	let { open = false, location = null, onClose, onSuccess }: Props = $props();

	let name = $state('');
	let isActive = $state(true);
	let error = $state('');
	let nameError = $state('');
	let isLoading = $state(false);
	let formEl: HTMLFormElement | undefined = $state();

	const isEditMode = $derived(location !== null);
	const title = $derived(isEditMode ? 'Edit Storage Location' : 'Add Storage Location');

	// Reset state when modal opens
	$effect(() => {
		if (open) {
			if (location) {
				name = location.name;
				isActive = location.is_active;
			} else {
				name = '';
				isActive = true;
			}
			error = '';
			nameError = '';
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

		if (!name.trim()) {
			nameError = 'Name is required';
			return false;
		}

		if (name.trim().length > 255) {
			nameError = 'Name must be 255 characters or less';
			return false;
		}

		return true;
	}

	async function handleSubmit() {
		if (!validateForm()) {
			return;
		}

		isLoading = true;
		error = '';

		try {
			let result: StorageLocationSummary;

			if (isEditMode && location) {
				result = await updateStorageLocation(location.location_id, {
					name: name.trim(),
					is_active: isActive
				});
			} else {
				result = await createStorageLocation({
					name: name.trim()
				});
			}

			onSuccess?.(result);
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isCode('CONFLICT')) {
					nameError = 'A location with this name already exists';
				} else if (err.isValidationError()) {
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

<Modal {open} {title} onClose={handleClose} closeOnBackdrop={!isLoading} closeOnEsc={!isLoading}>
	<form
		bind:this={formEl}
		class="location-form"
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
				label="Location Name"
				placeholder="e.g., Bin A, Safe Drawer 1"
				bind:value={name}
				error={nameError}
				disabled={isLoading}
				required
			/>

			{#if isEditMode}
				<div class="checkbox-field">
					<label class="checkbox-label">
						<input
							type="checkbox"
							bind:checked={isActive}
							disabled={isLoading}
							class="checkbox-input"
						/>
						<span class="checkbox-text">Active</span>
					</label>
					<p class="checkbox-hint">Inactive locations won't appear in ticket forms</p>
				</div>
			{/if}
		</div>

		<div class="form-actions">
			<Button variant="secondary" onclick={handleClose} disabled={isLoading}>Cancel</Button>
			<Button variant="primary" type="submit" loading={isLoading} disabled={isLoading}>
				{isEditMode ? 'Save Changes' : 'Add Location'}
			</Button>
		</div>
	</form>
</Modal>

<style>
	.location-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg, 1.5rem);
		min-width: 350px;
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

	.checkbox-field {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.checkbox-label {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
		cursor: pointer;
	}

	.checkbox-input {
		width: 16px;
		height: 16px;
		accent-color: var(--color-primary, #1e40af);
		cursor: pointer;
	}

	.checkbox-input:disabled {
		cursor: not-allowed;
	}

	.checkbox-text {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.checkbox-hint {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
		padding-left: calc(16px + var(--space-sm, 0.5rem));
	}
</style>

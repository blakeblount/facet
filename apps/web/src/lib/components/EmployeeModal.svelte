<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Select from './Select.svelte';
	import Button from './Button.svelte';
	import { createEmployee, updateEmployee, ApiClientError } from '$lib/services/api';
	import type { EmployeeSummary, EmployeeRole } from '$lib/types/api';

	interface Props {
		/** Whether the modal is open */
		open?: boolean;
		/** Employee to edit (null for create mode) */
		employee?: EmployeeSummary | null;
		/** Callback when modal requests close */
		onClose?: () => void;
		/** Callback when employee is successfully saved */
		onSuccess?: (employee: EmployeeSummary) => void;
	}

	let { open = false, employee = null, onClose, onSuccess }: Props = $props();

	let name = $state('');
	let pin = $state('');
	let confirmPin = $state('');
	let role = $state<EmployeeRole>('staff');
	let isActive = $state(true);
	let error = $state('');
	let nameError = $state('');
	let pinError = $state('');
	let confirmPinError = $state('');
	let isLoading = $state(false);
	let formEl: HTMLFormElement | undefined = $state();

	const isEditMode = $derived(employee !== null);
	const title = $derived(isEditMode ? 'Edit Employee' : 'Add Employee');

	const roleOptions = [
		{ value: 'staff', label: 'Staff' },
		{ value: 'admin', label: 'Admin' }
	];

	// Reset state when modal opens
	$effect(() => {
		if (open) {
			if (employee) {
				name = employee.name;
				role = employee.role;
				isActive = employee.is_active;
				pin = '';
				confirmPin = '';
			} else {
				name = '';
				pin = '';
				confirmPin = '';
				role = 'staff';
				isActive = true;
			}
			error = '';
			nameError = '';
			pinError = '';
			confirmPinError = '';
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
		pinError = '';
		confirmPinError = '';

		let isValid = true;

		if (!name.trim()) {
			nameError = 'Name is required';
			isValid = false;
		} else if (name.trim().length > 255) {
			nameError = 'Name must be 255 characters or less';
			isValid = false;
		}

		// PIN is required for new employees, optional for edits
		if (!isEditMode) {
			if (!pin) {
				pinError = 'PIN is required';
				isValid = false;
			} else if (pin.length < 4) {
				pinError = 'PIN must be at least 4 characters';
				isValid = false;
			}
		} else if (pin) {
			// If editing and PIN is provided, validate it
			if (pin.length < 4) {
				pinError = 'PIN must be at least 4 characters';
				isValid = false;
			}
		}

		// Confirm PIN must match if PIN is provided
		if (pin && pin !== confirmPin) {
			confirmPinError = 'PINs do not match';
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
			let result: EmployeeSummary;

			if (isEditMode && employee) {
				const updateRequest: {
					name?: string;
					pin?: string;
					role?: EmployeeRole;
					is_active?: boolean;
				} = {
					name: name.trim(),
					role,
					is_active: isActive
				};
				// Only include PIN if it was changed
				if (pin) {
					updateRequest.pin = pin;
				}
				result = await updateEmployee(employee.employee_id, updateRequest);
			} else {
				result = await createEmployee({
					name: name.trim(),
					pin,
					role
				});
			}

			onSuccess?.(result);
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isCode('CONFLICT')) {
					error = 'An employee with this name already exists';
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
		class="employee-form"
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
				label="Name"
				placeholder="Employee name"
				bind:value={name}
				error={nameError}
				disabled={isLoading}
				required
			/>

			<Input
				label={isEditMode ? 'New PIN (leave blank to keep current)' : 'PIN'}
				type="password"
				placeholder={isEditMode ? 'Enter new PIN to change' : 'Enter PIN'}
				bind:value={pin}
				error={pinError}
				disabled={isLoading}
				required={!isEditMode}
			/>

			{#if pin}
				<Input
					label="Confirm PIN"
					type="password"
					placeholder="Re-enter PIN"
					bind:value={confirmPin}
					error={confirmPinError}
					disabled={isLoading}
					required
				/>
			{/if}

			<Select label="Role" options={roleOptions} bind:value={role} disabled={isLoading} />

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
					<p class="checkbox-hint">Inactive employees cannot log in</p>
				</div>
			{/if}
		</div>

		<div class="form-actions">
			<Button variant="secondary" onclick={handleClose} disabled={isLoading}>Cancel</Button>
			<Button variant="primary" type="submit" loading={isLoading} disabled={isLoading}>
				{isEditMode ? 'Save Changes' : 'Add Employee'}
			</Button>
		</div>
	</form>
</Modal>

<style>
	.employee-form {
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

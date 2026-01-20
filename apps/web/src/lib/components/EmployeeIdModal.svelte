<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Button from './Button.svelte';
	import { verifyEmployeePin, ApiClientError, type VerifyPinResponse } from '$lib/services/api';
	import { employeeCacheStore } from '$lib/services/employeeCache.svelte';
	import { offlineStore } from '$lib/stores/offline.svelte';

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

		// If offline, go straight to cached verification
		if (offlineStore.isOffline) {
			await handleOfflineVerification();
			return;
		}

		// Try online verification first
		try {
			const employee = await verifyEmployeePin(pin);
			// Cache credentials for future offline use
			await employeeCacheStore.cache(employee, pin);
			onSuccess?.(employee);
		} catch (err) {
			// Check if this is a network error - if so, try offline verification
			if (isNetworkError(err)) {
				await handleOfflineVerification();
				return;
			}

			if (err instanceof ApiClientError) {
				if (err.isCode('INVALID_PIN')) {
					error = 'Invalid PIN. Please try again.';
				} else {
					error = err.message || 'An error occurred. Please try again.';
				}
			} else {
				error = 'An error occurred. Please try again.';
			}
			isLoading = false;
		}
	}

	/**
	 * Check if an error is a network error (fetch failed, no connection, etc.)
	 */
	function isNetworkError(err: unknown): boolean {
		if (err instanceof TypeError && err.message.includes('fetch')) {
			return true;
		}
		if (err instanceof ApiClientError && err.status === 0) {
			return true;
		}
		return false;
	}

	/**
	 * Verify PIN against cached credentials when offline
	 */
	async function handleOfflineVerification() {
		try {
			const employee = await employeeCacheStore.verifyOffline(pin);
			if (employee) {
				onSuccess?.(employee);
			} else {
				// No matching cached credentials
				if (employeeCacheStore.hasCachedCredentials) {
					error = 'Invalid PIN. Please try again.';
				} else {
					error = 'You must verify your PIN online at least once before using offline mode.';
				}
			}
		} catch {
			error = 'Failed to verify PIN offline. Please try again.';
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
		<!-- Offline indicator -->
		{#if offlineStore.isOffline}
			<div class="offline-notice" role="status">
				<svg
					class="offline-icon"
					xmlns="http://www.w3.org/2000/svg"
					width="16"
					height="16"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<line x1="2" x2="22" y1="2" y2="22" />
					<path d="M8.5 16.5a5 5 0 0 1 7 0" />
					<path d="M2 8.82a15 15 0 0 1 4.17-2.65" />
					<path d="M10.66 5c4.01-.36 8.14.9 11.34 3.76" />
					<path d="M16.85 11.25a10 10 0 0 1 2.22 1.68" />
					<path d="M5 13a10 10 0 0 1 5.24-2.76" />
					<line x1="12" x2="12.01" y1="20" y2="20" />
				</svg>
				<span class="offline-text">Verifying offline</span>
			</div>
		{/if}

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

	.offline-notice {
		display: flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		background-color: rgba(251, 191, 36, 0.1);
		border: 1px solid #fbbf24;
		border-radius: var(--radius-md, 0.5rem);
		color: #92400e;
		font-size: 0.75rem;
	}

	.offline-icon {
		flex-shrink: 0;
	}

	.offline-text {
		font-weight: 500;
	}
</style>

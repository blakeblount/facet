<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Button from './Button.svelte';
	import { verifyEmployeePin, ApiClientError, type VerifyPinResponse } from '$lib/services/api';

	type SettingsTab = 'store-info' | 'employees' | 'locations' | 'appearance';

	interface Props {
		/** Whether the modal is open */
		open?: boolean;
		/** Callback when modal requests close */
		onClose?: () => void;
	}

	let { open = false, onClose }: Props = $props();

	// Authentication state
	let isAuthenticated = $state(false);
	let authenticatedEmployee: VerifyPinResponse | null = $state(null);
	let pin = $state('');
	let pinError = $state('');
	let isVerifying = $state(false);
	let formEl: HTMLFormElement | undefined = $state();

	// Tab state
	let activeTab: SettingsTab = $state('store-info');

	const tabs: { id: SettingsTab; label: string }[] = [
		{ id: 'store-info', label: 'Store Info' },
		{ id: 'employees', label: 'Employees' },
		{ id: 'locations', label: 'Locations' },
		{ id: 'appearance', label: 'Appearance' }
	];

	// Focus input when modal opens
	$effect(() => {
		if (open && !isAuthenticated && formEl) {
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
			// Reset to PIN entry state when opening
			isAuthenticated = false;
			authenticatedEmployee = null;
			pin = '';
			pinError = '';
			isVerifying = false;
			activeTab = 'store-info';
		}
	});

	async function handlePinSubmit() {
		if (!pin.trim()) {
			pinError = 'Please enter your PIN';
			return;
		}

		isVerifying = true;
		pinError = '';

		try {
			const employee = await verifyEmployeePin(pin);

			// Check if employee is an admin
			if (employee.role !== 'admin') {
				pinError = 'Admin access required. Please use an admin PIN.';
				return;
			}

			// Authentication successful
			authenticatedEmployee = employee;
			isAuthenticated = true;
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isCode('INVALID_PIN')) {
					pinError = 'Invalid PIN. Please try again.';
				} else {
					pinError = err.message || 'An error occurred. Please try again.';
				}
			} else {
				pinError = 'An error occurred. Please try again.';
			}
		} finally {
			isVerifying = false;
		}
	}

	function handleClose() {
		if (!isVerifying) {
			onClose?.();
		}
	}

	function handleTabClick(tabId: SettingsTab) {
		activeTab = tabId;
	}

	function handleTabKeydown(event: KeyboardEvent, tabId: SettingsTab) {
		const currentIndex = tabs.findIndex((t) => t.id === tabId);

		if (event.key === 'ArrowRight' || event.key === 'ArrowDown') {
			event.preventDefault();
			const nextIndex = (currentIndex + 1) % tabs.length;
			activeTab = tabs[nextIndex].id;
			focusTab(tabs[nextIndex].id);
		} else if (event.key === 'ArrowLeft' || event.key === 'ArrowUp') {
			event.preventDefault();
			const prevIndex = (currentIndex - 1 + tabs.length) % tabs.length;
			activeTab = tabs[prevIndex].id;
			focusTab(tabs[prevIndex].id);
		} else if (event.key === 'Home') {
			event.preventDefault();
			activeTab = tabs[0].id;
			focusTab(tabs[0].id);
		} else if (event.key === 'End') {
			event.preventDefault();
			activeTab = tabs[tabs.length - 1].id;
			focusTab(tabs[tabs.length - 1].id);
		}
	}

	function focusTab(tabId: SettingsTab) {
		const tabEl = document.querySelector(`[data-tab-id="${tabId}"]`) as HTMLElement | null;
		tabEl?.focus();
	}
</script>

<Modal
	{open}
	title="Settings"
	onClose={handleClose}
	closeOnBackdrop={!isVerifying}
	closeOnEsc={!isVerifying}
>
	<div class="admin-settings-modal">
		{#if !isAuthenticated}
			<!-- PIN Entry -->
			<form
				bind:this={formEl}
				class="pin-form"
				onsubmit={(e) => {
					e.preventDefault();
					handlePinSubmit();
				}}
			>
				<div class="pin-form-content">
					<p class="pin-description">Enter your admin PIN to access settings.</p>
					<Input
						label="Admin PIN"
						type="password"
						placeholder="Enter your PIN"
						bind:value={pin}
						error={pinError}
						disabled={isVerifying}
						required
					/>
				</div>

				<div class="pin-form-actions">
					<Button variant="secondary" onclick={handleClose} disabled={isVerifying}>Cancel</Button>
					<Button variant="primary" type="submit" loading={isVerifying} disabled={isVerifying}>
						Verify
					</Button>
				</div>
			</form>
		{:else}
			<!-- Settings Tabs -->
			<div class="settings-container">
				<!-- Tab List -->
				<div class="tab-list" role="tablist" aria-label="Settings sections">
					{#each tabs as tab (tab.id)}
						<button
							type="button"
							role="tab"
							data-tab-id={tab.id}
							class="tab-button"
							class:active={activeTab === tab.id}
							aria-selected={activeTab === tab.id}
							aria-controls="panel-{tab.id}"
							tabindex={activeTab === tab.id ? 0 : -1}
							onclick={() => handleTabClick(tab.id)}
							onkeydown={(e) => handleTabKeydown(e, tab.id)}
						>
							{tab.label}
						</button>
					{/each}
				</div>

				<!-- Tab Panels -->
				<div class="tab-panels">
					<div
						id="panel-store-info"
						role="tabpanel"
						aria-labelledby="tab-store-info"
						class="tab-panel"
						class:active={activeTab === 'store-info'}
						hidden={activeTab !== 'store-info'}
					>
						<div class="panel-placeholder">
							<h3 class="placeholder-title">Store Information</h3>
							<p class="placeholder-text">
								Store name, address, and contact details will be configurable here.
							</p>
						</div>
					</div>

					<div
						id="panel-employees"
						role="tabpanel"
						aria-labelledby="tab-employees"
						class="tab-panel"
						class:active={activeTab === 'employees'}
						hidden={activeTab !== 'employees'}
					>
						<div class="panel-placeholder">
							<h3 class="placeholder-title">Employee Management</h3>
							<p class="placeholder-text">
								Add, edit, and manage employee accounts and permissions here.
							</p>
						</div>
					</div>

					<div
						id="panel-locations"
						role="tabpanel"
						aria-labelledby="tab-locations"
						class="tab-panel"
						class:active={activeTab === 'locations'}
						hidden={activeTab !== 'locations'}
					>
						<div class="panel-placeholder">
							<h3 class="placeholder-title">Storage Locations</h3>
							<p class="placeholder-text">
								Configure storage bins, safes, and other item locations here.
							</p>
						</div>
					</div>

					<div
						id="panel-appearance"
						role="tabpanel"
						aria-labelledby="tab-appearance"
						class="tab-panel"
						class:active={activeTab === 'appearance'}
						hidden={activeTab !== 'appearance'}
					>
						<div class="panel-placeholder">
							<h3 class="placeholder-title">Appearance</h3>
							<p class="placeholder-text">
								Customize colors, logos, and receipt printing options here.
							</p>
						</div>
					</div>
				</div>

				<!-- Footer with close button -->
				<div class="settings-footer">
					<span class="authenticated-user">
						Logged in as: <strong>{authenticatedEmployee?.name}</strong>
					</span>
					<Button variant="secondary" onclick={handleClose}>Close</Button>
				</div>
			</div>
		{/if}
	</div>
</Modal>

<style>
	.admin-settings-modal {
		min-width: 500px;
		max-width: 90vw;
	}

	/* PIN Form */
	.pin-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg, 1.5rem);
		min-width: 300px;
	}

	.pin-form-content {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	.pin-description {
		margin: 0;
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
		line-height: 1.5;
	}

	.pin-form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
	}

	/* Settings Container */
	.settings-container {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	/* Tab List */
	.tab-list {
		display: flex;
		gap: var(--space-xs, 0.25rem);
		border-bottom: 1px solid var(--color-border, #e2e8f0);
		padding-bottom: var(--space-xs, 0.25rem);
	}

	.tab-button {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text-muted, #64748b);
		background: none;
		border: none;
		border-bottom: 2px solid transparent;
		border-radius: var(--radius-sm, 0.25rem) var(--radius-sm, 0.25rem) 0 0;
		cursor: pointer;
		transition:
			color var(--transition-fast, 150ms ease),
			border-color var(--transition-fast, 150ms ease),
			background-color var(--transition-fast, 150ms ease);
	}

	.tab-button:hover {
		color: var(--color-text, #1e293b);
		background-color: var(--color-bg, #f8fafc);
	}

	.tab-button:focus-visible {
		outline: 2px solid var(--color-primary, #1e40af);
		outline-offset: -2px;
	}

	.tab-button.active {
		color: var(--color-primary, #1e40af);
		border-bottom-color: var(--color-primary, #1e40af);
	}

	/* Tab Panels */
	.tab-panels {
		min-height: 300px;
	}

	.tab-panel {
		display: none;
		animation: fadeIn var(--transition-fast, 150ms ease) forwards;
	}

	.tab-panel.active {
		display: block;
	}

	@keyframes fadeIn {
		from {
			opacity: 0;
		}
		to {
			opacity: 1;
		}
	}

	/* Placeholder content */
	.panel-placeholder {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl, 2rem);
		text-align: center;
		color: var(--color-text-muted, #64748b);
	}

	.placeholder-title {
		margin: 0 0 var(--space-sm, 0.5rem);
		font-size: 1rem;
		font-weight: 600;
		color: var(--color-text, #1e293b);
	}

	.placeholder-text {
		margin: 0;
		font-size: 0.875rem;
		max-width: 300px;
		line-height: 1.5;
	}

	/* Footer */
	.settings-footer {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-top: var(--space-md, 1rem);
		border-top: 1px solid var(--color-border, #e2e8f0);
	}

	.authenticated-user {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	.authenticated-user strong {
		color: var(--color-text, #1e293b);
	}

	/* Responsive */
	@media (max-width: 600px) {
		.admin-settings-modal {
			min-width: auto;
			width: 100%;
		}

		.tab-list {
			flex-wrap: wrap;
		}

		.tab-button {
			flex: 1 0 auto;
			text-align: center;
		}
	}
</style>

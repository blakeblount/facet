<script lang="ts">
	import { SvelteMap } from 'svelte/reactivity';
	import Modal from './Modal.svelte';
	import Button from './Button.svelte';
	import Input from './Input.svelte';
	import Select from './Select.svelte';
	import Textarea from './Textarea.svelte';
	import PhotoUpload from './PhotoUpload.svelte';
	import EmployeeIdModal from './EmployeeIdModal.svelte';
	import {
		listStorageLocations,
		listCustomers,
		createTicket,
		uploadTicketPhoto,
		setCurrentEmployee,
		getReceiptPdfUrl,
		ApiClientError,
		type StorageLocationSummary,
		type CreateTicketRequest,
		type InlineCustomer,
		type Customer
	} from '$lib/services/api';
	import type { VerifyPinResponse, EmployeeInfo } from '$lib/types/api';
	import { offlineStore } from '$lib/stores/offline.svelte';
	import { syncQueueStore } from '$lib/services/syncQueue.svelte';

	interface Props {
		/** Whether the modal is open */
		open: boolean;
		/** Callback when modal closes */
		onClose: () => void;
		/** Callback when form is successfully submitted */
		onSuccess?: (ticketId: string, friendlyCode: string) => void;
	}

	let { open, onClose, onSuccess }: Props = $props();

	// Form state - Customer section
	let customerName = $state('');
	let customerPhone = $state('');
	let customerEmail = $state('');

	// Customer search autocomplete state
	let selectedCustomerId = $state<string | null>(null);
	let customerSearchResults = $state<Customer[]>([]);
	let isSearchingCustomers = $state(false);
	let showCustomerDropdown = $state(false);
	let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

	// Form state - Item section
	let itemType = $state('');
	let itemDescription = $state('');
	let conditionNotes = $state('');
	let requestedWork = $state('');

	// Form state - Details section
	let isRush = $state(false);
	let promiseDate = $state('');
	let storageLocationId = $state('');
	let quoteAmount = $state('');

	// Form state - Photos section
	let photos: File[] = $state([]);

	// Validation errors
	let errors = $state<Record<string, string>>({});

	// Storage locations
	let storageLocations = $state<StorageLocationSummary[]>([]);
	let isLoadingLocations = $state(false);

	// Form submission state
	let isSubmitting = $state(false);
	let showEmployeeModal = $state(false);
	let submissionError = $state('');
	let photoUploadProgress = new SvelteMap<number, number>();
	let successMessage = $state('');

	// Fetch storage locations when modal opens
	$effect(() => {
		if (open) {
			loadStorageLocations();
		}
	});

	// Reset form when modal closes
	$effect(() => {
		if (!open) {
			resetForm();
		}
	});

	async function loadStorageLocations() {
		isLoadingLocations = true;
		try {
			const response = await listStorageLocations(false);
			storageLocations = response.locations;
		} catch (e) {
			console.error('Failed to load storage locations:', e);
		} finally {
			isLoadingLocations = false;
		}
	}

	// Customer search with debounce
	function handleCustomerNameInput(e: Event) {
		const target = e.target as HTMLInputElement;
		customerName = target.value;

		// Clear selected customer when typing
		if (selectedCustomerId) {
			selectedCustomerId = null;
		}

		// Clear previous timer
		if (searchDebounceTimer) {
			clearTimeout(searchDebounceTimer);
		}

		// Don't search if input is empty or too short
		if (customerName.trim().length < 2) {
			customerSearchResults = [];
			showCustomerDropdown = false;
			return;
		}

		// Debounce the search
		searchDebounceTimer = setTimeout(() => {
			searchCustomers(customerName.trim());
		}, 300);
	}

	async function searchCustomers(query: string) {
		isSearchingCustomers = true;
		showCustomerDropdown = true;
		try {
			customerSearchResults = await listCustomers(query);
		} catch (e) {
			console.error('Failed to search customers:', e);
			customerSearchResults = [];
		} finally {
			isSearchingCustomers = false;
		}
	}

	function selectCustomer(customer: Customer) {
		selectedCustomerId = customer.customer_id;
		customerName = customer.name;
		customerPhone = customer.phone ?? '';
		customerEmail = customer.email ?? '';
		customerSearchResults = [];
		showCustomerDropdown = false;
	}

	function clearCustomerSelection() {
		selectedCustomerId = null;
		customerName = '';
		customerPhone = '';
		customerEmail = '';
		customerSearchResults = [];
		showCustomerDropdown = false;
	}

	function handleCustomerInputFocus() {
		// Show dropdown if we have results and no customer selected
		if (customerSearchResults.length > 0 && !selectedCustomerId) {
			showCustomerDropdown = true;
		}
	}

	function handleCustomerInputBlur() {
		// Delay hiding to allow click on dropdown items
		setTimeout(() => {
			showCustomerDropdown = false;
		}, 200);
		// Clear error if field is now valid
		clearFieldErrorOnBlur('customerName', !!customerName.trim());
	}

	// Clear individual field error when the field becomes valid on blur
	function clearFieldErrorOnBlur(fieldName: string, isValid: boolean) {
		if (isValid && errors[fieldName]) {
			errors = Object.fromEntries(Object.entries(errors).filter(([key]) => key !== fieldName));
		}
	}

	function resetForm() {
		// Reset customer fields
		customerName = '';
		customerPhone = '';
		customerEmail = '';
		selectedCustomerId = null;
		customerSearchResults = [];
		showCustomerDropdown = false;
		if (searchDebounceTimer) {
			clearTimeout(searchDebounceTimer);
			searchDebounceTimer = null;
		}

		// Reset item fields
		itemType = '';
		itemDescription = '';
		conditionNotes = '';
		requestedWork = '';

		// Reset details fields
		isRush = false;
		promiseDate = '';
		storageLocationId = '';
		quoteAmount = '';

		// Reset photos
		photos = [];

		// Reset errors
		errors = {};

		// Reset submission state
		isSubmitting = false;
		showEmployeeModal = false;
		submissionError = '';
		photoUploadProgress.clear();
		successMessage = '';
	}

	function validateForm(): boolean {
		const newErrors: Record<string, string> = {};

		// Customer validation
		if (!customerName.trim()) {
			newErrors.customerName = 'Customer name is required';
		}

		// Item validation
		if (!itemDescription.trim()) {
			newErrors.itemDescription = 'Item description is required';
		}
		if (!conditionNotes.trim()) {
			newErrors.conditionNotes = 'Condition notes are required';
		}
		if (!requestedWork.trim()) {
			newErrors.requestedWork = 'Requested work is required';
		}

		// Details validation
		if (!storageLocationId) {
			newErrors.storageLocationId = 'Storage location is required';
		}

		// Quote amount validation (if provided, must be valid number)
		if (quoteAmount.trim()) {
			const amount = parseFloat(quoteAmount);
			if (isNaN(amount) || amount < 0) {
				newErrors.quoteAmount = 'Please enter a valid amount';
			}
		}

		// Photos validation
		if (photos.length === 0) {
			newErrors.photos = 'At least one photo is required';
		}

		errors = newErrors;
		return Object.keys(newErrors).length === 0;
	}

	function buildCreateTicketRequest(): CreateTicketRequest {
		// Use existing customer_id if selected, otherwise create inline customer
		const baseRequest = {
			item_type: itemType.trim() || null,
			item_description: itemDescription.trim(),
			condition_notes: conditionNotes.trim(),
			requested_work: requestedWork.trim(),
			is_rush: isRush,
			promise_date: promiseDate || null,
			storage_location_id: storageLocationId,
			quote_amount: quoteAmount.trim() || null
		};

		if (selectedCustomerId) {
			return {
				...baseRequest,
				customer_id: selectedCustomerId
			};
		}

		const customer: InlineCustomer = {
			name: customerName.trim(),
			phone: customerPhone.trim() || null,
			email: customerEmail.trim() || null
		};

		return {
			...baseRequest,
			customer
		};
	}

	function handleSubmit(e: SubmitEvent) {
		e.preventDefault();

		if (!validateForm()) {
			return;
		}

		// Clear any previous errors
		submissionError = '';

		// Show employee PIN modal
		showEmployeeModal = true;
	}

	function handleEmployeeModalClose() {
		showEmployeeModal = false;
	}

	async function handleEmployeeVerified(employee: VerifyPinResponse | EmployeeInfo) {
		showEmployeeModal = false;
		isSubmitting = true;
		submissionError = '';

		// Set the employee ID for API requests
		setCurrentEmployee(employee.employee_id);

		// Build the request
		const request = buildCreateTicketRequest();

		// Check if we're offline - if so, queue the ticket
		if (offlineStore.isOffline) {
			try {
				const queuedTicket = await syncQueueStore.queue(
					request,
					photos,
					employee.employee_id,
					employee.name
				);

				// Show offline success message
				successMessage = `Ticket saved offline! It will sync when you're back online.`;

				// Close after a brief delay to show the message
				setTimeout(() => {
					// Use client ID for offline tickets (no server ID yet)
					onSuccess?.(queuedTicket.clientId, 'OFFLINE');
					onClose();
				}, 1500);
			} catch (err) {
				submissionError = 'Failed to save ticket offline. Please try again.';
				console.error('Failed to queue ticket offline:', err);
			} finally {
				isSubmitting = false;
			}
			return;
		}

		// Online submission
		try {
			const response = await createTicket(request);

			// Upload photos
			const totalPhotos = photos.length;
			for (let i = 0; i < totalPhotos; i++) {
				const file = photos[i];
				await uploadTicketPhoto(response.ticket_id, file, (progress) => {
					photoUploadProgress.set(i, progress);
				});
			}

			// Open receipt PDF in new tab
			const receiptUrl = getReceiptPdfUrl(response.ticket_id);
			window.open(receiptUrl, '_blank');

			// Show success message
			successMessage = `Ticket ${response.friendly_code} created successfully!`;

			// Call success callback and close after a brief delay to show the message
			setTimeout(() => {
				onSuccess?.(response.ticket_id, response.friendly_code);
				onClose();
			}, 1500);
		} catch (err) {
			// If we get a network error, try to queue offline
			if (err instanceof TypeError && err.message.includes('fetch')) {
				try {
					const queuedTicket = await syncQueueStore.queue(
						request,
						photos,
						employee.employee_id,
						employee.name
					);

					successMessage = `Network unavailable. Ticket saved offline and will sync automatically.`;

					setTimeout(() => {
						onSuccess?.(queuedTicket.clientId, 'OFFLINE');
						onClose();
					}, 1500);
					return;
				} catch (queueErr) {
					submissionError = 'Failed to save ticket. Please check your connection.';
					console.error('Failed to queue ticket:', queueErr);
				}
			} else if (err instanceof ApiClientError) {
				submissionError = err.message || 'Failed to create ticket. Please try again.';
			} else {
				submissionError = 'An unexpected error occurred. Please try again.';
			}
			console.error('Failed to create ticket:', err);
		} finally {
			isSubmitting = false;
			photoUploadProgress.clear();
		}
	}

	function handleClose() {
		if (isSubmitting) return;
		onClose();
	}

	// Convert storage locations to Select options
	const locationOptions = $derived(
		storageLocations.map((loc) => ({
			value: loc.location_id,
			label: loc.name
		}))
	);
</script>

<Modal {open} title="New Repair Ticket" onClose={handleClose} closeOnBackdrop={!isSubmitting}>
	<form class="intake-form" onsubmit={handleSubmit}>
		<!-- Offline Banner -->
		{#if offlineStore.isOffline}
			<div class="offline-banner" role="status">
				<svg
					class="offline-icon"
					xmlns="http://www.w3.org/2000/svg"
					width="18"
					height="18"
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
				<div class="offline-text">
					<span class="offline-title">You're offline</span>
					<span class="offline-description"
						>Ticket will be saved locally and synced when back online</span
					>
				</div>
			</div>
		{/if}

		<!-- Customer Section -->
		<section class="form-section">
			<h3 class="section-title">Customer Information</h3>
			<div class="form-grid">
				<!-- Customer Name with Autocomplete -->
				<div class="customer-search-wrapper">
					<label for="customer-name" class="input-label">
						Customer Name
						<span class="required-indicator" aria-hidden="true">*</span>
					</label>
					<div class="search-input-container">
						<input
							type="text"
							id="customer-name"
							class="input-field"
							class:has-error={!!errors.customerName}
							placeholder="Search or enter customer name"
							value={customerName}
							oninput={handleCustomerNameInput}
							onfocus={handleCustomerInputFocus}
							onblur={handleCustomerInputBlur}
							disabled={isSubmitting}
							autocomplete="off"
							required
						/>
						{#if selectedCustomerId}
							<button
								type="button"
								class="clear-selection-btn"
								onclick={clearCustomerSelection}
								disabled={isSubmitting}
								aria-label="Clear customer selection"
							>
								<svg width="16" height="16" viewBox="0 0 16 16" fill="none">
									<path
										d="M12 4L4 12M4 4L12 12"
										stroke="currentColor"
										stroke-width="2"
										stroke-linecap="round"
									/>
								</svg>
							</button>
						{/if}
						{#if isSearchingCustomers}
							<span class="search-spinner" aria-label="Searching..."></span>
						{/if}
					</div>
					{#if showCustomerDropdown && (customerSearchResults.length > 0 || (customerName.trim().length >= 2 && !isSearchingCustomers))}
						<div class="customer-dropdown" role="listbox">
							{#each customerSearchResults as customer (customer.customer_id)}
								<button
									type="button"
									class="customer-option"
									onclick={() => selectCustomer(customer)}
									role="option"
									aria-selected="false"
								>
									<span class="customer-option-name">{customer.name}</span>
									{#if customer.phone || customer.email}
										<span class="customer-option-details">
											{customer.phone ?? ''}{customer.phone && customer.email
												? ' · '
												: ''}{customer.email ?? ''}
										</span>
									{/if}
								</button>
							{/each}
							{#if customerSearchResults.length === 0 && !isSearchingCustomers}
								<div class="customer-no-results">
									<span class="no-results-text">No matching customers</span>
									<span class="no-results-hint">A new customer will be created</span>
								</div>
							{/if}
						</div>
					{/if}
					{#if errors.customerName}
						<p class="input-error" role="alert">{errors.customerName}</p>
					{/if}
					{#if selectedCustomerId}
						<p class="customer-selected-hint">Existing customer selected</p>
					{/if}
				</div>

				<Input
					label="Phone"
					type="tel"
					placeholder="(555) 123-4567"
					bind:value={customerPhone}
					disabled={isSubmitting || !!selectedCustomerId}
				/>
				<Input
					label="Email"
					type="email"
					placeholder="customer@example.com"
					bind:value={customerEmail}
					disabled={isSubmitting || !!selectedCustomerId}
				/>
			</div>
		</section>

		<!-- Item Section -->
		<section class="form-section">
			<h3 class="section-title">Item Details</h3>
			<div class="form-grid">
				<Input
					label="Item Type"
					placeholder="e.g., Ring, Necklace, Watch"
					bind:value={itemType}
					disabled={isSubmitting}
				/>
				<Input
					label="Item Description"
					placeholder="Describe the item"
					bind:value={itemDescription}
					error={errors.itemDescription}
					required
					disabled={isSubmitting}
					class="full-width"
					onblur={() => clearFieldErrorOnBlur('itemDescription', !!itemDescription.trim())}
				/>
				<Textarea
					label="Condition Notes"
					placeholder="Describe the current condition of the item"
					bind:value={conditionNotes}
					error={errors.conditionNotes}
					required
					rows={3}
					disabled={isSubmitting}
					class="full-width"
					onblur={() => clearFieldErrorOnBlur('conditionNotes', !!conditionNotes.trim())}
				/>
				<Textarea
					label="Requested Work"
					placeholder="Describe the work to be done"
					bind:value={requestedWork}
					error={errors.requestedWork}
					required
					rows={3}
					disabled={isSubmitting}
					class="full-width"
					onblur={() => clearFieldErrorOnBlur('requestedWork', !!requestedWork.trim())}
				/>
			</div>
		</section>

		<!-- Details Section -->
		<section class="form-section">
			<h3 class="section-title">Repair Details</h3>
			<div class="form-grid">
				<div class="rush-toggle">
					<label class="rush-label">
						<input
							type="checkbox"
							bind:checked={isRush}
							disabled={isSubmitting}
							class="rush-checkbox"
						/>
						<span class="rush-text">
							<span class="rush-title">Rush Order</span>
							<span class="rush-description">Prioritize this repair over others</span>
						</span>
					</label>
				</div>

				<div class="date-field">
					<label for="promise-date" class="date-label">Promise Date</label>
					<input
						type="date"
						id="promise-date"
						class="date-input"
						bind:value={promiseDate}
						disabled={isSubmitting}
					/>
				</div>

				<Select
					label="Storage Location"
					options={locationOptions}
					bind:value={storageLocationId}
					placeholder={isLoadingLocations ? 'Loading...' : 'Select location'}
					error={errors.storageLocationId}
					required
					disabled={isSubmitting || isLoadingLocations}
					onblur={() => clearFieldErrorOnBlur('storageLocationId', !!storageLocationId)}
				/>

				<Input
					label="Quote Amount"
					type="number"
					placeholder="0.00"
					bind:value={quoteAmount}
					error={errors.quoteAmount}
					disabled={isSubmitting}
				/>
			</div>
		</section>

		<!-- Photos Section -->
		<section class="form-section">
			<h3 class="section-title">Photos</h3>
			<PhotoUpload
				label="Item Photos"
				bind:files={photos}
				maxFiles={10}
				required
				error={errors.photos}
				disabled={isSubmitting}
				onchange={() => clearFieldErrorOnBlur('photos', photos.length > 0)}
			/>
		</section>

		<!-- Success Message -->
		{#if successMessage}
			<div class="success-message" role="status">
				<svg class="success-icon" width="20" height="20" viewBox="0 0 20 20" fill="none">
					<path
						d="M10 0C4.48 0 0 4.48 0 10C0 15.52 4.48 20 10 20C15.52 20 20 15.52 20 10C20 4.48 15.52 0 10 0ZM8 15L3 10L4.41 8.59L8 12.17L15.59 4.58L17 6L8 15Z"
						fill="currentColor"
					/>
				</svg>
				<span>{successMessage}</span>
			</div>
		{/if}

		<!-- Submission Error -->
		{#if submissionError}
			<div class="submission-error" role="alert">
				<svg class="error-icon" width="20" height="20" viewBox="0 0 20 20" fill="none">
					<path
						d="M10 0C4.48 0 0 4.48 0 10C0 15.52 4.48 20 10 20C15.52 20 20 15.52 20 10C20 4.48 15.52 0 10 0ZM11 15H9V13H11V15ZM11 11H9V5H11V11Z"
						fill="currentColor"
					/>
				</svg>
				<span>{submissionError}</span>
			</div>
		{/if}

		<!-- Photo Upload Progress -->
		{#if isSubmitting && photoUploadProgress.size > 0}
			<div class="upload-progress">
				<span class="upload-label">Uploading photos...</span>
				<div class="progress-bar">
					<div
						class="progress-fill"
						style="width: {Math.round(
							([...photoUploadProgress.values()].reduce((a, b) => a + b, 0) /
								(photos.length * 100)) *
								100
						)}%"
					></div>
				</div>
			</div>
		{/if}

		<!-- Form Actions -->
		<div class="form-actions">
			<Button variant="secondary" onclick={handleClose} disabled={isSubmitting}>Cancel</Button>
			<Button variant="primary" type="submit" loading={isSubmitting} disabled={!!successMessage}
				>Create & Print</Button
			>
		</div>
	</form>
</Modal>

<!-- Employee PIN Modal -->
<EmployeeIdModal
	open={showEmployeeModal}
	title="Enter Employee PIN"
	onClose={handleEmployeeModalClose}
	onSuccess={handleEmployeeVerified}
/>

<style>
	.intake-form {
		width: 600px;
		max-width: 90vw;
		display: flex;
		flex-direction: column;
		gap: var(--space-lg, 1.5rem);
	}

	.form-section {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	.section-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text-muted, #64748b);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin: 0;
		padding-bottom: var(--space-sm, 0.5rem);
		border-bottom: 1px solid var(--color-border, #e2e8f0);
	}

	.form-grid {
		display: grid;
		grid-template-columns: repeat(2, 1fr);
		gap: var(--space-md, 1rem);
	}

	/* Full-width fields span both columns */
	:global(.form-grid .full-width) {
		grid-column: 1 / -1;
	}

	/* Customer search autocomplete styles */
	.customer-search-wrapper {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.input-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.required-indicator {
		color: var(--color-rush, #ef4444);
		margin-left: 0.125rem;
	}

	.search-input-container {
		position: relative;
		display: flex;
		align-items: center;
	}

	.input-field {
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		padding-right: 2.5rem;
		font-size: 0.875rem;
		line-height: 1.5;
		color: var(--color-text, #1e293b);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.input-field:hover:not(:disabled) {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.input-field:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.15);
	}

	.input-field:disabled {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text-muted, #64748b);
		cursor: not-allowed;
		opacity: 0.7;
	}

	.input-field.has-error {
		border-color: var(--color-rush, #ef4444);
	}

	.clear-selection-btn {
		position: absolute;
		right: 0.5rem;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		padding: 0;
		background: var(--color-bg, #f8fafc);
		border: none;
		border-radius: var(--radius-sm, 0.25rem);
		color: var(--color-text-muted, #64748b);
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.clear-selection-btn:hover {
		background: var(--color-border, #e2e8f0);
		color: var(--color-text, #1e293b);
	}

	.clear-selection-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.search-spinner {
		position: absolute;
		right: 0.75rem;
		width: 1rem;
		height: 1rem;
		border: 2px solid var(--color-border, #e2e8f0);
		border-top-color: var(--color-primary, #1e40af);
		border-radius: 50%;
		animation: spin 0.6s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	.customer-dropdown {
		position: absolute;
		top: 100%;
		left: 0;
		right: 0;
		z-index: 100;
		margin-top: 0.25rem;
		background: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		box-shadow:
			0 4px 6px -1px rgba(0, 0, 0, 0.1),
			0 2px 4px -1px rgba(0, 0, 0, 0.06);
		max-height: 200px;
		overflow-y: auto;
	}

	.customer-option {
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		text-align: left;
		background: none;
		border: none;
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.customer-option:hover {
		background-color: var(--color-bg, #f8fafc);
	}

	.customer-option:not(:last-child) {
		border-bottom: 1px solid var(--color-border, #e2e8f0);
	}

	.customer-option-name {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.customer-option-details {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	.customer-no-results {
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
	}

	.no-results-text {
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
	}

	.no-results-hint {
		font-size: 0.75rem;
		color: var(--color-primary, #1e40af);
	}

	.input-error {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		line-height: 1.4;
	}

	.customer-selected-hint {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-success, #22c55e);
		line-height: 1.4;
	}

	/* Rush toggle styling */
	.rush-toggle {
		grid-column: 1 / -1;
	}

	.rush-label {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-md, 1rem);
		background-color: var(--color-bg, #f8fafc);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		transition:
			border-color var(--transition-fast, 150ms ease),
			background-color var(--transition-fast, 150ms ease);
	}

	.rush-label:hover {
		border-color: var(--color-rush, #ef4444);
		background-color: rgba(239, 68, 68, 0.05);
	}

	.rush-label:has(.rush-checkbox:checked) {
		border-color: var(--color-rush, #ef4444);
		background-color: rgba(239, 68, 68, 0.1);
	}

	.rush-checkbox {
		width: 1.25rem;
		height: 1.25rem;
		margin-top: 0.125rem;
		accent-color: var(--color-rush, #ef4444);
		cursor: pointer;
	}

	.rush-text {
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
	}

	.rush-title {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.rush-description {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	/* Date field styling to match Input component */
	.date-field {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.date-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.date-input {
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		font-family: inherit;
		line-height: 1.5;
		color: var(--color-text, #1e293b);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.date-input:hover:not(:disabled) {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.date-input:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.15);
	}

	.date-input:disabled {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text-muted, #64748b);
		cursor: not-allowed;
		opacity: 0.7;
	}

	/* Form actions */
	.form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
		padding-top: var(--space-md, 1rem);
		border-top: 1px solid var(--color-border, #e2e8f0);
	}

	/* Success message */
	.success-message {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-md, 1rem);
		background-color: rgba(34, 197, 94, 0.1);
		border: 1px solid var(--color-success, #22c55e);
		border-radius: var(--radius-md, 0.5rem);
		color: var(--color-success, #22c55e);
		font-size: 0.875rem;
		font-weight: 500;
	}

	.success-icon {
		flex-shrink: 0;
	}

	/* Submission error */
	.submission-error {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-md, 1rem);
		background-color: rgba(239, 68, 68, 0.1);
		border: 1px solid var(--color-rush, #ef4444);
		border-radius: var(--radius-md, 0.5rem);
		color: var(--color-rush, #ef4444);
		font-size: 0.875rem;
	}

	.error-icon {
		flex-shrink: 0;
	}

	/* Upload progress */
	.upload-progress {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.upload-label {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	.progress-bar {
		height: 4px;
		background-color: var(--color-border, #e2e8f0);
		border-radius: 2px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-primary, #1e40af);
		transition: width var(--transition-fast, 150ms ease);
	}

	/* Offline banner */
	.offline-banner {
		display: flex;
		align-items: flex-start;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-md, 1rem);
		background-color: rgba(251, 191, 36, 0.1);
		border: 1px solid #fbbf24;
		border-radius: var(--radius-md, 0.5rem);
		color: #92400e;
	}

	.offline-icon {
		flex-shrink: 0;
		margin-top: 0.125rem;
	}

	.offline-text {
		display: flex;
		flex-direction: column;
		gap: 0.125rem;
	}

	.offline-title {
		font-size: 0.875rem;
		font-weight: 600;
	}

	.offline-description {
		font-size: 0.75rem;
		opacity: 0.9;
	}

	/* Responsive adjustments */
	@media (max-width: 640px) {
		.form-grid {
			grid-template-columns: 1fr;
		}

		.intake-form {
			width: 100%;
		}
	}
</style>

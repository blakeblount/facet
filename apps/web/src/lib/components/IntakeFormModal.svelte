<script lang="ts">
	import Modal from './Modal.svelte';
	import Button from './Button.svelte';
	import Input from './Input.svelte';
	import Select from './Select.svelte';
	import Textarea from './Textarea.svelte';
	import PhotoUpload from './PhotoUpload.svelte';
	import {
		listStorageLocations,
		listCustomers,
		type StorageLocationSummary,
		type CreateTicketRequest,
		type InlineCustomer,
		type Customer
	} from '$lib/services/api';

	interface Props {
		/** Whether the modal is open */
		open: boolean;
		/** Callback when modal closes */
		onClose: () => void;
		/** Callback when form is successfully submitted */
		onSuccess?: (ticketId: string) => void;
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

		// Build the request object (for display to integrating code)
		const request = buildCreateTicketRequest();
		console.log('Form data ready for submission:', request);
		console.log('Photos to upload:', photos);

		// Note: Actual submission will be implemented in a separate task
		// This modal focuses on the form structure and validation
		// For now, just close the modal to demonstrate the flow
		onSuccess?.('placeholder-ticket-id');
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
			/>
		</section>

		<!-- Form Actions -->
		<div class="form-actions">
			<Button variant="secondary" onclick={handleClose} disabled={isSubmitting}>Cancel</Button>
			<Button variant="primary" type="submit" loading={isSubmitting}>Create Ticket</Button>
		</div>
	</form>
</Modal>

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

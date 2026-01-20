<script lang="ts">
	import Modal from './Modal.svelte';
	import Button from './Button.svelte';
	import Input from './Input.svelte';
	import Select from './Select.svelte';
	import Textarea from './Textarea.svelte';
	import PhotoUpload from './PhotoUpload.svelte';
	import {
		listStorageLocations,
		type StorageLocationSummary,
		type CreateTicketRequest,
		type InlineCustomer
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

	function resetForm() {
		// Reset customer fields
		customerName = '';
		customerPhone = '';
		customerEmail = '';

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
		const customer: InlineCustomer = {
			name: customerName.trim(),
			phone: customerPhone.trim() || null,
			email: customerEmail.trim() || null
		};

		return {
			customer,
			item_type: itemType.trim() || null,
			item_description: itemDescription.trim(),
			condition_notes: conditionNotes.trim(),
			requested_work: requestedWork.trim(),
			is_rush: isRush,
			promise_date: promiseDate || null,
			storage_location_id: storageLocationId,
			quote_amount: quoteAmount.trim() || null
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
				<Input
					label="Customer Name"
					placeholder="Enter customer name"
					bind:value={customerName}
					error={errors.customerName}
					required
					disabled={isSubmitting}
				/>
				<Input
					label="Phone"
					type="tel"
					placeholder="(555) 123-4567"
					bind:value={customerPhone}
					disabled={isSubmitting}
				/>
				<Input
					label="Email"
					type="email"
					placeholder="customer@example.com"
					bind:value={customerEmail}
					disabled={isSubmitting}
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

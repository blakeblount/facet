<script lang="ts">
	import Modal from './Modal.svelte';
	import Button from './Button.svelte';
	import Input from './Input.svelte';
	import PhotoUpload from './PhotoUpload.svelte';
	import EmployeeIdModal from './EmployeeIdModal.svelte';
	import {
		getTicket,
		fetchReceiptPdf,
		fetchLabelPdf,
		closeTicket,
		toggleRush,
		addTicketNote,
		setCurrentEmployee,
		verifyEmployeePin,
		type TicketDetailResponse,
		type TicketStatus,
		type TicketPhoto,
		type VerifyPinResponse
	} from '$lib/services/api';
	import { uploadTicketPhoto } from '$lib/services/api';

	interface Props {
		/** The ticket ID to display */
		ticketId: string | null;
		/** Whether the modal is open */
		open: boolean;
		/** Callback when modal closes */
		onClose: () => void;
		/** Whether editing mode is enabled (shows edit indicators) */
		isEditing?: boolean;
		/** Callback when edit mode is requested */
		onEdit?: () => void;
		/** Callback when ticket is closed successfully */
		onTicketClosed?: () => void;
	}

	let { ticketId, open, onClose, isEditing = false, onEdit, onTicketClosed }: Props = $props();

	let ticket: TicketDetailResponse | null = $state(null);
	let loading: boolean = $state(false);
	let error: string | null = $state(null);

	// Close ticket flow state
	let showCloseModal: boolean = $state(false);
	let closeStep: 'amount' | 'employee' = $state('amount');
	let actualAmount: string = $state('');
	let employeePin: string = $state('');
	let closeError: string | null = $state(null);
	let isClosing: boolean = $state(false);

	// Print state
	let isPrintingReceipt: boolean = $state(false);
	let isPrintingTag: boolean = $state(false);
	let printError: string | null = $state(null);

	// Lightbox state
	let lightboxPhoto: TicketPhoto | null = $state(null);
	let lightboxIndex: number = $state(0);

	// Photo upload state
	let showUploadModal: boolean = $state(false);
	let uploadFiles: File[] = $state([]);
	let isUploading: boolean = $state(false);
	let uploadProgress: number = $state(0);
	let uploadError: string | null = $state(null);

	// Rush toggle state
	let isTogglingRush: boolean = $state(false);
	let rushError: string | null = $state(null);
	let showRushEmployeeModal: boolean = $state(false);
	let pendingRushValue: boolean = $state(false);

	// Notes state
	let newNoteContent: string = $state('');
	let isAddingNote: boolean = $state(false);
	let noteError: string | null = $state(null);
	let showNoteEmployeeModal: boolean = $state(false);

	// Fetch ticket when ticketId changes and modal is open
	$effect(() => {
		if (open && ticketId) {
			fetchTicket(ticketId);
		} else if (!open) {
			// Reset state when modal closes
			ticket = null;
			error = null;
		}
	});

	async function fetchTicket(id: string) {
		loading = true;
		error = null;
		try {
			ticket = await getTicket(id);
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load ticket';
			ticket = null;
		} finally {
			loading = false;
		}
	}

	function formatDate(dateString: string | null): string {
		if (!dateString) return '—';
		const date = new Date(dateString);
		return date.toLocaleDateString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric'
		});
	}

	function formatDateTime(dateString: string): string {
		const date = new Date(dateString);
		return date.toLocaleString('en-US', {
			month: 'short',
			day: 'numeric',
			year: 'numeric',
			hour: 'numeric',
			minute: '2-digit'
		});
	}

	function formatCurrency(amount: string | null): string {
		if (!amount) return '—';
		const num = parseFloat(amount);
		return new Intl.NumberFormat('en-US', {
			style: 'currency',
			currency: 'USD'
		}).format(num);
	}

	function getStatusLabel(status: TicketStatus): string {
		const labels: Record<TicketStatus, string> = {
			intake: 'Intake',
			in_progress: 'In Progress',
			waiting_on_parts: 'Waiting on Parts',
			ready_for_pickup: 'Ready for Pickup',
			closed: 'Closed',
			archived: 'Archived'
		};
		return labels[status];
	}

	function getStatusClass(status: TicketStatus): string {
		const classes: Record<TicketStatus, string> = {
			intake: 'status-intake',
			in_progress: 'status-in-progress',
			waiting_on_parts: 'status-waiting',
			ready_for_pickup: 'status-ready',
			closed: 'status-closed',
			archived: 'status-closed'
		};
		return classes[status];
	}

	// Action handlers
	function handleEditTicket() {
		onEdit?.();
	}

	// Lightbox handlers
	function openLightbox(photo: TicketPhoto) {
		if (!ticket) return;
		lightboxPhoto = photo;
		lightboxIndex = ticket.photos.findIndex((p) => p.photo_id === photo.photo_id);
	}

	function closeLightbox() {
		lightboxPhoto = null;
	}

	function navigateLightbox(direction: 'prev' | 'next') {
		if (!ticket || ticket.photos.length === 0) return;
		if (direction === 'prev') {
			lightboxIndex = lightboxIndex > 0 ? lightboxIndex - 1 : ticket.photos.length - 1;
		} else {
			lightboxIndex = lightboxIndex < ticket.photos.length - 1 ? lightboxIndex + 1 : 0;
		}
		lightboxPhoto = ticket.photos[lightboxIndex];
	}

	function handleLightboxKeydown(e: KeyboardEvent) {
		if (!lightboxPhoto) return;
		if (e.key === 'Escape') {
			closeLightbox();
		} else if (e.key === 'ArrowLeft') {
			navigateLightbox('prev');
		} else if (e.key === 'ArrowRight') {
			navigateLightbox('next');
		}
	}

	// Photo upload handlers
	function openUploadModal() {
		uploadFiles = [];
		uploadError = null;
		uploadProgress = 0;
		showUploadModal = true;
	}

	function closeUploadModal() {
		showUploadModal = false;
	}

	async function handleUploadPhotos() {
		if (!ticket || uploadFiles.length === 0 || isUploading) return;

		isUploading = true;
		uploadError = null;
		uploadProgress = 0;

		try {
			// Upload each file sequentially
			for (let i = 0; i < uploadFiles.length; i++) {
				const file = uploadFiles[i];
				const fileProgress = (completed: number) => {
					// Calculate overall progress across all files
					const baseProgress = (i / uploadFiles.length) * 100;
					const fileContribution = completed / uploadFiles.length;
					uploadProgress = Math.round(baseProgress + fileContribution);
				};

				await uploadTicketPhoto(ticket.ticket_id, file, fileProgress);
			}

			uploadProgress = 100;
			showUploadModal = false;
			// Refresh ticket to show new photos
			await fetchTicket(ticket.ticket_id);
		} catch (e) {
			uploadError = e instanceof Error ? e.message : 'Failed to upload photo';
		} finally {
			isUploading = false;
		}
	}

	async function handlePrintReceipt() {
		if (!ticket || isPrintingReceipt) return;
		isPrintingReceipt = true;
		printError = null;
		try {
			const blob = await fetchReceiptPdf(ticket.ticket_id);
			const url = URL.createObjectURL(blob);
			window.open(url, '_blank');
			// Clean up the object URL after a delay to allow the new tab to load
			setTimeout(() => URL.revokeObjectURL(url), 60000);
		} catch (e) {
			printError = e instanceof Error ? e.message : 'Failed to load receipt PDF';
		} finally {
			isPrintingReceipt = false;
		}
	}

	async function handlePrintTag() {
		if (!ticket || isPrintingTag) return;
		isPrintingTag = true;
		printError = null;
		try {
			const blob = await fetchLabelPdf(ticket.ticket_id);
			const url = URL.createObjectURL(blob);
			window.open(url, '_blank');
			// Clean up the object URL after a delay to allow the new tab to load
			setTimeout(() => URL.revokeObjectURL(url), 60000);
		} catch (e) {
			printError = e instanceof Error ? e.message : 'Failed to load label PDF';
		} finally {
			isPrintingTag = false;
		}
	}

	function openCloseModal() {
		// Reset close flow state
		closeStep = 'amount';
		actualAmount = ticket?.quote_amount ?? '';
		employeePin = '';
		closeError = null;
		showCloseModal = true;
	}

	function closeCloseModal() {
		showCloseModal = false;
	}

	function handleAmountSubmit() {
		// Convert to string in case type="number" input returns a number
		const amountStr = String(actualAmount).trim();
		if (!amountStr) {
			closeError = 'Please enter the actual amount charged';
			return;
		}
		// Validate it's a valid number
		const amount = parseFloat(amountStr);
		if (isNaN(amount) || amount < 0) {
			closeError = 'Please enter a valid amount';
			return;
		}
		closeError = null;
		closeStep = 'employee';
	}

	async function handleEmployeeSubmit() {
		if (!employeePin.trim()) {
			closeError = 'Please enter your employee PIN';
			return;
		}
		if (!ticket) return;

		isClosing = true;
		closeError = null;

		try {
			// Verify the employee PIN first
			const employee = await verifyEmployeePin(employeePin);
			setCurrentEmployee(employee.employee_id);

			await closeTicket(ticket.ticket_id, String(actualAmount));
			showCloseModal = false;
			onTicketClosed?.();
			// Refresh the ticket to show updated status
			await fetchTicket(ticket.ticket_id);
		} catch (e) {
			closeError = e instanceof Error ? e.message : 'Failed to close ticket';
		} finally {
			isClosing = false;
			setCurrentEmployee(null);
		}
	}

	// Check if ticket can be closed (only Ready for Pickup status)
	function canCloseTicket(): boolean {
		return ticket?.status === 'ready_for_pickup';
	}

	// Check if ticket is already closed/archived
	function isTicketClosed(): boolean {
		return ticket?.status === 'closed' || ticket?.status === 'archived';
	}

	// Rush toggle handlers
	function handleRushToggleClick() {
		if (!ticket || isTogglingRush || isTicketClosed()) return;
		pendingRushValue = !ticket.is_rush;
		showRushEmployeeModal = true;
	}

	async function handleRushEmployeeSuccess(employee: VerifyPinResponse) {
		if (!ticket) return;
		showRushEmployeeModal = false;
		isTogglingRush = true;
		rushError = null;

		try {
			setCurrentEmployee(employee.employee_id);
			await toggleRush(ticket.ticket_id, pendingRushValue);
			// Refresh ticket to show updated data
			await fetchTicket(ticket.ticket_id);
		} catch (e) {
			rushError = e instanceof Error ? e.message : 'Failed to update rush status';
		} finally {
			isTogglingRush = false;
			setCurrentEmployee(null);
		}
	}

	// Notes handlers
	function handleAddNoteClick() {
		if (!newNoteContent.trim()) {
			noteError = 'Please enter a note';
			return;
		}
		noteError = null;
		showNoteEmployeeModal = true;
	}

	async function handleNoteEmployeeSuccess(employee: VerifyPinResponse) {
		if (!ticket || !newNoteContent.trim()) return;
		showNoteEmployeeModal = false;
		isAddingNote = true;
		noteError = null;

		try {
			setCurrentEmployee(employee.employee_id);
			await addTicketNote(ticket.ticket_id, newNoteContent.trim());
			newNoteContent = '';
			// Refresh ticket to show new note
			await fetchTicket(ticket.ticket_id);
		} catch (e) {
			noteError = e instanceof Error ? e.message : 'Failed to add note';
		} finally {
			isAddingNote = false;
			setCurrentEmployee(null);
		}
	}
</script>

<Modal {open} title={ticket?.friendly_code ?? 'Ticket Details'} {onClose}>
	<div class="ticket-detail-content">
		{#if loading}
			<div class="loading-state">
				<div class="spinner"></div>
				<p>Loading ticket details...</p>
			</div>
		{:else if error}
			<div class="error-state">
				<p class="error-message">{error}</p>
				<button class="retry-button" onclick={() => ticketId && fetchTicket(ticketId)}>
					Retry
				</button>
			</div>
		{:else if ticket}
			<!-- Header with status and rush indicator -->
			<div class="ticket-header">
				<div class="ticket-code-row">
					<span class="ticket-code">{ticket.friendly_code}</span>
					{#if ticket.is_rush}
						<span class="rush-badge">RUSH</span>
					{/if}
				</div>
				<span class="status-badge {getStatusClass(ticket.status)}">
					{getStatusLabel(ticket.status)}
				</span>
			</div>

			<!-- Customer Section -->
			<section class="detail-section">
				<h3 class="section-title">Customer</h3>
				<div class="section-content">
					<div class="info-row">
						<span class="info-label">Name</span>
						<span class="info-value">{ticket.customer.name}</span>
					</div>
					{#if ticket.customer.phone}
						<div class="info-row">
							<span class="info-label">Phone</span>
							<span class="info-value">{ticket.customer.phone}</span>
						</div>
					{/if}
					{#if ticket.customer.email}
						<div class="info-row">
							<span class="info-label">Email</span>
							<span class="info-value">{ticket.customer.email}</span>
						</div>
					{/if}
				</div>
			</section>

			<!-- Item Section -->
			<section class="detail-section">
				<h3 class="section-title">Item Details</h3>
				<div class="section-content">
					{#if ticket.item_type}
						<div class="info-row" class:editable={isEditing}>
							<span class="info-label">Type</span>
							<span class="info-value">
								{ticket.item_type}
								{#if isEditing}
									<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
										<path
											d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
										/>
									</svg>
								{/if}
							</span>
						</div>
					{/if}
					<div class="info-row" class:editable={isEditing}>
						<span class="info-label">Description</span>
						<span class="info-value">
							{ticket.item_description}
							{#if isEditing}
								<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
									<path
										d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
									/>
								</svg>
							{/if}
						</span>
					</div>
					<div class="info-row" class:editable={isEditing}>
						<span class="info-label">Condition</span>
						<span class="info-value text-block">
							{ticket.condition_notes}
							{#if isEditing}
								<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
									<path
										d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
									/>
								</svg>
							{/if}
						</span>
					</div>
					<div class="info-row" class:editable={isEditing}>
						<span class="info-label">Requested Work</span>
						<span class="info-value text-block">
							{ticket.requested_work}
							{#if isEditing}
								<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
									<path
										d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
									/>
								</svg>
							{/if}
						</span>
					</div>
				</div>
			</section>

			<!-- Photos Section -->
			<section class="detail-section">
				<div class="section-header">
					<h3 class="section-title">Photos ({ticket.photos.length})</h3>
					{#if !isTicketClosed() && ticket.photos.length < 10}
						<button class="add-photo-btn" type="button" onclick={openUploadModal}>
							<svg
								viewBox="0 0 16 16"
								fill="currentColor"
								aria-hidden="true"
								class="add-photo-icon"
							>
								<path
									d="M8 0a8 8 0 1 1 0 16A8 8 0 0 1 8 0ZM1.5 8a6.5 6.5 0 1 0 13 0 6.5 6.5 0 0 0-13 0Zm7.25-3.25v2.5h2.5a.75.75 0 0 1 0 1.5h-2.5v2.5a.75.75 0 0 1-1.5 0v-2.5h-2.5a.75.75 0 0 1 0-1.5h2.5v-2.5a.75.75 0 0 1 1.5 0Z"
								/>
							</svg>
							Add Photo
						</button>
					{/if}
				</div>
				<div class="section-content">
					{#if ticket.photos.length > 0}
						<div class="photos-grid">
							{#each ticket.photos as photo (photo.photo_id)}
								<button
									type="button"
									class="photo-item"
									onclick={() => openLightbox(photo)}
									aria-label="View photo uploaded {formatDateTime(photo.uploaded_at)} by {photo
										.uploaded_by.name}"
								>
									<img src={photo.url} alt="Ticket item" class="photo-thumbnail" />
									<span class="photo-meta">
										{formatDateTime(photo.uploaded_at)} by {photo.uploaded_by.name}
									</span>
								</button>
							{/each}
						</div>
					{:else}
						<div class="empty-photos">
							<p class="empty-message">No photos attached</p>
							{#if !isTicketClosed()}
								<Button variant="secondary" onclick={openUploadModal}>Add Photo</Button>
							{/if}
						</div>
					{/if}
				</div>
			</section>

			<!-- Pricing Section -->
			<section class="detail-section">
				<h3 class="section-title">Pricing</h3>
				<div class="section-content">
					<div class="info-row" class:editable={isEditing}>
						<span class="info-label">Quote</span>
						<span class="info-value">
							{formatCurrency(ticket.quote_amount)}
							{#if isEditing}
								<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
									<path
										d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
									/>
								</svg>
							{/if}
						</span>
					</div>
					<div class="info-row" class:editable={isEditing}>
						<span class="info-label">Actual Charged</span>
						<span class="info-value">
							{formatCurrency(ticket.actual_amount)}
							{#if isEditing}
								<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
									<path
										d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
									/>
								</svg>
							{/if}
						</span>
					</div>
				</div>
			</section>

			<!-- Status & Location Section -->
			<section class="detail-section">
				<h3 class="section-title">Status & Location</h3>
				<div class="section-content">
					<div class="info-row">
						<span class="info-label">Current Status</span>
						<span class="info-value">
							<span class="status-badge inline {getStatusClass(ticket.status)}">
								{getStatusLabel(ticket.status)}
							</span>
						</span>
					</div>
					<div class="info-row">
						<span class="info-label">Rush</span>
						<span class="info-value rush-row">
							{#if ticket.is_rush}
								<span class="rush-badge">RUSH</span>
							{:else}
								<span class="rush-off">No</span>
							{/if}
							{#if !isTicketClosed()}
								<button
									type="button"
									class="rush-toggle-btn"
									onclick={handleRushToggleClick}
									disabled={isTogglingRush}
								>
									{#if isTogglingRush}
										<span class="spinner-small"></span>
									{:else if ticket.is_rush}
										Remove Rush
									{:else}
										Mark Rush
									{/if}
								</button>
							{/if}
						</span>
					</div>
					{#if rushError}
						<div class="inline-error">{rushError}</div>
					{/if}
					<div class="info-row" class:editable={isEditing}>
						<span class="info-label">Promise Date</span>
						<span class="info-value">
							{formatDate(ticket.promise_date)}
							{#if isEditing}
								<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
									<path
										d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
									/>
								</svg>
							{/if}
						</span>
					</div>
					<div class="info-row" class:editable={isEditing}>
						<span class="info-label">Storage Location</span>
						<span class="info-value">
							{ticket.storage_location.name}
							{#if isEditing}
								<svg class="edit-icon" viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
									<path
										d="M11.013 1.427a1.75 1.75 0 0 1 2.474 0l1.086 1.086a1.75 1.75 0 0 1 0 2.474l-8.61 8.61c-.21.21-.47.364-.756.445l-3.251.93a.75.75 0 0 1-.927-.928l.929-3.25c.081-.286.235-.547.445-.758l8.61-8.61Zm1.414 1.06a.25.25 0 0 0-.354 0L10.811 3.75l1.439 1.44 1.263-1.263a.25.25 0 0 0 0-.354l-1.086-1.086ZM11.189 6.25 9.75 4.81l-6.286 6.287a.253.253 0 0 0-.064.108l-.558 1.953 1.953-.558a.253.253 0 0 0 .108-.064l6.286-6.286Z"
									/>
								</svg>
							{/if}
						</span>
					</div>
				</div>
			</section>

			<!-- Status History Section -->
			<section class="detail-section">
				<h3 class="section-title">Status History ({ticket.status_history.length})</h3>
				<div class="section-content">
					{#if ticket.status_history.length > 0}
						<ul class="status-history-list">
							{#each ticket.status_history as entry, index (index)}
								<li class="status-history-item">
									<div class="status-history-change">
										{#if entry.from_status}
											<span class="status-badge small {getStatusClass(entry.from_status)}">
												{getStatusLabel(entry.from_status)}
											</span>
											<svg
												class="status-arrow"
												viewBox="0 0 16 16"
												fill="currentColor"
												aria-hidden="true"
											>
												<path
													fill-rule="evenodd"
													d="M8.22 2.97a.75.75 0 0 1 1.06 0l4.25 4.25a.75.75 0 0 1 0 1.06l-4.25 4.25a.75.75 0 0 1-1.06-1.06l2.97-2.97H3.75a.75.75 0 0 1 0-1.5h7.44L8.22 4.03a.75.75 0 0 1 0-1.06Z"
													clip-rule="evenodd"
												/>
											</svg>
										{/if}
										<span class="status-badge small {getStatusClass(entry.to_status)}">
											{getStatusLabel(entry.to_status)}
										</span>
									</div>
									<span class="status-history-meta">
										{formatDateTime(entry.changed_at)} by {entry.changed_by.name}
									</span>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="empty-message">No status changes recorded</p>
					{/if}
				</div>
			</section>

			<!-- Notes Section -->
			<section class="detail-section">
				<h3 class="section-title">Notes ({ticket.notes.length})</h3>
				<div class="section-content">
					{#if !isTicketClosed()}
						<div class="add-note-form">
							<div class="add-note-input-wrapper">
								<textarea
									class="add-note-input"
									placeholder="Add a note..."
									bind:value={newNoteContent}
									disabled={isAddingNote}
									rows="2"
								></textarea>
							</div>
							<div class="add-note-actions">
								{#if noteError}
									<span class="add-note-error">{noteError}</span>
								{/if}
								<Button
									variant="secondary"
									onclick={handleAddNoteClick}
									disabled={isAddingNote || !newNoteContent.trim()}
									loading={isAddingNote}
								>
									Add Note
								</Button>
							</div>
						</div>
					{/if}
					{#if ticket.notes.length > 0}
						<ul class="notes-list">
							{#each ticket.notes as note (note.note_id)}
								<li class="note-item">
									<p class="note-content">{note.content}</p>
									<span class="note-meta">
										{formatDateTime(note.created_at)} by {note.created_by.name}
									</span>
								</li>
							{/each}
						</ul>
					{:else if isTicketClosed()}
						<p class="empty-message">No notes</p>
					{/if}
				</div>
			</section>

			<!-- Activity / Attribution Section -->
			<section class="detail-section">
				<h3 class="section-title">Activity</h3>
				<div class="section-content">
					<div class="info-row">
						<span class="info-label">Taken in by</span>
						<span class="info-value">{ticket.taken_in_by.name}</span>
					</div>
					{#if ticket.worked_by}
						<div class="info-row">
							<span class="info-label">Worked by</span>
							<span class="info-value">{ticket.worked_by.name}</span>
						</div>
					{/if}
					{#if ticket.closed_by}
						<div class="info-row">
							<span class="info-label">Closed by</span>
							<span class="info-value">{ticket.closed_by.name}</span>
						</div>
					{/if}
					<div class="info-row">
						<span class="info-label">Created</span>
						<span class="info-value">{formatDateTime(ticket.created_at)}</span>
					</div>
					{#if ticket.closed_at}
						<div class="info-row">
							<span class="info-label">Closed</span>
							<span class="info-value">{formatDateTime(ticket.closed_at)}</span>
						</div>
					{/if}
				</div>
			</section>

			<!-- Action Buttons -->
			<section class="detail-section actions-section">
				{#if printError}
					<div class="print-error">
						<span class="print-error-message">{printError}</span>
						<button class="print-error-dismiss" onclick={() => (printError = null)}>Dismiss</button>
					</div>
				{/if}
				<div class="actions-row">
					{#if !isTicketClosed()}
						<Button variant="secondary" onclick={handleEditTicket}>Edit Ticket</Button>
					{/if}
					<Button variant="secondary" onclick={handlePrintReceipt} loading={isPrintingReceipt}>
						Print Receipt
					</Button>
					<Button variant="secondary" onclick={handlePrintTag} loading={isPrintingTag}>
						Print Tag
					</Button>
					{#if canCloseTicket()}
						<Button variant="primary" onclick={openCloseModal}>Close Ticket</Button>
					{/if}
				</div>
			</section>
		{/if}
	</div>
</Modal>

<!-- Close Ticket Modal -->
<Modal open={showCloseModal} title="Close Ticket" onClose={closeCloseModal}>
	<div class="close-ticket-modal">
		{#if closeStep === 'amount'}
			<form
				onsubmit={(e) => {
					e.preventDefault();
					handleAmountSubmit();
				}}
			>
				<p class="close-modal-description">Enter the actual amount charged for this repair.</p>
				<Input
					label="Actual Amount"
					type="number"
					placeholder="0.00"
					bind:value={actualAmount}
					error={closeError ?? undefined}
					required
				/>
				<div class="close-modal-actions">
					<Button variant="secondary" onclick={closeCloseModal}>Cancel</Button>
					<Button variant="primary" type="submit">Next</Button>
				</div>
			</form>
		{:else}
			<form
				onsubmit={(e) => {
					e.preventDefault();
					handleEmployeeSubmit();
				}}
			>
				<p class="close-modal-description">
					Enter your employee PIN to confirm closing this ticket.
				</p>
				<Input
					label="Employee PIN"
					type="password"
					placeholder="Enter PIN"
					bind:value={employeePin}
					error={closeError ?? undefined}
					required
				/>
				<div class="close-modal-actions">
					<Button
						variant="secondary"
						onclick={() => {
							closeStep = 'amount';
							closeError = null;
						}}
					>
						Back
					</Button>
					<Button variant="primary" type="submit" loading={isClosing}>Close Ticket</Button>
				</div>
			</form>
		{/if}
	</div>
</Modal>

<!-- Photo Lightbox -->
{#if lightboxPhoto}
	<div
		class="lightbox-overlay"
		role="dialog"
		tabindex="-1"
		aria-modal="true"
		aria-label="Photo viewer"
		onclick={closeLightbox}
		onkeydown={handleLightboxKeydown}
	>
		<button
			type="button"
			class="lightbox-close"
			onclick={closeLightbox}
			aria-label="Close photo viewer"
		>
			<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
				<line x1="18" y1="6" x2="6" y2="18" />
				<line x1="6" y1="6" x2="18" y2="18" />
			</svg>
		</button>

		{#if ticket && ticket.photos.length > 1}
			<button
				type="button"
				class="lightbox-nav lightbox-prev"
				onclick={(e) => {
					e.stopPropagation();
					navigateLightbox('prev');
				}}
				aria-label="Previous photo"
			>
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<polyline points="15 18 9 12 15 6" />
				</svg>
			</button>
			<button
				type="button"
				class="lightbox-nav lightbox-next"
				onclick={(e) => {
					e.stopPropagation();
					navigateLightbox('next');
				}}
				aria-label="Next photo"
			>
				<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
					<polyline points="9 18 15 12 9 6" />
				</svg>
			</button>
		{/if}

		<div class="lightbox-content" role="presentation" onclick={(e) => e.stopPropagation()}>
			<img src={lightboxPhoto.url} alt="Ticket item full size" class="lightbox-image" />
			<div class="lightbox-meta">
				<span>
					{formatDateTime(lightboxPhoto.uploaded_at)} by {lightboxPhoto.uploaded_by.name}
				</span>
				{#if ticket && ticket.photos.length > 1}
					<span class="lightbox-counter">
						{lightboxIndex + 1} / {ticket.photos.length}
					</span>
				{/if}
			</div>
		</div>
	</div>
{/if}

<!-- Upload Photo Modal -->
<Modal open={showUploadModal} title="Add Photos" onClose={closeUploadModal}>
	<div class="upload-photo-modal">
		{#if uploadError}
			<div class="upload-error-banner" role="alert">
				<span>{uploadError}</span>
				<button type="button" onclick={() => (uploadError = null)}>Dismiss</button>
			</div>
		{/if}

		<PhotoUpload
			label="Select photos to upload"
			maxFiles={ticket ? 10 - ticket.photos.length : 10}
			bind:files={uploadFiles}
			disabled={isUploading}
		/>

		{#if isUploading}
			<div class="upload-progress">
				<div class="progress-bar">
					<div class="progress-fill" style="width: {uploadProgress}%"></div>
				</div>
				<span class="progress-text">Uploading... {uploadProgress}%</span>
			</div>
		{/if}

		<div class="upload-modal-actions">
			<Button variant="secondary" onclick={closeUploadModal} disabled={isUploading}>Cancel</Button>
			<Button
				variant="primary"
				onclick={handleUploadPhotos}
				disabled={uploadFiles.length === 0}
				loading={isUploading}
			>
				Upload {uploadFiles.length > 0 ? `(${uploadFiles.length})` : ''}
			</Button>
		</div>
	</div>
</Modal>

<!-- Employee PIN Modal for Rush Toggle -->
<EmployeeIdModal
	open={showRushEmployeeModal}
	title="Verify Employee"
	onClose={() => (showRushEmployeeModal = false)}
	onSuccess={handleRushEmployeeSuccess}
/>

<!-- Employee PIN Modal for Adding Notes -->
<EmployeeIdModal
	open={showNoteEmployeeModal}
	title="Verify Employee"
	onClose={() => (showNoteEmployeeModal = false)}
	onSuccess={handleNoteEmployeeSuccess}
/>

<style>
	.ticket-detail-content {
		width: 600px;
		max-width: 90vw;
	}

	/* Loading state */
	.loading-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl, 2rem);
		gap: var(--space-md, 1rem);
	}

	.spinner {
		width: 32px;
		height: 32px;
		border: 3px solid var(--color-border, #e2e8f0);
		border-top-color: var(--color-primary, #1e40af);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to {
			transform: rotate(360deg);
		}
	}

	/* Error state */
	.error-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl, 2rem);
		gap: var(--space-md, 1rem);
	}

	.error-message {
		color: var(--color-rush, #ef4444);
		text-align: center;
	}

	.retry-button {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		background-color: var(--color-primary, #1e40af);
		color: white;
		border: none;
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		font-weight: 500;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.retry-button:hover {
		background-color: var(--color-primary-dark, #1e3a8a);
	}

	/* Ticket header */
	.ticket-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding-bottom: var(--space-md, 1rem);
		margin-bottom: var(--space-md, 1rem);
		border-bottom: 1px solid var(--color-border, #e2e8f0);
	}

	.ticket-code-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
	}

	.ticket-code {
		font-size: 1.25rem;
		font-weight: 600;
		font-family: var(--font-mono, monospace);
		color: var(--color-text, #1e293b);
	}

	.rush-badge {
		padding: 0.125rem 0.5rem;
		background-color: var(--color-rush, #ef4444);
		color: white;
		font-size: 0.75rem;
		font-weight: 600;
		border-radius: var(--radius-sm, 0.25rem);
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.status-badge {
		padding: 0.25rem 0.75rem;
		font-size: 0.875rem;
		font-weight: 500;
		border-radius: var(--radius-md, 0.5rem);
	}

	/* Sections */
	.detail-section {
		margin-bottom: var(--space-lg, 1.5rem);
	}

	.detail-section:last-child {
		margin-bottom: 0;
	}

	.section-title {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text-muted, #64748b);
		text-transform: uppercase;
		letter-spacing: 0.05em;
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.section-content {
		background-color: var(--color-bg, #f8fafc);
		border-radius: var(--radius-md, 0.5rem);
		padding: var(--space-md, 1rem);
	}

	/* Info rows */
	.info-row {
		display: flex;
		padding: var(--space-xs, 0.25rem) 0;
	}

	.info-row:not(:last-child) {
		border-bottom: 1px solid var(--color-border, #e2e8f0);
		padding-bottom: var(--space-sm, 0.5rem);
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.info-label {
		flex-shrink: 0;
		width: 140px;
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
	}

	.info-value {
		flex: 1;
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
	}

	.info-value.text-block {
		white-space: pre-wrap;
		line-height: 1.5;
	}

	/* Editable indicators */
	.info-row.editable {
		cursor: pointer;
		border-radius: var(--radius-sm, 0.25rem);
		margin-left: calc(-1 * var(--space-xs, 0.25rem));
		margin-right: calc(-1 * var(--space-xs, 0.25rem));
		padding-left: var(--space-xs, 0.25rem);
		padding-right: var(--space-xs, 0.25rem);
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.info-row.editable:hover {
		background-color: var(--color-bg-hover, rgba(0, 0, 0, 0.04));
	}

	.edit-icon {
		width: 14px;
		height: 14px;
		margin-left: var(--space-xs, 0.25rem);
		color: var(--color-primary, #1e40af);
		opacity: 0.6;
		vertical-align: middle;
		flex-shrink: 0;
		transition: opacity var(--transition-fast, 150ms ease);
	}

	.info-row.editable:hover .edit-icon {
		opacity: 1;
	}

	/* Section header with title and action */
	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.section-header .section-title {
		margin-bottom: 0;
	}

	.add-photo-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
		padding: var(--space-xs, 0.25rem) var(--space-sm, 0.5rem);
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--color-primary, #1e40af);
		background: none;
		border: 1px solid var(--color-primary, #1e40af);
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.add-photo-btn:hover {
		background-color: var(--color-primary, #1e40af);
		color: white;
	}

	.add-photo-icon {
		width: 14px;
		height: 14px;
	}

	/* Photos */
	.photos-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
		gap: var(--space-md, 1rem);
	}

	.photo-item {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
		padding: 0;
		background: none;
		border: none;
		text-align: left;
		cursor: pointer;
	}

	.photo-item:focus-visible {
		outline: 2px solid var(--color-primary, #1e40af);
		outline-offset: 2px;
		border-radius: var(--radius-md, 0.5rem);
	}

	.photo-thumbnail {
		width: 100%;
		aspect-ratio: 1;
		object-fit: cover;
		border-radius: var(--radius-md, 0.5rem);
		transition: transform var(--transition-fast, 150ms ease);
	}

	.photo-item:hover .photo-thumbnail {
		transform: scale(1.02);
	}

	.empty-photos {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-md, 1rem);
		padding: var(--space-md, 1rem);
	}

	.photo-meta {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
		line-height: 1.3;
	}

	/* Notes */
	.notes-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.note-item {
		padding: var(--space-sm, 0.5rem) 0;
	}

	.note-item:not(:last-child) {
		border-bottom: 1px solid var(--color-border, #e2e8f0);
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.note-content {
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
		line-height: 1.5;
		margin-bottom: var(--space-xs, 0.25rem);
		white-space: pre-wrap;
	}

	.note-meta {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	/* Status badge inline variant */
	.status-badge.inline {
		display: inline-block;
		vertical-align: middle;
	}

	.status-badge.small {
		padding: 0.125rem 0.375rem;
		font-size: 0.75rem;
	}

	/* Rush toggle row */
	.rush-row {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
	}

	.rush-off {
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
	}

	.rush-toggle-btn {
		display: inline-flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
		padding: var(--space-xs, 0.25rem) var(--space-sm, 0.5rem);
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--color-primary, #1e40af);
		background: none;
		border: 1px solid var(--color-primary, #1e40af);
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.rush-toggle-btn:hover:not(:disabled) {
		background-color: var(--color-primary, #1e40af);
		color: white;
	}

	.rush-toggle-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.spinner-small {
		width: 12px;
		height: 12px;
		border: 2px solid var(--color-border, #e2e8f0);
		border-top-color: var(--color-primary, #1e40af);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	.inline-error {
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		margin-top: var(--space-xs, 0.25rem);
		padding-left: 140px;
	}

	/* Status history */
	.status-history-list {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.status-history-item {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
		padding: var(--space-sm, 0.5rem) 0;
	}

	.status-history-item:not(:last-child) {
		border-bottom: 1px solid var(--color-border, #e2e8f0);
	}

	.status-history-change {
		display: flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
	}

	.status-arrow {
		width: 14px;
		height: 14px;
		color: var(--color-text-muted, #64748b);
	}

	.status-history-meta {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	/* Add note form */
	.add-note-form {
		margin-bottom: var(--space-md, 1rem);
		padding-bottom: var(--space-md, 1rem);
		border-bottom: 1px solid var(--color-border, #e2e8f0);
	}

	.add-note-input-wrapper {
		margin-bottom: var(--space-sm, 0.5rem);
	}

	.add-note-input {
		width: 100%;
		padding: var(--space-sm, 0.5rem);
		font-size: 0.875rem;
		font-family: inherit;
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		resize: vertical;
		min-height: 60px;
		transition: border-color var(--transition-fast, 150ms ease);
	}

	.add-note-input:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
	}

	.add-note-input:disabled {
		background-color: var(--color-bg, #f8fafc);
		cursor: not-allowed;
	}

	.add-note-actions {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
	}

	.add-note-error {
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
	}

	/* Empty state */
	.empty-message {
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
		font-style: italic;
	}

	/* Actions section */
	.actions-section {
		margin-top: var(--space-lg, 1.5rem);
		padding-top: var(--space-md, 1rem);
		border-top: 1px solid var(--color-border, #e2e8f0);
	}

	.print-error {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		margin-bottom: var(--space-sm, 0.5rem);
		background-color: var(--color-error-bg, #fef2f2);
		border: 1px solid var(--color-error-border, #fecaca);
		border-radius: var(--radius-md, 0.5rem);
	}

	.print-error-message {
		font-size: 0.875rem;
		color: var(--color-rush, #ef4444);
	}

	.print-error-dismiss {
		padding: var(--space-xs, 0.25rem) var(--space-sm, 0.5rem);
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		background: none;
		border: 1px solid var(--color-rush, #ef4444);
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.print-error-dismiss:hover {
		background-color: var(--color-error-bg, #fef2f2);
	}

	.actions-row {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-sm, 0.5rem);
	}

	/* Close ticket modal */
	.close-ticket-modal {
		width: 320px;
		max-width: 90vw;
	}

	.close-modal-description {
		margin: 0 0 var(--space-md, 1rem);
		font-size: 0.875rem;
		color: var(--color-text-muted, #64748b);
		line-height: 1.5;
	}

	.close-modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
		margin-top: var(--space-lg, 1.5rem);
	}

	/* Lightbox */
	.lightbox-overlay {
		position: fixed;
		inset: 0;
		z-index: 1000;
		display: flex;
		align-items: center;
		justify-content: center;
		background-color: rgba(0, 0, 0, 0.9);
		padding: var(--space-xl, 2rem);
	}

	.lightbox-close {
		position: absolute;
		top: var(--space-md, 1rem);
		right: var(--space-md, 1rem);
		display: flex;
		align-items: center;
		justify-content: center;
		width: 2.5rem;
		height: 2.5rem;
		padding: 0;
		background: rgba(255, 255, 255, 0.1);
		border: none;
		border-radius: 50%;
		color: white;
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.lightbox-close:hover {
		background: rgba(255, 255, 255, 0.2);
	}

	.lightbox-close svg {
		width: 1.25rem;
		height: 1.25rem;
	}

	.lightbox-nav {
		position: absolute;
		top: 50%;
		transform: translateY(-50%);
		display: flex;
		align-items: center;
		justify-content: center;
		width: 3rem;
		height: 3rem;
		padding: 0;
		background: rgba(255, 255, 255, 0.1);
		border: none;
		border-radius: 50%;
		color: white;
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.lightbox-nav:hover {
		background: rgba(255, 255, 255, 0.2);
	}

	.lightbox-nav svg {
		width: 1.5rem;
		height: 1.5rem;
	}

	.lightbox-prev {
		left: var(--space-md, 1rem);
	}

	.lightbox-next {
		right: var(--space-md, 1rem);
	}

	.lightbox-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		max-width: 90vw;
		max-height: 90vh;
	}

	.lightbox-image {
		max-width: 100%;
		max-height: calc(90vh - 3rem);
		object-fit: contain;
		border-radius: var(--radius-md, 0.5rem);
	}

	.lightbox-meta {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		margin-top: var(--space-sm, 0.5rem);
		padding: var(--space-sm, 0.5rem);
		color: rgba(255, 255, 255, 0.8);
		font-size: 0.875rem;
	}

	.lightbox-counter {
		font-weight: 500;
	}

	/* Upload photo modal */
	.upload-photo-modal {
		width: 400px;
		max-width: 90vw;
	}

	.upload-error-banner {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-sm, 0.5rem);
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		margin-bottom: var(--space-md, 1rem);
		background-color: var(--color-error-bg, #fef2f2);
		border: 1px solid var(--color-error-border, #fecaca);
		border-radius: var(--radius-md, 0.5rem);
		font-size: 0.875rem;
		color: var(--color-rush, #ef4444);
	}

	.upload-error-banner button {
		padding: var(--space-xs, 0.25rem) var(--space-sm, 0.5rem);
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		background: none;
		border: 1px solid var(--color-rush, #ef4444);
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
	}

	.upload-progress {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
		margin-top: var(--space-md, 1rem);
	}

	.progress-bar {
		height: 8px;
		background-color: var(--color-border, #e2e8f0);
		border-radius: 4px;
		overflow: hidden;
	}

	.progress-fill {
		height: 100%;
		background-color: var(--color-primary, #1e40af);
		transition: width 150ms ease;
	}

	.progress-text {
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
		text-align: center;
	}

	.upload-modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
		margin-top: var(--space-lg, 1.5rem);
	}
</style>

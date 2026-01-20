<script lang="ts">
	interface PhotoFile {
		file: File;
		id: string;
		preview: string;
	}

	interface Props {
		/** Label text displayed above the upload zone */
		label?: string;
		/** Maximum number of files allowed */
		maxFiles?: number;
		/** Maximum file size in bytes (default: 10MB) */
		maxSize?: number;
		/** Accepted file types (MIME types) */
		accept?: string;
		/** Whether the component is disabled */
		disabled?: boolean;
		/** Whether at least one photo is required */
		required?: boolean;
		/** Error message to display */
		error?: string;
		/** Currently selected files (two-way bindable) */
		files?: File[];
		/** Additional CSS class for the wrapper */
		class?: string;
	}

	let {
		label,
		maxFiles = 10,
		maxSize = 10 * 1024 * 1024,
		accept = 'image/*',
		disabled = false,
		required = false,
		error,
		files = $bindable([]),
		class: className = ''
	}: Props = $props();

	let photoFiles = $state<PhotoFile[]>([]);
	let isDragging = $state(false);
	let fileInput: HTMLInputElement;
	let validationError = $state<string | null>(null);

	const displayError = $derived(error || validationError);
	const canAddMore = $derived(photoFiles.length < maxFiles);

	const generatedId = `photo-upload-${Math.random().toString(36).substring(2, 9)}`;
	const errorId = $derived(`${generatedId}-error`);

	function generateId(): string {
		return Math.random().toString(36).substring(2, 11);
	}

	function formatFileSize(bytes: number): string {
		if (bytes < 1024) return `${bytes} B`;
		if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
		return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
	}

	function validateFile(file: File): string | null {
		if (file.size > maxSize) {
			return `File "${file.name}" exceeds ${formatFileSize(maxSize)} limit`;
		}

		if (accept !== '*' && accept !== '*/*') {
			const acceptedTypes = accept.split(',').map((t) => t.trim());
			const isAccepted = acceptedTypes.some((type) => {
				if (type.endsWith('/*')) {
					const category = type.slice(0, -2);
					return file.type.startsWith(category);
				}
				return file.type === type;
			});

			if (!isAccepted) {
				return `File "${file.name}" is not an accepted file type`;
			}
		}

		return null;
	}

	async function processFiles(newFiles: FileList | File[]): Promise<void> {
		validationError = null;
		const fileArray = Array.from(newFiles);

		const availableSlots = maxFiles - photoFiles.length;
		if (fileArray.length > availableSlots) {
			validationError = `Can only add ${availableSlots} more file${availableSlots === 1 ? '' : 's'}`;
			return;
		}

		const validFiles: PhotoFile[] = [];

		for (const file of fileArray) {
			const error = validateFile(file);
			if (error) {
				validationError = error;
				return;
			}

			const preview = await createPreview(file);
			validFiles.push({
				file,
				id: generateId(),
				preview
			});
		}

		photoFiles = [...photoFiles, ...validFiles];
		syncFiles();
	}

	function createPreview(file: File): Promise<string> {
		return new Promise((resolve, reject) => {
			const reader = new FileReader();
			reader.onload = () => resolve(reader.result as string);
			reader.onerror = () => reject(new Error('Failed to read file'));
			reader.readAsDataURL(file);
		});
	}

	function removePhoto(id: string): void {
		const photo = photoFiles.find((p) => p.id === id);
		if (photo) {
			URL.revokeObjectURL(photo.preview);
		}
		photoFiles = photoFiles.filter((p) => p.id !== id);
		syncFiles();
		validationError = null;
	}

	function syncFiles(): void {
		files = photoFiles.map((p) => p.file);
	}

	function handleDragEnter(e: DragEvent): void {
		e.preventDefault();
		e.stopPropagation();
		if (!disabled) {
			isDragging = true;
		}
	}

	function handleDragLeave(e: DragEvent): void {
		e.preventDefault();
		e.stopPropagation();
		isDragging = false;
	}

	function handleDragOver(e: DragEvent): void {
		e.preventDefault();
		e.stopPropagation();
	}

	function handleDrop(e: DragEvent): void {
		e.preventDefault();
		e.stopPropagation();
		isDragging = false;

		if (disabled || !canAddMore) return;

		const droppedFiles = e.dataTransfer?.files;
		if (droppedFiles && droppedFiles.length > 0) {
			processFiles(droppedFiles);
		}
	}

	function handleFileInputChange(e: Event): void {
		const input = e.target as HTMLInputElement;
		if (input.files && input.files.length > 0) {
			processFiles(input.files);
		}
		input.value = '';
	}

	function openFilePicker(): void {
		if (!disabled && canAddMore) {
			fileInput?.click();
		}
	}

	function handleKeyDown(e: KeyboardEvent): void {
		if (e.key === 'Enter' || e.key === ' ') {
			e.preventDefault();
			openFilePicker();
		}
	}
</script>

<div
	class="photo-upload-wrapper {className}"
	class:has-error={!!displayError}
	class:is-disabled={disabled}
>
	{#if label}
		<span class="upload-label">
			{label}
			{#if required}
				<span class="required-indicator" aria-hidden="true">*</span>
			{/if}
		</span>
	{/if}

	<input
		bind:this={fileInput}
		type="file"
		{accept}
		multiple={maxFiles > 1}
		{disabled}
		class="file-input"
		onchange={handleFileInputChange}
		aria-hidden="true"
		tabindex="-1"
	/>

	{#if canAddMore}
		<div
			class="dropzone"
			class:is-dragging={isDragging}
			class:is-disabled={disabled}
			role="button"
			tabindex={disabled ? -1 : 0}
			aria-label="Upload photos. Click or drag and drop files here."
			aria-describedby={displayError ? errorId : undefined}
			ondragenter={handleDragEnter}
			ondragleave={handleDragLeave}
			ondragover={handleDragOver}
			ondrop={handleDrop}
			onclick={openFilePicker}
			onkeydown={handleKeyDown}
		>
			<div class="dropzone-content">
				<svg
					class="upload-icon"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
					aria-hidden="true"
				>
					<path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
					<polyline points="17 8 12 3 7 8" />
					<line x1="12" y1="3" x2="12" y2="15" />
				</svg>
				<p class="dropzone-text">
					<span class="dropzone-primary">Click to upload</span>
					<span class="dropzone-secondary">or drag and drop</span>
				</p>
				<p class="dropzone-hint">
					{#if maxFiles > 1}
						Up to {maxFiles} images, max {formatFileSize(maxSize)} each
					{:else}
						Max {formatFileSize(maxSize)}
					{/if}
				</p>
			</div>
		</div>
	{/if}

	{#if photoFiles.length > 0}
		<div class="previews" role="list" aria-label="Uploaded photos">
			{#each photoFiles as photo (photo.id)}
				<div class="preview-item" role="listitem">
					<img src={photo.preview} alt={photo.file.name} class="preview-image" />
					<button
						type="button"
						class="remove-btn"
						onclick={() => removePhoto(photo.id)}
						aria-label="Remove {photo.file.name}"
						{disabled}
					>
						<svg
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
						>
							<line x1="18" y1="6" x2="6" y2="18" />
							<line x1="6" y1="6" x2="18" y2="18" />
						</svg>
					</button>
					<span class="preview-name">{photo.file.name}</span>
				</div>
			{/each}
		</div>
	{/if}

	{#if displayError}
		<p id={errorId} class="upload-error" role="alert">
			{displayError}
		</p>
	{/if}
</div>

<style>
	.photo-upload-wrapper {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm, 0.5rem);
	}

	.upload-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.required-indicator {
		color: var(--color-rush, #ef4444);
		margin-left: 0.125rem;
	}

	.file-input {
		position: absolute;
		width: 1px;
		height: 1px;
		padding: 0;
		margin: -1px;
		overflow: hidden;
		clip: rect(0, 0, 0, 0);
		white-space: nowrap;
		border: 0;
	}

	.dropzone {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: var(--space-xl, 2rem);
		border: 2px dashed var(--color-border, #e2e8f0);
		border-radius: var(--radius-lg, 0.75rem);
		background-color: var(--color-surface, #ffffff);
		cursor: pointer;
		transition:
			border-color var(--transition-fast, 150ms ease),
			background-color var(--transition-fast, 150ms ease);
	}

	.dropzone:hover:not(.is-disabled) {
		border-color: var(--color-primary-light, #3b82f6);
		background-color: var(--color-bg, #f8fafc);
	}

	.dropzone:focus-visible {
		outline: 2px solid var(--color-primary, #1e40af);
		outline-offset: 2px;
	}

	.dropzone.is-dragging {
		border-color: var(--color-primary, #1e40af);
		background-color: rgba(30, 64, 175, 0.05);
		border-style: solid;
	}

	.dropzone.is-disabled {
		opacity: 0.6;
		cursor: not-allowed;
		background-color: var(--color-bg, #f8fafc);
	}

	.dropzone-content {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
		text-align: center;
	}

	.upload-icon {
		width: 2.5rem;
		height: 2.5rem;
		color: var(--color-text-muted, #64748b);
	}

	.dropzone-text {
		margin: 0;
		font-size: 0.875rem;
	}

	.dropzone-primary {
		font-weight: 500;
		color: var(--color-primary, #1e40af);
	}

	.dropzone-secondary {
		color: var(--color-text-muted, #64748b);
		margin-left: 0.25rem;
	}

	.dropzone-hint {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
	}

	.previews {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(100px, 1fr));
		gap: var(--space-sm, 0.5rem);
	}

	.preview-item {
		position: relative;
		aspect-ratio: 1;
		border-radius: var(--radius-md, 0.5rem);
		overflow: hidden;
		background-color: var(--color-bg, #f8fafc);
	}

	.preview-image {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.remove-btn {
		position: absolute;
		top: 0.25rem;
		right: 0.25rem;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 1.5rem;
		height: 1.5rem;
		padding: 0;
		background-color: rgba(0, 0, 0, 0.6);
		border: none;
		border-radius: 50%;
		color: white;
		cursor: pointer;
		opacity: 0;
		transition: opacity var(--transition-fast, 150ms ease);
	}

	.preview-item:hover .remove-btn,
	.remove-btn:focus-visible {
		opacity: 1;
	}

	.remove-btn:focus-visible {
		outline: 2px solid white;
		outline-offset: 1px;
	}

	.remove-btn:disabled {
		cursor: not-allowed;
	}

	.remove-btn svg {
		width: 0.875rem;
		height: 0.875rem;
	}

	.preview-name {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		padding: 0.25rem;
		background: linear-gradient(transparent, rgba(0, 0, 0, 0.7));
		color: white;
		font-size: 0.625rem;
		text-overflow: ellipsis;
		white-space: nowrap;
		overflow: hidden;
	}

	/* Error state */
	.has-error .dropzone {
		border-color: var(--color-rush, #ef4444);
	}

	.upload-error {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		line-height: 1.4;
	}

	/* Disabled state */
	.is-disabled .upload-label {
		color: var(--color-text-muted, #64748b);
	}
</style>

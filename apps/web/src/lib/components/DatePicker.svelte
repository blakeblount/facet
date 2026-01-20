<script lang="ts">
	interface Props {
		/** Label text displayed above the date picker */
		label?: string;
		/** Current value of the date picker (ISO string, two-way bindable) */
		value?: string;
		/** Minimum selectable date (ISO string) */
		minDate?: string;
		/** Maximum selectable date (ISO string) */
		maxDate?: string;
		/** Error message to display below the picker */
		error?: string;
		/** Whether the picker is disabled */
		disabled?: boolean;
		/** Whether the picker is required */
		required?: boolean;
		/** Input name attribute for forms */
		name?: string;
		/** Placeholder text shown when no date is selected */
		placeholder?: string;
		/** Additional CSS class for the wrapper */
		class?: string;
		/** Unique ID for the picker (auto-generated if not provided) */
		id?: string;
	}

	let {
		label,
		value = $bindable(''),
		minDate,
		maxDate,
		error,
		disabled = false,
		required = false,
		name,
		placeholder = 'Select a date',
		class: className = '',
		id
	}: Props = $props();

	// Generate a stable unique ID if not provided
	const generatedId = `datepicker-${Math.random().toString(36).substring(2, 9)}`;
	const pickerId = $derived(id ?? generatedId);
	const errorId = $derived(`${pickerId}-error`);
	const calendarId = $derived(`${pickerId}-calendar`);

	// Component state
	let isOpen = $state(false);
	let viewDate = $state(new Date()); // The month being viewed
	let buttonRef: HTMLButtonElement | undefined = $state();
	let calendarRef: HTMLDivElement | undefined = $state();
	let focusedDate = $state<Date | null>(null);

	// Parse value to Date
	const selectedDate = $derived(value ? new Date(value + 'T00:00:00') : null);

	// Parse min/max dates
	const minDateObj = $derived(minDate ? new Date(minDate + 'T00:00:00') : null);
	const maxDateObj = $derived(maxDate ? new Date(maxDate + 'T00:00:00') : null);

	// Format date for display
	const displayText = $derived(
		selectedDate
			? selectedDate.toLocaleDateString('en-US', {
					weekday: 'short',
					year: 'numeric',
					month: 'short',
					day: 'numeric'
				})
			: placeholder
	);

	// Calendar helpers
	const DAYS = ['Su', 'Mo', 'Tu', 'We', 'Th', 'Fr', 'Sa'];
	const MONTHS = [
		'January',
		'February',
		'March',
		'April',
		'May',
		'June',
		'July',
		'August',
		'September',
		'October',
		'November',
		'December'
	];

	const viewMonth = $derived(viewDate.getMonth());
	const viewYear = $derived(viewDate.getFullYear());
	const monthLabel = $derived(`${MONTHS[viewMonth]} ${viewYear}`);

	// Get days to display in calendar grid
	const calendarDays = $derived.by(() => {
		const days: Array<{ key: string; date: Date; isCurrentMonth: boolean; isDisabled: boolean }> = [];

		// First day of the month
		const firstDay = new Date(viewYear, viewMonth, 1);
		// Last day of the month
		const lastDay = new Date(viewYear, viewMonth + 1, 0);

		// Start from the Sunday of the week containing the first day
		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- non-reactive computation
		const startDate = new Date(firstDay);
		startDate.setDate(startDate.getDate() - startDate.getDay());

		// End on the Saturday of the week containing the last day
		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- non-reactive computation
		const endDate = new Date(lastDay);
		endDate.setDate(endDate.getDate() + (6 - endDate.getDay()));

		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- non-reactive computation
		const current = new Date(startDate);
		while (current <= endDate) {
			const isCurrentMonth = current.getMonth() === viewMonth;
			const isDisabled = isDateDisabled(current);
			const dateKey = `${current.getFullYear()}-${current.getMonth()}-${current.getDate()}`;
			days.push({
				key: dateKey,
				date: new Date(current),
				isCurrentMonth,
				isDisabled
			});
			current.setDate(current.getDate() + 1);
		}

		return days;
	});

	function isDateDisabled(date: Date): boolean {
		if (minDateObj && date < minDateObj) return true;
		if (maxDateObj && date > maxDateObj) return true;
		return false;
	}

	function isSameDay(a: Date | null, b: Date | null): boolean {
		if (!a || !b) return false;
		return (
			a.getFullYear() === b.getFullYear() &&
			a.getMonth() === b.getMonth() &&
			a.getDate() === b.getDate()
		);
	}

	function isToday(date: Date): boolean {
		return isSameDay(date, new Date());
	}

	function formatISODate(date: Date): string {
		const year = date.getFullYear();
		const month = String(date.getMonth() + 1).padStart(2, '0');
		const day = String(date.getDate()).padStart(2, '0');
		return `${year}-${month}-${day}`;
	}

	function open() {
		if (disabled) return;
		isOpen = true;
		// Set view to selected date's month or current month
		viewDate = selectedDate ? new Date(selectedDate) : new Date();
		focusedDate = selectedDate ? new Date(selectedDate) : new Date();
	}

	function close() {
		isOpen = false;
		focusedDate = null;
	}

	function toggle() {
		if (isOpen) {
			close();
		} else {
			open();
		}
	}

	function selectDate(date: Date) {
		if (isDateDisabled(date)) return;
		value = formatISODate(date);
		close();
		buttonRef?.focus();
	}

	function prevMonth() {
		viewDate = new Date(viewYear, viewMonth - 1, 1);
	}

	function nextMonth() {
		viewDate = new Date(viewYear, viewMonth + 1, 1);
	}

	function handleKeydown(event: KeyboardEvent) {
		if (disabled) return;

		if (!isOpen) {
			if (event.key === 'Enter' || event.key === ' ' || event.key === 'ArrowDown') {
				event.preventDefault();
				open();
			}
			return;
		}

		switch (event.key) {
			case 'Escape':
				event.preventDefault();
				close();
				buttonRef?.focus();
				break;

			case 'Enter':
			case ' ':
				event.preventDefault();
				if (focusedDate && !isDateDisabled(focusedDate)) {
					selectDate(focusedDate);
				}
				break;

			case 'ArrowLeft':
				event.preventDefault();
				moveFocus(-1);
				break;

			case 'ArrowRight':
				event.preventDefault();
				moveFocus(1);
				break;

			case 'ArrowUp':
				event.preventDefault();
				moveFocus(-7);
				break;

			case 'ArrowDown':
				event.preventDefault();
				moveFocus(7);
				break;

			case 'Home':
				event.preventDefault();
				// Go to first day of month
				focusedDate = new Date(viewYear, viewMonth, 1);
				break;

			case 'End':
				event.preventDefault();
				// Go to last day of month
				focusedDate = new Date(viewYear, viewMonth + 1, 0);
				break;

			case 'PageUp':
				event.preventDefault();
				if (event.shiftKey) {
					// Previous year
					viewDate = new Date(viewYear - 1, viewMonth, 1);
					if (focusedDate) {
						focusedDate = new Date(
							focusedDate.getFullYear() - 1,
							focusedDate.getMonth(),
							focusedDate.getDate()
						);
					}
				} else {
					// Previous month
					prevMonth();
					if (focusedDate) {
						focusedDate = new Date(
							focusedDate.getFullYear(),
							focusedDate.getMonth() - 1,
							focusedDate.getDate()
						);
					}
				}
				break;

			case 'PageDown':
				event.preventDefault();
				if (event.shiftKey) {
					// Next year
					viewDate = new Date(viewYear + 1, viewMonth, 1);
					if (focusedDate) {
						focusedDate = new Date(
							focusedDate.getFullYear() + 1,
							focusedDate.getMonth(),
							focusedDate.getDate()
						);
					}
				} else {
					// Next month
					nextMonth();
					if (focusedDate) {
						focusedDate = new Date(
							focusedDate.getFullYear(),
							focusedDate.getMonth() + 1,
							focusedDate.getDate()
						);
					}
				}
				break;

			case 'Tab':
				close();
				break;
		}
	}

	function moveFocus(days: number) {
		if (!focusedDate) {
			focusedDate = selectedDate ? new Date(selectedDate) : new Date();
			return;
		}

		// eslint-disable-next-line svelte/prefer-svelte-reactivity -- event handler, not reactive
		const newDate = new Date(focusedDate);
		newDate.setDate(newDate.getDate() + days);
		focusedDate = newDate;

		// Update view if needed
		if (newDate.getMonth() !== viewMonth || newDate.getFullYear() !== viewYear) {
			viewDate = new Date(newDate.getFullYear(), newDate.getMonth(), 1);
		}
	}

	function handleClickOutside(event: MouseEvent) {
		const target = event.target as Node;
		if (buttonRef && !buttonRef.contains(target) && calendarRef && !calendarRef.contains(target)) {
			close();
		}
	}

	// Close on click outside
	$effect(() => {
		if (isOpen) {
			document.addEventListener('click', handleClickOutside, true);
			return () => {
				document.removeEventListener('click', handleClickOutside, true);
			};
		}
	});
</script>

<div class="datepicker-wrapper {className}" class:has-error={!!error} class:is-disabled={disabled}>
	{#if label}
		<label for={pickerId} class="datepicker-label">
			{label}
			{#if required}
				<span class="required-indicator" aria-hidden="true">*</span>
			{/if}
		</label>
	{/if}

	<!-- Hidden native input for form submission -->
	{#if name}
		<input type="hidden" {name} {value} />
	{/if}

	<div class="datepicker-container">
		<button
			bind:this={buttonRef}
			type="button"
			id={pickerId}
			class="datepicker-trigger"
			class:is-placeholder={!selectedDate}
			aria-haspopup="dialog"
			aria-expanded={isOpen}
			aria-controls={calendarId}
			aria-describedby={error ? errorId : undefined}
			{disabled}
			onclick={toggle}
			onkeydown={handleKeydown}
		>
			<span class="datepicker-value">{displayText}</span>
			<span class="datepicker-icon" aria-hidden="true">
				<svg
					width="16"
					height="16"
					viewBox="0 0 16 16"
					fill="none"
					stroke="currentColor"
					stroke-width="1.5"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<rect x="2" y="3" width="12" height="11" rx="1" />
					<path d="M5 1v3" />
					<path d="M11 1v3" />
					<path d="M2 7h12" />
				</svg>
			</span>
		</button>

		{#if isOpen}
			<div
				bind:this={calendarRef}
				id={calendarId}
				class="calendar-popup"
				role="dialog"
				aria-modal="true"
				aria-label="Choose date"
			>
				<div class="calendar-header">
					<button
						type="button"
						class="calendar-nav-btn"
						onclick={prevMonth}
						aria-label="Previous month"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 16 16"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M10 12L6 8L10 4" />
						</svg>
					</button>
					<span class="calendar-month-label">{monthLabel}</span>
					<button
						type="button"
						class="calendar-nav-btn"
						onclick={nextMonth}
						aria-label="Next month"
					>
						<svg
							width="16"
							height="16"
							viewBox="0 0 16 16"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
							stroke-linecap="round"
							stroke-linejoin="round"
						>
							<path d="M6 12L10 8L6 4" />
						</svg>
					</button>
				</div>

				<div class="calendar-grid" role="grid" aria-label={monthLabel}>
					<div class="calendar-weekdays" role="row">
						{#each DAYS as day (day)}
							<span class="calendar-weekday" role="columnheader">{day}</span>
						{/each}
					</div>

					<div class="calendar-days">
						{#each calendarDays as { key, date, isCurrentMonth, isDisabled } (key)}
							<button
								type="button"
								role="gridcell"
								class="calendar-day"
								class:is-other-month={!isCurrentMonth}
								class:is-today={isToday(date)}
								class:is-selected={isSameDay(date, selectedDate)}
								class:is-focused={isSameDay(date, focusedDate)}
								class:is-disabled={isDisabled}
								disabled={isDisabled}
								tabindex={isSameDay(date, focusedDate) ? 0 : -1}
								aria-label={date.toLocaleDateString('en-US', {
									weekday: 'long',
									year: 'numeric',
									month: 'long',
									day: 'numeric'
								})}
								aria-selected={isSameDay(date, selectedDate)}
								aria-current={isToday(date) ? 'date' : undefined}
								onclick={() => selectDate(date)}
								onkeydown={handleKeydown}
							>
								{date.getDate()}
							</button>
						{/each}
					</div>
				</div>

				<div class="calendar-footer">
					<button type="button" class="calendar-today-btn" onclick={() => selectDate(new Date())}>
						Today
					</button>
				</div>
			</div>
		{/if}
	</div>

	{#if error}
		<p id={errorId} class="datepicker-error" role="alert">
			{error}
		</p>
	{/if}
</div>

<style>
	.datepicker-wrapper {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.datepicker-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.required-indicator {
		color: var(--color-rush, #ef4444);
		margin-left: 0.125rem;
	}

	.datepicker-container {
		position: relative;
	}

	.datepicker-trigger {
		display: flex;
		align-items: center;
		justify-content: space-between;
		width: 100%;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		font-size: 0.875rem;
		font-family: inherit;
		line-height: 1.5;
		text-align: left;
		color: var(--color-text, #1e293b);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		cursor: pointer;
		transition:
			border-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.datepicker-trigger.is-placeholder {
		color: var(--color-text-muted, #64748b);
	}

	.datepicker-trigger:hover:not(:disabled) {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.datepicker-trigger:focus {
		outline: none;
		border-color: var(--color-primary, #1e40af);
		box-shadow: 0 0 0 3px rgba(30, 64, 175, 0.15);
	}

	.datepicker-trigger:disabled {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text-muted, #64748b);
		cursor: not-allowed;
		opacity: 0.7;
	}

	.datepicker-value {
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}

	.datepicker-icon {
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		color: var(--color-text-muted, #64748b);
	}

	/* Calendar Popup */
	.calendar-popup {
		position: absolute;
		top: calc(100% + 4px);
		left: 0;
		z-index: 50;
		width: 280px;
		padding: var(--space-sm, 0.5rem);
		background-color: var(--color-surface, #ffffff);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
		box-shadow: var(--shadow-lg, 0 10px 15px -3px rgb(0 0 0 / 0.1));
	}

	.calendar-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-xs, 0.25rem) var(--space-xs, 0.25rem) var(--space-sm, 0.5rem);
	}

	.calendar-month-label {
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text, #1e293b);
	}

	.calendar-nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		padding: 0;
		color: var(--color-text-muted, #64748b);
		background: none;
		border: none;
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.calendar-nav-btn:hover {
		background-color: var(--color-bg, #f8fafc);
		color: var(--color-text, #1e293b);
	}

	.calendar-nav-btn:focus {
		outline: none;
		box-shadow: 0 0 0 2px rgba(30, 64, 175, 0.3);
	}

	.calendar-grid {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.calendar-weekdays {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 2px;
	}

	.calendar-weekday {
		display: flex;
		align-items: center;
		justify-content: center;
		height: 28px;
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--color-text-muted, #64748b);
		text-transform: uppercase;
	}

	.calendar-days {
		display: grid;
		grid-template-columns: repeat(7, 1fr);
		gap: 2px;
	}

	.calendar-day {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 36px;
		height: 36px;
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
		background: none;
		border: none;
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.calendar-day:hover:not(:disabled) {
		background-color: var(--color-bg, #f8fafc);
	}

	.calendar-day:focus {
		outline: none;
	}

	.calendar-day.is-focused {
		box-shadow: inset 0 0 0 2px var(--color-primary, #1e40af);
	}

	.calendar-day.is-other-month {
		color: var(--color-text-muted, #64748b);
		opacity: 0.5;
	}

	.calendar-day.is-today {
		font-weight: 600;
		color: var(--color-primary, #1e40af);
	}

	.calendar-day.is-selected {
		background-color: var(--color-primary, #1e40af);
		color: white;
		font-weight: 500;
	}

	.calendar-day.is-selected:hover {
		background-color: var(--color-primary-dark, #1e3a8a);
	}

	.calendar-day.is-disabled {
		color: var(--color-text-muted, #64748b);
		opacity: 0.3;
		cursor: not-allowed;
	}

	.calendar-footer {
		display: flex;
		justify-content: center;
		padding-top: var(--space-sm, 0.5rem);
		border-top: 1px solid var(--color-border, #e2e8f0);
		margin-top: var(--space-sm, 0.5rem);
	}

	.calendar-today-btn {
		padding: var(--space-xs, 0.25rem) var(--space-md, 1rem);
		font-size: 0.75rem;
		font-weight: 500;
		color: var(--color-primary, #1e40af);
		background: none;
		border: none;
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.calendar-today-btn:hover {
		background-color: var(--color-bg, #f8fafc);
	}

	.calendar-today-btn:focus {
		outline: none;
		box-shadow: 0 0 0 2px rgba(30, 64, 175, 0.3);
	}

	/* Error state */
	.has-error .datepicker-trigger {
		border-color: var(--color-rush, #ef4444);
	}

	.has-error .datepicker-trigger:focus {
		border-color: var(--color-rush, #ef4444);
		box-shadow: 0 0 0 3px rgba(239, 68, 68, 0.15);
	}

	.datepicker-error {
		margin: 0;
		font-size: 0.75rem;
		color: var(--color-rush, #ef4444);
		line-height: 1.4;
	}

	/* Disabled state */
	.is-disabled .datepicker-label {
		color: var(--color-text-muted, #64748b);
	}
</style>

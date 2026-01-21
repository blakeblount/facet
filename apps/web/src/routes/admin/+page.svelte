<script lang="ts">
	import { onMount } from 'svelte';
	import type { PageData } from './$types';
	import type { EmployeeSummary, StorageLocationSummary, StoreSettings } from '$lib/types/api';
	import {
		themeStore,
		THEMES,
		THEME_NAMES,
		THEME_DESCRIPTIONS,
		type Theme
	} from '$lib/stores/theme.svelte';
	import { adminAuthStore } from '$lib/stores/adminAuth.svelte';
	import {
		listEmployees,
		listStorageLocations,
		getStoreSettings,
		ApiClientError
	} from '$lib/services/api';
	import AdminPinModal from '$lib/components/AdminPinModal.svelte';
	import StorageLocationModal from '$lib/components/StorageLocationModal.svelte';
	import EmployeeModal from '$lib/components/EmployeeModal.svelte';
	import StoreInfoModal from '$lib/components/StoreInfoModal.svelte';

	let { data }: { data: PageData } = $props();

	// Employee list state (fetched client-side after auth)
	let employees = $state<EmployeeSummary[]>([]);
	let employeesLoading = $state(false);
	let employeesError = $state<string | null>(null);

	// Storage locations state (fetched client-side for real-time updates)
	let locations = $state<StorageLocationSummary[]>(data.locations || []);

	// Store settings state (fetched client-side for real-time updates)
	let settings = $state<StoreSettings | null>(data.settings || null);

	// Whether to show the auth modal
	let showAuthModal = $state(false);

	// Modal states
	let showLocationModal = $state(false);
	let editingLocation = $state<StorageLocationSummary | null>(null);

	let showEmployeeModal = $state(false);
	let editingEmployee = $state<EmployeeSummary | null>(null);

	let showStoreInfoModal = $state(false);

	// Initialize auth store on mount
	onMount(() => {
		adminAuthStore.init();

		// If already authenticated, fetch employees
		if (adminAuthStore.isAuthenticated) {
			fetchEmployees();
		} else {
			// Show auth modal on first load
			showAuthModal = true;
		}
	});

	// When auth state changes, fetch employees
	$effect(() => {
		if (adminAuthStore.isAuthenticated) {
			fetchEmployees();
		}
	});

	async function fetchEmployees() {
		if (!adminAuthStore.isAuthenticated) return;

		employeesLoading = true;
		employeesError = null;

		try {
			const response = await listEmployees(true); // Include inactive
			employees = response.employees || [];
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isCode('UNAUTHORIZED') || err.isCode('INVALID_PIN')) {
					// Session is no longer valid, logout and show auth modal
					adminAuthStore.logout();
					showAuthModal = true;
					employeesError = null;
				} else {
					employeesError = err.message || 'Failed to load employees.';
				}
			} else {
				employeesError = 'Failed to load employees.';
			}
		} finally {
			employeesLoading = false;
		}
	}

	async function fetchLocations() {
		try {
			const response = await listStorageLocations(true); // Include inactive
			locations = response.locations || [];
		} catch {
			// Silently fail - we already have initial data
		}
	}

	async function fetchSettings() {
		try {
			settings = await getStoreSettings();
		} catch {
			// Silently fail - we already have initial data
		}
	}

	function handleThemeChange(theme: Theme) {
		themeStore.set(theme);
	}

	function handleAuthSuccess() {
		showAuthModal = false;
	}

	function handleAuthClose() {
		showAuthModal = false;
	}

	function handleLogout() {
		adminAuthStore.logout();
		employees = [];
		showAuthModal = true;
	}

	// Location modal handlers
	function handleAddLocation() {
		editingLocation = null;
		showLocationModal = true;
	}

	function handleEditLocation(location: StorageLocationSummary) {
		editingLocation = location;
		showLocationModal = true;
	}

	function handleLocationModalClose() {
		showLocationModal = false;
		editingLocation = null;
	}

	function handleLocationSuccess() {
		showLocationModal = false;
		editingLocation = null;
		fetchLocations();
	}

	// Employee modal handlers
	function handleAddEmployee() {
		editingEmployee = null;
		showEmployeeModal = true;
	}

	function handleEditEmployee(employee: EmployeeSummary) {
		editingEmployee = employee;
		showEmployeeModal = true;
	}

	function handleEmployeeModalClose() {
		showEmployeeModal = false;
		editingEmployee = null;
	}

	function handleEmployeeSuccess() {
		showEmployeeModal = false;
		editingEmployee = null;
		fetchEmployees();
	}

	// Store info modal handlers
	function handleEditStoreInfo() {
		showStoreInfoModal = true;
	}

	function handleStoreInfoModalClose() {
		showStoreInfoModal = false;
	}

	function handleStoreInfoSuccess() {
		showStoreInfoModal = false;
		fetchSettings();
	}
</script>

<AdminPinModal open={showAuthModal} onClose={handleAuthClose} onSuccess={handleAuthSuccess} />
<StorageLocationModal
	open={showLocationModal}
	location={editingLocation}
	onClose={handleLocationModalClose}
	onSuccess={handleLocationSuccess}
/>
<EmployeeModal
	open={showEmployeeModal}
	employee={editingEmployee}
	onClose={handleEmployeeModalClose}
	onSuccess={handleEmployeeSuccess}
/>
<StoreInfoModal
	open={showStoreInfoModal}
	{settings}
	onClose={handleStoreInfoModalClose}
	onSuccess={handleStoreInfoSuccess}
/>

<div class="admin-page">
	<div class="admin-header">
		<div class="header-content">
			<h1 class="page-title">Admin Settings</h1>
			<p class="page-subtitle">Manage store settings, employees, and storage locations.</p>
		</div>
		{#if adminAuthStore.isAuthenticated}
			<button class="logout-button" onclick={handleLogout} title="Lock admin access">
				<svg
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
					<rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
					<path d="M7 11V7a5 5 0 0 1 10 0v4" />
				</svg>
				Lock
			</button>
		{/if}
	</div>

	{#if data.error}
		<div class="error-message">
			<p>Failed to load settings: {data.error}</p>
		</div>
	{:else}
		<div class="admin-sections">
			<section class="admin-section">
				<div class="section-header">
					<h2 class="section-title">Appearance</h2>
				</div>
				<div class="section-content">
					<div class="theme-toggle">
						{#each THEMES as theme (theme)}
							<button
								class="theme-option"
								class:active={themeStore.current === theme}
								onclick={() => handleThemeChange(theme)}
							>
								<span class="theme-name">{THEME_NAMES[theme]}</span>
								<span class="theme-description">{THEME_DESCRIPTIONS[theme]}</span>
							</button>
						{/each}
					</div>
				</div>
			</section>

			<section class="admin-section">
				<div class="section-header">
					<h2 class="section-title">Store Information</h2>
					{#if adminAuthStore.isAuthenticated}
						<button
							class="edit-button"
							onclick={handleEditStoreInfo}
							title="Edit store information"
						>
							<svg
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
								<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
								<path d="m15 5 4 4" />
							</svg>
							Edit
						</button>
					{/if}
				</div>
				<div class="section-content">
					{#if settings}
						<dl class="info-list">
							<div class="info-item">
								<dt>Store Name</dt>
								<dd>{settings.store_name || 'Not configured'}</dd>
							</div>
							<div class="info-item">
								<dt>Phone</dt>
								<dd>{settings.store_phone || 'Not set'}</dd>
							</div>
							<div class="info-item">
								<dt>Address</dt>
								<dd>{settings.store_address || 'Not set'}</dd>
							</div>
						</dl>
					{:else}
						<p class="placeholder-text">Store settings not yet configured.</p>
					{/if}
				</div>
			</section>

			<section class="admin-section">
				<div class="section-header">
					<h2 class="section-title">Employees</h2>
					{#if !adminAuthStore.isAuthenticated}
						<span class="auth-required-badge">Authentication Required</span>
					{:else}
						<button class="add-button" onclick={handleAddEmployee} title="Add new employee">
							<svg
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
								<path d="M12 5v14" />
								<path d="M5 12h14" />
							</svg>
							Add
						</button>
					{/if}
				</div>
				<div class="section-content">
					{#if !adminAuthStore.isAuthenticated}
						<div class="auth-prompt">
							<p class="placeholder-text">Enter admin PIN to view and manage employees.</p>
							<button class="unlock-button" onclick={() => (showAuthModal = true)}>
								<svg
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
									<rect width="18" height="11" x="3" y="11" rx="2" ry="2" />
									<path d="M7 11V7a5 5 0 0 1 9.9-1" />
								</svg>
								Unlock
							</button>
						</div>
					{:else if employeesLoading}
						<p class="placeholder-text">Loading employees...</p>
					{:else if employeesError}
						<div class="error-inline">
							<p>{employeesError}</p>
							<button class="retry-button" onclick={fetchEmployees}>Retry</button>
						</div>
					{:else if employees.length > 0}
						<ul class="employee-list">
							{#each employees as employee (employee.employee_id)}
								<li class="employee-item">
									<div class="employee-info">
										<span class="employee-name">{employee.name}</span>
										<span class="employee-role role-{employee.role}">{employee.role}</span>
										{#if !employee.is_active}
											<span class="employee-inactive">Inactive</span>
										{/if}
									</div>
									<button
										class="item-edit-button"
										onclick={() => handleEditEmployee(employee)}
										title="Edit employee"
									>
										<svg
											xmlns="http://www.w3.org/2000/svg"
											width="14"
											height="14"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2"
											stroke-linecap="round"
											stroke-linejoin="round"
										>
											<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
											<path d="m15 5 4 4" />
										</svg>
									</button>
								</li>
							{/each}
						</ul>
					{:else}
						<p class="placeholder-text">No employees configured yet.</p>
					{/if}
				</div>
			</section>

			<section class="admin-section">
				<div class="section-header">
					<h2 class="section-title">Storage Locations</h2>
					{#if adminAuthStore.isAuthenticated}
						<button class="add-button" onclick={handleAddLocation} title="Add new storage location">
							<svg
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
								<path d="M12 5v14" />
								<path d="M5 12h14" />
							</svg>
							Add
						</button>
					{/if}
				</div>
				<div class="section-content">
					{#if locations && locations.length > 0}
						<ul class="location-list">
							{#each locations as location (location.location_id)}
								<li class="location-item">
									<div class="location-info">
										<span class="location-name">{location.name}</span>
										{#if !location.is_active}
											<span class="location-inactive">Inactive</span>
										{/if}
									</div>
									{#if adminAuthStore.isAuthenticated}
										<button
											class="item-edit-button"
											onclick={() => handleEditLocation(location)}
											title="Edit location"
										>
											<svg
												xmlns="http://www.w3.org/2000/svg"
												width="14"
												height="14"
												viewBox="0 0 24 24"
												fill="none"
												stroke="currentColor"
												stroke-width="2"
												stroke-linecap="round"
												stroke-linejoin="round"
											>
												<path d="M17 3a2.85 2.83 0 1 1 4 4L7.5 20.5 2 22l1.5-5.5Z" />
												<path d="m15 5 4 4" />
											</svg>
										</button>
									{/if}
								</li>
							{/each}
						</ul>
					{:else}
						<p class="placeholder-text">No storage locations configured yet.</p>
					{/if}
				</div>
			</section>
		</div>
	{/if}
</div>

<style>
	.admin-page {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
		max-width: 900px;
		margin: 0 auto;
	}

	.admin-header {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-md);
		margin-bottom: var(--space-sm);
	}

	.header-content {
		flex: 1;
	}

	.logout-button {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: var(--space-sm) var(--space-md);
		background-color: var(--color-bg);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		color: var(--color-text-muted);
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			border-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.logout-button:hover {
		background-color: var(--color-bg-card);
		border-color: var(--color-text-muted);
		color: var(--color-text);
	}

	.page-title {
		font-size: 1.75rem;
		margin-bottom: var(--space-xs);
	}

	.page-subtitle {
		color: var(--color-text-muted);
	}

	.error-message {
		padding: var(--space-lg);
		background-color: #fef2f2;
		border: 1px solid #fecaca;
		border-radius: var(--radius-md);
		color: #991b1b;
	}

	.admin-sections {
		display: flex;
		flex-direction: column;
		gap: var(--space-lg);
	}

	.admin-section {
		background-color: var(--color-bg-card);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-lg);
		overflow: hidden;
	}

	.section-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-md) var(--space-lg);
		background-color: var(--color-bg);
		border-bottom: 1px solid var(--color-border);
	}

	.section-title {
		font-size: 1.125rem;
		font-weight: 600;
	}

	.section-content {
		padding: var(--space-lg);
	}

	.placeholder-text {
		color: var(--color-text-muted);
		font-style: italic;
	}

	.info-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-md);
		margin: 0;
	}

	.info-item {
		display: flex;
		gap: var(--space-md);
	}

	.info-item dt {
		flex: 0 0 150px;
		font-weight: 500;
		color: var(--color-text-muted);
	}

	.info-item dd {
		margin: 0;
	}

	.employee-list,
	.location-list {
		list-style: none;
		padding: 0;
		margin: 0;
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.employee-item,
	.location-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		background-color: var(--color-bg);
		border-radius: var(--radius-md);
	}

	.employee-info,
	.location-info {
		display: flex;
		align-items: center;
		gap: var(--space-md);
		flex: 1;
	}

	.employee-name,
	.location-name {
		font-weight: 500;
	}

	.employee-role {
		padding: var(--space-xs) var(--space-sm);
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: uppercase;
		border-radius: var(--radius-sm);
	}

	.role-admin {
		background-color: var(--color-primary);
		color: white;
	}

	.role-staff {
		background-color: var(--color-text-muted);
		color: white;
	}

	.employee-inactive,
	.location-inactive {
		margin-left: auto;
		padding: var(--space-xs) var(--space-sm);
		font-size: 0.75rem;
		background-color: #fef2f2;
		color: #991b1b;
		border-radius: var(--radius-sm);
	}

	.auth-required-badge {
		padding: var(--space-xs) var(--space-sm);
		font-size: 0.75rem;
		font-weight: 500;
		background-color: rgba(251, 191, 36, 0.1);
		color: #92400e;
		border: 1px solid #fbbf24;
		border-radius: var(--radius-sm);
	}

	.auth-prompt {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: var(--space-md);
	}

	.unlock-button {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: var(--space-sm) var(--space-md);
		background-color: var(--color-primary);
		border: none;
		border-radius: var(--radius-md);
		color: white;
		font-size: 0.875rem;
		font-weight: 500;
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.unlock-button:hover {
		background-color: var(--color-primary-dark, #1a2e4a);
	}

	.error-inline {
		display: flex;
		flex-direction: column;
		gap: var(--space-sm);
	}

	.error-inline p {
		margin: 0;
		color: #991b1b;
	}

	.retry-button {
		align-self: flex-start;
		padding: var(--space-xs) var(--space-sm);
		background-color: transparent;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text);
		font-size: 0.875rem;
		cursor: pointer;
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.retry-button:hover {
		background-color: var(--color-bg);
	}

	.theme-toggle {
		display: flex;
		gap: var(--space-md);
	}

	.theme-option {
		flex: 1;
		display: flex;
		flex-direction: column;
		gap: var(--space-xs);
		padding: var(--space-md);
		background-color: var(--color-bg);
		border: 2px solid var(--color-border);
		border-radius: var(--radius-md);
		cursor: pointer;
		text-align: left;
		transition:
			border-color var(--transition-fast, 150ms ease),
			background-color var(--transition-fast, 150ms ease);
	}

	.theme-option:hover {
		border-color: var(--color-primary-light, #3b82f6);
	}

	.theme-option.active {
		border-color: var(--color-primary);
		background-color: rgba(30, 64, 175, 0.05);
	}

	.theme-name {
		font-weight: 600;
		color: var(--color-text);
	}

	.theme-description {
		font-size: 0.875rem;
		color: var(--color-text-muted);
	}

	.add-button,
	.edit-button {
		display: flex;
		align-items: center;
		gap: var(--space-xs);
		padding: var(--space-xs) var(--space-sm);
		background-color: transparent;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-sm);
		color: var(--color-text-muted);
		font-size: 0.75rem;
		font-weight: 500;
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			border-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.add-button:hover,
	.edit-button:hover {
		background-color: var(--color-bg-card);
		border-color: var(--color-primary);
		color: var(--color-primary);
	}

	.item-edit-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		padding: 0;
		background-color: transparent;
		border: 1px solid transparent;
		border-radius: var(--radius-sm);
		color: var(--color-text-muted);
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			border-color var(--transition-fast, 150ms ease),
			color var(--transition-fast, 150ms ease);
	}

	.item-edit-button:hover {
		background-color: var(--color-bg-card);
		border-color: var(--color-border);
		color: var(--color-primary);
	}
</style>

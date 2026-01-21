<script lang="ts">
	import Modal from './Modal.svelte';
	import Input from './Input.svelte';
	import Button from './Button.svelte';
	import {
		verifyAdminPin,
		adminLogout,
		hasAdminSession,
		listEmployees,
		createEmployee,
		updateEmployee,
		deleteEmployee,
		getStoreSettings,
		updateStoreSettings,
		listStorageLocations,
		createStorageLocation,
		updateStorageLocation,
		ApiClientError,
		type EmployeeSummary,
		type EmployeeRole,
		type StoreSettings,
		type StorageLocationSummary
	} from '$lib/services/api';

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
	let authenticatedEmployee: { name: string; role: string } | null = $state(null);
	let pin = $state('');
	let pinError = $state('');
	let isVerifying = $state(false);
	let formEl: HTMLFormElement | undefined = $state();

	// Employees tab state
	let employees: EmployeeSummary[] = $state([]);
	let employeesLoading = $state(false);
	let employeesError = $state('');
	let showInactive = $state(false);

	// Employee form state
	let showEmployeeForm = $state(false);
	let editingEmployee: EmployeeSummary | null = $state(null);
	let employeeFormName = $state('');
	let employeeFormPin = $state('');
	let employeeFormRole: EmployeeRole = $state('staff');
	let employeeFormError = $state('');
	let employeeFormSaving = $state(false);

	// Delete confirmation state
	let deleteConfirmEmployee: EmployeeSummary | null = $state(null);
	let deleteError = $state('');
	let deleteLoading = $state(false);

	// Store info tab state
	let storeSettings: StoreSettings | null = $state(null);
	let storeInfoLoading = $state(false);
	let storeInfoError = $state('');
	let storeInfoSaving = $state(false);
	let storeInfoSuccess = $state('');

	// Store info form state (editable copies)
	let storeFormName = $state('');
	let storeFormPhone = $state('');
	let storeFormAddress = $state('');
	let storeFormTicketPrefix = $state('');

	// Locations tab state
	let locations: StorageLocationSummary[] = $state([]);
	let locationsLoading = $state(false);
	let locationsError = $state('');
	let showInactiveLocations = $state(false);

	// Location form state
	let showLocationForm = $state(false);
	let editingLocation: StorageLocationSummary | null = $state(null);
	let locationFormName = $state('');
	let locationFormError = $state('');
	let locationFormSaving = $state(false);

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

			// Reset employees state
			employees = [];
			employeesLoading = false;
			employeesError = '';
			showInactive = false;
			resetEmployeeForm();
			deleteConfirmEmployee = null;
			deleteError = '';
			deleteLoading = false;

			// Reset store info state
			storeSettings = null;
			storeInfoLoading = false;
			storeInfoError = '';
			storeInfoSaving = false;
			storeInfoSuccess = '';
			resetStoreInfoForm();

			// Reset locations state
			locations = [];
			locationsLoading = false;
			locationsError = '';
			showInactiveLocations = false;
			resetLocationForm();
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
			// Verify admin PIN and create session (session token is stored automatically)
			const response = await verifyAdminPin(pin);

			if (response.valid) {
				// Authentication successful - session is now active
				authenticatedEmployee = { name: 'Admin', role: 'admin' };
				isAuthenticated = true;
			}
		} catch (err) {
			if (err instanceof ApiClientError) {
				if (err.isCode('INVALID_PIN')) {
					pinError = 'Invalid admin PIN. Please try again.';
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

	async function handleClose() {
		if (!isVerifying) {
			// Log out when closing the settings modal
			await adminLogout();
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

	// =============================================================================
	// Store Info Tab Functions
	// =============================================================================

	function resetStoreInfoForm() {
		storeFormName = '';
		storeFormPhone = '';
		storeFormAddress = '';
		storeFormTicketPrefix = '';
	}

	function populateStoreInfoForm(settings: StoreSettings) {
		storeFormName = settings.store_name;
		storeFormPhone = settings.store_phone ?? '';
		storeFormAddress = settings.store_address ?? '';
		storeFormTicketPrefix = settings.ticket_prefix;
	}

	async function loadStoreSettings() {
		storeInfoLoading = true;
		storeInfoError = '';
		storeInfoSuccess = '';

		try {
			const settings = await getStoreSettings();
			storeSettings = settings;
			populateStoreInfoForm(settings);
		} catch (err) {
			if (err instanceof ApiClientError) {
				storeInfoError = err.message || 'Failed to load store settings';
			} else {
				storeInfoError = 'An error occurred while loading settings';
			}
		} finally {
			storeInfoLoading = false;
		}
	}

	// Load store settings when store-info tab becomes active
	$effect(() => {
		if (isAuthenticated && activeTab === 'store-info' && !storeSettings && !storeInfoLoading) {
			loadStoreSettings();
		}
	});

	async function handleSaveStoreInfo() {
		if (!hasAdminSession()) return;

		// Validate required fields
		if (!storeFormName.trim()) {
			storeInfoError = 'Store name is required';
			return;
		}

		if (!storeFormTicketPrefix.trim()) {
			storeInfoError = 'Ticket prefix is required';
			return;
		}

		storeInfoSaving = true;
		storeInfoError = '';
		storeInfoSuccess = '';

		try {
			const updates = {
				store_name: storeFormName.trim(),
				store_phone: storeFormPhone.trim() || undefined,
				store_address: storeFormAddress.trim() || undefined,
				ticket_prefix: storeFormTicketPrefix.trim()
			};

			const updatedSettings = await updateStoreSettings(updates);
			storeSettings = updatedSettings;
			populateStoreInfoForm(updatedSettings);
			storeInfoSuccess = 'Settings saved successfully';

			// Clear success message after 3 seconds
			setTimeout(() => {
				storeInfoSuccess = '';
			}, 3000);
		} catch (err) {
			if (err instanceof ApiClientError) {
				storeInfoError = err.message || 'Failed to save settings';
			} else {
				storeInfoError = 'An error occurred while saving settings';
			}
		} finally {
			storeInfoSaving = false;
		}
	}

	// =============================================================================
	// Employees Tab Functions
	// =============================================================================

	async function loadEmployees() {
		if (!hasAdminSession()) return;

		employeesLoading = true;
		employeesError = '';

		try {
			const response = await listEmployees(showInactive);
			employees = response.employees;
		} catch (err) {
			if (err instanceof ApiClientError) {
				employeesError = err.message || 'Failed to load employees';
			} else {
				employeesError = 'An error occurred while loading employees';
			}
		} finally {
			employeesLoading = false;
		}
	}

	// Load/reload employees when tab becomes active or showInactive changes
	// Using a derived value to track showInactive without unused variable warnings
	let currentShowInactive = $derived(showInactive);

	$effect(() => {
		// Access currentShowInactive to make this effect depend on it
		if (isAuthenticated && activeTab === 'employees' && currentShowInactive !== undefined) {
			loadEmployees();
		}
	});

	function resetEmployeeForm() {
		showEmployeeForm = false;
		editingEmployee = null;
		employeeFormName = '';
		employeeFormPin = '';
		employeeFormRole = 'staff';
		employeeFormError = '';
		employeeFormSaving = false;
	}

	function handleAddEmployee() {
		resetEmployeeForm();
		showEmployeeForm = true;
	}

	function handleEditEmployee(employee: EmployeeSummary) {
		resetEmployeeForm();
		editingEmployee = employee;
		employeeFormName = employee.name;
		employeeFormPin = ''; // Don't populate PIN for security
		employeeFormRole = employee.role;
		showEmployeeForm = true;
	}

	function handleCancelEmployeeForm() {
		resetEmployeeForm();
	}

	async function handleSaveEmployee() {
		if (!hasAdminSession()) return;

		// Validate
		if (!employeeFormName.trim()) {
			employeeFormError = 'Name is required';
			return;
		}

		if (!editingEmployee && !employeeFormPin.trim()) {
			employeeFormError = 'PIN is required for new employees';
			return;
		}

		employeeFormSaving = true;
		employeeFormError = '';

		try {
			if (editingEmployee) {
				// Update existing employee
				await updateEmployee(editingEmployee.employee_id, {
					name: employeeFormName.trim(),
					pin: employeeFormPin.trim() || undefined,
					role: employeeFormRole
				});
			} else {
				// Create new employee
				await createEmployee({
					name: employeeFormName.trim(),
					pin: employeeFormPin.trim(),
					role: employeeFormRole
				});
			}

			resetEmployeeForm();
			await loadEmployees();
		} catch (err) {
			if (err instanceof ApiClientError) {
				employeeFormError = err.message || 'Failed to save employee';
			} else {
				employeeFormError = 'An error occurred while saving';
			}
		} finally {
			employeeFormSaving = false;
		}
	}

	async function handleToggleActive(employee: EmployeeSummary) {
		if (!hasAdminSession()) return;

		try {
			await updateEmployee(employee.employee_id, {
				is_active: !employee.is_active
			});
			await loadEmployees();
		} catch (err) {
			// Show error in the employees section
			if (err instanceof ApiClientError) {
				employeesError = err.message || 'Failed to update employee';
			} else {
				employeesError = 'An error occurred';
			}
		}
	}

	function handleDeleteClick(employee: EmployeeSummary) {
		deleteConfirmEmployee = employee;
		deleteError = '';
	}

	function handleCancelDelete() {
		deleteConfirmEmployee = null;
		deleteError = '';
	}

	async function handleConfirmDelete() {
		if (!hasAdminSession() || !deleteConfirmEmployee) return;

		deleteLoading = true;
		deleteError = '';

		try {
			const response = await deleteEmployee(deleteConfirmEmployee.employee_id);
			if (response.warning) {
				// Could show this warning somewhere, but for now just log it
				console.warn(response.warning);
			}
			deleteConfirmEmployee = null;
			await loadEmployees();
		} catch (err) {
			if (err instanceof ApiClientError) {
				deleteError = err.message || 'Failed to delete employee';
			} else {
				deleteError = 'An error occurred while deleting';
			}
		} finally {
			deleteLoading = false;
		}
	}

	// =============================================================================
	// Locations Tab Functions
	// =============================================================================

	async function loadLocations() {
		locationsLoading = true;
		locationsError = '';

		try {
			const response = await listStorageLocations(showInactiveLocations);
			locations = response.locations;
		} catch (err) {
			if (err instanceof ApiClientError) {
				locationsError = err.message || 'Failed to load locations';
			} else {
				locationsError = 'An error occurred while loading locations';
			}
		} finally {
			locationsLoading = false;
		}
	}

	// Load/reload locations when tab becomes active or showInactiveLocations changes
	let currentShowInactiveLocations = $derived(showInactiveLocations);

	$effect(() => {
		if (
			isAuthenticated &&
			activeTab === 'locations' &&
			currentShowInactiveLocations !== undefined
		) {
			loadLocations();
		}
	});

	function resetLocationForm() {
		showLocationForm = false;
		editingLocation = null;
		locationFormName = '';
		locationFormError = '';
		locationFormSaving = false;
	}

	function handleAddLocation() {
		resetLocationForm();
		showLocationForm = true;
	}

	function handleEditLocation(location: StorageLocationSummary) {
		resetLocationForm();
		editingLocation = location;
		locationFormName = location.name;
		showLocationForm = true;
	}

	function handleCancelLocationForm() {
		resetLocationForm();
	}

	async function handleSaveLocation() {
		if (!hasAdminSession()) return;

		// Validate
		if (!locationFormName.trim()) {
			locationFormError = 'Name is required';
			return;
		}

		locationFormSaving = true;
		locationFormError = '';

		try {
			if (editingLocation) {
				// Update existing location
				await updateStorageLocation(editingLocation.location_id, {
					name: locationFormName.trim()
				});
			} else {
				// Create new location
				await createStorageLocation({
					name: locationFormName.trim()
				});
			}

			resetLocationForm();
			await loadLocations();
		} catch (err) {
			if (err instanceof ApiClientError) {
				locationFormError = err.message || 'Failed to save location';
			} else {
				locationFormError = 'An error occurred while saving';
			}
		} finally {
			locationFormSaving = false;
		}
	}

	async function handleToggleLocationActive(location: StorageLocationSummary) {
		if (!hasAdminSession()) return;

		try {
			await updateStorageLocation(location.location_id, {
				is_active: !location.is_active
			});
			await loadLocations();
		} catch (err) {
			// Show error in the locations section
			if (err instanceof ApiClientError) {
				locationsError = err.message || 'Failed to update location';
			} else {
				locationsError = 'An error occurred';
			}
		}
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
						<div class="store-info-panel">
							<h3 class="panel-title">Store Information</h3>

							{#if storeInfoError}
								<div class="store-info-message error">{storeInfoError}</div>
							{/if}

							{#if storeInfoSuccess}
								<div class="store-info-message success">{storeInfoSuccess}</div>
							{/if}

							{#if storeInfoLoading}
								<div class="store-info-loading">Loading settings...</div>
							{:else}
								<form
									class="store-info-form"
									onsubmit={(e) => {
										e.preventDefault();
										handleSaveStoreInfo();
									}}
								>
									<div class="store-info-fields">
										<Input
											label="Store Name"
											placeholder="Enter store name"
											bind:value={storeFormName}
											disabled={storeInfoSaving}
											required
										/>

										<Input
											label="Phone Number"
											type="tel"
											placeholder="(555) 555-5555"
											bind:value={storeFormPhone}
											disabled={storeInfoSaving}
										/>

										<Input
											label="Address"
											placeholder="123 Main St, City, State ZIP"
											bind:value={storeFormAddress}
											disabled={storeInfoSaving}
										/>

										<Input
											label="Ticket Prefix"
											placeholder="JR"
											bind:value={storeFormTicketPrefix}
											disabled={storeInfoSaving}
											required
										/>
										<p class="field-hint">
											The prefix used for ticket IDs (e.g., "JR" for JR-9F3K2)
										</p>
									</div>

									<div class="store-info-actions">
										<Button
											variant="primary"
											type="submit"
											loading={storeInfoSaving}
											disabled={storeInfoSaving}
										>
											Save Changes
										</Button>
									</div>
								</form>
							{/if}
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
						<div class="employees-panel">
							<!-- Header with Add button and filter -->
							<div class="employees-header">
								<h3 class="panel-title">Employees</h3>
								<div class="employees-actions">
									<label class="show-inactive-label">
										<input
											type="checkbox"
											bind:checked={showInactive}
											class="show-inactive-checkbox"
										/>
										Show inactive
									</label>
									<Button
										variant="primary"
										size="sm"
										onclick={handleAddEmployee}
										disabled={showEmployeeForm}
									>
										Add Employee
									</Button>
								</div>
							</div>

							<!-- Error message -->
							{#if employeesError}
								<div class="employees-error">{employeesError}</div>
							{/if}

							<!-- Loading state -->
							{#if employeesLoading}
								<div class="employees-loading">Loading employees...</div>
							{:else}
								<!-- Employee form (add/edit) -->
								{#if showEmployeeForm}
									<div class="employee-form">
										<h4 class="employee-form-title">
											{editingEmployee ? 'Edit Employee' : 'Add Employee'}
										</h4>

										<div class="employee-form-fields">
											<Input
												label="Name"
												placeholder="Employee name"
												bind:value={employeeFormName}
												disabled={employeeFormSaving}
												required
											/>

											<Input
												label={editingEmployee ? 'New PIN (leave blank to keep current)' : 'PIN'}
												type="password"
												placeholder={editingEmployee ? 'Enter new PIN' : 'Enter PIN'}
												bind:value={employeeFormPin}
												disabled={employeeFormSaving}
												required={!editingEmployee}
											/>

											<div class="form-field">
												<label class="form-label" for="employee-role-select">Role</label>
												<select
													id="employee-role-select"
													bind:value={employeeFormRole}
													disabled={employeeFormSaving}
													class="role-select"
												>
													<option value="staff">Staff</option>
													<option value="admin">Admin</option>
												</select>
											</div>
										</div>

										{#if employeeFormError}
											<div class="employee-form-error">{employeeFormError}</div>
										{/if}

										<div class="employee-form-actions">
											<Button
												variant="secondary"
												size="sm"
												onclick={handleCancelEmployeeForm}
												disabled={employeeFormSaving}
											>
												Cancel
											</Button>
											<Button
												variant="primary"
												size="sm"
												onclick={handleSaveEmployee}
												loading={employeeFormSaving}
												disabled={employeeFormSaving}
											>
												{editingEmployee ? 'Save Changes' : 'Add Employee'}
											</Button>
										</div>
									</div>
								{/if}

								<!-- Delete confirmation -->
								{#if deleteConfirmEmployee}
									<div class="delete-confirm">
										<p class="delete-confirm-message">
											Are you sure you want to delete <strong>{deleteConfirmEmployee.name}</strong>?
											This action cannot be undone.
										</p>

										{#if deleteError}
											<div class="delete-error">{deleteError}</div>
										{/if}

										<div class="delete-confirm-actions">
											<Button
												variant="secondary"
												size="sm"
												onclick={handleCancelDelete}
												disabled={deleteLoading}
											>
												Cancel
											</Button>
											<Button
												variant="danger"
												size="sm"
												onclick={handleConfirmDelete}
												loading={deleteLoading}
												disabled={deleteLoading}
											>
												Delete
											</Button>
										</div>
									</div>
								{/if}

								<!-- Employee list -->
								{#if employees.length === 0}
									<div class="employees-empty">
										{showInactive ? 'No employees found.' : 'No active employees found.'}
									</div>
								{:else}
									<div class="employees-list">
										{#each employees as employee (employee.employee_id)}
											<div class="employee-row" class:inactive={!employee.is_active}>
												<div class="employee-info">
													<span class="employee-name">{employee.name}</span>
													<span class="employee-role" class:admin={employee.role === 'admin'}>
														{employee.role}
													</span>
													{#if !employee.is_active}
														<span class="employee-status-badge">Inactive</span>
													{/if}
												</div>
												<div class="employee-actions">
													<button
														type="button"
														class="action-btn edit-btn"
														onclick={() => handleEditEmployee(employee)}
														title="Edit"
														disabled={showEmployeeForm || deleteConfirmEmployee !== null}
													>
														Edit
													</button>
													<button
														type="button"
														class="action-btn toggle-btn"
														onclick={() => handleToggleActive(employee)}
														title={employee.is_active ? 'Deactivate' : 'Activate'}
														disabled={showEmployeeForm || deleteConfirmEmployee !== null}
													>
														{employee.is_active ? 'Deactivate' : 'Activate'}
													</button>
													<button
														type="button"
														class="action-btn delete-btn"
														onclick={() => handleDeleteClick(employee)}
														title="Delete"
														disabled={showEmployeeForm || deleteConfirmEmployee !== null}
													>
														Delete
													</button>
												</div>
											</div>
										{/each}
									</div>
								{/if}
							{/if}
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
						<div class="locations-panel">
							<!-- Header with Add button and filter -->
							<div class="locations-header">
								<h3 class="panel-title">Storage Locations</h3>
								<div class="locations-actions">
									<label class="show-inactive-label">
										<input
											type="checkbox"
											bind:checked={showInactiveLocations}
											class="show-inactive-checkbox"
										/>
										Show inactive
									</label>
									<Button
										variant="primary"
										size="sm"
										onclick={handleAddLocation}
										disabled={showLocationForm}
									>
										Add Location
									</Button>
								</div>
							</div>

							<!-- Error message -->
							{#if locationsError}
								<div class="locations-error">{locationsError}</div>
							{/if}

							<!-- Loading state -->
							{#if locationsLoading}
								<div class="locations-loading">Loading locations...</div>
							{:else}
								<!-- Location form (add/edit) -->
								{#if showLocationForm}
									<div class="location-form">
										<h4 class="location-form-title">
											{editingLocation ? 'Edit Location' : 'Add Location'}
										</h4>

										<div class="location-form-fields">
											<Input
												label="Name"
												placeholder="e.g., Safe Drawer 1, Workbench A"
												bind:value={locationFormName}
												disabled={locationFormSaving}
												required
											/>
										</div>

										{#if locationFormError}
											<div class="location-form-error">{locationFormError}</div>
										{/if}

										<div class="location-form-actions">
											<Button
												variant="secondary"
												size="sm"
												onclick={handleCancelLocationForm}
												disabled={locationFormSaving}
											>
												Cancel
											</Button>
											<Button
												variant="primary"
												size="sm"
												onclick={handleSaveLocation}
												loading={locationFormSaving}
												disabled={locationFormSaving}
											>
												{editingLocation ? 'Save Changes' : 'Add Location'}
											</Button>
										</div>
									</div>
								{/if}

								<!-- Location list -->
								{#if locations.length === 0}
									<div class="locations-empty">
										{showInactiveLocations ? 'No locations found.' : 'No active locations found.'}
									</div>
								{:else}
									<div class="locations-list">
										{#each locations as location (location.location_id)}
											<div class="location-row" class:inactive={!location.is_active}>
												<div class="location-info">
													<span class="location-name">{location.name}</span>
													{#if !location.is_active}
														<span class="location-status-badge">Inactive</span>
													{/if}
												</div>
												<div class="location-actions">
													<button
														type="button"
														class="action-btn edit-btn"
														onclick={() => handleEditLocation(location)}
														title="Edit"
														disabled={showLocationForm}
													>
														Edit
													</button>
													<button
														type="button"
														class="action-btn toggle-btn"
														onclick={() => handleToggleLocationActive(location)}
														title={location.is_active ? 'Deactivate' : 'Activate'}
														disabled={showLocationForm}
													>
														{location.is_active ? 'Deactivate' : 'Activate'}
													</button>
												</div>
											</div>
										{/each}
									</div>
								{/if}
							{/if}
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

	/* Store Info Panel */
	.store-info-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
		padding: var(--space-sm, 0.5rem) 0;
	}

	.store-info-message {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		border-radius: var(--radius-sm, 0.25rem);
		font-size: 0.875rem;
	}

	.store-info-message.error {
		background-color: var(--color-danger-light, #fef2f2);
		border: 1px solid var(--color-danger, #dc2626);
		color: var(--color-danger, #dc2626);
	}

	.store-info-message.success {
		background-color: var(--color-success-light, #f0fdf4);
		border: 1px solid var(--color-success, #16a34a);
		color: var(--color-success, #16a34a);
	}

	.store-info-loading {
		text-align: center;
		padding: var(--space-lg, 1.5rem);
		color: var(--color-text-muted, #64748b);
		font-size: 0.875rem;
	}

	.store-info-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	.store-info-fields {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	.field-hint {
		margin: calc(var(--space-xs, 0.25rem) * -1) 0 0;
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
		line-height: 1.4;
	}

	.store-info-actions {
		display: flex;
		justify-content: flex-end;
		padding-top: var(--space-sm, 0.5rem);
	}

	/* Employees Panel */
	.employees-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
		padding: var(--space-sm, 0.5rem) 0;
	}

	.employees-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md, 1rem);
	}

	.panel-title {
		margin: 0;
		font-size: 1rem;
		font-weight: 600;
		color: var(--color-text, #1e293b);
	}

	.employees-actions {
		display: flex;
		align-items: center;
		gap: var(--space-md, 1rem);
	}

	.show-inactive-label {
		display: flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
		font-size: 0.75rem;
		color: var(--color-text-muted, #64748b);
		cursor: pointer;
	}

	.show-inactive-checkbox {
		cursor: pointer;
	}

	.employees-error,
	.delete-error,
	.employee-form-error,
	.locations-error,
	.location-form-error {
		padding: var(--space-sm, 0.5rem);
		background-color: var(--color-danger-light, #fef2f2);
		border: 1px solid var(--color-danger, #dc2626);
		border-radius: var(--radius-sm, 0.25rem);
		font-size: 0.875rem;
		color: var(--color-danger, #dc2626);
	}

	.employees-loading,
	.employees-empty,
	.locations-loading,
	.locations-empty {
		text-align: center;
		padding: var(--space-lg, 1.5rem);
		color: var(--color-text-muted, #64748b);
		font-size: 0.875rem;
	}

	/* Employee Form */
	.employee-form {
		padding: var(--space-md, 1rem);
		background-color: var(--color-bg-subtle, #f8fafc);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
	}

	.employee-form-title {
		margin: 0 0 var(--space-md, 1rem);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text, #1e293b);
	}

	.employee-form-fields {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	.form-field {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.form-label {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.role-select {
		padding: var(--space-sm, 0.5rem);
		font-size: 0.875rem;
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-sm, 0.25rem);
		background-color: white;
		cursor: pointer;
	}

	.role-select:focus {
		outline: 2px solid var(--color-primary, #1e40af);
		outline-offset: -1px;
	}

	.role-select:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.employee-form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
		margin-top: var(--space-md, 1rem);
	}

	/* Delete Confirmation */
	.delete-confirm {
		padding: var(--space-md, 1rem);
		background-color: var(--color-danger-light, #fef2f2);
		border: 1px solid var(--color-danger-muted, #fecaca);
		border-radius: var(--radius-md, 0.5rem);
	}

	.delete-confirm-message {
		margin: 0 0 var(--space-md, 1rem);
		font-size: 0.875rem;
		color: var(--color-text, #1e293b);
	}

	.delete-confirm-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
	}

	/* Employee List */
	.employees-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.employee-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		background-color: white;
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-sm, 0.25rem);
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.employee-row:hover {
		background-color: var(--color-bg-subtle, #f8fafc);
	}

	.employee-row.inactive {
		opacity: 0.6;
	}

	.employee-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
	}

	.employee-name {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.employee-role {
		padding: 0.125rem 0.5rem;
		font-size: 0.75rem;
		font-weight: 500;
		text-transform: capitalize;
		background-color: var(--color-bg-subtle, #f1f5f9);
		color: var(--color-text-muted, #64748b);
		border-radius: var(--radius-full, 9999px);
	}

	.employee-role.admin {
		background-color: var(--color-primary-light, #dbeafe);
		color: var(--color-primary, #1e40af);
	}

	.employee-status-badge {
		padding: 0.125rem 0.5rem;
		font-size: 0.75rem;
		font-weight: 500;
		background-color: var(--color-warning-light, #fef3c7);
		color: var(--color-warning-dark, #92400e);
		border-radius: var(--radius-full, 9999px);
	}

	.employee-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
	}

	.action-btn {
		padding: 0.25rem 0.5rem;
		font-size: 0.75rem;
		font-weight: 500;
		border: none;
		border-radius: var(--radius-sm, 0.25rem);
		cursor: pointer;
		transition:
			background-color var(--transition-fast, 150ms ease),
			opacity var(--transition-fast, 150ms ease);
	}

	.action-btn:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.edit-btn {
		background-color: var(--color-bg-subtle, #f1f5f9);
		color: var(--color-text, #1e293b);
	}

	.edit-btn:hover:not(:disabled) {
		background-color: var(--color-border, #e2e8f0);
	}

	.toggle-btn {
		background-color: var(--color-warning-light, #fef3c7);
		color: var(--color-warning-dark, #92400e);
	}

	.toggle-btn:hover:not(:disabled) {
		background-color: var(--color-warning-muted, #fde68a);
	}

	.delete-btn {
		background-color: var(--color-danger-light, #fef2f2);
		color: var(--color-danger, #dc2626);
	}

	.delete-btn:hover:not(:disabled) {
		background-color: var(--color-danger-muted, #fecaca);
	}

	/* Locations Panel */
	.locations-panel {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
		padding: var(--space-sm, 0.5rem) 0;
	}

	.locations-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-md, 1rem);
	}

	.locations-actions {
		display: flex;
		align-items: center;
		gap: var(--space-md, 1rem);
	}

	/* Location Form */
	.location-form {
		padding: var(--space-md, 1rem);
		background-color: var(--color-bg-subtle, #f8fafc);
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-md, 0.5rem);
	}

	.location-form-title {
		margin: 0 0 var(--space-md, 1rem);
		font-size: 0.875rem;
		font-weight: 600;
		color: var(--color-text, #1e293b);
	}

	.location-form-fields {
		display: flex;
		flex-direction: column;
		gap: var(--space-md, 1rem);
	}

	.location-form-actions {
		display: flex;
		justify-content: flex-end;
		gap: var(--space-sm, 0.5rem);
		margin-top: var(--space-md, 1rem);
	}

	/* Location List */
	.locations-list {
		display: flex;
		flex-direction: column;
		gap: var(--space-xs, 0.25rem);
	}

	.location-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		background-color: white;
		border: 1px solid var(--color-border, #e2e8f0);
		border-radius: var(--radius-sm, 0.25rem);
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.location-row:hover {
		background-color: var(--color-bg-subtle, #f8fafc);
	}

	.location-row.inactive {
		opacity: 0.6;
	}

	.location-info {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
	}

	.location-name {
		font-size: 0.875rem;
		font-weight: 500;
		color: var(--color-text, #1e293b);
	}

	.location-status-badge {
		padding: 0.125rem 0.5rem;
		font-size: 0.75rem;
		font-weight: 500;
		background-color: var(--color-warning-light, #fef3c7);
		color: var(--color-warning-dark, #92400e);
		border-radius: var(--radius-full, 9999px);
	}

	.location-actions {
		display: flex;
		align-items: center;
		gap: var(--space-xs, 0.25rem);
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

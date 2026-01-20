<script lang="ts">
	import type { PageData } from './$types';
	import { themeStore, THEMES, THEME_NAMES, THEME_DESCRIPTIONS, type Theme } from '$lib/stores/theme.svelte';

	let { data }: { data: PageData } = $props();

	function handleThemeChange(theme: Theme) {
		themeStore.set(theme);
	}
</script>

<div class="admin-page">
	<div class="admin-header">
		<h1 class="page-title">Admin Settings</h1>
		<p class="page-subtitle">Manage store settings, employees, and storage locations.</p>
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
				</div>
				<div class="section-content">
					{#if data.settings}
						<dl class="info-list">
							<div class="info-item">
								<dt>Store Name</dt>
								<dd>{data.settings.store_name || 'Not configured'}</dd>
							</div>
							<div class="info-item">
								<dt>Phone</dt>
								<dd>{data.settings.store_phone || 'Not set'}</dd>
							</div>
							<div class="info-item">
								<dt>Address</dt>
								<dd>{data.settings.store_address || 'Not set'}</dd>
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
				</div>
				<div class="section-content">
					{#if data.employees && data.employees.length > 0}
						<ul class="employee-list">
							{#each data.employees as employee (employee.employee_id)}
								<li class="employee-item">
									<span class="employee-name">{employee.name}</span>
									<span class="employee-role role-{employee.role}">{employee.role}</span>
									{#if !employee.is_active}
										<span class="employee-inactive">Inactive</span>
									{/if}
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
				</div>
				<div class="section-content">
					{#if data.locations && data.locations.length > 0}
						<ul class="location-list">
							{#each data.locations as location (location.location_id)}
								<li class="location-item">
									<span class="location-name">{location.name}</span>
									{#if !location.is_active}
										<span class="location-inactive">Inactive</span>
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
	}

	.admin-header {
		margin-bottom: var(--space-sm);
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
		gap: var(--space-md);
		padding: var(--space-sm) var(--space-md);
		background-color: var(--color-bg);
		border-radius: var(--radius-md);
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
		transition: border-color var(--transition-fast, 150ms ease),
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
</style>

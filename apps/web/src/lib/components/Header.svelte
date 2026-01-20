<script lang="ts">
	import { resolve } from '$app/paths';

	interface Props {
		storeName?: string;
	}

	let { storeName = 'Facet' }: Props = $props();

	let searchQuery = $state('');
</script>

<header class="app-header">
	<div class="header-content">
		<a href={resolve('/')} class="logo">
			<span class="logo-text">{storeName}</span>
		</a>

		<div class="header-center">
			<form class="search-form" method="GET" action={resolve('/search')}>
				<div class="search-wrapper">
					<svg
						class="search-icon"
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
						<circle cx="11" cy="11" r="8" />
						<path d="m21 21-4.35-4.35" />
					</svg>
					<input
						type="search"
						name="q"
						class="search-input"
						placeholder="Search tickets..."
						bind:value={searchQuery}
						autocomplete="off"
					/>
				</div>
			</form>
		</div>

		<div class="header-right">
			<nav class="nav-links">
				<a href={resolve('/')} class="nav-link">Workboard</a>
				<a href={resolve('/search')} class="nav-link">Search</a>
			</nav>
			<a href={resolve('/admin')} class="settings-link" title="Settings">
				<svg
					class="settings-icon"
					xmlns="http://www.w3.org/2000/svg"
					width="22"
					height="22"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
					stroke-linecap="round"
					stroke-linejoin="round"
				>
					<path
						d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"
					/>
					<circle cx="12" cy="12" r="3" />
				</svg>
			</a>
		</div>
	</div>
</header>

<style>
	.app-header {
		position: sticky;
		top: 0;
		z-index: 100;
		height: var(--header-height, 60px);
		background-color: var(--color-primary);
		color: white;
		box-shadow: var(--shadow-md);
	}

	.header-content {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: var(--space-lg, 1.5rem);
		height: 100%;
		max-width: var(--max-content-width, 1600px);
		margin: 0 auto;
		padding: 0 var(--space-lg, 1.5rem);
	}

	.logo {
		display: flex;
		align-items: center;
		gap: var(--space-sm, 0.5rem);
		color: white;
		text-decoration: none;
		flex-shrink: 0;
	}

	.logo:hover {
		text-decoration: none;
	}

	.logo-text {
		font-size: 1.5rem;
		font-weight: 700;
		letter-spacing: -0.02em;
	}

	.header-center {
		flex: 1;
		display: flex;
		justify-content: center;
		max-width: 400px;
	}

	.search-form {
		width: 100%;
	}

	.search-wrapper {
		position: relative;
		display: flex;
		align-items: center;
	}

	.search-icon {
		position: absolute;
		left: 12px;
		color: var(--color-text-muted, #64748b);
		pointer-events: none;
	}

	.search-input {
		width: 100%;
		padding: 8px 12px 8px 40px;
		font-size: 0.875rem;
		background-color: rgba(255, 255, 255, 0.95);
		border: none;
		border-radius: var(--radius-md, 0.5rem);
		color: var(--color-text, #1e293b);
		transition:
			background-color var(--transition-fast, 150ms ease),
			box-shadow var(--transition-fast, 150ms ease);
	}

	.search-input::placeholder {
		color: var(--color-text-muted, #64748b);
	}

	.search-input:focus {
		outline: none;
		background-color: white;
		box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.3);
	}

	.header-right {
		display: flex;
		align-items: center;
		gap: var(--space-md, 1rem);
		flex-shrink: 0;
	}

	.nav-links {
		display: flex;
		gap: var(--space-sm, 0.5rem);
	}

	.nav-link {
		padding: var(--space-sm, 0.5rem) var(--space-md, 1rem);
		color: rgba(255, 255, 255, 0.9);
		font-size: 0.875rem;
		font-weight: 500;
		text-decoration: none;
		border-radius: var(--radius-md, 0.5rem);
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.nav-link:hover {
		background-color: rgba(255, 255, 255, 0.1);
		text-decoration: none;
	}

	.settings-link {
		display: flex;
		align-items: center;
		justify-content: center;
		padding: var(--space-sm, 0.5rem);
		color: rgba(255, 255, 255, 0.9);
		border-radius: var(--radius-md, 0.5rem);
		transition: background-color var(--transition-fast, 150ms ease);
	}

	.settings-link:hover {
		background-color: rgba(255, 255, 255, 0.1);
		text-decoration: none;
	}

	.settings-icon {
		display: block;
	}

	/* Responsive adjustments */
	@media (max-width: 768px) {
		.header-center {
			display: none;
		}

		.nav-links {
			display: none;
		}
	}
</style>

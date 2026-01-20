<script lang="ts">
	interface Props {
		/** Badge variant - determines color scheme */
		variant: 'rush' | 'overdue' | 'intake' | 'in-progress' | 'waiting' | 'ready' | 'closed';
		/** Text to display in the badge */
		text: string;
		/** Badge size */
		size?: 'sm' | 'md';
		/** Additional CSS class for the badge */
		class?: string;
	}

	let { variant, text, size = 'sm', class: className = '' }: Props = $props();
</script>

<span class="badge badge-{variant} badge-{size} {className}" role="status">
	{text}
</span>

<style>
	.badge {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		font-weight: 500;
		line-height: 1;
		white-space: nowrap;
		border-radius: 9999px;
		text-transform: uppercase;
		letter-spacing: 0.025em;
	}

	/* Size variants */
	.badge-sm {
		padding: 0.125rem 0.5rem;
		font-size: 0.625rem;
		min-height: 1.25rem;
	}

	.badge-md {
		padding: 0.25rem 0.75rem;
		font-size: 0.75rem;
		min-height: 1.5rem;
	}

	/* Rush variant - red for urgent items */
	.badge-rush {
		background-color: var(--color-rush, #ef4444);
		color: white;
	}

	/* Overdue variant - darker red for overdue items */
	.badge-overdue {
		background-color: #b91c1c;
		color: white;
	}

	/* Status variants */
	.badge-intake {
		background-color: var(--color-intake, #8b5cf6);
		color: white;
	}

	.badge-in-progress {
		background-color: var(--color-in-progress, #3b82f6);
		color: white;
	}

	.badge-waiting {
		background-color: var(--color-waiting, #f59e0b);
		color: white;
	}

	.badge-ready {
		background-color: var(--color-ready, #10b981);
		color: white;
	}

	.badge-closed {
		background-color: var(--color-closed, #6b7280);
		color: white;
	}
</style>

/**
 * Modal store for global modal management
 *
 * Provides a reactive store for controlling modal visibility and content.
 * Supports stacking multiple modals with proper z-index management.
 */

import type { Component, ComponentProps } from 'svelte';

export interface ModalConfig<T extends Component = Component> {
	/** Unique identifier for this modal instance */
	id: string;
	/** Modal title displayed in header */
	title: string;
	/** Svelte component to render inside the modal */
	component: T;
	/** Props to pass to the component */
	props?: ComponentProps<T>;
	/** Callback when modal is closed */
	onClose?: () => void;
	/** Whether clicking backdrop closes modal (default: true) */
	closeOnBackdrop?: boolean;
	/** Whether ESC key closes modal (default: true) */
	closeOnEsc?: boolean;
}

interface ModalState {
	/** Stack of open modals (last is topmost) */
	stack: ModalConfig[];
}

function createModalStore() {
	const state = $state<ModalState>({ stack: [] });

	return {
		/** Get current modal stack (reactive) */
		get stack() {
			return state.stack;
		},

		/** Check if any modal is open */
		get isOpen() {
			return state.stack.length > 0;
		},

		/** Get the topmost modal */
		get current() {
			return state.stack.length > 0 ? state.stack[state.stack.length - 1] : null;
		},

		/**
		 * Open a new modal
		 * @param config Modal configuration
		 */
		open<T extends Component>(config: ModalConfig<T>) {
			const id = config.id || crypto.randomUUID();
			const modalConfig: ModalConfig = {
				...config,
				id,
				closeOnBackdrop: config.closeOnBackdrop ?? true,
				closeOnEsc: config.closeOnEsc ?? true
			};
			state.stack = [...state.stack, modalConfig];
			return id;
		},

		/**
		 * Close a specific modal by ID, or close the topmost modal if no ID provided
		 * @param id Optional modal ID to close
		 */
		close(id?: string) {
			if (state.stack.length === 0) return;

			if (id) {
				const index = state.stack.findIndex((m) => m.id === id);
				if (index !== -1) {
					const modal = state.stack[index];
					modal.onClose?.();
					state.stack = [...state.stack.slice(0, index), ...state.stack.slice(index + 1)];
				}
			} else {
				// Close topmost modal
				const modal = state.stack[state.stack.length - 1];
				modal.onClose?.();
				state.stack = state.stack.slice(0, -1);
			}
		},

		/**
		 * Close all open modals
		 */
		closeAll() {
			for (const modal of [...state.stack].reverse()) {
				modal.onClose?.();
			}
			state.stack = [];
		},

		/**
		 * Check if a specific modal is open
		 * @param id Modal ID to check
		 */
		isModalOpen(id: string) {
			return state.stack.some((m) => m.id === id);
		}
	};
}

/**
 * Global modal store instance
 *
 * Usage:
 * ```ts
 * import { modalStore } from '$lib/stores/modal';
 * import MyModalContent from './MyModalContent.svelte';
 *
 * // Open a modal
 * modalStore.open({
 *   id: 'my-modal',
 *   title: 'My Modal',
 *   component: MyModalContent,
 *   props: { someData: 123 }
 * });
 *
 * // Close the modal
 * modalStore.close('my-modal');
 * // or close topmost:
 * modalStore.close();
 * ```
 */
export const modalStore = createModalStore();

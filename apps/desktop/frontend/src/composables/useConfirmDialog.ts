import { ref } from 'vue';

interface ConfirmState {
  visible: boolean;
  title: string;
  message: string;
  resolve: ((value: boolean) => void) | null;
}

const state = ref<ConfirmState>({ visible: false, title: '', message: '', resolve: null });

/** Global confirm dialog composable — pairs with ConfirmDialog.vue */
export function useConfirmDialog() {
  function confirm(title: string, message: string): Promise<boolean> {
    return new Promise((resolve) => {
      state.value = { visible: true, title, message, resolve };
    });
  }

  function accept() {
    const r = state.value.resolve;
    state.value = { visible: false, title: '', message: '', resolve: null };
    r?.(true);
  }

  function cancel() {
    const r = state.value.resolve;
    state.value = { visible: false, title: '', message: '', resolve: null };
    r?.(false);
  }

  return { state, confirm, accept, cancel };
}

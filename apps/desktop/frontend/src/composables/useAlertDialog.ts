import { ref } from 'vue';

interface AlertState {
  visible: boolean;
  title: string;
  message: string;
  resolve: (() => void) | null;
}

const state = ref<AlertState>({ visible: false, title: '', message: '', resolve: null });

/** Global alert dialog composable — pairs with AlertDialog.vue */
export function useAlertDialog() {
  function alert(title: string, message: string): Promise<void> {
    return new Promise((resolve) => {
      state.value = { visible: true, title, message, resolve };
    });
  }

  function close() {
    const r = state.value.resolve;
    state.value = { visible: false, title: '', message: '', resolve: null };
    r?.();
  }

  return { state, alert, close };
}

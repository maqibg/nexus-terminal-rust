import { ref } from 'vue';

export interface PromptDialogOptions {
  title?: string;
  message?: string;
  initialValue?: string;
  placeholder?: string;
  confirmText?: string;
  cancelText?: string;
  inputType?: 'text' | 'password';
  validate?: ((value: string) => string | null | undefined) | null;
}

interface PromptState {
  visible: boolean;
  title: string;
  message: string;
  initialValue: string;
  placeholder: string;
  confirmText: string;
  cancelText: string;
  inputType: 'text' | 'password';
  validate: ((value: string) => string | null | undefined) | null;
  resolve: ((value: string | null) => void) | null;
}

const createDefaultState = (): PromptState => ({
  visible: false,
  title: '请输入',
  message: '',
  initialValue: '',
  placeholder: '',
  confirmText: '确定',
  cancelText: '取消',
  inputType: 'text',
  validate: null,
  resolve: null,
});

const state = ref<PromptState>(createDefaultState());

export function usePromptDialog() {
  function prompt(options: PromptDialogOptions): Promise<string | null> {
    return new Promise((resolve) => {
      state.value = {
        ...createDefaultState(),
        ...options,
        visible: true,
        resolve,
      };
    });
  }

  function accept(value: string) {
    const resolver = state.value.resolve;
    state.value = createDefaultState();
    resolver?.(value);
  }

  function cancel() {
    const resolver = state.value.resolve;
    state.value = createDefaultState();
    resolver?.(null);
  }

  return { state, prompt, accept, cancel };
}

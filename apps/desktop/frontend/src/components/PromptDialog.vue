<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="cancel">
      <div class="dialog-card">
        <div class="dialog-title">{{ title }}</div>
        <div v-if="message" class="dialog-message">{{ message }}</div>
        <input
          ref="inputRef"
          v-model="inputValue"
          class="dialog-input"
          :type="inputType"
          :placeholder="placeholder"
          @input="clearError"
          @keydown.enter.prevent="confirm"
        >
        <div v-if="errorMessage" class="dialog-error">{{ errorMessage }}</div>
        <div class="dialog-actions">
          <button class="btn btn-cancel" @click="cancel">{{ cancelText }}</button>
          <button class="btn btn-confirm" @click="confirm">{{ confirmText }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { nextTick, ref, watch } from 'vue';

const props = withDefaults(defineProps<{
  visible: boolean;
  title?: string;
  message?: string;
  initialValue?: string;
  placeholder?: string;
  confirmText?: string;
  cancelText?: string;
  inputType?: 'text' | 'password';
  validator?: ((value: string) => string | null | undefined) | null;
}>(), {
  title: '请输入',
  message: '',
  initialValue: '',
  placeholder: '',
  confirmText: '确定',
  cancelText: '取消',
  inputType: 'text',
  validator: null,
});

const emit = defineEmits<{
  confirm: [value: string];
  cancel: [];
}>();

const inputValue = ref('');
const errorMessage = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

function clearError() {
  errorMessage.value = '';
}

function confirm() {
  const validationMessage = props.validator?.(inputValue.value);
  if (validationMessage) {
    errorMessage.value = validationMessage;
    return;
  }
  emit('confirm', inputValue.value);
}

function cancel() {
  emit('cancel');
}

watch(
  () => props.visible,
  async (visible) => {
    if (!visible) {
      inputValue.value = '';
      errorMessage.value = '';
      return;
    }
    inputValue.value = props.initialValue;
    errorMessage.value = '';
    await nextTick();
    inputRef.value?.focus();
    inputRef.value?.select();
  },
  { immediate: true },
);
</script>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9000;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-card {
  width: min(420px, calc(100vw - 24px));
  background: var(--bg-surface0);
  border-radius: 10px;
  padding: 20px;
  border: 1px solid var(--border);
  box-shadow: 0 18px 40px rgba(0, 0, 0, 0.35);
}

.dialog-title {
  font-size: calc(16px + var(--ui-font-size-offset));
  font-weight: 600;
  margin-bottom: 10px;
  color: var(--text);
}

.dialog-message {
  font-size: calc(13px + var(--ui-font-size-offset));
  color: var(--text-sub);
  margin-bottom: 12px;
  line-height: 1.6;
  white-space: pre-wrap;
}

.dialog-input {
  width: 100%;
  min-height: 38px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  padding: 8px 12px;
  font-size: calc(13px + var(--ui-font-size-offset));
  box-sizing: border-box;
}

.dialog-input:focus {
  outline: none;
  border-color: var(--blue);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--blue) 22%, transparent);
}

.dialog-error {
  margin-top: 8px;
  color: var(--red);
  font-size: calc(12px + var(--ui-font-size-offset));
  line-height: 1.4;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 16px;
}

.btn {
  min-width: 78px;
  height: 34px;
  border-radius: 8px;
  border: 1px solid var(--border);
  cursor: pointer;
  font-size: calc(13px + var(--ui-font-size-offset));
}

.btn-cancel {
  background: transparent;
  color: var(--text-sub);
}

.btn-cancel:hover {
  background: var(--bg-surface1);
  color: var(--text);
}

.btn-confirm {
  border-color: var(--button-bg-color);
  background: var(--button-bg-color);
  color: var(--button-text-color);
}

.btn-confirm:hover {
  background: var(--button-hover-bg-color);
}
</style>

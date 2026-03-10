<template>
  <Teleport to="body">
    <div
      v-if="visible"
      class="favorite-dialog-overlay"
      @click.self="handleClose"
    >
      <div class="favorite-dialog">
        <h2 class="dialog-title">{{ isEditMode ? '编辑收藏路径' : '添加新收藏路径' }}</h2>

        <form class="dialog-form" @submit.prevent="handleSave">
          <label class="dialog-field" for="favorite-name-input">
            <span>名称（可选）</span>
            <input
              id="favorite-name-input"
              v-model="name"
              type="text"
              class="dialog-input"
              placeholder="我的文档"
            />
          </label>

          <label class="dialog-field" for="favorite-path-input">
            <span>路径 <em>*</em></span>
            <input
              id="favorite-path-input"
              v-model="path"
              type="text"
              class="dialog-input"
              placeholder="/example/folder/path"
              required
            />
          </label>

          <p v-if="errorMessage" class="dialog-error">{{ errorMessage }}</p>
        </form>

        <div class="dialog-footer">
          <button
            type="button"
            class="dialog-btn cancel"
            @click="handleClose"
          >
            取消
          </button>
          <button
            type="button"
            class="dialog-btn save"
            :disabled="!trimmedPath"
            @click="handleSave"
          >
            保存
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { FavoritePath } from '@/lib/api';

const props = defineProps<{ visible: boolean; editData?: FavoritePath | null; connectionId?: number }>();
const emit = defineEmits<{ close: []; save: [data: { name: string; path: string }] }>();

const name = ref('');
const path = ref('');
const errorMessage = ref('');

const isEditMode = computed(() => !!props.editData?.id);
const trimmedPath = computed(() => path.value.trim());

watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      return;
    }

    errorMessage.value = '';
    if (props.editData) {
      name.value = props.editData.name || '';
      path.value = props.editData.path || '';
      return;
    }

    name.value = '';
    path.value = '';
  },
  { immediate: true },
);

function handleClose() {
  emit('close');
}

function handleSave() {
  if (!trimmedPath.value) {
    errorMessage.value = '路径不能为空';
    return;
  }

  errorMessage.value = '';
  emit('save', {
    name: name.value.trim(),
    path: trimmedPath.value,
  });
}
</script>

<style scoped>
.favorite-dialog-overlay {
  position: fixed;
  inset: 0;
  z-index: 4300;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--overlay-bg-color, rgba(0, 0, 0, 0.56));
}

.favorite-dialog {
  width: min(460px, calc(100vw - 32px));
  border-radius: 10px;
  border: 1px solid rgba(148, 163, 184, 0.22);
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  box-shadow: 0 16px 42px rgba(0, 0, 0, 0.46);
  padding: 24px;
}

.dialog-title {
  margin: 0 0 22px;
  text-align: center;
  font-size: calc(22px + var(--ui-font-size-offset));
  font-weight: 650;
  letter-spacing: 0.1px;
}

.dialog-form {
  display: flex;
  flex-direction: column;
  gap: 14px;
}

.dialog-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  color: var(--text-sub, #a6adc8);
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 500;
}

.dialog-field em {
  font-style: normal;
  color: var(--red, #f38ba8);
}

.dialog-input {
  height: 38px;
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
  font-size: calc(13px + var(--ui-font-size-offset));
  padding: 0 12px;
  outline: none;
  transition: border-color 0.15s, box-shadow 0.15s;
}

.dialog-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.22);
}

.dialog-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.dialog-error {
  margin: 0;
  padding: 8px 10px;
  border-radius: 8px;
  color: var(--red, #f38ba8);
  font-size: calc(12px + var(--ui-font-size-offset));
  background: rgba(243, 139, 168, 0.14);
}

.dialog-footer {
  margin-top: 28px;
  padding-top: 16px;
  border-top: 1px solid rgba(148, 163, 184, 0.2);
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}

.dialog-btn {
  min-width: 72px;
  height: 36px;
  border-radius: 8px;
  border: 1px solid transparent;
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 600;
  cursor: pointer;
  transition: all 0.15s;
}

.dialog-btn.cancel {
  background: transparent;
  border-color: var(--border, #45475a);
  color: var(--text-sub, #a6adc8);
}

.dialog-btn.cancel:hover {
  background: rgba(137, 180, 250, 0.08);
  color: var(--text, #cdd6f4);
}

.dialog-btn.save {
  background: var(--blue, #89b4fa);
  border-color: var(--blue, #89b4fa);
  color: var(--bg-base, #1e1e2e);
}

.dialog-btn.save:hover:not(:disabled) {
  opacity: 0.88;
}

.dialog-btn.save:disabled {
  opacity: 0.48;
  cursor: not-allowed;
}
</style>
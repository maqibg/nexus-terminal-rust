<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="closeDialog">
      <div class="dialog-card">
        <div class="dialog-header">
          <h3 class="dialog-title">手动添加 AI 模型</h3>
          <button class="icon-btn" type="button" @click="closeDialog">
            <i class="fas fa-times"></i>
          </button>
        </div>

        <div class="dialog-body">
          <p v-if="errorText" class="error-text">{{ errorText }}</p>

          <label class="field">
            <span class="field-label">所属渠道</span>
            <AppSelect
              v-model="form.channelId"
              :options="channelOptions"
              variant="input"
              placeholder="请选择渠道"
            />
          </label>

          <label class="field">
            <span class="field-label">模型 ID</span>
            <input v-model.trim="form.modelId" class="form-control" type="text" placeholder="例如：gpt-4o / claude-3-5-sonnet-latest" />
          </label>

          <label class="field">
            <span class="field-label">显示名称</span>
            <input v-model.trim="form.displayName" class="form-control" type="text" placeholder="例如：GPT-4o" />
          </label>

          <label class="field">
            <span class="field-label">上下文窗口 (tokens)</span>
            <input v-model.number="form.contextWindow" class="form-control" type="number" min="1000" max="1000000" step="1000" />
          </label>
        </div>

        <div class="dialog-footer">
          <button class="btn" type="button" @click="closeDialog">取消</button>
          <button class="btn btn-primary" type="button" :disabled="saving" @click="submit">
            {{ saving ? '保存中...' : '添加' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue';
import AppSelect from '@/components/AppSelect.vue';
import { useAIStore } from '@/stores/ai';
import { useUiNotificationsStore } from '@/stores/uiNotifications';
import type { AIChannel } from '@/types/ai';

const props = defineProps<{
  visible: boolean;
  channels: AIChannel[];
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'saved'): void;
}>();

const aiStore = useAIStore();
const notifications = useUiNotificationsStore();

const saving = ref(false);
const errorText = ref('');

const form = reactive({
  channelId: '',
  modelId: '',
  displayName: '',
  contextWindow: 8192,
});

const channelOptions = computed(() => {
  return props.channels.map((channel) => ({
    label: channel.name,
    value: channel.id,
  }));
});

const resetForm = () => {
  form.channelId = channelOptions.value[0]?.value?.toString() ?? '';
  form.modelId = '';
  form.displayName = '';
  form.contextWindow = 8192;
  errorText.value = '';
};

watch(
  () => props.visible,
  (visible) => {
    if (visible) {
      resetForm();
    }
  },
  { immediate: true },
);

const closeDialog = () => emit('update:visible', false);

const submit = async () => {
  if (!form.channelId) {
    errorText.value = '请选择所属渠道';
    return;
  }
  if (!form.modelId.trim()) {
    errorText.value = '模型 ID 不能为空';
    return;
  }
  if (!form.displayName.trim()) {
    errorText.value = '显示名称不能为空';
    return;
  }

  const contextWindow = Math.max(1000, Math.min(1_000_000, Number(form.contextWindow) || 8192));

  saving.value = true;
  errorText.value = '';
  try {
    await aiStore.addModel({
      channelId: form.channelId,
      modelId: form.modelId.trim(),
      displayName: form.displayName.trim(),
      contextWindow,
      type: 'manual',
    });
    notifications.addNotification('success', 'AI 模型添加成功');
    emit('saved');
    emit('update:visible', false);
  } catch (error) {
    errorText.value = error instanceof Error ? error.message : '添加失败';
    notifications.addNotification('error', errorText.value);
  } finally {
    saving.value = false;
  }
};
</script>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 12000;
  background: var(--ui-overlay);
  display: flex;
  justify-content: center;
  align-items: center;
}

.dialog-card {
  width: min(560px, 92vw);
  max-height: 88vh;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-surface0);
  box-shadow: 0 18px 42px rgba(0, 0, 0, 0.45);
  overflow: hidden;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-mantle);
}

.dialog-title {
  margin: 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}

.icon-btn {
  width: 30px;
  height: 30px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
}

.icon-btn:hover {
  color: var(--text);
  background: var(--bg-surface1);
}

.dialog-body {
  padding: 14px;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 13px;
  color: var(--text-sub);
}

.form-control {
  width: 100%;
  min-height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  padding: 6px 10px;
  font-size: 13px;
}

.form-control:focus {
  outline: none;
  border-color: var(--blue);
}

.error-text {
  color: var(--red);
  font-size: 12px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 12px 14px;
  border-top: 1px solid var(--border);
  background: var(--bg-mantle);
}

.btn {
  height: 32px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0 12px;
  font-size: 13px;
  cursor: pointer;
  background: transparent;
  color: var(--text);
}

.btn:hover {
  background: var(--bg-surface1);
}

.btn-primary {
  border-color: var(--button-bg-color);
  background: var(--button-bg-color);
  color: var(--button-text-color);
}

.btn-primary:hover {
  background: var(--button-hover-bg-color);
}
</style>

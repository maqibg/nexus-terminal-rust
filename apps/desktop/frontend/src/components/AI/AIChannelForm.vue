<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="closeDialog">
      <div class="dialog-card">
        <div class="dialog-header">
          <h3 class="dialog-title">{{ isEditing ? '编辑 AI 渠道' : '添加 AI 渠道' }}</h3>
          <button class="icon-btn" type="button" @click="closeDialog">
            <i class="fas fa-times"></i>
          </button>
        </div>

        <div class="dialog-body">
          <p v-if="errorText" class="error-text">{{ errorText }}</p>

          <label class="field">
            <span class="field-label">渠道名称</span>
            <input v-model.trim="form.name" class="form-control" type="text" placeholder="例如：OpenAI 官方" />
          </label>

          <label class="field">
            <span class="field-label">渠道类型</span>
            <AppSelect
              v-model="form.type"
              :options="typeOptions"
              variant="input"
              class="field-select"
              placeholder="请选择渠道类型"
            />
          </label>

          <label class="field">
            <span class="field-label">API Key</span>
            <input
              v-model.trim="form.apiKey"
              class="form-control"
              :type="showApiKey ? 'text' : 'password'"
              :placeholder="isEditing ? '留空表示保持不变' : '请输入 API Key'"
            />
            <button class="text-btn" type="button" @click="showApiKey = !showApiKey">
              {{ showApiKey ? '隐藏' : '显示' }}
            </button>
          </label>

          <label class="field">
            <span class="field-label">API 地址 (可选)</span>
            <input
              v-model.trim="form.apiEndpoint"
              class="form-control"
              type="text"
              :placeholder="apiEndpointPlaceholder"
            />
            <small class="field-hint">OpenAI 兼容服务建议填写完整端点，如 `https://api.example.com/v1`。</small>
          </label>

          <label class="switch-row">
            <input v-model="form.enabled" type="checkbox" class="switch-input" />
            <span>启用该渠道</span>
          </label>
        </div>

        <div class="dialog-footer">
          <button class="btn" type="button" @click="closeDialog">取消</button>
          <button class="btn btn-primary" type="button" :disabled="saving" @click="submit">
            {{ saving ? '保存中...' : isEditing ? '保存' : '添加' }}
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
import type { AIChannel, AIProviderType } from '@/types/ai';

const props = defineProps<{
  visible: boolean;
  channel?: AIChannel | null;
}>();

const emit = defineEmits<{
  (e: 'update:visible', value: boolean): void;
  (e: 'saved'): void;
}>();

const aiStore = useAIStore();
const notifications = useUiNotificationsStore();

const saving = ref(false);
const showApiKey = ref(false);
const errorText = ref('');

const form = reactive({
  name: '',
  type: 'openai' as AIProviderType,
  apiKey: '',
  apiEndpoint: '',
  enabled: true,
});

const isEditing = computed(() => !!props.channel);

const typeOptions: Array<{ label: string; value: AIProviderType }> = [
  { label: 'OpenAI', value: 'openai' },
  { label: 'Anthropic Claude', value: 'anthropic' },
  { label: 'Google Gemini', value: 'gemini' },
  { label: 'OpenAI 兼容', value: 'openai-compatible' },
];

const apiEndpointPlaceholder = computed(() => {
  if (form.type === 'anthropic') {
    return 'https://api.anthropic.com/v1';
  }
  if (form.type === 'gemini') {
    return 'https://generativelanguage.googleapis.com/v1beta';
  }
  if (form.type === 'openai-compatible') {
    return 'https://api.example.com/v1';
  }
  return 'https://api.openai.com/v1';
});

const resetForm = () => {
  form.name = '';
  form.type = 'openai';
  form.apiKey = '';
  form.apiEndpoint = '';
  form.enabled = true;
  errorText.value = '';
  showApiKey.value = false;
};

const fillForm = (channel: AIChannel) => {
  form.name = channel.name;
  form.type = channel.type;
  form.apiKey = '';
  form.apiEndpoint = channel.apiEndpoint ?? '';
  form.enabled = channel.enabled;
  errorText.value = '';
  showApiKey.value = false;
};

watch(
  () => props.visible,
  (visible) => {
    if (!visible) {
      return;
    }
    if (props.channel) {
      fillForm(props.channel);
      return;
    }
    resetForm();
  },
  { immediate: true },
);

watch(
  () => props.channel,
  (channel) => {
    if (!props.visible) {
      return;
    }
    if (channel) {
      fillForm(channel);
    } else {
      resetForm();
    }
  },
);

const closeDialog = () => {
  emit('update:visible', false);
};

const submit = async () => {
  if (!form.name.trim()) {
    errorText.value = '渠道名称不能为空';
    return;
  }

  if (!isEditing.value && !form.apiKey.trim()) {
    errorText.value = 'API Key 不能为空';
    return;
  }

  saving.value = true;
  errorText.value = '';
  try {
    if (isEditing.value && props.channel) {
      await aiStore.updateChannel(props.channel.id, {
        name: form.name.trim(),
        type: form.type,
        apiKey: form.apiKey.trim() ? form.apiKey.trim() : undefined,
        apiEndpoint: form.apiEndpoint.trim() || undefined,
        enabled: form.enabled,
      });
      notifications.addNotification('success', 'AI 渠道更新成功');
    } else {
      await aiStore.addChannel({
        name: form.name.trim(),
        type: form.type,
        apiKey: form.apiKey.trim(),
        apiEndpoint: form.apiEndpoint.trim() || undefined,
        enabled: form.enabled,
      });
      notifications.addNotification('success', 'AI 渠道添加成功');
    }

    emit('saved');
    emit('update:visible', false);
  } catch (error) {
    errorText.value = error instanceof Error ? error.message : '保存失败';
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
  width: min(620px, 92vw);
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
  font-size: calc(14px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
}

.form-control:focus {
  outline: none;
  border-color: var(--blue);
}

.field-select :deep(.app-select-trigger) {
  background: var(--bg-base);
}

.field-hint {
  color: var(--text-sub);
  font-size: calc(12px + var(--ui-font-size-offset));
}

.text-btn {
  align-self: flex-start;
  border: none;
  background: transparent;
  color: var(--blue);
  cursor: pointer;
  font-size: calc(12px + var(--ui-font-size-offset));
}

.switch-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: calc(13px + var(--ui-font-size-offset));
  color: var(--text);
}

.switch-input {
  width: 14px;
  height: 14px;
  accent-color: var(--blue);
}

.error-text {
  color: var(--red);
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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

<template>
  <div class="terminal-ai-panel">
    <div class="chat-header">
      <span class="title">AI 助手</span>
      <button class="model-pill" type="button" @click="openModelDialog">
        {{ selectedModelName }}
      </button>
      <div class="header-actions">
        <button v-if="isLoading" class="icon-btn danger" type="button" title="停止生成" @click="handleStop">
          <i class="fas fa-stop"></i>
        </button>
        <button v-if="canRegenerate" class="icon-btn" type="button" title="重新生成" @click="handleRegenerate">
          <i class="fas fa-rotate-right"></i>
        </button>
        <button class="icon-btn" type="button" title="清空对话" @click="handleClear">
          <i class="fas fa-trash-can"></i>
        </button>
      </div>
    </div>

    <div ref="scrollContainer" class="chat-messages">
      <template v-if="messages.length > 0">
        <div v-for="(message, index) in messages" :key="message.id" class="message-item" :class="message.role">
          <div class="message-avatar">
            <i :class="message.role === 'user' ? 'fas fa-user' : 'fas fa-robot'"></i>
          </div>
          <div class="message-content">
            <div class="message-meta">
              <span>{{ formatTime(message.timestamp) }}</span>
              <span v-if="message.modelId">{{ message.modelId }}</span>
            </div>
            <pre class="message-text">{{ message.content }}</pre>
            <div v-if="message.status === 'error'" class="error-row">
              <span>{{ message.error || '请求失败' }}</span>
              <button class="text-btn" type="button" @click="retryMessage(index)">重试</button>
            </div>
          </div>
        </div>
      </template>

      <div v-else class="empty-state">
        <i class="fas fa-robot empty-icon"></i>
        <p>终端 AI 助手已就绪</p>
        <p class="hint">可用于命令解释、故障排查、脚本生成。</p>
        <div class="quick-actions">
          <button class="mini-btn" type="button" @click="setInput('帮我解释上一条命令的作用')">解释命令</button>
          <button class="mini-btn" type="button" @click="setInput('帮我排查当前终端报错')">排查报错</button>
          <button class="mini-btn" type="button" @click="setInput('帮我生成一份部署脚本')">生成脚本</button>
        </div>
      </div>

      <div v-if="isLoading" class="loading-row">
        <span class="dot"></span><span class="dot"></span><span class="dot"></span>
      </div>
    </div>

    <div class="chat-footer">
      <textarea
        ref="inputRef"
        v-model="inputValue"
        class="input-textarea"
        placeholder="输入问题，Enter 发送，Shift+Enter 换行"
        rows="4"
        @keydown.enter.exact.prevent="sendMessage"
      ></textarea>
      <div class="footer-actions">
        <button class="btn" type="button" @click="inputValue = ''">清空输入</button>
        <button class="btn btn-primary" type="button" :disabled="isLoading" @click="sendMessage">
          {{ isLoading ? '生成中...' : '发送' }}
        </button>
      </div>
    </div>
  </div>

  <Teleport to="body">
    <div v-if="showModelDialog" class="dialog-backdrop" @click.self="showModelDialog = false">
      <div class="model-dialog">
        <div class="dialog-header">
          <h3>选择 AI 模型</h3>
          <button class="icon-btn" type="button" @click="showModelDialog = false">
            <i class="fas fa-times"></i>
          </button>
        </div>

        <div class="dialog-body">
          <div class="dialog-column">
            <p class="column-title">渠道</p>
            <button
              v-for="channel in aiStore.enabledChannels"
              :key="channel.id"
              class="list-btn"
              :class="{ active: dialogSelectedChannelId === channel.id }"
              type="button"
              @click="dialogSelectedChannelId = channel.id"
            >
              {{ channel.name }}
            </button>
            <p v-if="aiStore.enabledChannels.length === 0" class="empty-hint">暂无已启用渠道</p>
          </div>

          <div class="dialog-column">
            <p class="column-title">模型</p>
            <button
              v-for="model in dialogModels"
              :key="model.id"
              class="list-btn"
              :class="{ active: dialogSelectedModelId === model.id }"
              type="button"
              @click="dialogSelectedModelId = model.id"
            >
              {{ model.displayName }}
            </button>
            <p v-if="dialogModels.length === 0" class="empty-hint">该渠道暂无模型</p>
          </div>
        </div>

        <div class="dialog-footer">
          <button class="btn" type="button" @click="showModelDialog = false">取消</button>
          <button class="btn btn-primary" type="button" :disabled="!dialogSelectedModelId" @click="confirmModel">
            确定
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { useAIStore } from '@/stores/ai';
import { useUiNotificationsStore } from '@/stores/uiNotifications';
import type { AIChatMessage } from '@/types/ai';

const props = defineProps<{
  sessionId?: string | null;
  sessionName?: string;
  storageId?: string;
}>();

const aiStore = useAIStore();
const notifications = useUiNotificationsStore();

const scrollContainer = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);
const inputValue = ref('');
const isLoading = ref(false);
const showModelDialog = ref(false);
const dialogSelectedChannelId = ref('');
const dialogSelectedModelId = ref('');
const selectedModelId = ref('');

const messages = ref<AIChatMessage[]>([]);

let requestSerial = 0;
let cancelledRequestSerial = 0;

const storageId = computed(() => props.storageId || props.sessionId || 'global');

const selectedModel = computed(() => {
  if (selectedModelId.value) {
    return aiStore.models.find((model) => model.id === selectedModelId.value) ?? null;
  }
  return aiStore.defaultModel;
});

const selectedModelName = computed(() => selectedModel.value?.displayName ?? '选择模型');

const dialogModels = computed(() => {
  if (!dialogSelectedChannelId.value) {
    return [];
  }
  return aiStore.models.filter((model) => model.channelId === dialogSelectedChannelId.value);
});

const canRegenerate = computed(() => {
  for (let index = messages.value.length - 1; index >= 0; index -= 1) {
    if (messages.value[index].role === 'user') {
      return true;
    }
  }
  return false;
});

const formatTime = (timestamp: number) => {
  const date = new Date(timestamp);
  return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`;
};

const createMessage = (
  role: 'user' | 'assistant' | 'system',
  content: string,
  extra: Partial<AIChatMessage> = {},
): AIChatMessage => ({
  id: `${Date.now()}-${Math.random().toString(16).slice(2, 10)}`,
  role,
  content,
  timestamp: Date.now(),
  ...extra,
});

const scrollToBottom = async () => {
  await nextTick();
  if (!scrollContainer.value) {
    return;
  }
  scrollContainer.value.scrollTop = scrollContainer.value.scrollHeight;
};

const saveHistory = async () => {
  await aiStore.saveTerminalChatHistory(storageId.value, messages.value);
};

const loadHistory = async () => {
  messages.value = await aiStore.getTerminalChatHistory(storageId.value);
  await scrollToBottom();
};

const loadSavedModel = () => {
  const key = `nexus-terminal:ai-model:${storageId.value}`;
  const modelId = localStorage.getItem(key);
  if (modelId && aiStore.models.some((model) => model.id === modelId)) {
    selectedModelId.value = modelId;
  }
};

const persistSelectedModel = () => {
  const key = `nexus-terminal:ai-model:${storageId.value}`;
  if (!selectedModelId.value) {
    localStorage.removeItem(key);
    return;
  }
  localStorage.setItem(key, selectedModelId.value);
};

const ensureAiReady = async () => {
  if (aiStore.channels.length === 0 && aiStore.models.length === 0) {
    await aiStore.loadAll();
  }
};

const openModelDialog = () => {
  showModelDialog.value = true;
  const selected = selectedModel.value;
  if (selected) {
    dialogSelectedModelId.value = selected.id;
    dialogSelectedChannelId.value = selected.channelId;
    return;
  }
  dialogSelectedChannelId.value = aiStore.enabledChannels[0]?.id ?? '';
  dialogSelectedModelId.value = dialogModels.value[0]?.id ?? '';
};

const confirmModel = () => {
  selectedModelId.value = dialogSelectedModelId.value;
  persistSelectedModel();
  showModelDialog.value = false;
  if (selectedModel.value) {
    notifications.addNotification('success', `已切换到模型：${selectedModel.value.displayName}`);
  }
};

const buildContextPrompt = (message: string) => {
  if (!props.sessionName?.trim()) {
    return message;
  }
  return `[当前终端会话: ${props.sessionName.trim()}]\n\n用户问题: ${message}`;
};

const sendMessageInternal = async (override?: string) => {
  const content = (override ?? inputValue.value).trim();
  if (!content || isLoading.value) {
    return;
  }

  await ensureAiReady();
  if (!selectedModel.value && !aiStore.defaultModel) {
    notifications.addNotification('warning', '请先在设置- AI 助手中配置模型');
    return;
  }

  const runId = ++requestSerial;
  isLoading.value = true;

  if (!override) {
    inputValue.value = '';
  }

  messages.value.push(createMessage('user', content, { status: 'success' }));
  await saveHistory();
  await scrollToBottom();

  try {
    const modelId = selectedModel.value?.id || aiStore.defaultModel?.id;
    if (!modelId) {
      throw new Error('未找到可用模型');
    }

    const response = await aiStore.sendRequestWithModel('chat', buildContextPrompt(content), modelId);
    if (cancelledRequestSerial >= runId) {
      return;
    }

    messages.value.push(
      createMessage('assistant', response, {
        modelId: selectedModel.value?.displayName ?? aiStore.defaultModel?.displayName,
        status: 'success',
      }),
    );
    await saveHistory();
    await scrollToBottom();
  } catch (error) {
    const message = error instanceof Error ? error.message : 'AI 请求失败';
    messages.value.push(createMessage('assistant', `错误：${message}`, { status: 'error', error: message }));
    await saveHistory();
    await scrollToBottom();
    notifications.addNotification('error', message);
  } finally {
    if (requestSerial === runId) {
      isLoading.value = false;
    }
  }
};

const sendMessage = async () => {
  await sendMessageInternal();
};

const retryMessage = async (errorIndex: number) => {
  for (let index = errorIndex - 1; index >= 0; index -= 1) {
    if (messages.value[index].role === 'user') {
      await sendMessageInternal(messages.value[index].content);
      return;
    }
  }
};

const handleClear = async () => {
  if (!messages.value.length) {
    return;
  }
  const confirmed = window.confirm('确定清空当前会话 AI 历史？');
  if (!confirmed) {
    return;
  }
  messages.value = [];
  await aiStore.clearTerminalChatHistory(storageId.value);
};

const handleStop = () => {
  cancelledRequestSerial = requestSerial;
  isLoading.value = false;
  notifications.addNotification('info', '已停止等待当前响应');
};

const handleRegenerate = async () => {
  for (let index = messages.value.length - 1; index >= 0; index -= 1) {
    if (messages.value[index].role === 'user') {
      await sendMessageInternal(messages.value[index].content);
      return;
    }
  }
};

const setInput = (value: string) => {
  inputValue.value = value;
  inputRef.value?.focus();
};

watch(storageId, async () => {
  await loadHistory();
  loadSavedModel();
});

onMounted(async () => {
  await ensureAiReady();
  await loadHistory();
  loadSavedModel();
});

defineExpose({
  sendMessage: sendMessageInternal,
  setInput,
});
</script>

<style scoped>
.terminal-ai-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-surface0);
}

.chat-header {
  height: 38px;
  padding: 0 8px 0 10px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-mantle);
  display: flex;
  align-items: center;
  gap: 8px;
}

.title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.model-pill {
  max-width: 180px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text-sub);
  border-radius: 999px;
  padding: 2px 10px;
  font-size: 12px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  cursor: pointer;
}

.model-pill:hover {
  color: var(--text);
  border-color: var(--blue);
}

.header-actions {
  margin-left: auto;
  display: flex;
  align-items: center;
  gap: 4px;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
}

.icon-btn:hover {
  color: var(--text);
  background: var(--bg-surface1);
}

.icon-btn.danger {
  color: var(--red);
}

.chat-messages {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.message-item {
  display: flex;
  gap: 8px;
}

.message-item.user {
  flex-direction: row-reverse;
}

.message-avatar {
  width: 24px;
  height: 24px;
  border-radius: 50%;
  background: var(--bg-mantle);
  color: var(--text-sub);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 12px;
  flex-shrink: 0;
}

.message-content {
  max-width: calc(100% - 32px);
  min-width: 0;
}

.message-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  color: var(--text-sub);
  font-size: 11px;
  margin-bottom: 4px;
}

.message-item.user .message-meta {
  justify-content: flex-end;
}

.message-text {
  margin: 0;
  white-space: pre-wrap;
  word-break: break-word;
  line-height: 1.5;
  font-size: 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 8px 10px;
  background: var(--bg-base);
  color: var(--text);
}

.message-item.user .message-text {
  background: color-mix(in srgb, var(--blue) 25%, var(--bg-base));
  border-color: color-mix(in srgb, var(--blue) 45%, var(--border));
}

.error-row {
  margin-top: 4px;
  color: var(--red);
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.text-btn {
  border: none;
  background: transparent;
  color: var(--blue);
  cursor: pointer;
  font-size: 12px;
}

.empty-state {
  flex: 1;
  border: 1px dashed var(--border);
  border-radius: 10px;
  padding: 24px 14px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  text-align: center;
  color: var(--text-sub);
  gap: 6px;
}

.empty-icon {
  font-size: 28px;
}

.hint {
  font-size: 12px;
}

.quick-actions {
  margin-top: 6px;
  display: flex;
  flex-wrap: wrap;
  justify-content: center;
  gap: 6px;
}

.mini-btn {
  border: 1px solid var(--border);
  background: transparent;
  color: var(--text-sub);
  border-radius: 6px;
  font-size: 12px;
  padding: 4px 8px;
  cursor: pointer;
}

.mini-btn:hover {
  color: var(--text);
  border-color: var(--blue);
}

.loading-row {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 5px;
  margin-top: 4px;
}

.dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--text-sub);
  animation: blink 1.2s infinite ease-in-out;
}

.dot:nth-child(2) {
  animation-delay: 0.18s;
}

.dot:nth-child(3) {
  animation-delay: 0.32s;
}

@keyframes blink {
  0%, 80%, 100% {
    transform: scale(0.8);
    opacity: 0.5;
  }
  40% {
    transform: scale(1.1);
    opacity: 1;
  }
}

.chat-footer {
  border-top: 1px solid var(--border);
  background: var(--bg-mantle);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.input-textarea {
  width: 100%;
  min-height: 86px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  padding: 8px 10px;
  font-size: 13px;
  resize: vertical;
}

.input-textarea:focus {
  outline: none;
  border-color: var(--blue);
}

.footer-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  height: 30px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0 10px;
  font-size: 12px;
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

.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 12000;
  background: var(--ui-overlay);
  display: flex;
  justify-content: center;
  align-items: center;
}

.model-dialog {
  width: min(760px, 92vw);
  max-height: 82vh;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-surface0);
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dialog-header {
  height: 42px;
  padding: 0 10px 0 14px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-mantle);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.dialog-header h3 {
  margin: 0;
  font-size: 14px;
  color: var(--text);
}

.dialog-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
  padding: 12px;
}

.dialog-column {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-base);
  min-height: 260px;
  overflow: auto;
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.column-title {
  margin: 0 0 4px;
  font-size: 12px;
  color: var(--text-sub);
}

.list-btn {
  border: 1px solid var(--border);
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub);
  padding: 6px 8px;
  font-size: 12px;
  text-align: left;
  cursor: pointer;
}

.list-btn:hover {
  color: var(--text);
  background: var(--bg-surface1);
}

.list-btn.active {
  border-color: var(--blue);
  background: color-mix(in srgb, var(--blue) 24%, var(--bg-base));
  color: var(--text);
}

.empty-hint {
  margin: 0;
  font-size: 12px;
  color: var(--text-sub);
}

.dialog-footer {
  height: 48px;
  border-top: 1px solid var(--border);
  background: var(--bg-mantle);
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 12px;
  gap: 8px;
}
</style>

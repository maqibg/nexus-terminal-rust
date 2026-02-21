<template>
  <div class="terminal-ai-panel">
    <div class="chat-header">
      <span class="title">AI 助手</span>
      <button class="model-pill" type="button" @click="openModelDialog">
        {{ selectedModelName }}
      </button>
      <div class="header-actions">
        <button v-if="props.closable" class="icon-btn" type="button" title="关闭" @click="emit('close')">
          <i class="fas fa-times"></i>
        </button>
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

    <div ref="scrollContainer" class="chat-messages" @click="handleMessageClick">
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
            <div class="message-text markdown-body" v-html="renderMessageHtml(message)"></div>
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
      <div v-if="attachedFiles.length > 0" class="attached-files">
        <div v-for="(file, index) in attachedFiles" :key="file.path" class="attached-file">
          <i class="fas fa-file-lines"></i>
          <span class="file-name" :title="file.path">{{ file.name }}</span>
          <button class="remove-file-btn" type="button" title="移除" @click="removeAttachedFile(index)">
            <i class="fas fa-times"></i>
          </button>
        </div>
      </div>

      <div class="input-shell">
        <textarea
          ref="inputRef"
          v-model="inputValue"
          class="input-textarea"
          placeholder="输入问题，@ 附加文件，Enter 发送，Shift+Enter 换行"
          rows="4"
          @input="handleInputChange"
          @keydown.enter.exact.prevent="sendMessage"
        ></textarea>

        <div v-if="showFileSelector" class="file-selector-popup">
          <div class="file-selector-header">
            <span>选择文件</span>
            <div class="file-selector-header-actions">
              <button class="popup-icon-btn" type="button" title="刷新" @click="refreshFileList">
                <i class="fas fa-rotate-right"></i>
              </button>
              <button class="popup-icon-btn" type="button" title="关闭" @click="showFileSelector = false">
                <i class="fas fa-times"></i>
              </button>
            </div>
          </div>

          <div class="path-row">
            <input
              v-model.trim="pathInput"
              class="path-input"
              type="text"
              placeholder="输入路径并回车"
              @keydown.enter.prevent="goToPath"
            >
            <button class="btn btn-small" type="button" @click="goToPath">跳转</button>
          </div>

          <div class="quick-paths">
            <button class="quick-path-btn" type="button" @click="goToQuickPath('/')">/ 根目录</button>
            <button class="quick-path-btn" type="button" @click="goToQuickPath('~')">~ 主目录</button>
            <button class="quick-path-btn" type="button" @click="goToQuickPath('/tmp')">/tmp</button>
            <button class="quick-path-btn" type="button" @click="goToQuickPath('/var/log')">/var/log</button>
          </div>

          <div v-if="loadingFiles" class="file-loading">
            <i class="fas fa-spinner fa-spin"></i>
            <span>加载中...</span>
          </div>

          <div v-else class="file-list">
            <button
              v-if="currentDir !== '/'"
              class="file-item file-item-dir"
              type="button"
              @click="navigateToParent"
            >
              <i class="fas fa-arrow-left"></i>
              <span>..</span>
            </button>

            <button
              v-for="file in remoteFiles"
              :key="file.path"
              class="file-item"
              :class="{ 'file-item-dir': file.isDir }"
              type="button"
              @click="handleFileClick(file)"
            >
              <i :class="file.isDir ? 'fas fa-folder' : 'fas fa-file-lines'"></i>
              <span class="file-item-name">{{ file.name }}</span>
              <span v-if="!file.isDir" class="file-size">{{ formatFileSize(file.size) }}</span>
            </button>

            <div v-if="remoteFiles.length === 0" class="empty-file-list">目录为空</div>
          </div>
        </div>
      </div>

      <div class="footer-actions">
        <button class="btn attach-trigger" type="button" title="附加文件 (@)" @click="toggleFileSelector">
          <i class="fas fa-folder-open"></i>
        </button>
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
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import type { UnlistenFn } from '@tauri-apps/api/event';
import { aiApi, sftpApi, sshApi } from '@/lib/api';
import { useAIStore } from '@/stores/ai';
import { useSessionStore } from '@/stores/session';
import { useUiNotificationsStore } from '@/stores/uiNotifications';
import type { AIChatMessage } from '@/types/ai';

interface AttachedFile {
  name: string;
  path: string;
  content: string;
}

interface RemoteFileEntry {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
}

const props = defineProps<{
  sessionId?: string | null;
  connectionId?: number | null;
  sessionName?: string;
  storageId?: string;
  closable?: boolean;
}>();
const emit = defineEmits<{
  close: [];
}>();

const aiStore = useAIStore();
const sessionStore = useSessionStore();
const notifications = useUiNotificationsStore();

const scrollContainer = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLTextAreaElement | null>(null);
const inputValue = ref('');
const isLoading = ref(false);
const showModelDialog = ref(false);
const dialogSelectedChannelId = ref('');
const dialogSelectedModelId = ref('');
const selectedModelId = ref('');
const currentRequestId = ref<string | null>(null);

const showFileSelector = ref(false);
const loadingFiles = ref(false);
const remoteFiles = ref<RemoteFileEntry[]>([]);
const currentDir = ref('/');
const pathInput = ref('/');
const attachedFiles = ref<AttachedFile[]>([]);

const messages = ref<AIChatMessage[]>([]);

let requestSerial = 0;
let cancelledRequestSerial = 0;
let clickOutsideHandler: ((event: MouseEvent) => void) | null = null;

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
  if (isLoading.value) {
    return false;
  }
  for (let index = messages.value.length - 1; index >= 0; index -= 1) {
    if (messages.value[index].role === 'user') {
      return true;
    }
  }
  return false;
});

const activeSession = computed(() => {
  if (!props.sessionId) {
    return null;
  }
  return sessionStore.getSession(props.sessionId) ?? null;
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

const escapeHtml = (value: string): string =>
  value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');

const renderTextBlock = (raw: string): string => {
  if (!raw.trim()) {
    return '';
  }
  const escaped = escapeHtml(raw).replace(/`([^`\n]+)`/g, '<code>$1</code>');
  const paragraphs = escaped
    .split(/\n{2,}/)
    .map((part) => part.trim())
    .filter(Boolean)
    .map((part) => `<p>${part.replace(/\n/g, '<br>')}</p>`);
  return paragraphs.join('');
};

const renderAssistantMarkdown = (content: string): string => {
  const codeRegex = /```([\w+-]*)\n([\s\S]*?)```/g;
  let cursor = 0;
  let html = '';
  let match: RegExpExecArray | null;

  while ((match = codeRegex.exec(content)) !== null) {
    html += renderTextBlock(content.slice(cursor, match.index));
    const language = escapeHtml(match[1] || 'text');
    const code = match[2].replace(/\n$/, '');
    const escapedCode = escapeHtml(code);
    const encodedCode = encodeURIComponent(code);
    html += `<div class="code-block-wrapper">
      <div class="code-toolbar">
        <button class="code-btn" data-action="copy" data-code="${encodedCode}">复制</button>
        <button class="code-btn" data-action="insert" data-code="${encodedCode}">插入终端</button>
        <button class="code-btn run-btn" data-action="run" data-code="${encodedCode}">执行</button>
      </div>
      <pre><code class="language-${language}">${escapedCode}</code></pre>
    </div>`;
    cursor = codeRegex.lastIndex;
  }

  html += renderTextBlock(content.slice(cursor));
  return html || '<p></p>';
};

const renderMessageHtml = (message: AIChatMessage): string => {
  if (message.role === 'assistant') {
    return renderAssistantMarkdown(message.content);
  }
  return renderTextBlock(message.content);
};

const toBase64Utf8 = (value: string): string => {
  const bytes = new TextEncoder().encode(value);
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
};

const fromBase64Utf8 = (base64: string): string => {
  const binary = atob(base64);
  const bytes = Uint8Array.from(binary, (char) => char.charCodeAt(0));
  return new TextDecoder().decode(bytes);
};

const decodeEncodedCode = (encoded: string): string => {
  try {
    return decodeURIComponent(encoded);
  } catch {
    return encoded;
  }
};

const sendCodeToTerminal = async (code: string, execute: boolean) => {
  if (!props.sessionId) {
    notifications.addNotification('warning', '当前不是终端会话，无法插入命令');
    return;
  }

  const payload = execute ? `${code}\n` : code;
  await sshApi.write(props.sessionId, toBase64Utf8(payload));
  notifications.addNotification('success', execute ? '已执行命令' : '已插入到终端');
};

const handleMessageClick = async (event: MouseEvent) => {
  const target = (event.target as HTMLElement | null)?.closest<HTMLButtonElement>('.code-btn');
  if (!target) {
    return;
  }

  const action = target.dataset.action;
  const encodedCode = target.dataset.code || '';
  const code = decodeEncodedCode(encodedCode);

  if (!code) {
    return;
  }

  try {
    if (action === 'copy') {
      await navigator.clipboard.writeText(code);
      notifications.addNotification('success', '已复制到剪贴板');
      return;
    }
    if (action === 'insert') {
      await sendCodeToTerminal(code, false);
      return;
    }
    if (action === 'run') {
      await sendCodeToTerminal(code, true);
    }
  } catch (error) {
    notifications.addNotification('error', error instanceof Error ? error.message : '操作失败');
  }
};

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

const inferHomePath = (): string => {
  const source = activeSession.value?.currentPath?.trim() || currentDir.value || '/';
  if (source.startsWith('/home/')) {
    const seg = source.split('/').filter(Boolean);
    return seg.length >= 2 ? `/home/${seg[1]}` : '/home';
  }
  if (source.startsWith('/root')) {
    return '/root';
  }
  return '/';
};

const normalizeRemotePath = (value: string): string => {
  const raw = value.trim();
  if (!raw) {
    return currentDir.value || '/';
  }

  if (raw === '~') {
    return inferHomePath();
  }

  let normalized = raw.replace(/\\/g, '/');
  if (!normalized.startsWith('/')) {
    const baseDir = (currentDir.value || '/').replace(/\/+$/, '') || '/';
    normalized = `${baseDir}/${normalized}`;
  }
  normalized = normalized.replace(/\/{2,}/g, '/');
  return normalized || '/';
};

const ensureSftpSession = async (): Promise<string | null> => {
  if (!props.sessionId || !props.connectionId) {
    return null;
  }

  const session = sessionStore.getSession(props.sessionId);
  if (session?.sftpSessionId) {
    return session.sftpSessionId;
  }

  try {
    const opened = await sftpApi.open(props.connectionId);
    sessionStore.setSftpSession(props.sessionId, opened);
    return opened;
  } catch {
    return null;
  }
};

const loadFileList = async (targetPath: string) => {
  const sid = await ensureSftpSession();
  if (!sid) {
    notifications.addNotification('warning', '当前会话未就绪，无法附加文件');
    remoteFiles.value = [];
    return;
  }

  loadingFiles.value = true;
  try {
    const path = normalizeRemotePath(targetPath);
    const entries = await sftpApi.listDir(sid, path);
    currentDir.value = path;
    pathInput.value = path;

    remoteFiles.value = entries
      .filter((entry) => entry.name && entry.name !== '.' && entry.name !== '..')
      .map((entry) => ({
        name: entry.name,
        path: entry.path,
        isDir: entry.is_dir,
        size: entry.size,
      }))
      .sort((left, right) => {
        if (left.isDir !== right.isDir) {
          return left.isDir ? -1 : 1;
        }
        return left.name.localeCompare(right.name, 'zh-CN');
      });
  } catch (error) {
    remoteFiles.value = [];
    notifications.addNotification('error', error instanceof Error ? error.message : '读取目录失败');
  } finally {
    loadingFiles.value = false;
  }
};

const loadCurrentDirectory = async () => {
  const defaultDir = activeSession.value?.currentPath || '/';
  await loadFileList(defaultDir);
};

const refreshFileList = async () => {
  await loadFileList(currentDir.value || '/');
};

const goToPath = async () => {
  await loadFileList(pathInput.value || currentDir.value || '/');
};

const goToQuickPath = async (path: string) => {
  await loadFileList(path);
};

const navigateToParent = async () => {
  const nextPath = normalizeRemotePath(`${currentDir.value}/..`);
  await loadFileList(nextPath);
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) {
    return `${bytes}B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)}KB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
};

const attachFile = async (file: RemoteFileEntry) => {
  if (file.size > 100 * 1024) {
    notifications.addNotification('warning', '文件过大，最大支持 100KB');
    return;
  }
  if (attachedFiles.value.some((item) => item.path === file.path)) {
    notifications.addNotification('info', '该文件已附加');
    return;
  }

  const sid = await ensureSftpSession();
  if (!sid) {
    notifications.addNotification('warning', 'SFTP 会话未就绪，无法读取文件');
    return;
  }

  try {
    const fileBase64 = await sftpApi.readFile(sid, file.path);
    const content = fromBase64Utf8(fileBase64);
    attachedFiles.value.push({
      name: file.name,
      path: file.path,
      content,
    });
    if (inputValue.value.endsWith('@')) {
      inputValue.value = inputValue.value.slice(0, -1);
    }
    showFileSelector.value = false;
    notifications.addNotification('success', `已附加 ${file.name}`);
  } catch (error) {
    notifications.addNotification('error', error instanceof Error ? error.message : '读取文件失败');
  }
};

const handleFileClick = async (file: RemoteFileEntry) => {
  if (file.isDir) {
    await loadFileList(file.path);
    return;
  }
  await attachFile(file);
};

const removeAttachedFile = (index: number) => {
  attachedFiles.value.splice(index, 1);
};

const handleInputChange = () => {
  if (inputValue.value.endsWith('@') && !showFileSelector.value) {
    void toggleFileSelector();
  }
};

const toggleFileSelector = async () => {
  showFileSelector.value = !showFileSelector.value;
  if (showFileSelector.value) {
    await loadCurrentDirectory();
  }
};

const createRequestId = (): string => {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return `ai-${Date.now()}-${Math.random().toString(16).slice(2, 10)}`;
};

const sendMessageInternal = async (override?: string, options?: { skipUserMessage?: boolean }) => {
  const content = (override ?? inputValue.value).trim();
  if (!content || isLoading.value) {
    return;
  }

  await ensureAiReady();
  const model = selectedModel.value || aiStore.defaultModel;
  if (!model) {
    notifications.addNotification('warning', '请先在设置-AI 助手中配置模型');
    return;
  }

  const runId = ++requestSerial;
  const requestId = createRequestId();
  currentRequestId.value = requestId;
  isLoading.value = true;

  if (!override) {
    inputValue.value = '';
  }

  const useAttachments = !options?.skipUserMessage && attachedFiles.value.length > 0;
  let promptContent = content;
  let displayContent = content;

  if (useAttachments) {
    const fileDescriptions = attachedFiles.value.map((file) => file.name).join(', ');
    const fileContents = attachedFiles.value
      .map((file) => `\n\n--- 文件: ${file.path} ---\n${file.content}\n--- 文件结束 ---`)
      .join('');
    promptContent = `${content}${fileContents}`;
    displayContent = `${content}\n\n📎 附加文件: ${fileDescriptions}`;
  }

  if (!options?.skipUserMessage) {
    messages.value.push(createMessage('user', displayContent, { status: 'success' }));
    attachedFiles.value = [];
  }

  await saveHistory();
  await scrollToBottom();

  let streamMsgIndex = -1;
  let streamReceived = false;
  let streamError = '';
  let streamCancelled = false;
  const unlisteners: UnlistenFn[] = [];

  const cleanupListeners = () => {
    while (unlisteners.length > 0) {
      const unlisten = unlisteners.pop();
      unlisten?.();
    }
  };

  try {
    unlisteners.push(
      await aiApi.onStreamChunk((payload) => {
        if (payload.requestId !== requestId) {
          return;
        }

        streamReceived = true;
        if (streamMsgIndex === -1) {
          messages.value.push(
            createMessage('assistant', payload.chunk, {
              modelId: model.displayName,
              status: 'success',
            }),
          );
          streamMsgIndex = messages.value.length - 1;
        } else {
          messages.value[streamMsgIndex].content += payload.chunk;
        }
        void scrollToBottom();
      }),
    );

    unlisteners.push(
      await aiApi.onError((payload) => {
        if (payload.requestId === requestId) {
          streamError = payload.error || 'AI 请求失败';
        }
      }),
    );

    unlisteners.push(
      await aiApi.onCancelled((payload) => {
        if (payload.requestId === requestId) {
          streamCancelled = true;
        }
      }),
    );

    const response = await aiStore.sendRequestWithModel(
      'chat',
      buildContextPrompt(promptContent),
      model.id,
      undefined,
      requestId,
    );

    if (cancelledRequestSerial >= runId || streamCancelled) {
      return;
    }

    if (streamError) {
      throw new Error(streamError);
    }

    if (!streamReceived) {
      messages.value.push(
        createMessage('assistant', response, {
          modelId: model.displayName,
          status: 'success',
        }),
      );
    } else if (streamMsgIndex >= 0 && !messages.value[streamMsgIndex].content.trim() && response.trim()) {
      messages.value[streamMsgIndex].content = response;
    }
  } catch (error) {
    if (cancelledRequestSerial >= runId || streamCancelled) {
      notifications.addNotification('info', '已停止当前响应');
      return;
    }

    const message = error instanceof Error ? error.message : 'AI 请求失败';
    messages.value.push(createMessage('assistant', `错误：${message}`, { status: 'error', error: message }));
    notifications.addNotification('error', message);
  } finally {
    cleanupListeners();
    await saveHistory();
    await scrollToBottom();
    if (requestSerial === runId) {
      isLoading.value = false;
      currentRequestId.value = null;
    }
  }
};

const sendMessage = async () => {
  await sendMessageInternal();
};

const retryMessage = async (errorIndex: number) => {
  for (let index = errorIndex - 1; index >= 0; index -= 1) {
    if (messages.value[index].role === 'user') {
      await sendMessageInternal(messages.value[index].content, { skipUserMessage: true });
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
  attachedFiles.value = [];
  await aiStore.clearTerminalChatHistory(storageId.value);
};

const handleStop = async () => {
  cancelledRequestSerial = requestSerial;
  const requestId = currentRequestId.value;
  if (requestId) {
    await aiStore.cancelRequest(requestId);
  }
  isLoading.value = false;
  notifications.addNotification('info', '已停止等待当前响应');
};

const handleRegenerate = async () => {
  for (let index = messages.value.length - 1; index >= 0; index -= 1) {
    if (messages.value[index].role === 'user') {
      await sendMessageInternal(messages.value[index].content, { skipUserMessage: true });
      return;
    }
  }
};

const setInput = (value: string) => {
  inputValue.value = value;
  inputRef.value?.focus();
};

watch(storageId, async () => {
  attachedFiles.value = [];
  showFileSelector.value = false;
  await loadHistory();
  loadSavedModel();
});

onMounted(async () => {
  await ensureAiReady();
  await loadHistory();
  loadSavedModel();

  clickOutsideHandler = (event: MouseEvent) => {
    if (!showFileSelector.value) {
      return;
    }
    const target = event.target as HTMLElement | null;
    if (target?.closest('.file-selector-popup') || target?.closest('.attach-trigger') || target?.closest('.path-row')) {
      return;
    }
    showFileSelector.value = false;
  };
  document.addEventListener('mousedown', clickOutsideHandler);
});

onUnmounted(() => {
  if (clickOutsideHandler) {
    document.removeEventListener('mousedown', clickOutsideHandler);
    clickOutsideHandler = null;
  }
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
  line-height: 1.55;
  font-size: 13px;
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 8px 10px;
  background: var(--bg-base);
  color: var(--text);
  word-break: break-word;
}

.message-item.user .message-text {
  background: color-mix(in srgb, var(--blue) 25%, var(--bg-base));
  border-color: color-mix(in srgb, var(--blue) 45%, var(--border));
}

.markdown-body :deep(p) {
  margin: 0 0 8px;
}

.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-body :deep(code) {
  font-family: 'Cascadia Mono', Consolas, 'Courier New', monospace;
  font-size: 12px;
  padding: 1px 4px;
  border-radius: 4px;
  background: color-mix(in srgb, var(--bg-mantle) 85%, black);
}

.markdown-body :deep(.code-block-wrapper) {
  margin: 8px 0;
}

.markdown-body :deep(.code-toolbar) {
  display: flex;
  gap: 6px;
  margin-bottom: 4px;
}

.markdown-body :deep(.code-btn) {
  height: 24px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-surface1);
  color: var(--text-sub);
  padding: 0 8px;
  font-size: 11px;
  cursor: pointer;
}

.markdown-body :deep(.code-btn:hover) {
  color: var(--text);
  border-color: var(--blue);
}

.markdown-body :deep(.run-btn) {
  border-color: color-mix(in srgb, var(--color-success) 64%, var(--border));
  color: color-mix(in srgb, var(--color-success) 88%, white);
}

.markdown-body :deep(.run-btn:hover) {
  border-color: var(--color-success);
  background: color-mix(in srgb, var(--color-success) 18%, var(--bg-surface1));
}

.markdown-body :deep(pre) {
  margin: 0;
  padding: 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: color-mix(in srgb, var(--bg-mantle) 88%, black);
  overflow: auto;
}

.markdown-body :deep(pre code) {
  display: block;
  background: transparent;
  padding: 0;
  border-radius: 0;
  white-space: pre;
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
  0%,
  80%,
  100% {
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

.attached-files {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.attached-file {
  height: 24px;
  border-radius: 999px;
  border: 1px solid color-mix(in srgb, var(--blue) 48%, var(--border));
  background: color-mix(in srgb, var(--blue) 18%, var(--bg-base));
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 0 8px;
  font-size: 12px;
  color: var(--text);
}

.file-name {
  max-width: 150px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.remove-file-btn {
  width: 16px;
  height: 16px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.remove-file-btn:hover {
  color: var(--text);
}

.input-shell {
  position: relative;
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

.file-selector-popup {
  position: absolute;
  left: 0;
  right: 0;
  bottom: calc(100% + 6px);
  border-radius: 10px;
  border: 1px solid var(--border);
  background: var(--bg-surface0);
  box-shadow: 0 -10px 28px color-mix(in srgb, var(--bg-base) 75%, transparent);
  max-height: 360px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  z-index: 12000;
}

.file-selector-header {
  height: 34px;
  padding: 0 10px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-mantle);
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 12px;
  color: var(--text);
}

.file-selector-header-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.popup-icon-btn {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
}

.popup-icon-btn:hover {
  color: var(--text);
  background: var(--bg-surface1);
}

.path-row {
  padding: 8px;
  display: flex;
  gap: 6px;
  border-bottom: 1px solid var(--border);
}

.path-input {
  flex: 1;
  min-width: 0;
  height: 30px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  padding: 0 8px;
  font-size: 12px;
}

.path-input:focus {
  outline: none;
  border-color: var(--blue);
}

.quick-paths {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 8px;
  border-bottom: 1px solid var(--border);
}

.quick-path-btn {
  border: 1px solid var(--border);
  border-radius: 999px;
  background: transparent;
  color: var(--text-sub);
  font-size: 11px;
  height: 24px;
  padding: 0 8px;
  cursor: pointer;
}

.quick-path-btn:hover {
  color: var(--text);
  border-color: var(--blue);
}

.file-loading {
  padding: 18px 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-sub);
}

.file-list {
  flex: 1;
  min-height: 0;
  overflow: auto;
  padding: 6px;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.file-item {
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text);
  min-height: 30px;
  padding: 0 8px;
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  text-align: left;
}

.file-item:hover {
  background: var(--bg-surface1);
}

.file-item-dir {
  color: var(--blue);
}

.file-item-name {
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-size {
  font-size: 11px;
  color: var(--text-sub);
}

.empty-file-list {
  padding: 18px 10px;
  font-size: 12px;
  color: var(--text-sub);
  text-align: center;
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

.btn-small {
  height: 30px;
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

<template>
  <div class="vnc-session-wrapper">
    <template v-if="!session || session.protocol !== 'VNC'">
      <div class="vnc-placeholder">
        <i class="fas fa-desktop"></i>
        <span>当前会话不是 VNC 连接</span>
      </div>
    </template>

    <template v-else>
      <div class="vnc-toolbar">
        <div class="vnc-toolbar-left">
          <span class="vnc-status-dot" :class="statusClass"></span>
          <span class="vnc-toolbar-title">{{ session.connectionName }}</span>
          <span class="vnc-toolbar-subtitle">{{ connectionStatusText }}</span>
        </div>

        <div class="vnc-toolbar-actions">
          <button class="toolbar-btn" title="全屏" @click="toggleFullscreen">
            <i class="fas fa-expand"></i>
          </button>
          <button class="toolbar-btn" title="发送 Ctrl+Alt+Del" @click="sendCtrlAltDel">
            CAD
          </button>
          <button class="toolbar-btn" :class="{ active: showClipboard }" title="剪贴板" @click="showClipboard = !showClipboard">
            <i class="fas fa-copy"></i>
          </button>
          <button class="toolbar-btn" title="重新连接" @click="reconnect">
            <i class="fas fa-sync-alt"></i>
          </button>
        </div>
      </div>

      <div class="vnc-body" ref="vncContainerRef">
        <div v-if="connectionStatus === 'connecting'" class="vnc-overlay">
          <i class="fas fa-spinner fa-spin"></i>
          <span>正在连接 {{ session.connectionName }} ...</span>
        </div>

        <div v-else-if="connectionStatus === 'error'" class="vnc-overlay error">
          <i class="fas fa-exclamation-triangle"></i>
          <span>{{ errorMessage || 'VNC 连接失败' }}</span>
        </div>

        <div v-show="connectionStatus === 'connected'" ref="vncScreenRef" class="vnc-screen"></div>
      </div>

      <div v-if="showClipboard" class="clipboard-panel">
        <div class="clipboard-title">剪贴板</div>
        <textarea
          v-model="clipboardText"
          rows="4"
          class="clipboard-textarea"
          placeholder="输入要发送到远程端的内容..."
        ></textarea>
        <div class="clipboard-actions">
          <button class="toolbar-btn" @click="pasteFromLocal">从本地粘贴</button>
          <button class="toolbar-btn" @click="sendToRemote">发送到远程</button>
        </div>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue';
import RFB from '@novnc/novnc/lib/rfb';

import { useSessionStore } from '@/stores/session';

type RfbInstance = {
  disconnect: () => void;
  sendCtrlAltDel: () => void;
  clipboardPasteFrom: (text: string) => void;
  addEventListener: (name: string, handler: (event: any) => void) => void;
  removeEventListener?: (name: string, handler: (event: any) => void) => void;
  scaleViewport: boolean;
  resizeSession: boolean;
  showDotCursor: boolean;
  viewOnly: boolean;
  qualityLevel?: number;
  compressionLevel?: number;
  sendCredentials?: (credentials: { password: string }) => void;
};

const props = defineProps<{
  sessionId: string;
}>();

const sessionStore = useSessionStore();
const vncContainerRef = ref<HTMLElement>();
const vncScreenRef = ref<HTMLElement>();
const showClipboard = ref(false);
const clipboardText = ref('');
const connectionStatus = ref<'connecting' | 'connected' | 'disconnected' | 'error'>('connecting');
const errorMessage = ref('');

let rfb: RfbInstance | null = null;

const session = computed(() => sessionStore.getSession(props.sessionId));

const statusClass = computed(() => {
  if (connectionStatus.value === 'connected') return 'connected';
  if (connectionStatus.value === 'connecting') return 'connecting';
  if (connectionStatus.value === 'error') return 'error';
  return 'disconnected';
});

const connectionStatusText = computed(() => {
  if (connectionStatus.value === 'connected') return '已连接';
  if (connectionStatus.value === 'connecting') return '连接中';
  if (connectionStatus.value === 'error') return '连接错误';
  return '已断开';
});

function parseVncOptions(rawOptions: unknown): Record<string, unknown> {
  if (!rawOptions) {
    return {};
  }
  if (typeof rawOptions === 'string') {
    try {
      const parsed = JSON.parse(rawOptions);
      return typeof parsed === 'object' && parsed ? parsed as Record<string, unknown> : {};
    } catch {
      return {};
    }
  }
  if (typeof rawOptions === 'object') {
    return rawOptions as Record<string, unknown>;
  }
  return {};
}

async function connect() {
  const current = session.value;
  if (!current || current.protocol !== 'VNC') {
    return;
  }

  const wsPort = current.vncWsPort;
  if (!wsPort || !current.desktopSessionId) {
    connectionStatus.value = 'error';
    errorMessage.value = 'VNC 会话参数缺失';
    return;
  }

  connectionStatus.value = 'connecting';
  errorMessage.value = '';

  try {
    rfb?.disconnect();
    rfb = null;

    await nextTick();
    if (!vncScreenRef.value) {
      throw new Error('VNC 画布未准备就绪');
    }

    const wsUrl = `ws://127.0.0.1:${wsPort}`;
    const options = parseVncOptions((current as unknown as Record<string, unknown>).vnc_options);
    const password = typeof current.vncPassword === 'string' && current.vncPassword.length > 0
      ? current.vncPassword
      : undefined;

    const instance = new (RFB as unknown as new (
      target: HTMLElement,
      url: string,
      options?: Record<string, unknown>,
    ) => RfbInstance)(vncScreenRef.value, wsUrl, {
      credentials: password ? { password } : undefined,
      shared: options.sharedConnection !== false,
      wsProtocols: ['binary'],
    });

    instance.scaleViewport = true;
    instance.resizeSession = true;
    instance.showDotCursor = options.localCursor !== false;
    instance.viewOnly = options.viewOnly === true;
    if (typeof options.quality === 'number') {
      instance.qualityLevel = Math.max(0, Math.min(9, Math.round(options.quality)));
    }
    if (typeof options.compression === 'number') {
      instance.compressionLevel = Math.max(0, Math.min(9, Math.round(options.compression)));
    }

    instance.addEventListener('connect', () => {
      connectionStatus.value = 'connected';
      sessionStore.updateStatus(props.sessionId, 'connected');
    });

    instance.addEventListener('disconnect', (event: any) => {
      const clean = Boolean(event?.detail?.clean);
      connectionStatus.value = clean ? 'disconnected' : 'error';
      if (!clean) {
        errorMessage.value = '连接意外断开';
      }
      sessionStore.updateStatus(props.sessionId, clean ? 'disconnected' : 'disconnected');
    });

    instance.addEventListener('credentialsrequired', () => {
      if (password && instance.sendCredentials) {
        instance.sendCredentials({ password });
      } else {
        connectionStatus.value = 'error';
        errorMessage.value = 'VNC 服务器需要密码';
      }
    });

    instance.addEventListener('securityfailure', (event: any) => {
      connectionStatus.value = 'error';
      const reason = event?.detail?.reason;
      errorMessage.value = reason ? `认证失败: ${reason}` : '认证失败';
    });

    rfb = instance;
  } catch (error: any) {
    connectionStatus.value = 'error';
    errorMessage.value = error?.message ?? '连接失败';
    sessionStore.updateStatus(props.sessionId, 'disconnected');
  }
}

function reconnect() {
  void connect();
}

function toggleFullscreen() {
  const container = vncContainerRef.value;
  if (!container) {
    return;
  }

  if (document.fullscreenElement) {
    void document.exitFullscreen();
  } else {
    void container.requestFullscreen();
  }
}

function sendCtrlAltDel() {
  rfb?.sendCtrlAltDel();
}

async function pasteFromLocal() {
  try {
    clipboardText.value = await navigator.clipboard.readText();
  } catch {
    // ignore clipboard errors
  }
}

function sendToRemote() {
  if (!clipboardText.value.trim()) {
    return;
  }
  rfb?.clipboardPasteFrom(clipboardText.value);
}

watch(
  () => [props.sessionId, session.value?.desktopSessionId, session.value?.vncWsPort].join('|'),
  () => {
    void connect();
  },
  { immediate: true },
);

onBeforeUnmount(() => {
  try {
    rfb?.disconnect();
  } catch {
    // ignore dispose errors
  }
  rfb = null;
});
</script>

<style scoped>
.vnc-session-wrapper {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-base);
  color: var(--text);
}

.vnc-placeholder {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--text-dim);
}

.vnc-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border);
  background: var(--bg-surface0);
}

.vnc-toolbar-left {
  display: flex;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.vnc-status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.vnc-status-dot.connected { background: var(--green); }
.vnc-status-dot.connecting { background: var(--yellow); }
.vnc-status-dot.disconnected { background: var(--text-dim); }
.vnc-status-dot.error { background: var(--red); }

.vnc-toolbar-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--text);
}

.vnc-toolbar-subtitle {
  font-size: 11px;
  color: var(--text-sub);
}

.vnc-toolbar-actions {
  display: flex;
  align-items: center;
  gap: 6px;
}

.toolbar-btn {
  height: 28px;
  min-width: 28px;
  padding: 0 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-surface1);
  color: var(--text-sub);
  font-size: 12px;
  cursor: pointer;
}

.toolbar-btn:hover {
  color: var(--text);
  border-color: var(--blue);
}

.toolbar-btn.active {
  color: #fff;
  background: var(--blue);
  border-color: var(--blue);
}

.vnc-body {
  flex: 1;
  min-height: 0;
  position: relative;
  overflow: hidden;
  background: #111;
}

.vnc-screen {
  width: 100%;
  height: 100%;
}

.vnc-overlay {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  font-size: 13px;
  background: rgba(0, 0, 0, 0.45);
}

.vnc-overlay.error {
  color: #fecaca;
}

.clipboard-panel {
  border-top: 1px solid var(--border);
  background: var(--bg-surface0);
  padding: 10px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.clipboard-title {
  font-size: 12px;
  color: var(--text-sub);
}

.clipboard-textarea {
  width: 100%;
  resize: vertical;
  min-height: 80px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  padding: 8px;
}

.clipboard-actions {
  display: flex;
  gap: 8px;
}
</style>
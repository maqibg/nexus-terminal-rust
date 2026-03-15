<template>
  <div class="quick-sessions">
    <div class="quick-title">快捷会话</div>

    <div class="quick-row">
      <div class="quick-group">
        <div class="quick-label">本地终端</div>
        <div class="quick-fields local">
          <input class="quick-input" type="text" value="默认 Shell" disabled />
          <button type="button" class="quick-btn" :disabled="localConnecting" @click="openLocalTerminal">
            {{ localConnecting ? '打开中...' : '打开' }}
          </button>
        </div>
      </div>

      <div class="quick-group">
        <div class="quick-label">Telnet</div>
        <div class="quick-fields">
          <input v-model="telnetHost" class="quick-input" type="text" placeholder="host / IP" />
          <input v-model.number="telnetPort" class="quick-input" type="number" min="1" max="65535" placeholder="port" />
          <button type="button" class="quick-btn" :disabled="telnetConnecting" @click="connectTelnet">
            {{ telnetConnecting ? '连接中...' : '连接' }}
          </button>
        </div>
      </div>
    </div>

    <div v-if="errorText" class="quick-error">{{ errorText }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { localTerminalApi, telnetApi } from '@/lib/api';
import { toAppError } from '@/lib/errors';
import { useSessionStore } from '@/stores/session';

const sessionStore = useSessionStore();

const localConnecting = ref(false);
const telnetHost = ref('');
const telnetPort = ref<number | null>(23);
const telnetConnecting = ref(false);
const errorText = ref('');

async function openLocalTerminal() {
  errorText.value = '';
  localConnecting.value = true;
  try {
    const sessionId = await localTerminalApi.open();
    sessionStore.addSession({
      id: sessionId,
      connectionId: 0,
      connectionName: 'LOCAL · 本地终端',
      protocol: 'LOCAL',
      status: 'connected',
      createdAt: new Date().toISOString(),
      sftpReady: false,
      sftpSessionId: null,
      currentPath: '/',
      desktopSessionId: null,
      vncWsPort: null,
      vncPassword: null,
    });
    sessionStore.setActive(sessionId);
  } catch (err) {
    errorText.value = toAppError(err).message;
  } finally {
    localConnecting.value = false;
  }
}

async function connectTelnet() {
  const host = telnetHost.value.trim();
  const port = Number(telnetPort.value ?? 0);
  errorText.value = '';

  if (!host) {
    errorText.value = '请输入 Telnet Host';
    return;
  }
  if (!port || port < 1 || port > 65535) {
    errorText.value = '请输入有效端口';
    return;
  }

  telnetConnecting.value = true;
  try {
    const sessionId = await telnetApi.connect(host, port);
    sessionStore.addSession({
      id: sessionId,
      connectionId: 0,
      connectionName: `TELNET · ${host}:${port}`,
      protocol: 'TELNET',
      status: 'connected',
      createdAt: new Date().toISOString(),
      sftpReady: false,
      sftpSessionId: null,
      currentPath: '/',
      desktopSessionId: null,
      vncWsPort: null,
      vncPassword: null,
    });
    sessionStore.setActive(sessionId);
    telnetHost.value = '';
  } catch (err) {
    errorText.value = toAppError(err).message;
  } finally {
    telnetConnecting.value = false;
  }
}
</script>

<style scoped>
.quick-sessions {
  border: 1px solid var(--border);
  border-radius: 10px;
  padding: 10px;
  background: color-mix(in srgb, var(--bg-surface1) 35%, transparent);
  margin-bottom: 8px;
}

.quick-title {
  font-size: calc(12px + var(--ui-font-size-offset));
  color: var(--text-sub);
  font-weight: 650;
  margin-bottom: 8px;
}

.quick-row {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.quick-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.quick-label {
  font-size: calc(12px + var(--ui-font-size-offset));
  color: var(--text-dim);
}

.quick-fields {
  display: grid;
  grid-template-columns: 1fr 90px 84px;
  gap: 8px;
}

.quick-fields.local {
  grid-template-columns: 1fr 84px;
}

.quick-input {
  width: 100%;
  height: 30px;
  padding: 0 10px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: calc(12px + var(--ui-font-size-offset));
  outline: none;
}

.quick-input:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.quick-btn {
  height: 30px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-surface0);
  color: var(--text);
  cursor: pointer;
  font-size: calc(12px + var(--ui-font-size-offset));
}

.quick-btn:hover {
  background: var(--ui-action-hover);
}

.quick-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.quick-error {
  margin-top: 8px;
  font-size: calc(12px + var(--ui-font-size-offset));
  color: var(--color-error);
}
</style>

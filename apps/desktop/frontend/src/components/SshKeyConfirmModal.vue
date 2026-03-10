<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop">
      <div class="modal-card ssh-key-warn">
        <div class="modal-header warn">
          <span>SSH 主机密钥已变更</span>
        </div>
        <div class="modal-body">
          <p class="warn-text">
            主机 <strong>{{ event?.host }}:{{ event?.port }}</strong> 的 SSH 公钥已与存储的指纹不匹配，可能遭受中间人攻击。请仅在确认主机合法时接受新密钥。
          </p>
          <div class="fp-row">
            <span class="fp-label">旧指纹</span>
            <code class="fp-value old">{{ event?.old_fingerprint }}</code>
          </div>
          <div class="fp-row">
            <span class="fp-label">新指纹</span>
            <code class="fp-value new">{{ event?.new_fingerprint }}</code>
          </div>
        </div>
        <div class="modal-footer">
          <button class="btn danger" @click="accept" :disabled="busy">接受新密钥</button>
          <button class="btn" @click="reject" :disabled="busy">拒绝</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { sshApi } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

interface HostKeyChangedEvent {
  host: string;
  port: number;
  old_fingerprint: string;
  new_fingerprint: string;
}

const notify = useUINotificationStore();
const visible = ref(false);
const busy = ref(false);
const event = ref<HostKeyChangedEvent | null>(null);

let unlisten: UnlistenFn | null = null;

onMounted(async () => {
  unlisten = await listen<HostKeyChangedEvent>('ssh:host_key_changed', (e) => {
    event.value = e.payload;
    visible.value = true;
  });
});

onUnmounted(() => {
  unlisten?.();
  unlisten = null;
});

async function accept() {
  if (!event.value) return;
  busy.value = true;
  try {
    await sshApi.acceptHostKey(event.value.host, event.value.port, event.value.new_fingerprint);
    notify.addNotification('success', `已接受 ${event.value.host} 的新 SSH 主机密钥`);
    visible.value = false;
    event.value = null;
  } catch (e: unknown) {
    const err = e instanceof Error ? e.message : String(e);
    notify.addNotification('error', `更新主机密钥失败: ${err}`);
  } finally {
    busy.value = false;
  }
}

function reject() {
  visible.value = false;
  event.value = null;
  notify.addNotification('warning', '已拒绝新 SSH 主机密钥，连接已中止');
}
</script>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.55);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.modal-card.ssh-key-warn {
  background: var(--bg-secondary, #1e1e2e);
  border: 1px solid var(--border-color, #313244);
  border-radius: 8px;
  width: 520px;
  max-width: 95vw;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

.modal-header.warn {
  padding: 14px 18px;
  font-weight: 600;
  font-size: calc(15px + var(--ui-font-size-offset));
  border-bottom: 1px solid var(--border-color, #313244);
  color: var(--color-warning, #f9e2af);
  display: flex;
  align-items: center;
  gap: 8px;
}

.modal-header.warn::before {
  content: '⚠';
  font-size: calc(18px + var(--ui-font-size-offset));
}

.modal-body {
  padding: 18px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.warn-text {
  font-size: calc(13px + var(--ui-font-size-offset));
  color: var(--text-secondary, #cdd6f4);
  line-height: 1.6;
}

.fp-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.fp-label {
  font-size: calc(11px + var(--ui-font-size-offset));
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-muted, #6c7086);
}

.fp-value {
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: calc(12px + var(--ui-font-size-offset));
  padding: 6px 10px;
  border-radius: 4px;
  word-break: break-all;
}

.fp-value.old {
  background: rgba(243, 139, 168, 0.1);
  color: var(--color-red, #f38ba8);
  border: 1px solid rgba(243, 139, 168, 0.2);
}

.fp-value.new {
  background: rgba(166, 227, 161, 0.1);
  color: var(--color-green, #a6e3a1);
  border: 1px solid rgba(166, 227, 161, 0.2);
}

.modal-footer {
  padding: 12px 18px;
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  border-top: 1px solid var(--border-color, #313244);
}

.btn {
  padding: 7px 16px;
  border-radius: 5px;
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
  border: 1px solid var(--border-color, #313244);
  background: var(--bg-tertiary, #313244);
  color: var(--text-primary, #cdd6f4);
  transition: background 0.15s;
}

.btn:hover:not(:disabled) {
  background: var(--bg-hover, #45475a);
}

.btn.danger {
  background: var(--color-red, #f38ba8);
  color: #1e1e2e;
  border-color: var(--color-red, #f38ba8);
}

.btn.danger:hover:not(:disabled) {
  background: #eb6f92;
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>

<template>
  <div class="workspace">
    <TerminalTabBar
      :sessions="sessionList"
      :active-session-id="activeSessionId"
      @activate="sessionStore.setActive($event)"
      @close="closeSession"
      @close-others="closeOthers"
      @close-right="closeRight"
      @close-left="closeLeft"
      @open-transfers="showTransferModal = true"
      @add="showConnList = true"
    />

    <Splitpanes
      class="workspace-layout"
      @resize="handleWorkspacePaneResize"
      @resized="handleWorkspacePaneResize"
    >
      <Pane v-if="leftSidebarVisible" :size="leftSidebarSize" :min-size="10" :max-size="30">
        <LayoutRenderer v-if="layoutConfig.leftSidebar" :node="layoutConfig.leftSidebar" />
      </Pane>

      <Pane :size="mainSize" :min-size="40">
        <LayoutRenderer :node="layoutConfig.root" />
      </Pane>

      <Pane v-if="rightSidebarVisible" :size="rightSidebarSize" :min-size="15" :max-size="40">
        <LayoutRenderer v-if="layoutConfig.rightSidebar" :node="layoutConfig.rightSidebar" />
      </Pane>
    </Splitpanes>

    <Teleport to="body">
      <div v-if="showConnList" class="dialog-backdrop" @click.self="showConnList = false">
        <div class="conn-popup">
          <div class="popup-title">选择连接</div>
          <WorkspaceConnectionList @select="handleConnect" />
        </div>
      </div>
    </Teleport>

    <TransferProgressModal
      :visible="showTransferModal"
      :tasks="taskList"
      @close="showTransferModal = false"
      @cancel="cancelTask"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { storeToRefs } from 'pinia';
import { Splitpanes, Pane } from 'splitpanes';
import 'splitpanes/dist/splitpanes.css';
import { useSessionStore } from '@/stores/session';
import { useLayoutStore } from '@/stores/layout';
import { sshApi, sftpApi, desktopApi, type Connection } from '@/lib/api';
import { useTransferProgress } from '@/composables/useTransferProgress';
import LayoutRenderer from '@/components/LayoutRenderer.vue';
import TerminalTabBar from '@/components/TerminalTabBar.vue';
import WorkspaceConnectionList from '@/components/WorkspaceConnectionList.vue';
import TransferProgressModal from '@/components/TransferProgressModal.vue';

const sessionStore = useSessionStore();
const layoutStore = useLayoutStore();
const { activeSessionId, sessionList } = storeToRefs(sessionStore);
const { layoutConfig, leftSidebarVisible, rightSidebarVisible, leftSidebarSize, rightSidebarSize } =
  storeToRefs(layoutStore);
const showConnList = ref(false);
const showTransferModal = ref(false);
const { taskList, startListening, cancelTask, cleanup } = useTransferProgress();

let workspaceResizeDispatchRaf = 0;

function extractPaneSizes(payload: unknown): number[] {
  if (Array.isArray(payload)) {
    return payload
      .map((item) => (item && typeof item === 'object' ? (item as { size?: number }).size : undefined))
      .filter((size): size is number => Number.isFinite(size));
  }

  if (payload && typeof payload === 'object') {
    const panes = (payload as { panes?: Array<{ size?: number }> }).panes;
    if (Array.isArray(panes)) {
      return panes
        .map((item) => item.size)
        .filter((size): size is number => Number.isFinite(size));
    }
  }

  return [];
}

function clampSize(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function notifyWorkspaceLayoutResized() {
  if (typeof window === 'undefined') {
    return;
  }

  if (workspaceResizeDispatchRaf) {
    window.cancelAnimationFrame(workspaceResizeDispatchRaf);
  }

  workspaceResizeDispatchRaf = window.requestAnimationFrame(() => {
    window.dispatchEvent(new Event('resize'));
    window.dispatchEvent(new CustomEvent('nexus:layout-resized'));
    workspaceResizeDispatchRaf = 0;
  });
}

function handleWorkspacePaneResize(payload?: unknown) {
  const sizes = extractPaneSizes(payload);

  if (sizes.length > 0) {
    let cursor = 0;

    if (leftSidebarVisible.value && sizes[cursor] !== undefined) {
      layoutStore.setLeftSidebarSize(clampSize(sizes[cursor], 10, 30));
      cursor += 1;
    }

    if (sizes[cursor] !== undefined) {
      cursor += 1;
    }

    if (rightSidebarVisible.value && sizes[cursor] !== undefined) {
      layoutStore.setRightSidebarSize(clampSize(sizes[cursor], 15, 40));
    }
  }

  notifyWorkspaceLayoutResized();
}

const mainSize = computed(() => {
  let size = 100;
  if (leftSidebarVisible.value) size -= leftSidebarSize.value;
  if (rightSidebarVisible.value) size -= rightSidebarSize.value;
  return size;
});

async function warmupSftp(sessionId: string, connectionId: number) {
  try {
    const sftpSessionId = await sftpApi.open(connectionId);
    sessionStore.setSftpSession(sessionId, sftpSessionId);
  } catch {
    // ignore warmup failures and allow terminal-only usage
  }
}

async function handleConnect(conn: Connection) {
  showConnList.value = false;

  if (String(conn.type ?? 'SSH').toUpperCase() === 'RDP') {
    try {
      await desktopApi.openRdpConnection(conn.id);
    } catch (e: any) {
      window.alert(`RDP 启动失败: ${e.message ?? String(e)}`);
    }
    return;
  }

  const sid = sessionStore.createSession(conn.id, conn.name);
  try {
    const realSid = await sshApi.connect(conn.id);
    sessionStore.removeSession(sid);
    sessionStore.addSession({
      id: realSid,
      connectionId: conn.id,
      connectionName: conn.name,
      status: 'connected',
      createdAt: new Date().toISOString(),
      sftpReady: false,
      sftpSessionId: null,
      currentPath: '/',
    });
    sessionStore.setActive(realSid);
    void warmupSftp(realSid, conn.id);
  } catch {
    sessionStore.updateStatus(sid, 'disconnected');
  }
}

async function closeSession(sessionId: string) {
  const session = sessionStore.getSession(sessionId);
  if (session?.sftpSessionId) {
    try {
      await sftpApi.close(session.sftpSessionId);
    } catch {
      // ignore sftp close failures
    }
  }

  try {
    await sshApi.close(sessionId);
  } catch {
    // ignore backend close failures
  }
  sessionStore.removeSession(sessionId);
}

async function closeOthers(anchorSessionId: string) {
  const ids = sessionList.value
    .filter((session) => session.id !== anchorSessionId)
    .map((session) => session.id);
  for (const id of ids) {
    await closeSession(id);
  }
}

async function closeRight(anchorSessionId: string) {
  const index = sessionList.value.findIndex((session) => session.id === anchorSessionId);
  if (index < 0) return;
  const ids = sessionList.value.slice(index + 1).map((session) => session.id);
  for (const id of ids) {
    await closeSession(id);
  }
}

async function closeLeft(anchorSessionId: string) {
  const index = sessionList.value.findIndex((session) => session.id === anchorSessionId);
  if (index <= 0) return;
  const ids = sessionList.value.slice(0, index).map((session) => session.id);
  for (const id of ids) {
    await closeSession(id);
  }
}

function handleTransferCreated() {
  showTransferModal.value = true;
}

onMounted(() => {
  void startListening();
  window.addEventListener('transfer-created', handleTransferCreated);
});

onUnmounted(() => {
  window.removeEventListener('transfer-created', handleTransferCreated);
  if (workspaceResizeDispatchRaf) {
    window.cancelAnimationFrame(workspaceResizeDispatchRaf);
    workspaceResizeDispatchRaf = 0;
  }
  cleanup();
});
</script>

<style scoped>
.workspace {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base);
}

.workspace-layout {
  flex: 1;
  min-height: 0;
}

.workspace-layout :deep(.splitpanes__pane) {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.workspace-layout :deep(.splitpanes__splitter) {
  background: var(--border, #313244);
  position: relative;
  z-index: 1;
}

.workspace-layout :deep(.splitpanes__splitter) {
  width: 4px;
  margin: 0 -2px;
}

.workspace-layout :deep(.splitpanes__splitter:hover) {
  background: var(--blue, #89b4fa);
}

.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9000;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

.conn-popup {
  background: var(--bg-surface0);
  border: 1px solid var(--border);
  border-radius: 8px;
  width: 360px;
  max-height: 400px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.popup-title {
  padding: 12px 16px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
  border-bottom: 1px solid var(--border);
}
</style>

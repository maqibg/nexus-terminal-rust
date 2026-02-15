<template>
  <div class="workspace">
    <TerminalTabBar
      :sessions="sessionList"
      :active-session-id="activeSessionId"
      :header-visible="headerVisible"
      @activate="sessionStore.setActive($event)"
      @close="closeSession"
      @close-others="closeOthers"
      @close-right="closeRight"
      @close-left="closeLeft"
      @toggle-header="layoutStore.toggleHeaderVisibility()"
      @open-transfers="showTransferModal = true"
      @open-layout-configurator="showLayoutConfigurator = true"
      @add="showConnList = true"
    />

    <div class="workspace-body">
      <div class="workspace-left-tools">
        <button
          class="workspace-left-tool-btn"
          :class="{ 'workspace-left-tool-btn-active': activeLeftToolPane === 'connections' }"
          title="连接列表"
          @click="toggleLeftToolPane('connections')"
        >
          <i class="fas fa-network-wired"></i>
        </button>
        <button
          class="workspace-left-tool-btn"
          :class="{ 'workspace-left-tool-btn-active': activeLeftToolPane === 'docker' }"
          title="Docker 管理器"
          @click="toggleLeftToolPane('docker')"
        >
          <i class="fab fa-docker"></i>
        </button>
      </div>

      <div v-if="activeLeftToolPane" class="workspace-left-panel">
        <div class="workspace-left-panel-header">
          <div class="workspace-left-panel-title">
            {{ activeLeftToolPane === 'connections' ? '连接列表' : 'Docker 管理器' }}
          </div>
          <div class="workspace-left-panel-actions">
            <button
              v-if="activeLeftToolPane === 'connections'"
              class="workspace-left-panel-action-btn"
              title="新增连接"
              @click="showConnList = true"
            >
              <i class="fas fa-plus"></i>
            </button>
            <button
              class="workspace-left-panel-action-btn"
              title="关闭"
              @click="closeLeftToolPane"
            >
              <i class="fas fa-times"></i>
            </button>
          </div>
        </div>

        <WorkspaceConnectionList
          v-if="activeLeftToolPane === 'connections'"
          class="workspace-left-panel-content"
          @select="handleConnect"
        />

        <div v-else class="workspace-left-panel-content workspace-docker-empty">
          <i class="fab fa-docker workspace-docker-icon"></i>
          <div class="workspace-docker-title">远程主机 Docker 不可用</div>
          <div class="workspace-docker-desc">请确保远程主机上已安装并运行 Docker。</div>
        </div>
      </div>

      <Splitpanes
        :class="['workspace-layout', { 'workspace-layout-locked': layoutLocked }]"
        @resize="handleWorkspacePaneResize"
        @resized="handleWorkspacePaneResize"
      >
        <Pane v-if="effectiveLeftSidebarVisible" :size="leftSidebarSize" :min-size="10" :max-size="30">
          <LayoutRenderer v-if="layoutConfig.leftSidebar" :node="layoutConfig.leftSidebar" />
        </Pane>

        <Pane :size="mainSize" :min-size="40">
          <LayoutRenderer :node="layoutConfig.root" />
        </Pane>

        <Pane v-if="rightSidebarVisible" :size="rightSidebarSize" :min-size="15" :max-size="40">
          <LayoutRenderer v-if="layoutConfig.rightSidebar" :node="layoutConfig.rightSidebar" />
        </Pane>
      </Splitpanes>
    </div>

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

    <LayoutConfigurator
      :visible="showLayoutConfigurator"
      @close="showLayoutConfigurator = false"
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
import LayoutConfigurator from '@/components/LayoutConfigurator.vue';

const sessionStore = useSessionStore();
const layoutStore = useLayoutStore();
const { activeSessionId, sessionList } = storeToRefs(sessionStore);
const { layoutConfig, leftSidebarVisible, rightSidebarVisible, leftSidebarSize, rightSidebarSize, headerVisible, layoutLocked } =
  storeToRefs(layoutStore);
const showConnList = ref(false);
const showTransferModal = ref(false);
const showLayoutConfigurator = ref(false);
type LeftToolPane = 'connections' | 'docker';
const activeLeftToolPane = ref<LeftToolPane | null>(null);
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
  if (layoutLocked.value) {
    notifyWorkspaceLayoutResized();
    return;
  }

  const sizes = extractPaneSizes(payload);

  if (sizes.length > 0) {
    let cursor = 0;

    if (effectiveLeftSidebarVisible.value && sizes[cursor] !== undefined) {
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

function toggleLeftToolPane(pane: LeftToolPane) {
  activeLeftToolPane.value = activeLeftToolPane.value === pane ? null : pane;
}

function closeLeftToolPane() {
  activeLeftToolPane.value = null;
}

const effectiveLeftSidebarVisible = computed(() => leftSidebarVisible.value && !activeLeftToolPane.value);

const mainSize = computed(() => {
  let size = 100;
  if (effectiveLeftSidebarVisible.value) size -= leftSidebarSize.value;
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
  void layoutStore.loadLayout();
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

.workspace-body {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;
}

.workspace-left-tools {
  width: 42px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 4px 0;
  border-right: 1px solid var(--border);
  background: var(--bg-mantle);
}

.workspace-left-tool-btn {
  width: 34px;
  height: 34px;
  border: 0;
  border-radius: 8px;
  background: transparent;
  color: var(--text-dim);
  font-size: 15px;
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.workspace-left-tool-btn:hover {
  color: var(--text);
  background: var(--bg-surface0);
}

.workspace-left-tool-btn-active,
.workspace-left-tool-btn-active:hover {
  color: #fff;
  background: var(--blue);
}

.workspace-left-panel {
  width: 300px;
  min-width: 260px;
  max-width: 420px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-right: 1px solid var(--border);
  background: var(--bg-mantle);
}

.workspace-left-panel-header {
  height: 38px;
  padding: 0 6px 0 12px;
  border-bottom: 1px solid var(--border);
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.workspace-left-panel-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.workspace-left-panel-actions {
  display: flex;
  align-items: center;
  gap: 2px;
}

.workspace-left-panel-action-btn {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  font-size: 13px;
  transition: background-color 0.15s ease, color 0.15s ease;
}

.workspace-left-panel-action-btn:hover {
  color: var(--text);
  background: var(--bg-surface0);
}

.workspace-left-panel-content {
  flex: 1;
  min-height: 0;
}

.workspace-docker-empty {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  text-align: center;
  padding: 20px;
  gap: 10px;
}

.workspace-docker-icon {
  font-size: 42px;
  color: var(--text-dim);
}

.workspace-docker-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.workspace-docker-desc {
  max-width: 220px;
  font-size: 12px;
  line-height: 1.6;
  color: var(--text-sub);
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

.workspace-layout-locked :deep(.splitpanes__splitter) {
  pointer-events: none;
  opacity: 0.45;
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

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
      @open-ai-assistant="openAiAssistant()"
      @toggle-header="layoutStore.toggleHeaderVisibility()"
      @open-transfers="showTransferModal = true"
      @open-layout-configurator="showLayoutConfigurator = true"
      @add="showConnList = true"
    />

    <div class="workspace-body" @click="handleWorkspaceBodyClick">
      <div class="workspace-left-tools" @click.stop>
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
        <button
          class="workspace-left-tool-btn"
          :class="{ 'workspace-left-tool-btn-active': showTerminalAiPanel }"
          title="AI 助手"
          @click="toggleTerminalAiPanel()"
        >
          <i class="fas fa-robot"></i>
        </button>
      </div>

      <div v-if="activeLeftToolPane" class="workspace-left-panel" @click.stop>
        <div class="workspace-left-panel-header">
          <div class="workspace-left-panel-title">
            {{ leftPaneTitle }}
          </div>
          <div class="workspace-left-panel-actions">
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
        <DockerManager
          v-else-if="activeLeftToolPane === 'docker'"
          class="workspace-left-panel-content"
        />
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

      <div v-if="showTerminalAiPanel" class="workspace-right-ai-panel" @click.stop>
        <TerminalAIChatPanel
          ref="terminalAiPanelRef"
          class="workspace-right-ai-panel-content"
          :session-id="activeSession?.id ?? null"
          :connection-id="activeSession?.connectionId ?? null"
          :session-name="activeSession?.connectionName"
          :storage-id="activeSession?.id ?? undefined"
          :closable="true"
          @close="showTerminalAiPanel = false"
        />
      </div>
    </div>

    <Teleport to="body">
      <div v-if="showConnList" class="dialog-backdrop" @click.self="showConnList = false">
        <div class="conn-popup">
          <div class="popup-title">选择连接</div>
          <WorkspaceConnectionList @select="handleConnect" />
        </div>
      </div>
    </Teleport>

    <Teleport to="body">
      <div v-if="showFileManagerPopup" class="dialog-backdrop" @click.self="showFileManagerPopup = false">
        <div class="workspace-modal-popup">
          <div class="workspace-modal-header">
            <div class="workspace-modal-title">弹窗文件管理器</div>
            <button class="workspace-modal-close" title="关闭" @click="showFileManagerPopup = false">
              <i class="fas fa-times"></i>
            </button>
          </div>
          <div class="workspace-modal-body">
            <SftpBrowser />
          </div>
        </div>
      </div>
    </Teleport>

    <FileEditorOverlay />

    <TransferProgressModal
      :visible="showTransferModal"
      :tasks="taskList"
      @close="showTransferModal = false"
      @cancel="cancelTask"
      @pause-all="pauseAll"
      @resume-all="resumeAll"
      @cancel-all="cancelAll"
      @clear-completed="clearCompleted"
    />

    <LayoutConfigurator
      :visible="showLayoutConfigurator"
      @close="showLayoutConfigurator = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, onMounted, onUnmounted } from 'vue';
import { storeToRefs } from 'pinia';
import { Splitpanes, Pane } from 'splitpanes';
import 'splitpanes/dist/splitpanes.css';
import { useSessionStore } from '@/stores/session';
import { notifyGlobalLayoutResized, useLayoutStore } from '@/stores/layout';
import { useSettingsStore } from '@/stores/settings';
import { useFileEditorStore } from '@/stores/fileEditor';
import { type Connection } from '@/lib/api';
import { useTransferProgress } from '@/composables/useTransferProgress';
import { useAlertDialog } from '@/composables/useAlertDialog';
import { useSessionLifecycle } from '@/composables/useSessionLifecycle';
import LayoutRenderer from '@/components/LayoutRenderer.vue';
import TerminalTabBar from '@/components/TerminalTabBar.vue';
import WorkspaceConnectionList from '@/components/WorkspaceConnectionList.vue';
import TransferProgressModal from '@/components/TransferProgressModal.vue';
import LayoutConfigurator from '@/components/LayoutConfigurator.vue';
import SftpBrowser from '@/components/SftpBrowser.vue';
import FileEditorOverlay from '@/components/FileEditorOverlay.vue';
import TerminalAIChatPanel from '@/components/AI/TerminalAIChatPanel.vue';
import DockerManager from '@/components/DockerManager.vue';

const sessionStore = useSessionStore();
const layoutStore = useLayoutStore();
const settingsStore = useSettingsStore();
const fileEditorStore = useFileEditorStore();
const { alert } = useAlertDialog();
const { connectConnection, closeSession: closeManagedSession } = useSessionLifecycle(alert);
const { activeSessionId, activeSession, sessionList } = storeToRefs(sessionStore);
const { layoutConfig, leftSidebarVisible, rightSidebarVisible, leftSidebarSize, rightSidebarSize, headerVisible, layoutLocked } =
  storeToRefs(layoutStore);
const showConnList = ref(false);
const showTransferModal = ref(false);
const showLayoutConfigurator = ref(false);
const showFileManagerPopup = ref(false);

type LeftToolPane = 'connections' | 'docker';
const activeLeftToolPane = ref<LeftToolPane | null>(null);
const showTerminalAiPanel = ref(false);
interface TerminalAiActionDetail {
  prompt?: string;
  autoSend?: boolean;
}
interface TerminalAiPanelExpose {
  setInput: (value: string) => void;
  sendMessage: (override?: string) => Promise<void>;
}
const terminalAiPanelRef = ref<TerminalAiPanelExpose | null>(null);
const { taskList, startListening, cancelTask, pauseAll, resumeAll, cancelAll, clearCompleted, cleanup } = useTransferProgress();
const workspaceSidebarPersistent = computed(() => settingsStore.getBoolean('workspaceSidebarPersistent', false));

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
  if (workspaceResizeDispatchRaf) {
    window.cancelAnimationFrame(workspaceResizeDispatchRaf);
  }

  workspaceResizeDispatchRaf = window.requestAnimationFrame(() => {
    notifyGlobalLayoutResized();
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

function toggleTerminalAiPanel() {
  showTerminalAiPanel.value = !showTerminalAiPanel.value;
}

async function openAiAssistant(detail?: TerminalAiActionDetail) {
  const prompt = detail?.prompt?.trim();
  if (!prompt) {
    showTerminalAiPanel.value = !showTerminalAiPanel.value;
    return;
  }

  showTerminalAiPanel.value = true;
  await nextTick();
  const panel = terminalAiPanelRef.value;
  if (!panel) {
    return;
  }

  if (detail?.autoSend) {
    await panel.sendMessage(prompt);
    return;
  }

  panel.setInput(prompt);
}

function handleOpenAiAssistantEvent(event: Event) {
  const detail = (event as CustomEvent<TerminalAiActionDetail>).detail;
  void openAiAssistant(detail);
}

function handleWorkspaceBodyClick() {
  if (workspaceSidebarPersistent.value) {
    return;
  }
  closeLeftToolPane();
}

function handleOpenFileManagerPopup() {
  if (!settingsStore.getBoolean('showPopupFileManager', false)) {
    return;
  }
  showFileManagerPopup.value = true;
}

function handleOpenFileEditorPopup() {
  if (!settingsStore.getBoolean('showPopupFileEditor', false)) {
    return;
  }

  const sid = activeSessionId.value;
  if (!sid) {
    return;
  }

  fileEditorStore.triggerPopup('', sid);
}

const effectiveLeftSidebarVisible = computed(() => leftSidebarVisible.value && !activeLeftToolPane.value);

const leftPaneTitle = computed(() => {
  return activeLeftToolPane.value === 'docker' ? 'Docker 管理器' : '连接列表';
});

const mainSize = computed(() => {
  let size = 100;
  if (effectiveLeftSidebarVisible.value) size -= leftSidebarSize.value;
  if (rightSidebarVisible.value) size -= rightSidebarSize.value;
  return size;
});

async function handleConnect(conn: Connection) {
  showConnList.value = false;
  await connectConnection(conn);
}

async function closeSession(sessionId: string) {
  await closeManagedSession(sessionId);
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
  void settingsStore
    .loadAll()
    .then(() => undefined)
    .catch(() => undefined);

  window.addEventListener('transfer-created', handleTransferCreated);
  window.addEventListener('nexus:workspace:file-manager-popup:open', handleOpenFileManagerPopup as EventListener);
  window.addEventListener('nexus:workspace:file-editor-popup:open', handleOpenFileEditorPopup as EventListener);
  window.addEventListener('nexus:workspace:open-ai-assistant', handleOpenAiAssistantEvent as EventListener);
});

onUnmounted(() => {
  window.removeEventListener('transfer-created', handleTransferCreated);
  window.removeEventListener('nexus:workspace:file-manager-popup:open', handleOpenFileManagerPopup as EventListener);
  window.removeEventListener('nexus:workspace:file-editor-popup:open', handleOpenFileEditorPopup as EventListener);
  window.removeEventListener('nexus:workspace:open-ai-assistant', handleOpenAiAssistantEvent as EventListener);
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
  color: var(--button-text-color);
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

.workspace-layout {
  flex: 1;
  min-height: 0;
}

.workspace-right-ai-panel {
  width: 420px;
  min-width: 340px;
  max-width: 560px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  border-left: 1px solid var(--border);
  background: var(--bg-mantle);
}

.workspace-right-ai-panel-content {
  flex: 1;
  min-height: 0;
}

.workspace-layout :deep(.splitpanes__pane) {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.workspace-layout :deep(.splitpanes__splitter) {
  background: var(--border);
  position: relative;
  z-index: 1;
}

.workspace-layout :deep(.splitpanes__splitter) {
  width: 4px;
  margin: 0 -2px;
}

.workspace-layout :deep(.splitpanes__splitter:hover) {
  background: var(--blue);
}

.workspace-layout-locked :deep(.splitpanes__splitter) {
  pointer-events: none;
  opacity: 0.45;
}

.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9000;
  background: var(--ui-overlay);
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

.workspace-modal-popup {
  width: min(1180px, 92vw);
  height: min(840px, 86vh);
  display: flex;
  flex-direction: column;
  background: var(--bg-surface0);
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
}

.workspace-modal-header {
  height: 42px;
  padding: 0 10px 0 14px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border);
  background: var(--bg-mantle);
}

.workspace-modal-title {
  font-size: 13px;
  font-weight: 600;
  color: var(--text);
}

.workspace-modal-close {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
}

.workspace-modal-close:hover {
  color: var(--text);
  background: var(--bg-surface0);
}

.workspace-modal-body {
  flex: 1;
  min-height: 0;
}
</style>





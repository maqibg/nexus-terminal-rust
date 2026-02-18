<template>
  <div class="command-history-panel">
    <div class="panel-controls">
      <input
        ref="searchInputRef"
        v-model="searchTerm"
        type="text"
        class="search-input"
        data-focus-id="commandHistorySearch"
        placeholder="搜索历史记录..."
        @keydown="handleSearchInputKeydown"
      />
      <button class="clear-btn" title="清空历史记录" @click="confirmClearAll">
        <i class="fas fa-trash-alt"></i>
      </button>
    </div>

    <div class="history-list">
      <div v-if="historyStore.loading && filtered.length === 0" class="empty-state">
        <i class="fas fa-spinner fa-spin"></i>
        <span>加载历史记录...</span>
      </div>

      <div v-else-if="filtered.length === 0" class="empty-state">
        <i class="fas fa-history"></i>
        <span>{{ searchTerm.trim() ? '无匹配结果' : '没有历史记录' }}</span>
      </div>

      <ul v-else class="history-items">
        <li
          v-for="(entry, index) in filtered"
          :key="entry.id"
          class="history-item"
          :class="{ selected: index === selectedIndex }"
          :title="entry.command"
          @click="executeCommand(entry.command)"
          @contextmenu.prevent="showContextMenu($event, entry.command)"
        >
          <span class="command-text">{{ entry.command }}</span>
          <div class="item-actions">
            <button class="action-btn" title="复制" @click.stop="copyCommand(entry.command)">
              <i class="fas fa-copy"></i>
            </button>
            <button class="action-btn danger" title="删除" @click.stop="deleteEntry(entry.id)">
              <i class="fas fa-times"></i>
            </button>
          </div>
        </li>
      </ul>
    </div>

    <div
      v-if="contextMenuVisible && contextCommand"
      ref="contextMenuRef"
      class="command-history-context-menu"
      :style="{ left: `${contextMenuPosition.x}px`, top: `${contextMenuPosition.y}px` }"
      @click.stop
    >
      <button class="context-item" @click="sendContextCommandToAllSessions">发送到全部会话</button>
      <button class="context-item" @click="copyCommand(contextCommand)">复制命令</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { sshApi } from '@/lib/api';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { useCommandHistoryStore } from '@/stores/commandHistory';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { useSessionStore } from '@/stores/session';
import { useUINotificationStore } from '@/stores/uiNotifications';

const historyStore = useCommandHistoryStore();
const sessionStore = useSessionStore();
const focusSwitcherStore = useFocusSwitcherStore();
const notificationStore = useUINotificationStore();
const { confirm } = useConfirmDialog();

const searchInputRef = ref<HTMLInputElement>();
const searchTerm = ref('');
const selectedIndex = ref(-1);

const contextMenuVisible = ref(false);
const contextMenuRef = ref<HTMLDivElement>();
const contextCommand = ref('');
const contextMenuPosition = reactive({ x: 0, y: 0 });

interface CommandInputSyncEventDetail {
  term?: string;
}

let unregisterFocusAction: (() => void) | null = null;

const filtered = computed(() => {
  const keyword = searchTerm.value.trim().toLowerCase();
  const list = historyStore.items.slice(0, 300);
  if (!keyword) {
    return list;
  }
  return list.filter((entry) => entry.command.toLowerCase().includes(keyword));
});

watch(filtered, (list) => {
  if (!list.length) {
    selectedIndex.value = -1;
    return;
  }

  if (selectedIndex.value >= list.length) {
    selectedIndex.value = list.length - 1;
  }
});

function isVisibleInput(input: HTMLInputElement | undefined): input is HTMLInputElement {
  if (!input || !input.isConnected || input.disabled) {
    return false;
  }
  const style = window.getComputedStyle(input);
  if (style.display === 'none' || style.visibility === 'hidden') {
    return false;
  }
  const rect = input.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
}

function focusSearchInput(): boolean | undefined {
  if (!isVisibleInput(searchInputRef.value)) {
    return undefined;
  }

  searchInputRef.value.focus();
  searchInputRef.value.select();
  return document.activeElement === searchInputRef.value;
}

async function loadHistory() {
  await historyStore.fetchAll(300, 0);
}

async function executeCommand(command: string) {
  const sessionId = sessionStore.activeSessionId;
  if (!sessionId) {
    notificationStore.addNotification('warning', '没有活动会话，无法执行命令');
    return;
  }

  const payload = btoa(unescape(encodeURIComponent(`${command}\n`)));
  await sshApi.write(sessionId, payload);
}

async function copyCommand(command: string) {
  try {
    await navigator.clipboard.writeText(command);
    notificationStore.addNotification('success', '已复制到剪贴板');
  } catch {
    notificationStore.addNotification('error', '复制失败');
  }
}

async function deleteEntry(id: number) {
  const deleted = await historyStore.remove(id);
  if (!deleted) {
    notificationStore.addNotification('warning', '删除失败，记录可能已不存在');
  }
}

async function confirmClearAll() {
  const confirmed = await confirm('清空历史记录', '确定要清空所有历史记录吗？');
  if (!confirmed) {
    return;
  }

  await historyStore.clear();
}

function handleSearchInputKeydown(event: KeyboardEvent) {
  if (!filtered.value.length) {
    return;
  }

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      selectedIndex.value = selectedIndex.value >= filtered.value.length - 1 ? 0 : selectedIndex.value + 1;
      break;
    case 'ArrowUp':
      event.preventDefault();
      selectedIndex.value = selectedIndex.value <= 0 ? filtered.value.length - 1 : selectedIndex.value - 1;
      break;
    case 'Enter':
      event.preventDefault();
      if (selectedIndex.value >= 0 && selectedIndex.value < filtered.value.length) {
        void executeCommand(filtered.value[selectedIndex.value].command);
      }
      break;
    default:
      break;
  }
}

function closeContextMenu() {
  contextMenuVisible.value = false;
  contextCommand.value = '';
}

function adjustContextMenuPosition() {
  if (!contextMenuRef.value) {
    return;
  }

  const rect = contextMenuRef.value.getBoundingClientRect();
  if (contextMenuPosition.x + rect.width > window.innerWidth) {
    contextMenuPosition.x = Math.max(8, window.innerWidth - rect.width - 8);
  }
  if (contextMenuPosition.y + rect.height > window.innerHeight) {
    contextMenuPosition.y = Math.max(8, window.innerHeight - rect.height - 8);
  }
}

function showContextMenu(event: MouseEvent, command: string) {
  contextCommand.value = command;
  contextMenuPosition.x = event.clientX;
  contextMenuPosition.y = event.clientY;
  contextMenuVisible.value = true;
  void nextTick(adjustContextMenuPosition);
}

async function sendContextCommandToAllSessions() {
  const command = contextCommand.value;
  closeContextMenu();

  const connectedSessions = sessionStore.sessionList.filter((session) => session.status === 'connected');
  if (!connectedSessions.length) {
    notificationStore.addNotification('warning', '没有可用的在线会话');
    return;
  }

  const payload = btoa(unescape(encodeURIComponent(`${command}\n`)));
  await Promise.allSettled(connectedSessions.map((session) => sshApi.write(session.id, payload)));
  notificationStore.addNotification('success', `已发送到 ${connectedSessions.length} 个会话`);
}

function handleDocumentPointerDown(event: MouseEvent) {
  if (!contextMenuVisible.value) {
    return;
  }

  const target = event.target as Node;
  if (contextMenuRef.value?.contains(target)) {
    return;
  }

  closeContextMenu();
}


function handleCommandInputSearchSync(event: Event) {
  const detail = (event as CustomEvent<CommandInputSyncEventDetail>).detail;
  searchTerm.value = String(detail?.term ?? '');
}
function handleHistoryUpdated() {
  void loadHistory();
}

onMounted(async () => {
  await loadHistory();
  unregisterFocusAction = focusSwitcherStore.registerFocusAction('commandHistorySearch', focusSearchInput);
  document.addEventListener('mousedown', handleDocumentPointerDown);
  window.addEventListener('nexus:command-history-updated', handleHistoryUpdated);
  window.addEventListener('nexus:command-history:set-search', handleCommandInputSearchSync as EventListener);
});

onUnmounted(() => {
  unregisterFocusAction?.();
  unregisterFocusAction = null;
  document.removeEventListener('mousedown', handleDocumentPointerDown);
  window.removeEventListener('nexus:command-history-updated', handleHistoryUpdated);
  window.removeEventListener('nexus:command-history:set-search', handleCommandInputSearchSync as EventListener);
});
</script>

<style scoped>
.command-history-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  overflow: hidden;
}

.panel-controls {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border, #313244);
}

.search-input {
  flex: 1;
  min-width: 0;
  height: 30px;
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
  padding: 0 10px;
  font-size: 12px;
  outline: none;
  box-sizing: border-box;
}

.search-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.search-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.clear-btn {
  width: 30px;
  height: 30px;
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
}

.clear-btn:hover {
  color: var(--red, #f38ba8);
  border-color: rgba(243, 139, 168, 0.4);
  background: rgba(243, 139, 168, 0.1);
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 6px;
}

.history-items {
  margin: 0;
  padding: 0;
  list-style: none;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 2px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.15s;
}

.history-item:hover {
  background: rgba(137, 180, 250, 0.1);
}

.history-item.selected {
  background: rgba(137, 180, 250, 0.22);
}

.command-text {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  font-family: 'Cascadia Mono', 'Consolas', monospace;
  color: var(--text, #cdd6f4);
}

.item-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
}

.history-item:hover .item-actions,
.history-item:focus-within .item-actions {
  opacity: 1;
}

.action-btn {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 11px;
}

.action-btn:hover {
  background: rgba(137, 180, 250, 0.18);
  color: var(--blue, #89b4fa);
}

.action-btn.danger:hover {
  background: rgba(243, 139, 168, 0.18);
  color: var(--red, #f38ba8);
}

.empty-state {
  min-height: 120px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: var(--text-dim, #6c7086);
  font-size: 12px;
}

.empty-state i {
  font-size: 20px;
  opacity: 0.6;
}

.command-history-context-menu {
  position: fixed;
  z-index: 3200;
  min-width: 170px;
  background: var(--bg-surface0, #313244);
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
  padding: 4px;
}

.context-item {
  width: 100%;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text, #cdd6f4);
  font-size: 12px;
  text-align: left;
  padding: 7px 10px;
  cursor: pointer;
}

.context-item:hover {
  background: rgba(137, 180, 250, 0.12);
}
</style>



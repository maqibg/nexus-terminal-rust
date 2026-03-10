<template>
  <div class="command-history-panel">
    <div class="panel-shell">
      <div class="panel-controls">
        <input
          ref="searchInputRef"
          type="text"
          class="search-input"
          data-focus-id="commandHistorySearch"
          :value="searchTerm"
          placeholder="搜索历史记录..."
          @input="updateSearchTerm"
          @keydown="handleSearchInputKeydown"
          @blur="handleSearchInputBlur"
        />
        <button class="clear-btn" title="清空历史记录" @click="confirmClearAll">
          <i class="fas fa-trash-alt"></i>
        </button>
      </div>

      <div class="history-list">
        <div v-if="historyStore.loading && filteredHistory.length === 0" class="empty-state">
          <i class="fas fa-spinner fa-spin"></i>
          <span>加载历史记录...</span>
        </div>

        <div v-else-if="filteredHistory.length === 0" class="empty-state">
          <i class="fas fa-history"></i>
          <span>没有历史记录</span>
        </div>

        <ul v-else ref="historyListRef" class="history-items">
          <li
            v-for="(entry, index) in filteredHistory"
            :key="entry.id"
            class="history-item"
            :class="{ selected: index === selectedIndex }"
            :title="entry.command"
            @click="executeCommand(entry.command)"
            @contextmenu.prevent="showContextMenu($event, entry)"
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
    </div>

    <div
      v-if="contextMenuVisible && contextEntry"
      ref="contextMenuRef"
      class="command-history-context-menu"
      :style="{ left: `${contextMenuPosition.x}px`, top: `${contextMenuPosition.y}px` }"
      @click.stop
    >
      <button class="context-item" @click="sendContextCommandToAllSessions">发送到全部会话</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { sshApi, type CommandHistory } from '@/lib/api';
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
const historyListRef = ref<HTMLUListElement | null>(null);
const contextMenuVisible = ref(false);
const contextMenuRef = ref<HTMLDivElement>();
const contextEntry = ref<CommandHistory | null>(null);
const contextMenuPosition = reactive({ x: 0, y: 0 });

interface CommandInputSyncEventDetail {
  term?: string;
}

let unregisterFocusAction: (() => void) | null = null;

const searchTerm = computed(() => historyStore.searchTerm);
const filteredHistory = computed(() => historyStore.filteredItems);
const selectedIndex = computed(() => historyStore.selectedIndex);

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
  if (!historyStore.items.length) {
    await historyStore.fetchAll(300, 0);
  }
}

function updateSearchTerm(event: Event) {
  const target = event.target as HTMLInputElement;
  historyStore.setSearchTerm(target.value);
}

async function scrollToSelected(index: number) {
  await nextTick();
  if (index < 0 || !historyListRef.value) {
    return;
  }

  const target = historyListRef.value.children[index] as HTMLLIElement | undefined;
  target?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
}

watch(selectedIndex, (nextIndex) => {
  void scrollToSelected(nextIndex);
});

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
  if (!filteredHistory.value.length) {
    return;
  }

  switch (event.key) {
    case 'ArrowDown':
      event.preventDefault();
      historyStore.selectNext();
      break;
    case 'ArrowUp':
      event.preventDefault();
      historyStore.selectPrevious();
      break;
    case 'Enter':
      event.preventDefault();
      if (selectedIndex.value >= 0 && selectedIndex.value < filteredHistory.value.length) {
        void executeCommand(filteredHistory.value[selectedIndex.value].command);
      }
      break;
    default:
      break;
  }
}

function handleSearchInputBlur() {
  setTimeout(() => {
    const activeElement = document.activeElement;
    if (activeElement !== searchInputRef.value && !historyListRef.value?.contains(activeElement)) {
      historyStore.resetSelection();
    }
  }, 100);
}

function closeContextMenu() {
  contextMenuVisible.value = false;
  contextEntry.value = null;
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

function showContextMenu(event: MouseEvent, entry: CommandHistory) {
  contextEntry.value = entry;
  contextMenuPosition.x = event.clientX;
  contextMenuPosition.y = event.clientY;
  contextMenuVisible.value = true;
  void nextTick(adjustContextMenuPosition);
}

async function sendContextCommandToAllSessions() {
  const command = contextEntry.value?.command ?? '';
  closeContextMenu();
  if (!command) {
    return;
  }

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
  historyStore.setSearchTerm(String(detail?.term ?? ''));
}

function handleHistoryUpdated() {
  void historyStore.fetchAll(300, 0);
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
  overflow: hidden;
  background: var(--bg-base, #1e1e2e);
}

.panel-shell {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
}

.panel-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
  padding: 8px;
  background: var(--bg-base, #1e1e2e);
}

.search-input {
  flex: 1;
  min-width: 0;
  padding: 6px 12px;
  border: 1px solid color-mix(in srgb, var(--border, #45475a) 80%, transparent);
  border-radius: 10px;
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
  font-size: calc(13px + var(--ui-font-size-offset));
  outline: none;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.search-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.search-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.clear-btn {
  width: 32px;
  height: 32px;
  border: 1px solid color-mix(in srgb, var(--border, #45475a) 80%, transparent);
  border-radius: 10px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  flex-shrink: 0;
  transition: color 0.15s ease, background 0.15s ease, border-color 0.15s ease;
}

.clear-btn:hover {
  color: var(--red, #f38ba8);
  border-color: rgba(243, 139, 168, 0.45);
  background: rgba(243, 139, 168, 0.1);
}

.history-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 8px;
}

.history-items {
  list-style: none;
  padding: 0;
  margin: 0;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 4px;
  padding: 10px 12px;
  border-radius: 8px;
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
}

.history-item:hover {
  background: rgba(137, 180, 250, 0.1);
}

.history-item.selected {
  background: rgba(137, 180, 250, 0.2);
  font-weight: 600;
}

.command-text {
  flex: 1;
  min-width: 0;
  margin-right: 8px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: 'Cascadia Mono', 'Consolas', monospace;
  font-size: calc(13px + var(--ui-font-size-offset));
  color: var(--text, #cdd6f4);
}

.item-actions {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.15s ease;
}

.history-item:hover .item-actions,
.history-item:focus-within .item-actions {
  opacity: 1;
}

.action-btn {
  padding: 6px;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  transition: background 0.15s ease, color 0.15s ease;
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
  min-height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 24px;
  color: var(--text-dim, #6c7086);
  font-size: calc(12px + var(--ui-font-size-offset));
  text-align: center;
}

.empty-state i {
  font-size: calc(20px + var(--ui-font-size-offset));
  opacity: 0.65;
}

.command-history-context-menu {
  position: fixed;
  z-index: 3200;
  min-width: 180px;
  padding: 6px;
  background: var(--bg-surface0, #313244);
  border: 1px solid color-mix(in srgb, var(--border, #45475a) 85%, transparent);
  border-radius: 10px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
}

.context-item {
  width: 100%;
  border: none;
  border-radius: 8px;
  background: transparent;
  color: var(--text, #cdd6f4);
  font-size: calc(12px + var(--ui-font-size-offset));
  text-align: left;
  padding: 8px 10px;
  cursor: pointer;
}

.context-item:hover {
  background: rgba(137, 180, 250, 0.12);
  color: var(--blue, #89b4fa);
}
</style>

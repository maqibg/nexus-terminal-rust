<template>
  <div class="quick-commands-panel" :class="{ 'compact-mode': isCompactMode }">
    <div class="panel-controls">
      <input
        ref="searchInputRef"
        v-model="searchTerm"
        type="text"
        class="search-input"
        data-focus-id="quickCommandsSearch"
        placeholder="搜索名称或指令..."
      />
      <button class="ctrl-btn" :title="sortButtonTitle" @click="toggleSortBy">
        <i class="fas" :class="sortButtonIcon"></i>
      </button>
      <button
        class="ctrl-btn"
        :class="{ active: isCompactMode }"
        :title="isCompactMode ? '普通模式' : '紧凑模式'"
        @click="isCompactMode = !isCompactMode"
      >
        <i class="fas" :class="isCompactMode ? 'fa-compress-alt' : 'fa-expand-alt'"></i>
      </button>
      <button class="ctrl-btn add-btn" title="添加快捷指令" @click="openAddForm">
        <i class="fas fa-plus"></i>
      </button>
    </div>

    <div class="commands-list">
      <div v-if="quickCommandsStore.loading && groupedCommands.length === 0" class="empty-state">
        <i class="fas fa-spinner fa-spin"></i>
        <span>加载快捷指令...</span>
      </div>

      <div v-else-if="!quickCommandsStore.loading && groupedCommands.length === 0" class="empty-state">
        <i class="fas fa-bolt"></i>
        <span>{{ searchTerm.trim() ? '无匹配结果' : '没有快捷指令' }}</span>
      </div>

      <template v-else>
        <section v-for="group in groupedCommands" :key="group.name" class="command-group">
          <button class="group-header" @click="toggleGroup(group.name)">
            <i
              class="fas group-chevron"
              :class="expandedGroups[group.name] ? 'fa-chevron-down' : 'fa-chevron-right'"
            ></i>
            <span class="group-name">{{ group.name }}</span>
            <span class="group-count">{{ group.commands.length }}</span>
          </button>

          <ul v-show="expandedGroups[group.name]" class="group-items">
            <li
              v-for="command in group.commands"
              :key="command.id"
              class="command-item"
              :title="command.command"
              @click="prepareExecuteCommand(command)"
              @contextmenu.prevent="showContextMenu($event, command)"
            >
              <div class="command-main">
                <span class="command-name">{{ command.name }}</span>
                <span v-if="!isCompactMode" class="command-text">{{ command.command }}</span>
              </div>
              <div class="command-actions">
                <button class="item-btn" title="复制" @click.stop="copyCommand(command.command)">
                  <i class="fas fa-copy"></i>
                </button>
                <button class="item-btn" title="编辑" @click.stop="openEditForm(command)">
                  <i class="fas fa-pen"></i>
                </button>
                <button class="item-btn danger" title="删除" @click.stop="handleDelete(command)">
                  <i class="fas fa-times"></i>
                </button>
              </div>
            </li>
          </ul>
        </section>
      </template>
    </div>

    <div
      v-if="contextMenuVisible && contextTarget"
      ref="contextMenuRef"
      class="quick-command-context-menu"
      :style="{ left: `${contextMenuPosition.x}px`, top: `${contextMenuPosition.y}px` }"
      @click.stop
    >
      <button class="context-item" @click="sendContextCommandToAllSessions">
        发送到全部会话
      </button>
    </div>

    <AddEditQuickCommandForm
      :visible="showAddEditForm"
      :edit-data="editingCommand"
      @close="closeForm"
      @save="handleSave"
    />

    <Teleport to="body">
      <div v-if="showVariableDialog" class="variable-dialog-backdrop" @click.self="closeVariableDialog">
        <div class="variable-dialog">
          <h3>执行快捷指令</h3>
          <p class="dialog-hint">请填写命令变量</p>
          <label v-for="key in variableOrder" :key="key" class="variable-field">
            <span>{{ key }}</span>
            <input v-model="variableValues[key]" type="text" />
          </label>
          <div class="dialog-actions">
            <button class="dialog-btn secondary" @click="closeVariableDialog">取消</button>
            <button class="dialog-btn primary" @click="executeVariableCommand">执行</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, reactive, ref } from 'vue';
import { storeToRefs } from 'pinia';
import AddEditQuickCommandForm from '@/components/AddEditQuickCommandForm.vue';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { sshApi, type QuickCommand } from '@/lib/api';
import { useCommandHistoryStore } from '@/stores/commandHistory';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { useQuickCommandsStore } from '@/stores/quickCommands';
import { useSessionStore } from '@/stores/session';
import { useUINotificationStore } from '@/stores/uiNotifications';

interface GroupedCommands {
  name: string;
  commands: QuickCommand[];
}

const UNTAGGED_GROUP_NAME = '未标记';

const quickCommandsStore = useQuickCommandsStore();
const historyStore = useCommandHistoryStore();
const sessionStore = useSessionStore();
const focusSwitcherStore = useFocusSwitcherStore();
const notificationStore = useUINotificationStore();
const { confirm } = useConfirmDialog();

const { items: quickCommands } = storeToRefs(quickCommandsStore);

const searchInputRef = ref<HTMLInputElement>();
const searchTerm = ref('');
const sortBy = ref<'name' | 'usage'>('name');
const isCompactMode = ref(false);
const expandedGroups = reactive<Record<string, boolean>>({});

const showAddEditForm = ref(false);
const editingCommand = ref<QuickCommand | null>(null);

const contextMenuVisible = ref(false);
const contextMenuRef = ref<HTMLDivElement>();
const contextMenuTarget = ref<QuickCommand | null>(null);
const contextTarget = computed(() => contextMenuTarget.value);
const contextMenuPosition = reactive({ x: 0, y: 0 });

const showVariableDialog = ref(false);
const pendingCommand = ref<QuickCommand | null>(null);
const variableOrder = ref<string[]>([]);
const variableValues = reactive<Record<string, string>>({});

let unregisterFocusAction: (() => void) | null = null;

const sortButtonIcon = computed(() => (sortBy.value === 'name' ? 'fa-sort-alpha-down' : 'fa-clock'));
const sortButtonTitle = computed(() => (sortBy.value === 'name' ? '按名称排序' : '按使用频率排序'));

const filteredCommands = computed(() => {
  const keyword = searchTerm.value.trim().toLowerCase();
  const list = quickCommands.value.filter((item) => {
    if (!keyword) {
      return true;
    }

    return (
      item.name.toLowerCase().includes(keyword)
      || item.command.toLowerCase().includes(keyword)
      || (item.tags ?? []).some((tag) => tag.toLowerCase().includes(keyword))
    );
  });

  list.sort((a, b) => {
    if (sortBy.value === 'name') {
      return a.name.localeCompare(b.name, 'zh-Hans-CN', { sensitivity: 'base' });
    }

    if (a.usage_count !== b.usage_count) {
      return b.usage_count - a.usage_count;
    }

    return a.name.localeCompare(b.name, 'zh-Hans-CN', { sensitivity: 'base' });
  });

  return list;
});

const groupedCommands = computed<GroupedCommands[]>(() => {
  const groups: Record<string, QuickCommand[]> = {};

  for (const item of filteredCommands.value) {
    const tag = item.tags?.[0] || UNTAGGED_GROUP_NAME;
    if (!groups[tag]) {
      groups[tag] = [];
      if (expandedGroups[tag] === undefined) {
        expandedGroups[tag] = true;
      }
    }
    groups[tag].push(item);
  }

  const groupNames = Object.keys(groups).sort((a, b) => {
    if (a === UNTAGGED_GROUP_NAME) {
      return 1;
    }
    if (b === UNTAGGED_GROUP_NAME) {
      return -1;
    }
    return a.localeCompare(b, 'zh-Hans-CN', { sensitivity: 'base' });
  });

  return groupNames.map((name) => ({ name, commands: groups[name] }));
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

function toggleGroup(name: string) {
  expandedGroups[name] = !expandedGroups[name];
}

function toggleSortBy() {
  sortBy.value = sortBy.value === 'name' ? 'usage' : 'name';
}

function openAddForm() {
  editingCommand.value = null;
  showAddEditForm.value = true;
}

function openEditForm(command: QuickCommand) {
  editingCommand.value = { ...command };
  showAddEditForm.value = true;
}

function closeForm() {
  showAddEditForm.value = false;
  editingCommand.value = null;
}

async function handleSave(payload: { name: string; command: string; variables?: string; tags?: string[] }) {
  if (editingCommand.value) {
    await quickCommandsStore.update(editingCommand.value.id, payload);
  } else {
    await quickCommandsStore.create(payload);
  }

  closeForm();
}

async function handleDelete(command: QuickCommand) {
  const confirmed = await confirm('删除快捷指令', `确定删除“${command.name}”吗？`);
  if (!confirmed) {
    return;
  }

  await quickCommandsStore.remove(command.id);
}

async function copyCommand(command: string) {
  try {
    await navigator.clipboard.writeText(command);
    notificationStore.addNotification('success', '已复制到剪贴板');
  } catch {
    notificationStore.addNotification('error', '复制失败');
  }
}

function parseVariables(raw: string | undefined): Record<string, string> {
  if (!raw) {
    return {};
  }

  try {
    const parsed = JSON.parse(raw) as unknown;
    if (!parsed || typeof parsed !== 'object' || Array.isArray(parsed)) {
      return {};
    }

    return Object.fromEntries(
      Object.entries(parsed as Record<string, unknown>).map(([key, value]) => [key, String(value ?? '')]),
    );
  } catch {
    return {};
  }
}

function resetVariableValues() {
  for (const key of Object.keys(variableValues)) {
    delete variableValues[key];
  }
}

function openVariableDialog(command: QuickCommand): boolean {
  const placeholderSet = new Set<string>();
  for (const match of command.command.matchAll(/\$\{([^}]+)\}/g)) {
    const key = match[1]?.trim();
    if (key) {
      placeholderSet.add(key);
    }
  }

  const defaults = parseVariables(command.variables);
  for (const key of Object.keys(defaults)) {
    placeholderSet.add(key);
  }

  if (placeholderSet.size === 0) {
    return false;
  }

  variableOrder.value = Array.from(placeholderSet);
  resetVariableValues();
  for (const key of variableOrder.value) {
    variableValues[key] = defaults[key] ?? '';
  }

  pendingCommand.value = command;
  showVariableDialog.value = true;
  return true;
}

function closeVariableDialog() {
  showVariableDialog.value = false;
  pendingCommand.value = null;
  variableOrder.value = [];
  resetVariableValues();
}

function escapeRegExp(raw: string): string {
  return raw.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

async function runCommand(command: QuickCommand, finalCommand: string, sendToAllSessions = false) {
  const payload = btoa(unescape(encodeURIComponent(`${finalCommand}\n`)));

  if (sendToAllSessions) {
    const connectedSessions = sessionStore.sessionList.filter((session) => session.status === 'connected');
    if (!connectedSessions.length) {
      notificationStore.addNotification('warning', '没有可用的在线会话');
      return;
    }

    await Promise.allSettled(connectedSessions.map((session) => sshApi.write(session.id, payload)));
    notificationStore.addNotification('success', `已发送到 ${connectedSessions.length} 个会话`);
  } else {
    const sessionId = sessionStore.activeSessionId;
    if (!sessionId) {
      notificationStore.addNotification('warning', '没有活动会话，无法执行命令');
      return;
    }

    await sshApi.write(sessionId, payload);
    try {
      await historyStore.add(finalCommand, sessionId, sessionStore.activeSession?.connectionId);
      window.dispatchEvent(new Event('nexus:command-history-updated'));
    } catch {
      // ignore
    }
  }

  try {
    await quickCommandsStore.use(command.id);
    await quickCommandsStore.fetchAll();
  } catch {
    // ignore
  }
}

async function executeVariableCommand() {
  if (!pendingCommand.value) {
    return;
  }

  let finalCommand = pendingCommand.value.command;
  for (const key of variableOrder.value) {
    const pattern = new RegExp(`\\$\\{${escapeRegExp(key)}\\}`, 'g');
    finalCommand = finalCommand.replace(pattern, variableValues[key] ?? '');
  }

  const target = pendingCommand.value;
  closeVariableDialog();
  await runCommand(target, finalCommand);
}

async function prepareExecuteCommand(command: QuickCommand) {
  if (openVariableDialog(command)) {
    return;
  }

  await runCommand(command, command.command);
}

function closeContextMenu() {
  contextMenuVisible.value = false;
  contextMenuTarget.value = null;
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

function showContextMenu(event: MouseEvent, command: QuickCommand) {
  contextMenuTarget.value = command;
  contextMenuPosition.x = event.clientX;
  contextMenuPosition.y = event.clientY;
  contextMenuVisible.value = true;
  void nextTick(adjustContextMenuPosition);
}

async function sendContextCommandToAllSessions() {
  if (!contextMenuTarget.value) {
    return;
  }

  const command = contextMenuTarget.value;
  closeContextMenu();

  if (openVariableDialog(command)) {
    return;
  }

  await runCommand(command, command.command, true);
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

async function loadCommands() {
  await quickCommandsStore.fetchAll();
}

onMounted(async () => {
  await loadCommands();
  unregisterFocusAction = focusSwitcherStore.registerFocusAction('quickCommandsSearch', focusSearchInput);
  document.addEventListener('mousedown', handleDocumentPointerDown);
});

onUnmounted(() => {
  unregisterFocusAction?.();
  unregisterFocusAction = null;
  document.removeEventListener('mousedown', handleDocumentPointerDown);
});
</script>

<style scoped>
.quick-commands-panel {
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

.ctrl-btn {
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

.ctrl-btn:hover {
  background: rgba(137, 180, 250, 0.1);
  color: var(--text, #cdd6f4);
}

.ctrl-btn.active {
  background: rgba(137, 180, 250, 0.2);
  color: var(--blue, #89b4fa);
  border-color: rgba(137, 180, 250, 0.4);
}

.ctrl-btn.add-btn {
  background: var(--blue, #89b4fa);
  color: var(--bg-base, #1e1e2e);
  border-color: var(--blue, #89b4fa);
}

.ctrl-btn.add-btn:hover {
  opacity: 0.9;
}

.commands-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 6px;
}

.command-group {
  margin-bottom: 4px;
}

.group-header {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text, #cdd6f4);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
}

.group-header:hover {
  background: var(--bg-surface0, #313244);
}

.group-chevron {
  width: 14px;
  font-size: 10px;
  color: var(--text-dim, #6c7086);
}

.group-name {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  text-align: left;
}

.group-count {
  min-width: 20px;
  padding: 1px 6px;
  border-radius: 999px;
  background: var(--bg-surface0, #313244);
  color: var(--text-dim, #6c7086);
  font-size: 10px;
}

.group-items {
  margin: 0;
  padding: 0 0 0 14px;
  list-style: none;
}

.command-item {
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

.command-item:hover {
  background: rgba(137, 180, 250, 0.1);
}

.command-main {
  min-width: 0;
  flex: 1;
  display: flex;
  flex-direction: column;
}

.command-name,
.command-text {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.command-name {
  font-size: 12px;
  color: var(--text, #cdd6f4);
}

.command-text {
  margin-top: 1px;
  font-size: 11px;
  color: var(--text-dim, #6c7086);
  font-family: 'Cascadia Mono', 'Consolas', monospace;
}

.command-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0;
  transition: opacity 0.15s;
}

.command-item:hover .command-actions,
.command-item:focus-within .command-actions {
  opacity: 1;
}

.item-btn {
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

.item-btn:hover {
  background: rgba(137, 180, 250, 0.18);
  color: var(--blue, #89b4fa);
}

.item-btn.danger:hover {
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

.quick-command-context-menu {
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

.variable-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 4000;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

.variable-dialog {
  width: min(460px, 90vw);
  border-radius: 10px;
  border: 1px solid var(--border, #45475a);
  background: var(--bg-surface0, #313244);
  box-shadow: 0 14px 38px rgba(0, 0, 0, 0.5);
  padding: 16px;
}

.variable-dialog h3 {
  margin: 0;
  color: var(--text, #cdd6f4);
  font-size: 15px;
}

.dialog-hint {
  margin: 6px 0 12px;
  color: var(--text-dim, #6c7086);
  font-size: 12px;
}

.variable-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 10px;
  color: var(--text-sub, #a6adc8);
  font-size: 12px;
}

.variable-field input {
  height: 32px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  padding: 0 10px;
  outline: none;
}

.variable-field input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 4px;
}

.dialog-btn {
  min-width: 72px;
  height: 32px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 12px;
}

.dialog-btn.secondary {
  border: 1px solid var(--border, #45475a);
  background: transparent;
  color: var(--text-sub, #a6adc8);
}

.dialog-btn.secondary:hover {
  background: var(--bg-surface1, #45475a);
}

.dialog-btn.primary {
  border: none;
  background: var(--blue, #89b4fa);
  color: var(--bg-base, #1e1e2e);
  font-weight: 600;
}

.dialog-btn.primary:hover {
  opacity: 0.9;
}

.compact-mode .group-header {
  padding: 4px 8px;
}

.compact-mode .command-item {
  padding: 4px 8px;
}

.compact-mode .command-name {
  font-size: 11px;
}
</style>

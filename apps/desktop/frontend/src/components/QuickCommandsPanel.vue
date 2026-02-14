<template>
  <div class="quick-commands-panel" :class="{ 'compact-mode': isCompact }">
    <div class="panel-controls">
      <div class="search-box">
        <input
          ref="searchInputRef"
          v-model="searchTerm"
          type="text"
          placeholder="搜索名称或指令..."
          class="search-input"
          data-focus-id="quickCommandsSearch"
        />
      </div>
      <button class="ctrl-btn" @click="toggleSort" :title="sortAlpha ? '按名称排序' : '按最近使用排序'">
        <i class="fas" :class="sortAlpha ? 'fa-sort-alpha-down' : 'fa-clock'"></i>
      </button>
      <button class="ctrl-btn" @click="isCompact = !isCompact" :class="{ active: isCompact }" :title="isCompact ? '普通模式' : '紧凑模式'">
        <i class="fas" :class="isCompact ? 'fa-compress-alt' : 'fa-expand-alt'"></i>
      </button>
      <button class="ctrl-btn add-btn" title="添加快捷指令">
        <i class="fas fa-plus"></i>
      </button>
    </div>
    <div class="commands-list">
      <div v-if="filtered.length === 0" class="empty-state">
        <i class="fas fa-bolt"></i>
        <span>{{ searchTerm ? '无匹配结果' : '没有快捷指令' }}</span>
      </div>
      <template v-for="group in groupedCommands" :key="group.name">
        <div class="group-header" @click="toggleGroup(group.name)">
          <i class="fas group-chevron" :class="expandedGroups[group.name] ? 'fa-chevron-down' : 'fa-chevron-right'"></i>
          <span class="group-name">{{ group.name }}</span>
          <span class="group-count">{{ group.commands.length }}</span>
        </div>
        <div v-show="expandedGroups[group.name]" class="group-items">
          <div
            v-for="cmd in group.commands"
            :key="cmd.id"
            class="command-item"
            @click="executeCommand(cmd)"
          >
            <div class="cmd-info">
              <span class="cmd-name">{{ cmd.name }}</span>
              <span v-if="!isCompact" class="cmd-text">{{ cmd.command }}</span>
            </div>
            <div class="item-actions">
              <button class="act-btn" @click.stop="copyCommand(cmd.command)" title="复制">
                <i class="fas fa-copy"></i>
              </button>
              <button class="act-btn" @click.stop title="编辑">
                <i class="fas fa-edit"></i>
              </button>
              <button class="act-btn danger" @click.stop="quickCommandsStore.remove(cmd.id)" title="删除">
                <i class="fas fa-times"></i>
              </button>
            </div>
          </div>
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useSessionStore } from '@/stores/session';
import { useQuickCommandsStore } from '@/stores/quickCommands';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { sshApi } from '@/lib/api';

const sessionStore = useSessionStore();
const quickCommandsStore = useQuickCommandsStore();
const focusSwitcherStore = useFocusSwitcherStore();

const { items: quickCommands } = storeToRefs(quickCommandsStore);
const searchInputRef = ref<HTMLInputElement>();
const searchTerm = ref('');
const expandedGroups = reactive<Record<string, boolean>>({});
const sortAlpha = ref(true);
const isCompact = ref(false);

let unregisterFocusAction: (() => void) | null = null;

interface GroupedCmds {
  name: string;
  commands: any[];
}

const filtered = computed(() => {
  const q = searchTerm.value.toLowerCase().trim();
  let items = quickCommands.value.slice();
  if (q) {
    items = items.filter(
      (item) => item.name?.toLowerCase().includes(q) || item.command?.toLowerCase().includes(q),
    );
  }
  if (sortAlpha.value) {
    items.sort((a, b) => (a.name || '').localeCompare(b.name || ''));
  }
  return items;
});

const groupedCommands = computed<GroupedCmds[]>(() => {
  const groups: Record<string, any[]> = {};
  for (const cmd of filtered.value) {
    const tag = cmd.tags && cmd.tags.length > 0 ? cmd.tags[0] : '未标记';
    if (!groups[tag]) {
      groups[tag] = [];
      if (!(tag in expandedGroups)) {
        expandedGroups[tag] = true;
      }
    }
    groups[tag].push(cmd);
  }
  return Object.entries(groups).map(([name, commands]) => ({ name, commands }));
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

onMounted(() => {
  unregisterFocusAction = focusSwitcherStore.registerFocusAction('quickCommandsSearch', focusSearchInput);
});

onUnmounted(() => {
  unregisterFocusAction?.();
  unregisterFocusAction = null;
});

function toggleGroup(name: string) {
  expandedGroups[name] = !expandedGroups[name];
}

function toggleSort() {
  sortAlpha.value = !sortAlpha.value;
}

function executeCommand(cmd: any) {
  const sid = sessionStore.activeSessionId;
  if (!sid) return;
  const data = btoa(unescape(encodeURIComponent(`${cmd.command}\n`)));
  sshApi.write(sid, data).catch(() => {});
}

async function copyCommand(cmd: string) {
  try {
    await navigator.clipboard.writeText(cmd);
  } catch {
    // ignore
  }
}
</script>

<style scoped>
.quick-commands-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base, #1e1e2e);
  overflow: hidden;
  color: var(--text, #cdd6f4);
}

.panel-controls {
  display: flex;
  gap: 4px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border, #313244);
  flex-shrink: 0;
}

.search-box {
  flex: 1;
  min-width: 0;
}
.search-input {
  width: 100%;
  padding: 5px 10px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
  font-size: 12px;
  outline: none;
  transition: border-color 0.2s, box-shadow 0.2s;
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
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  flex-shrink: 0;
  font-size: 12px;
  transition: all 0.15s;
}
.ctrl-btn:hover {
  background: var(--bg-surface1, #45475a);
  color: var(--text, #cdd6f4);
  border-color: var(--text-dim, #6c7086);
}
.ctrl-btn.active {
  background: rgba(137, 180, 250, 0.15);
  color: var(--blue, #89b4fa);
  border-color: rgba(137, 180, 250, 0.3);
}
.ctrl-btn.add-btn {
  background: var(--blue, #89b4fa);
  color: var(--bg-base, #1e1e2e);
  border-color: var(--blue, #89b4fa);
  font-weight: 700;
}
.ctrl-btn.add-btn:hover {
  opacity: 0.85;
}

.commands-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 6px;
}

.group-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 8px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  color: var(--text, #cdd6f4);
  border-radius: 4px;
  margin-bottom: 1px;
  transition: background 0.15s;
}
.group-header:hover {
  background: var(--bg-surface0, #313244);
}
.group-chevron {
  font-size: 9px;
  width: 14px;
  color: var(--text-dim, #6c7086);
  transition: transform 0.15s;
}
.group-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.group-count {
  font-size: 10px;
  color: var(--text-dim, #6c7086);
  background: var(--bg-surface0, #313244);
  padding: 1px 6px;
  border-radius: 10px;
  font-weight: 400;
}

.group-items {
  padding-left: 14px;
}

.command-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 5px 8px;
  margin-bottom: 1px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
  position: relative;
}
.command-item:hover {
  background: rgba(137, 180, 250, 0.1);
}
.command-item:hover .item-actions {
  display: flex;
}

.cmd-info {
  display: flex;
  flex-direction: column;
  overflow: hidden;
  flex: 1;
  min-width: 0;
  margin-right: 8px;
}
.cmd-name {
  font-size: 12px;
  font-weight: 500;
  color: var(--text, #cdd6f4);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.cmd-text {
  font-size: 11px;
  color: var(--text-dim, #6c7086);
  font-family: 'Fira Code', 'Cascadia Code', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  margin-top: 1px;
}

.item-actions {
  display: none;
  gap: 2px;
  flex-shrink: 0;
}
.act-btn {
  width: 22px;
  height: 22px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  font-size: 10px;
  transition: all 0.15s;
}
.act-btn:hover {
  background: rgba(137, 180, 250, 0.15);
  color: var(--blue, #89b4fa);
}
.act-btn.danger:hover {
  background: rgba(243, 139, 168, 0.15);
  color: var(--red, #f38ba8);
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 30px 20px;
  color: var(--text-dim, #6c7086);
  font-size: 12px;
  gap: 8px;
  height: 100%;
}
.empty-state i {
  font-size: 24px;
  opacity: 0.5;
}

.compact-mode .command-item {
  padding: 3px 8px;
}
.compact-mode .cmd-name {
  font-size: 11px;
}
.compact-mode .group-header {
  padding: 4px 8px;
  font-size: 11px;
}
</style>
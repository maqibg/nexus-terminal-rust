<template>
  <div class="command-history-panel">
    <div class="panel-controls">
      <div class="search-box">
        <input
          ref="searchInputRef"
          v-model="searchTerm"
          type="text"
          placeholder="搜索历史记录..."
          class="search-input"
          data-focus-id="commandHistorySearch"
        />
        <i class="fas fa-search search-icon"></i>
      </div>
      <button class="clear-btn" @click="confirmClearAll" title="清空历史">
        <i class="fas fa-trash-alt"></i>
      </button>
    </div>
    <div class="history-list">
      <div v-if="filtered.length === 0" class="empty-state">
        <i class="fas fa-history"></i>
        <span>{{ searchTerm ? '无匹配结果' : '没有历史记录' }}</span>
      </div>
      <div
        v-for="(entry, idx) in filtered"
        :key="entry.id ?? idx"
        class="history-item"
        @click="executeCommand(entry.command)"
        :title="entry.command"
      >
        <span class="command-text">{{ entry.command }}</span>
        <div class="item-actions">
          <button class="act-btn" @click.stop="copyCommand(entry.command)" title="复制">
            <i class="fas fa-copy"></i>
          </button>
          <button class="act-btn danger" @click.stop="deleteEntry(entry.id)" title="删除">
            <i class="fas fa-times"></i>
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useCommandHistoryStore } from '@/stores/commandHistory';
import { useSessionStore } from '@/stores/session';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { sshApi } from '@/lib/api';

const sessionStore = useSessionStore();
const historyStore = useCommandHistoryStore();
const focusSwitcherStore = useFocusSwitcherStore();

const searchInputRef = ref<HTMLInputElement>();
const searchTerm = ref('');
let unregisterFocusAction: (() => void) | null = null;

const filtered = computed(() => {
  const q = searchTerm.value.toLowerCase().trim();
  const items = historyStore.items.slice(0, 50);
  if (!q) return items;
  return items.filter((item) => item.command.toLowerCase().includes(q));
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
  unregisterFocusAction = focusSwitcherStore.registerFocusAction('commandHistorySearch', focusSearchInput);
});

onUnmounted(() => {
  unregisterFocusAction?.();
  unregisterFocusAction = null;
});

function executeCommand(cmd: string) {
  const sid = sessionStore.activeSessionId;
  if (!sid) return;
  const data = btoa(unescape(encodeURIComponent(`${cmd}\n`)));
  sshApi.write(sid, data).catch(() => {});
}

async function copyCommand(cmd: string) {
  try {
    await navigator.clipboard.writeText(cmd);
  } catch {
    // ignore
  }
}

function deleteEntry(id: number) {
  historyStore.items = historyStore.items.filter((item) => item.id !== id);
}

function confirmClearAll() {
  if (confirm('确定要清空所有历史记录吗？')) {
    historyStore.clear();
  }
}
</script>

<style scoped>
.command-history-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base, #1e1e2e);
  overflow: hidden;
  color: var(--text, #cdd6f4);
}

.panel-controls {
  display: flex;
  gap: 6px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border, #313244);
  flex-shrink: 0;
}

.search-box {
  position: relative;
  flex: 1;
  min-width: 0;
}
.search-input {
  width: 100%;
  padding: 5px 30px 5px 10px;
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
.search-icon {
  position: absolute;
  right: 10px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-dim, #6c7086);
  font-size: 11px;
  pointer-events: none;
}

.clear-btn {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim, #6c7086);
  cursor: pointer;
  flex-shrink: 0;
  transition: all 0.15s;
  font-size: 12px;
}
.clear-btn:hover {
  background: rgba(243, 139, 168, 0.1);
  color: var(--red, #f38ba8);
  border-color: rgba(243, 139, 168, 0.3);
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px 6px;
}

.history-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 10px;
  margin-bottom: 1px;
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.15s;
}
.history-item:hover {
  background: rgba(137, 180, 250, 0.1);
}
.history-item:hover .item-actions {
  display: flex;
}

.command-text {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: 'Fira Code', 'Cascadia Code', 'Consolas', monospace;
  font-size: 12px;
  color: var(--text, #cdd6f4);
  margin-right: 8px;
}

.item-actions {
  display: none;
  gap: 2px;
  flex-shrink: 0;
}
.act-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  font-size: 11px;
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
</style>
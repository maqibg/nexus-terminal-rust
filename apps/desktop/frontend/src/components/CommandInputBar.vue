<template>
  <div class="command-input-bar">
    <button class="bar-btn" :disabled="!isSshActiveSession" @click="clearTerminal" title="清空终端">
      <i class="fas fa-eraser"></i>
    </button>
    <button class="bar-btn" @click="openFocusConfigurator" title="配置焦点切换器">
      <i class="fas fa-keyboard"></i>
    </button>

    <input
      ref="inputEl"
      v-model="command"
      class="command-input"
      data-focus-id="commandInput"
      placeholder="在此输入命令后按 Enter 发送到终端..."
      @keydown.enter="send"
      @keydown.up.prevent="historyUp"
      @keydown.down.prevent="historyDown"
      :disabled="!isSshActiveSession"
    />

    <button
      v-if="!isSearching"
      class="bar-btn"
      title="终端搜索"
      @click="openSearch"
    >
      <i class="fas fa-search"></i>
    </button>
    <button
      v-if="showPopupFileManager"
      class="bar-btn"
      title="弹窗文件管理器"
      @click="openFileManagerPopup"
    >
      <i class="fas fa-folder-open"></i>
    </button>

    <button
      v-if="showPopupFileEditor"
      class="bar-btn"
      title="弹窗文件编辑器"
      @click="openFileEditorPopup"
    >
      <i class="fas fa-pen-to-square"></i>
    </button>
    <div v-if="isSearching" class="search-controls">
      <input
        ref="searchInputEl"
        v-model="searchTerm"
        class="search-input"
        data-focus-id="terminalSearch"
        placeholder="在终端中搜索..."
        @input="emitSearchUpdate"
        @keydown.enter.prevent="findNext"
        @keydown.shift.enter.prevent="findPrevious"
        @keydown.up.prevent="findPrevious"
        @keydown.down.prevent="findNext"
        @keydown.esc.prevent="closeSearch"
      />
      <button class="search-btn" title="清空搜索" @click="clearSearchTerm">
        <i class="fas fa-times"></i>
      </button>
      <button class="search-btn" title="上一个" @click="findPrevious">
        <i class="fas fa-arrow-up"></i>
      </button>
      <button class="search-btn" title="下一个" @click="findNext">
        <i class="fas fa-arrow-down"></i>
      </button>
      <button class="search-btn" title="关闭搜索" @click="closeSearch">
        <i class="fas fa-chevron-right"></i>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { historyApi, sshApi } from '@/lib/api';
import { useSessionStore } from '@/stores/session';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { useSettingsStore } from '@/stores/settings';

const sessionStore = useSessionStore();
const focusSwitcherStore = useFocusSwitcherStore();
const settingsStore = useSettingsStore();

const inputEl = ref<HTMLInputElement>();
const searchInputEl = ref<HTMLInputElement>();

const command = ref('');
const searchTerm = ref('');
const isSearching = ref(false);
const commandInputSyncTarget = computed<'none' | 'quickCommands' | 'commandHistory'>(() => {
  const target = settingsStore.get('commandInputSyncTarget', 'none');
  if (target === 'quickCommands' || target === 'commandHistory') {
    return target;
  }
  return 'none';
});
const showPopupFileManager = computed(() => settingsStore.getBoolean('showPopupFileManager', false));
const showPopupFileEditor = computed(() => settingsStore.getBoolean('showPopupFileEditor', false));
const isSshActiveSession = computed(() => sessionStore.activeSession?.protocol === 'SSH');

const history = ref<string[]>([]);
const historyIdx = ref(-1);

let unregisterCommandInput: (() => void) | null = null;
let unregisterTerminalSearch: (() => void) | null = null;

interface TerminalSearchEventDetail {
  sessionId: string;
  term?: string;
}

function isElementVisibleAndFocusable(el: HTMLInputElement | undefined): el is HTMLInputElement {
  if (!el || !el.isConnected || el.disabled) {
    return false;
  }

  const style = window.getComputedStyle(el);
  if (style.display === 'none' || style.visibility === 'hidden') {
    return false;
  }

  const rect = el.getBoundingClientRect();
  return rect.width > 0 && rect.height > 0;
}

function dispatchTerminalSearchEvent(eventName: string, detail: TerminalSearchEventDetail): void {
  if (typeof window === 'undefined') {
    return;
  }
  window.dispatchEvent(new CustomEvent<TerminalSearchEventDetail>(eventName, { detail }));
}

function getActiveSessionId(): string | null {
  return sessionStore.activeSessionId ?? null;
}

function focusCommandInput(): boolean | undefined {
  if (!isElementVisibleAndFocusable(inputEl.value)) {
    return undefined;
  }

  inputEl.value.focus();
  inputEl.value.select();
  return document.activeElement === inputEl.value;
}

async function focusSearchInput(): Promise<boolean | undefined> {
  if (!isSearching.value) {
    isSearching.value = true;
    await nextTick();
  }

  if (!isElementVisibleAndFocusable(searchInputEl.value)) {
    return undefined;
  }

  searchInputEl.value.focus();
  searchInputEl.value.select();
  return document.activeElement === searchInputEl.value;
}

function emitSearchUpdate(): void {
  const sid = getActiveSessionId();
  if (!sid) {
    return;
  }

  dispatchTerminalSearchEvent('nexus:terminal-search:update', {
    sessionId: sid,
    term: searchTerm.value,
  });
}

function emitSearchClear(sessionId: string | null = getActiveSessionId()): void {
  if (!sessionId) {
    return;
  }

  dispatchTerminalSearchEvent('nexus:terminal-search:clear', {
    sessionId,
  });
}

function openSearch(): void {
  isSearching.value = true;
  void nextTick(() => {
    searchInputEl.value?.focus();
    searchInputEl.value?.select();
    emitSearchUpdate();
  });
}

function closeSearch(): void {
  const sid = getActiveSessionId();
  searchTerm.value = '';
  isSearching.value = false;
  emitSearchClear(sid);
}

function clearSearchTerm(): void {
  searchTerm.value = '';
  emitSearchUpdate();
  void nextTick(() => {
    searchInputEl.value?.focus();
  });
}

function findNext(): void {
  const sid = getActiveSessionId();
  if (!sid || !searchTerm.value.trim()) {
    void focusSearchInput();
    return;
  }

  dispatchTerminalSearchEvent('nexus:terminal-search:next', {
    sessionId: sid,
    term: searchTerm.value,
  });
}

function findPrevious(): void {
  const sid = getActiveSessionId();
  if (!sid || !searchTerm.value.trim()) {
    void focusSearchInput();
    return;
  }

  dispatchTerminalSearchEvent('nexus:terminal-search:previous', {
    sessionId: sid,
    term: searchTerm.value,
  });
}

onMounted(async () => {
  await settingsStore.loadAll().catch(() => undefined);

  try {
    const items = await historyApi.list(100);
    history.value = items.map((h) => h.command);
  } catch {
    // ignore
  }

  await nextTick();
  unregisterCommandInput = focusSwitcherStore.registerFocusAction('commandInput', focusCommandInput);
  unregisterTerminalSearch = focusSwitcherStore.registerFocusAction('terminalSearch', focusSearchInput);
  emitCommandInputSyncTarget(command.value);
});

onUnmounted(() => {
  unregisterCommandInput?.();
  unregisterTerminalSearch?.();
  unregisterCommandInput = null;
  unregisterTerminalSearch = null;
});

watch(
  () => sessionStore.activeSessionId,
  (newSid, oldSid) => {
    if (oldSid) {
      emitSearchClear(oldSid);
    }
    if (newSid && isSearching.value && searchTerm.value.trim()) {
      emitSearchUpdate();
    }
  },
);

function emitCommandInputSyncTarget(term: string): void {
  if (typeof window === 'undefined') {
    return;
  }

  const detail = { term };
  if (commandInputSyncTarget.value === 'quickCommands') {
    window.dispatchEvent(new CustomEvent('nexus:quick-commands:set-search', { detail }));
    return;
  }

  if (commandInputSyncTarget.value === 'commandHistory') {
    window.dispatchEvent(new CustomEvent('nexus:command-history:set-search', { detail }));
  }
}

watch(command, (nextCommand) => {
  emitCommandInputSyncTarget(nextCommand);
});

watch(commandInputSyncTarget, () => {
  emitCommandInputSyncTarget(command.value);
});

function openFocusConfigurator() {
  focusSwitcherStore.toggleConfigurator(true);
}

function openFileManagerPopup() {
  if (typeof window === 'undefined') {
    return;
  }
  window.dispatchEvent(new Event('nexus:workspace:file-manager-popup:open'));
}

function openFileEditorPopup() {
  if (typeof window === 'undefined') {
    return;
  }
  window.dispatchEvent(new Event('nexus:workspace:file-editor-popup:open'));
}

async function send() {
  const cmd = command.value.trim();
  if (!cmd) return;
  const sid = sessionStore.activeSessionId;
  if (!sid || !isSshActiveSession.value) return;

  const data = btoa(unescape(encodeURIComponent(`${cmd}\n`)));
  await sshApi.write(sid, data);

  history.value.unshift(cmd);
  historyIdx.value = -1;
  command.value = '';

  try {
    await historyApi.add(cmd, sid, sessionStore.activeSession?.connectionId);
    window.dispatchEvent(new Event('nexus:command-history-updated'));
  } catch {
    // ignore
  }
}

function clearTerminal() {
  const sid = sessionStore.activeSessionId;
  if (!sid || !isSshActiveSession.value) return;
  const data = btoa(unescape(encodeURIComponent('clear\n')));
  sshApi.write(sid, data).catch(() => {});
}

function historyUp() {
  if (historyIdx.value < history.value.length - 1) {
    historyIdx.value += 1;
    command.value = history.value[historyIdx.value];
  }
}

function historyDown() {
  if (historyIdx.value > 0) {
    historyIdx.value -= 1;
    command.value = history.value[historyIdx.value];
  } else {
    historyIdx.value = -1;
    command.value = '';
  }
}
</script>

<style scoped>
.command-input-bar {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: var(--bg-base, #1e1e2e);
  border-top: 1px solid var(--border, #313244);
  height: 100%;
  min-height: 36px;
  box-sizing: border-box;
}

.bar-btn {
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
  font-size: 13px;
  transition: all 0.15s;
}

.bar-btn:hover {
  background: var(--bg-surface1, #45475a);
  color: var(--text, #cdd6f4);
  border-color: var(--text-dim, #6c7086);
}

.command-input {
  flex: 1;
  padding: 6px 14px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
  font-family: 'Fira Code', 'Cascadia Code', 'Consolas', monospace;
  font-size: 13px;
  outline: none;
  min-width: 0;
  transition: border-color 0.2s, box-shadow 0.2s;
  box-sizing: border-box;
}

.command-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.command-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.search-controls {
  display: flex;
  align-items: center;
  gap: 4px;
  min-width: 0;
}

.search-input {
  width: 160px;
  height: 30px;
  padding: 0 9px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
  font-size: 12px;
  outline: none;
}

.search-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.search-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.search-btn {
  width: 26px;
  height: 26px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  font-size: 11px;
}

.search-btn:hover {
  background: var(--bg-surface1, #45475a);
  color: var(--text);
}

@media (max-width: 900px) {
  .search-input {
    width: 130px;
  }
}
</style>






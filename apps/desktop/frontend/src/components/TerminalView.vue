<template>
  <div class="terminal-wrapper">
    <div v-if="!sessionId" class="terminal-placeholder">
      <span class="placeholder-text">无活动会话</span>
    </div>
    <div v-else-if="isVncSession" class="terminal-container terminal-container-vnc">
      <VncSessionView :session-id="sessionId" />
    </div>
    <div v-else class="terminal-container" :style="terminalContainerStyle">
      <div ref="termRef" class="terminal-host"></div>
    </div>
    <CommandAutocomplete
      ref="autocompleteRef"
      :visible="showAutocomplete"
      :input="autocompleteInput"
      :cursor-position="autocompleteCursorPosition"
      :session-id="sessionId ?? ''"
      @select="handleAutocompleteSelect"
      @close="showAutocomplete = false"
    />

    <Teleport to="body">
      <div
        v-if="terminalContextMenu.visible"
        class="terminal-context-menu"
        :style="terminalContextMenuStyle"
        @click.stop
        @contextmenu.prevent
      >
        <button class="terminal-menu-item" :disabled="!terminalContextMenu.hasSelection" @click="handleContextMenuAction('copy')">
          复制
        </button>
        <button class="terminal-menu-item" @click="handleContextMenuAction('paste')">粘贴</button>
        <div class="terminal-menu-separator"></div>
        <button class="terminal-menu-item" @click="handleContextMenuAction('select-all')">全选</button>
        <button class="terminal-menu-item" @click="handleContextMenuAction('clear')">清屏</button>
        <div class="terminal-menu-separator"></div>
        <div class="terminal-menu-group-title">{{ aiMenuTitle }}</div>
        <button class="terminal-menu-item" :disabled="!terminalContextMenu.aiEnabled" @click="handleContextMenuAction('ai-write')">
          AI 撰写代码
        </button>
        <button
          class="terminal-menu-item"
          :disabled="!terminalContextMenu.aiEnabled || !terminalContextMenu.hasSelection"
          @click="handleContextMenuAction('ai-explain')"
        >
          {{ aiExplainLabel }}
        </button>
        <button
          class="terminal-menu-item"
          :disabled="!terminalContextMenu.aiEnabled || !terminalContextMenu.hasSelection"
          @click="handleContextMenuAction('ai-optimize')"
        >
          {{ aiOptimizeLabel }}
        </button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { FitAddon } from '@xterm/addon-fit';
import { SearchAddon } from '@xterm/addon-search';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { storeToRefs } from 'pinia';
import { onSshOutput, sshApi } from '@/lib/api';
import type { SshOutputChunk } from '@/lib/api';
import VncSessionView from '@/components/VncSessionView.vue';
import CommandAutocomplete from '@/components/CommandAutocomplete.vue';
import { useSessionStore } from '@/stores/session';
import { useAIStore } from '@/stores/ai';
import { useAppearanceStore } from '@/stores/appearance';
import { useSettingsStore } from '@/stores/settings';
import { useUiNotificationsStore } from '@/stores/uiNotifications';

const sessionStore = useSessionStore();
const aiStore = useAIStore();
const appearanceStore = useAppearanceStore();
const settingsStore = useSettingsStore();
const notifications = useUiNotificationsStore();
const { activeSessionId: sessionId, activeSession } = storeToRefs(sessionStore);
const { appearance, effectiveTerminalTheme } = storeToRefs(appearanceStore);
const { settings: runtimeSettings } = storeToRefs(settingsStore);
const isVncSession = computed(() => activeSession.value?.protocol === 'VNC');

const termRef = ref<HTMLElement>();
interface CommandAutocompleteExpose {
  selectNext: () => void;
  selectPrevious: () => void;
  selectCurrent: () => void;
  hasSuggestions: () => boolean;
  hasActiveSelection: () => boolean;
  resetSelection: () => void;
  forceReset: () => void;
}
const autocompleteRef = ref<CommandAutocompleteExpose | null>(null);
const showAutocomplete = ref(false);
const autocompleteInput = ref('');
const autocompleteCursorPosition = ref({ x: 0, y: 0 });
let terminalInputBuffer = '';
let lastOutputSeq = -1;

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let searchAddon: SearchAddon | null = null;
let unlisten: (() => void) | null = null;
let resizeObserver: ResizeObserver | null = null;
let currentSearchTerm = '';
let selectionDisposable: { dispose: () => void } | null = null;
type TerminalContextMenuAction = 'copy' | 'paste' | 'select-all' | 'clear' | 'ai-write' | 'ai-explain' | 'ai-optimize';
const terminalContextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  hasSelection: false,
  aiEnabled: false,
});

const FALLBACK_PROMPT_EXPLAIN =
  '请作为一名资深开发人员，详细分析并解释以下代码片段的主要功能和目的。\n\n```{language}\n{content}\n```';
const FALLBACK_PROMPT_OPTIMIZE =
  'Optimize this code:\n\n```{language}\n{content}\n```\n\nReturn only the optimized code without explanations or markdown code blocks.';
const FALLBACK_PROMPT_WRITE =
  'Write code based on this description: {content}\n\nLanguage: {language}\n\nReturn only the code without explanations or markdown code blocks.';

interface TerminalSearchEventDetail {
  sessionId?: string;
  term?: string;
}

interface WorkspaceAiActionDetail {
  prompt?: string;
  autoSend?: boolean;
}

const searchOptions = {
  caseSensitive: false,
  regex: false,
  wholeWord: false,
} as const;

function encodeUtf8Base64(value: string): string {
  return btoa(unescape(encodeURIComponent(value)));
}

function decodeBase64ToBytes(value: string): Uint8Array {
  return Uint8Array.from(atob(value), (char) => char.charCodeAt(0));
}

function isCommandAutocompleteEnabled(): boolean {
  return settingsStore.getBoolean('commandAutocomplete', true);
}

function resetAutocompleteState(forceResetComponent = true): void {
  showAutocomplete.value = false;
  autocompleteInput.value = '';
  if (forceResetComponent) {
    autocompleteRef.value?.forceReset?.();
  }
}

function syncAutocompleteCursorPosition(): void {
  if (!term || !termRef.value) {
    return;
  }
  const rect = termRef.value.getBoundingClientRect();
  const cols = Math.max(1, term.cols);
  const rows = Math.max(1, term.rows);
  const cellWidth = rect.width / cols;
  const cellHeight = rect.height / rows;
  const activeBuffer = term.buffer.active;
  autocompleteCursorPosition.value = {
    x: rect.left + Math.max(0, activeBuffer.cursorX) * cellWidth,
    y: rect.top + Math.max(0, activeBuffer.cursorY) * cellHeight,
  };
}

function refreshAutocompleteByBuffer(): void {
  if (!sessionId.value || activeSession.value?.protocol !== 'SSH') {
    resetAutocompleteState();
    return;
  }
  if (!isCommandAutocompleteEnabled()) {
    resetAutocompleteState();
    return;
  }
  const input = terminalInputBuffer;
  const trimmed = input.trim();
  if (!trimmed) {
    resetAutocompleteState();
    return;
  }
  const shouldShow = trimmed.startsWith('/') ? trimmed.length >= 1 : trimmed.length >= 2;
  if (!shouldShow) {
    resetAutocompleteState();
    return;
  }
  autocompleteInput.value = input;
  showAutocomplete.value = true;
  syncAutocompleteCursorPosition();
}

function applyTerminalInputDelta(data: string): void {
  if (!data) {
    return;
  }
  if (data === '\r' || data === '\n' || data.includes('\r') || data.includes('\n')) {
    terminalInputBuffer = '';
    resetAutocompleteState();
    return;
  }
  if (data === '\u0015' || data === '\u0003') {
    terminalInputBuffer = '';
    resetAutocompleteState();
    return;
  }
  if (/^\x1b\[[0-9;]*[A-Za-z~]$/.test(data) || /^\x1bO[A-Za-z]$/.test(data)) {
    return;
  }
  for (const ch of data) {
    const code = ch.charCodeAt(0);
    if (ch === '\u007f' || ch === '\b') {
      terminalInputBuffer = terminalInputBuffer.slice(0, -1);
      continue;
    }
    if (ch === '\t') {
      continue;
    }
    if (code < 32 || code === 127) {
      continue;
    }
    terminalInputBuffer += ch;
  }
  refreshAutocompleteByBuffer();
}

async function handleAutocompleteSelect(text: string): Promise<void> {
  if (!sessionId.value || !text) {
    return;
  }
  const currentInput = autocompleteInput.value;
  const words = currentInput.split(/\s+/);
  const lastWord = words[words.length - 1] ?? '';
  const deleteCount = currentInput.startsWith('/') ? currentInput.length : lastWord.length;
  const payload = '\u007f'.repeat(deleteCount) + text;
  await sshApi.write(sessionId.value, encodeUtf8Base64(payload)).catch(() => undefined);

  terminalInputBuffer = currentInput.slice(0, currentInput.length - deleteCount) + text;
  if (text.endsWith(' ')) {
    autocompleteInput.value = terminalInputBuffer;
    showAutocomplete.value = true;
    await nextTick();
    syncAutocompleteCursorPosition();
  } else {
    resetAutocompleteState();
  }
  term?.focus();
}

function shouldConsumeChunk(chunk?: SshOutputChunk): boolean {
  if (!chunk || chunk.seq < 0) {
    return true;
  }
  if (chunk.seq <= lastOutputSeq) {
    return false;
  }
  lastOutputSeq = chunk.seq;
  return true;
}

function handleAutocompleteKeydown(event: KeyboardEvent): boolean {
  if (!showAutocomplete.value) {
    return true;
  }
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return true;
  }
  const hasSuggestions = autocompleteRef.value?.hasSuggestions?.() ?? false;
  const hasActiveSelection = autocompleteRef.value?.hasActiveSelection?.() ?? false;
  switch (event.key) {
    case 'Enter':
      if (hasSuggestions && hasActiveSelection) {
        autocompleteRef.value?.selectCurrent?.();
        return false;
      }
      resetAutocompleteState();
      return true;
    case 'Tab':
      if (!hasSuggestions) {
        return true;
      }
      autocompleteRef.value?.selectCurrent?.();
      return false;
    case 'ArrowDown':
      if (!hasSuggestions) {
        return true;
      }
      autocompleteRef.value?.selectNext?.();
      return false;
    case 'ArrowUp':
      if (!hasSuggestions) {
        return true;
      }
      autocompleteRef.value?.selectPrevious?.();
      return false;
    case 'Escape':
      resetAutocompleteState();
      return false;
    default:
      return true;
  }
}

const terminalContextMenuStyle = computed<Record<string, string>>(() => {
  const menuWidth = 244;
  const menuHeight = 270;
  const viewportWidth = typeof window === 'undefined' ? menuWidth : window.innerWidth;
  const viewportHeight = typeof window === 'undefined' ? menuHeight : window.innerHeight;
  const left = Math.max(8, Math.min(terminalContextMenu.value.x, viewportWidth - menuWidth - 8));
  const top = Math.max(8, Math.min(terminalContextMenu.value.y, viewportHeight - menuHeight - 8));
  return {
    left: `${left}px`,
    top: `${top}px`,
  };
});

const aiMenuTitle = computed(() => (terminalContextMenu.value.aiEnabled ? 'AI 助手' : 'AI 助手 (未配置)'));
const aiExplainLabel = computed(() =>
  terminalContextMenu.value.hasSelection ? 'AI 解释代码' : 'AI 解释代码 (需要选中代码)',
);
const aiOptimizeLabel = computed(() =>
  terminalContextMenu.value.hasSelection ? 'AI 优化代码' : 'AI 优化代码 (需要选中代码)',
);

function getAppearanceValue(keys: string[], fallback: string): string {
  for (const key of keys) {
    const value = appearanceStore.get(key, '');
    if (value !== '') {
      return value;
    }
  }
  return fallback;
}

function parseBoolean(value: string): boolean {
  return String(value).toLowerCase() === 'true';
}

function parseNumber(value: string, fallback: number): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? parsed : fallback;
}

function getSettingBoolean(key: string, fallback: boolean): boolean {
  return settingsStore.getBoolean(key, fallback);
}

function getSettingInteger(key: string, fallback: number, min?: number): number {
  return settingsStore.getInteger(key, fallback, min);
}

function getTerminalFontFamily(): string {
  return getAppearanceValue(
    ['terminal_font_family', 'terminalFontFamily'],
    `Consolas, 'Courier New', monospace, 'Microsoft YaHei', '微软雅黑'`,
  );
}

function getTerminalScrollbackLimit(): number {
  return getSettingInteger('terminalScrollbackLimit', 5000, 0);
}

function isTerminalRightClickPasteEnabled(): boolean {
  return getSettingBoolean('terminalEnableRightClickPaste', true);
}

function isTerminalAutoCopyOnSelectEnabled(): boolean {
  return getSettingBoolean('autoCopyOnSelect', false);
}

function getTerminalFontSize(): number {
  const value = getAppearanceValue(['terminal_font_size', 'terminalFontSize'], '14');
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 14;
}

function normalizeImageUrl(pathValue: string): string {
  const trimmed = pathValue.trim();
  if (!trimmed) {
    return '';
  }

  if (/^(https?:|data:|file:)/i.test(trimmed)) {
    return trimmed;
  }

  const normalized = trimmed.replace(/\\/g, '/');
  if (/^[A-Za-z]:\//.test(normalized)) {
    return `file:///${normalized}`;
  }
  return normalized;
}

const terminalContainerStyle = computed<Record<string, string>>(() => {
  const enabled = parseBoolean(getAppearanceValue(['terminalBackgroundEnabled', 'terminal_background_enabled'], 'true'));
  const imagePath = getAppearanceValue(['terminalBackgroundImage', 'terminal_background_image'], '');
  if (!enabled || !imagePath) {
    return {} as Record<string, string>;
  }

  const imageUrl = normalizeImageUrl(imagePath);
  if (!imageUrl) {
    return {} as Record<string, string>;
  }

  const overlay = Math.min(
    1,
    Math.max(
      0,
      parseNumber(getAppearanceValue(['terminalBackgroundOverlayOpacity', 'terminal_background_overlay_opacity'], '0.50'), 0.5),
    ),
  );

  return {
    backgroundImage: `linear-gradient(rgba(0, 0, 0, ${overlay}), rgba(0, 0, 0, ${overlay})), url("${imageUrl}")`,
    backgroundSize: 'cover',
    backgroundPosition: 'center',
    backgroundRepeat: 'no-repeat',
  };
});

function buildTerminalShadow(): string {
  const shadowList: string[] = [];

  const strokeEnabled = parseBoolean(getAppearanceValue(['terminalTextStrokeEnabled', 'terminal_text_stroke_enabled'], 'false'));
  const strokeWidth = Math.max(0, parseNumber(getAppearanceValue(['terminalTextStrokeWidth', 'terminal_text_stroke_width'], '1'), 1));
  const strokeColor = getAppearanceValue(['terminalTextStrokeColor', 'terminal_text_stroke_color'], '#000000');

  if (strokeEnabled && strokeWidth > 0) {
    shadowList.push(
      `${strokeWidth}px 0 ${strokeColor}`,
      `-${strokeWidth}px 0 ${strokeColor}`,
      `0 ${strokeWidth}px ${strokeColor}`,
      `0 -${strokeWidth}px ${strokeColor}`,
    );
  }

  const shadowEnabled = parseBoolean(getAppearanceValue(['terminalTextShadowEnabled', 'terminal_text_shadow_enabled'], 'false'));
  if (shadowEnabled) {
    const offsetX = parseNumber(getAppearanceValue(['terminalTextShadowOffsetX', 'terminal_text_shadow_offset_x'], '0'), 0);
    const offsetY = parseNumber(getAppearanceValue(['terminalTextShadowOffsetY', 'terminal_text_shadow_offset_y'], '0'), 0);
    const blur = Math.max(0, parseNumber(getAppearanceValue(['terminalTextShadowBlur', 'terminal_text_shadow_blur'], '0'), 0));
    const color = getAppearanceValue(['terminalTextShadowColor', 'terminal_text_shadow_color'], 'rgba(0,0,0,0.5)');
    shadowList.push(`${offsetX}px ${offsetY}px ${blur}px ${color}`);
  }

  return shadowList.length > 0 ? shadowList.join(', ') : 'none';
}

function applyTerminalAppearance() {
  if (!term) {
    return;
  }

  term.options.fontFamily = getTerminalFontFamily();
  term.options.fontSize = getTerminalFontSize();

  const transparentBackground = parseBoolean(
    getAppearanceValue(['terminalBackgroundEnabled', 'terminal_background_enabled'], 'true'),
  );
  const activeTheme = effectiveTerminalTheme.value;
  term.options.theme = {
    ...activeTheme,
    background: transparentBackground ? '#00000000' : (activeTheme.background ?? '#1e1e2e'),
  };

  const hostElement = termRef.value;
  if (hostElement) {
    hostElement.style.setProperty('--terminal-text-shadow', buildTerminalShadow());
  }

  fitAddon?.fit();
}

function clearSearchDecorations(): void {
  const addon = searchAddon as (SearchAddon & { clearDecorations?: () => void }) | null;
  addon?.clearDecorations?.();
}

function matchesCurrentSession(detail: TerminalSearchEventDetail | undefined): boolean {
  return !!detail?.sessionId && detail.sessionId === sessionId.value;
}

function handleSearchUpdate(event: Event): void {
  const detail = (event as CustomEvent<TerminalSearchEventDetail>).detail;
  if (!matchesCurrentSession(detail) || !searchAddon) {
    return;
  }

  currentSearchTerm = (detail.term ?? '').trim();
  clearSearchDecorations();

  if (!currentSearchTerm) {
    return;
  }

  searchAddon.findNext(currentSearchTerm, { ...searchOptions, incremental: true });
}

function handleSearchNext(event: Event): void {
  const detail = (event as CustomEvent<TerminalSearchEventDetail>).detail;
  if (!matchesCurrentSession(detail) || !searchAddon) {
    return;
  }

  const termFromEvent = (detail.term ?? '').trim();
  if (termFromEvent) {
    currentSearchTerm = termFromEvent;
  }

  if (!currentSearchTerm) {
    return;
  }

  searchAddon.findNext(currentSearchTerm, searchOptions);
}

function handleSearchPrevious(event: Event): void {
  const detail = (event as CustomEvent<TerminalSearchEventDetail>).detail;
  if (!matchesCurrentSession(detail) || !searchAddon) {
    return;
  }

  const termFromEvent = (detail.term ?? '').trim();
  if (termFromEvent) {
    currentSearchTerm = termFromEvent;
  }

  if (!currentSearchTerm) {
    return;
  }

  searchAddon.findPrevious(currentSearchTerm, searchOptions);
}

function handleSearchClear(event: Event): void {
  const detail = (event as CustomEvent<TerminalSearchEventDetail>).detail;
  if (!matchesCurrentSession(detail)) {
    return;
  }

  currentSearchTerm = '';
  clearSearchDecorations();
}

function closeTerminalContextMenu(): void {
  terminalContextMenu.value.visible = false;
}

async function ensureAiConfigLoaded(): Promise<void> {
  if (aiStore.channels.length === 0 && aiStore.models.length === 0) {
    await aiStore.loadAll().catch(() => undefined);
    return;
  }
  if (!aiStore.config.prompts) {
    await aiStore.loadConfig().catch(() => undefined);
  }
}

function getSelectedText(): string {
  return term?.getSelection() ?? '';
}

function hasSelectedText(): boolean {
  return getSelectedText().trim().length > 0;
}

function getPromptTemplate(action: 'write' | 'explain' | 'optimize'): string {
  const prompts = aiStore.config.prompts;
  if (action === 'write') {
    return prompts?.write || FALLBACK_PROMPT_WRITE;
  }
  if (action === 'explain') {
    return prompts?.explain || FALLBACK_PROMPT_EXPLAIN;
  }
  return prompts?.optimize || FALLBACK_PROMPT_OPTIMIZE;
}

function buildAiPrompt(action: 'write' | 'explain' | 'optimize', content: string): string {
  return getPromptTemplate(action).replace('{content}', content).replace(/{language}/g, 'unknown');
}

function openWorkspaceAiAssistant(prompt: string): void {
  const detail: WorkspaceAiActionDetail = { prompt, autoSend: true };
  window.dispatchEvent(new CustomEvent<WorkspaceAiActionDetail>('nexus:workspace:open-ai-assistant', { detail }));
}

async function pasteClipboardToTerminal(): Promise<void> {
  if (!sessionId.value) {
    return;
  }
  const clipboardText = await navigator.clipboard.readText();
  if (!clipboardText) {
    return;
  }
  await sshApi.write(sessionId.value, btoa(clipboardText));
}

async function openTerminalContextMenu(event: MouseEvent): Promise<void> {
  await ensureAiConfigLoaded();
  terminalContextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    hasSelection: hasSelectedText(),
    aiEnabled: Boolean(aiStore.hasDefaultModel),
  };
}

async function handleContextMenuAction(action: TerminalContextMenuAction): Promise<void> {
  closeTerminalContextMenu();
  if (!sessionId.value) {
    return;
  }

  const selection = getSelectedText();

  if (action === 'copy') {
    if (!selection.trim()) {
      return;
    }
    await navigator.clipboard.writeText(selection).catch(() => undefined);
    return;
  }

  if (action === 'paste') {
    await pasteClipboardToTerminal().catch(() => undefined);
    return;
  }

  if (action === 'select-all') {
    term?.selectAll();
    return;
  }

  if (action === 'clear') {
    term?.clear();
    return;
  }

  await ensureAiConfigLoaded();
  if (!aiStore.hasDefaultModel) {
    notifications.addNotification('warning', '请先在设置- AI 助手中配置默认模型');
    return;
  }

  if (action === 'ai-write') {
    if (!selection.trim()) {
      notifications.addNotification('warning', '请先选中需求描述后再使用 AI 撰写代码');
      return;
    }
    openWorkspaceAiAssistant(buildAiPrompt('write', selection));
    return;
  }

  if (!selection.trim()) {
    notifications.addNotification('warning', '请先选中代码后再执行该 AI 操作');
    return;
  }

  if (action === 'ai-explain') {
    openWorkspaceAiAssistant(buildAiPrompt('explain', selection));
    return;
  }

  openWorkspaceAiAssistant(buildAiPrompt('optimize', selection));
}

function initTerminal(sid: string): void {
  cleanup();
  if (!termRef.value) return;
  terminalInputBuffer = '';
  lastOutputSeq = -1;
  resetAutocompleteState();

  const activeTheme = effectiveTerminalTheme.value;
  term = new Terminal({
    cursorBlink: true,
    scrollback: getTerminalScrollbackLimit(),
    fontSize: getTerminalFontSize(),
    fontFamily: getTerminalFontFamily(),
    theme: {
      ...activeTheme,
      background: parseBoolean(getAppearanceValue(['terminalBackgroundEnabled', 'terminal_background_enabled'], 'true'))
        ? '#00000000'
        : (activeTheme.background ?? '#1e1e2e'),
    },
  });

  fitAddon = new FitAddon();
  searchAddon = new SearchAddon();

  term.loadAddon(fitAddon);
  term.loadAddon(searchAddon);
  term.loadAddon(new WebLinksAddon());

  term.open(termRef.value);
  term.attachCustomKeyEventHandler(handleAutocompleteKeydown);
  syncAutocompleteCursorPosition();
  selectionDisposable = term.onSelectionChange(() => {
    if (!isTerminalAutoCopyOnSelectEnabled()) {
      return;
    }

    const selectedText = term?.getSelection();
    if (!selectedText) {
      return;
    }

    void navigator.clipboard.writeText(selectedText).catch(() => undefined);
  });
  termRef.value?.addEventListener('contextmenu', handleTerminalContextMenu);
  applyTerminalAppearance();

  term.onData((data) => {
    applyTerminalInputDelta(data);
    const b64 = encodeUtf8Base64(data);
    sshApi.write(sid, b64).catch(() => {});
  });

  onSshOutput(sid, {
    onData: (b64, chunk) => {
      if (!shouldConsumeChunk(chunk)) {
        return;
      }
      term?.write(decodeBase64ToBytes(b64));
    },
    onStderr: (b64, chunk) => {
      if (!shouldConsumeChunk(chunk)) {
        return;
      }
      term?.write(decodeBase64ToBytes(b64));
    },
    onExit: (code) => {
      term?.writeln(`\r\n[Process exited with code ${code}]`);
    },
    onClose: () => {
      term?.writeln('\r\n[Connection closed]');
    },
  })
    .then((fn) => {
      unlisten = fn;
      return sshApi.takeOutputBacklog(sid).catch(() => [] as SshOutputChunk[]);
    })
    .then((chunks) => {
      const ordered = chunks.slice().sort((a, b) => a.seq - b.seq);
      for (const chunk of ordered) {
        if (!shouldConsumeChunk(chunk)) {
          continue;
        }
        term?.write(decodeBase64ToBytes(chunk.data));
      }
    })
    .catch(() => undefined);

  resizeObserver = new ResizeObserver(() => {
    fitAddon?.fit();
    if (term) {
      const { cols, rows } = term;
      sshApi.resize(sid, cols, rows).catch(() => {});
      syncAutocompleteCursorPosition();
    }
  });
  resizeObserver.observe(termRef.value);
}


async function handleTerminalContextMenu(event: MouseEvent): Promise<void> {
  if (!sessionId.value) {
    return;
  }

  if (event.ctrlKey) {
    event.preventDefault();
    event.stopPropagation();
    await openTerminalContextMenu(event);
    return;
  }

  closeTerminalContextMenu();
  if (!isTerminalRightClickPasteEnabled()) {
    return;
  }

  try {
    event.preventDefault();
    event.stopPropagation();
    await pasteClipboardToTerminal();
  } catch {
  }
}

function handleGlobalPointerDown(event: MouseEvent): void {
  const target = event.target as HTMLElement | null;
  if (terminalContextMenu.value.visible && !target?.closest('.terminal-context-menu')) {
    closeTerminalContextMenu();
  }
  if (showAutocomplete.value && !target?.closest('.autocomplete-popup')) {
    resetAutocompleteState(false);
  }
}

function handleGlobalKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    closeTerminalContextMenu();
    resetAutocompleteState(false);
  }
}

function cleanup(): void {
  unlisten?.();
  unlisten = null;

  resizeObserver?.disconnect();
  resizeObserver = null;

  selectionDisposable?.dispose();
  selectionDisposable = null;
  termRef.value?.removeEventListener('contextmenu', handleTerminalContextMenu);
  closeTerminalContextMenu();
  resetAutocompleteState();
  terminalInputBuffer = '';
  lastOutputSeq = -1;

  term?.dispose();
  term = null;
  fitAddon = null;
  searchAddon = null;
}

watch([sessionId, () => activeSession.value?.protocol], ([newSid, protocol]) => {
  if (newSid && protocol === 'SSH') {
    setTimeout(() => initTerminal(newSid), 0);
  } else {
    cleanup();
    currentSearchTerm = '';
  }
});

watch(
  runtimeSettings,
  () => {
    if (!term) {
      return;
    }
    term.options.scrollback = getTerminalScrollbackLimit();
  },
  { deep: true },
);

watch(
  appearance,
  () => {
    if (term) {
      applyTerminalAppearance();
    }
  },
  { deep: true },
);

onMounted(async () => {
  if (!settingsStore.loaded) {
    await settingsStore.loadAll().catch(() => undefined);
  }

  if (!appearanceStore.loaded) {
    await appearanceStore.loadAll().catch(() => undefined);
  }

  if (sessionId.value && activeSession.value?.protocol === 'SSH') {
    initTerminal(sessionId.value);
  }

  window.addEventListener('nexus:terminal-search:update', handleSearchUpdate as EventListener);
  window.addEventListener('nexus:terminal-search:next', handleSearchNext as EventListener);
  window.addEventListener('nexus:terminal-search:previous', handleSearchPrevious as EventListener);
  window.addEventListener('nexus:terminal-search:clear', handleSearchClear as EventListener);
  window.addEventListener('mousedown', handleGlobalPointerDown);
  window.addEventListener('keydown', handleGlobalKeydown);
});

onBeforeUnmount(() => {
  window.removeEventListener('nexus:terminal-search:update', handleSearchUpdate as EventListener);
  window.removeEventListener('nexus:terminal-search:next', handleSearchNext as EventListener);
  window.removeEventListener('nexus:terminal-search:previous', handleSearchPrevious as EventListener);
  window.removeEventListener('nexus:terminal-search:clear', handleSearchClear as EventListener);
  window.removeEventListener('mousedown', handleGlobalPointerDown);
  window.removeEventListener('keydown', handleGlobalKeydown);

  cleanup();
});
</script>

<style scoped>
.terminal-wrapper {
  width: 100%;
  height: 100%;
  position: relative;
  overflow: hidden;
}

.terminal-container {
  width: 100%;
  height: 100%;
  box-sizing: border-box;
  padding: 10px 4px 2px 12px;
  background: var(--bg-base);
}

.terminal-container-vnc {
  padding: 0;
}

.terminal-host {
  width: calc(100% - 12px);
  height: calc(100% - 12px);
}

.terminal-host :deep(.xterm-rows) {
  text-shadow: var(--terminal-text-shadow, none);
}

.terminal-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: var(--bg-base);
}

.placeholder-text {
  color: var(--text-dim);
  font-size: 14px;
}

.terminal-context-menu {
  position: fixed;
  z-index: 12000;
  min-width: 220px;
  padding: 6px;
  border-radius: 10px;
  border: 1px solid var(--border);
  background: color-mix(in srgb, var(--bg-surface0) 92%, var(--bg-mantle) 8%);
  box-shadow: 0 16px 32px color-mix(in srgb, var(--bg-base) 74%, transparent);
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.terminal-menu-item {
  width: 100%;
  border: none;
  border-radius: 8px;
  padding: 7px 10px;
  background: transparent;
  color: var(--text);
  font-size: 12px;
  text-align: left;
  cursor: pointer;
  transition: background-color 0.14s ease, color 0.14s ease;
}

.terminal-menu-item:hover:not(:disabled) {
  background: var(--link-active-bg-color);
  color: var(--link-active-color);
}

.terminal-menu-item:disabled {
  color: var(--text-dim);
  cursor: not-allowed;
}

.terminal-menu-separator {
  height: 1px;
  margin: 4px 2px;
  background: var(--border);
}

.terminal-menu-group-title {
  padding: 4px 10px 3px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-sub);
}
</style>


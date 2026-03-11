<template>
  <div class="terminal-wrapper">
    <div v-if="!currentSessionId || !currentSession" class="terminal-placeholder">
      <span class="placeholder-text">无活动会话</span>
    </div>
    <div v-else-if="isVncSession" class="terminal-container terminal-container-vnc">
      <VncSessionView :session-id="currentSessionId" />
    </div>
    <div v-else class="terminal-container" :style="terminalContainerStyle">
      <iframe
        v-if="terminalBackgroundSrcdoc"
        class="terminal-background-html"
        :srcdoc="terminalBackgroundSrcdoc"
        sandbox="allow-scripts"
        aria-hidden="true"
        tabindex="-1"
      ></iframe>
      <div ref="termRef" class="terminal-host"></div>
    </div>
    <CommandAutocomplete
      ref="autocompleteRef"
      :visible="showAutocomplete"
      :input="autocompleteInput"
      :cursor-position="autocompleteCursorPosition"
      :session-id="currentSessionId"
      @select="handleAutocompleteSelect"
      @close="showAutocomplete = false"
    />

    <div
      v-if="shouldRenderInlineSuggestion"
      class="terminal-inline-suggestion"
      :style="inlineSuggestionStyle"
      aria-hidden="true"
    >
      {{ inlineSuggestion?.ghostText }}
    </div>

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

    <Teleport to="body">
      <div
        v-if="selectionAiToolbar.visible"
        class="terminal-selection-ai-toolbar"
        :style="selectionAiToolbarStyle"
        @mousedown.stop
        @click.stop
      >
        <div class="terminal-selection-ai-toolbar-title">{{ selectionAiToolbarTitle }}</div>
        <button class="terminal-selection-ai-toolbar-btn" @click="handleSelectionAiAction('ai-write')">AI 撰写代码</button>
        <button class="terminal-selection-ai-toolbar-btn" @click="handleSelectionAiAction('ai-explain')">AI 解释代码</button>
        <button class="terminal-selection-ai-toolbar-btn" @click="handleSelectionAiAction('ai-optimize')">AI 优化代码</button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onActivated, onBeforeUnmount, onDeactivated, onMounted, ref, watch } from 'vue';
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
import { getInlineSuggestion, shouldShowInlineSuggestion, type InlineSuggestion } from '@/utils/inline-suggest';
import { useSessionStore } from '@/stores/session';
import { useAIStore } from '@/stores/ai';
import { useAppearanceStore } from '@/stores/appearance';
import { useSettingsStore } from '@/stores/settings';
import { useUiNotificationsStore } from '@/stores/uiNotifications';

const props = defineProps<{
  sessionId: string;
}>();

const sessionStore = useSessionStore();
const aiStore = useAIStore();
const appearanceStore = useAppearanceStore();
const settingsStore = useSettingsStore();
const notifications = useUiNotificationsStore();
const { appearance, effectiveTerminalTheme, terminalCustomHTML } = storeToRefs(appearanceStore);
const { settings: runtimeSettings } = storeToRefs(settingsStore);
const currentSessionId = computed(() => props.sessionId);
const currentSession = computed(() => sessionStore.getSession(props.sessionId));
const isVncSession = computed(() => currentSession.value?.protocol === 'VNC');

const termRef = ref<HTMLElement>();
interface CommandAutocompleteExpose {
  selectNext: () => void;
  selectPrevious: () => void;
  selectPageDown: () => void;
  selectPageUp: () => void;
  selectFirst: () => void;
  selectLast: () => void;
  selectCurrent: () => string | null;
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
const terminalInputValue = ref('');
let lastOutputSeq = -1;
const inlineSuggestion = ref<InlineSuggestion | null>(null);
let inlineSuggestTimer: ReturnType<typeof setTimeout> | null = null;

let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let searchAddon: SearchAddon | null = null;
let unlisten: (() => void) | null = null;
let resizeObserver: ResizeObserver | null = null;
let currentSearchTerm = '';
let selectionDisposable: { dispose: () => void } | null = null;
let globalListenersAttached = false;
type TerminalContextMenuAction = 'copy' | 'paste' | 'select-all' | 'clear' | 'ai-write' | 'ai-explain' | 'ai-optimize';
const terminalContextMenu = ref({
  visible: false,
  x: 0,
  y: 0,
  hasSelection: false,
  aiEnabled: false,
});
const selectionAiToolbar = ref({
  visible: false,
  x: 0,
  y: 0,
  selection: '',
  aiEnabled: false,
});
const lastSelectionPointer = ref({ x: 0, y: 0 });
let selectionAiToolbarSerial = 0;

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

const inlineSuggestionStyle = computed<Record<string, string>>(() => {
  const { x, y } = autocompleteCursorPosition.value;
  if (x <= 0 && y <= 0) return { left: '-9999px', top: '-9999px', maxWidth: '0px', visibility: 'hidden' };
  const viewportWidth = typeof window === 'undefined' ? x + 380 : window.innerWidth;
  const maxWidth = Math.max(160, viewportWidth - x - 12);
  return {
    left: `${x}px`,
    top: `${y}px`,
    maxWidth: `${maxWidth}px`,
    visibility: 'visible',
  };
});

const shouldRenderInlineSuggestion = computed(() => {
  return Boolean(!showAutocomplete.value && inlineSuggestion.value?.ghostText);
});

function clearInlineSuggestion(): void {
  inlineSuggestion.value = null;
  if (inlineSuggestTimer) {
    clearTimeout(inlineSuggestTimer);
    inlineSuggestTimer = null;
  }
}

async function refreshInlineSuggestion(): Promise<void> {
  if (showAutocomplete.value) {
    inlineSuggestion.value = null;
    return;
  }
  const input = terminalInputValue.value;
  if (!shouldShowInlineSuggestion(input)) {
    inlineSuggestion.value = null;
    return;
  }
  inlineSuggestion.value = await getInlineSuggestion(input, currentSessionId.value).catch(() => null);
}

function scheduleInlineSuggestionRefresh(): void {
  if (inlineSuggestTimer) {
    clearTimeout(inlineSuggestTimer);
  }
  inlineSuggestTimer = setTimeout(() => {
    void refreshInlineSuggestion();
  }, 120);
}

watch(showAutocomplete, (visible) => {
  if (visible) {
    inlineSuggestion.value = null;
  }
});

function refreshAutocompleteByBuffer(): void {
  if (!currentSessionId.value || currentSession.value?.protocol !== 'SSH') {
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
    terminalInputValue.value = '';
    clearInlineSuggestion();
    resetAutocompleteState();
    return;
  }
  if (data === '\u0015' || data === '\u0003') {
    terminalInputBuffer = '';
    terminalInputValue.value = '';
    clearInlineSuggestion();
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
  terminalInputValue.value = terminalInputBuffer;
  syncAutocompleteCursorPosition();
  refreshAutocompleteByBuffer();
  scheduleInlineSuggestionRefresh();
}

async function handleAutocompleteSelect(text: string): Promise<void> {
  if (!currentSessionId.value || !text) {
    return;
  }
  clearInlineSuggestion();
  const currentInput = autocompleteInput.value;
  const words = currentInput.split(/\s+/);
  const lastWord = words[words.length - 1] ?? '';
  const deleteCount = currentInput.startsWith('/') ? currentInput.length : lastWord.length;
  const payload = '\u007f'.repeat(deleteCount) + text;
  await sshApi.write(currentSessionId.value, encodeUtf8Base64(payload)).catch(() => undefined);

  terminalInputBuffer = currentInput.slice(0, currentInput.length - deleteCount) + text;
  terminalInputValue.value = terminalInputBuffer;
  if (text.endsWith(' ')) {
    autocompleteInput.value = terminalInputBuffer;
    showAutocomplete.value = true;
    await nextTick();
    syncAutocompleteCursorPosition();
  } else {
    resetAutocompleteState();
  }
  scheduleInlineSuggestionRefresh();
  term?.focus();
}

function triggerAutocompleteSelection(): boolean {
  const selectedText = autocompleteRef.value?.selectCurrent?.();
  if (!selectedText) {
    return false;
  }
  void handleAutocompleteSelect(selectedText);
  return true;
}

function acceptInlineSuggestion(): boolean {
  const ghostText = inlineSuggestion.value?.ghostText;
  if (!currentSessionId.value || !ghostText) {
    return false;
  }
  clearInlineSuggestion();
  terminalInputBuffer += ghostText;
  terminalInputValue.value = terminalInputBuffer;
  sshApi.write(currentSessionId.value, encodeUtf8Base64(ghostText)).catch(() => undefined);
  syncAutocompleteCursorPosition();
  scheduleInlineSuggestionRefresh();
  term?.focus();
  return true;
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
  if (event.type !== 'keydown') {
    return true;
  }
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return true;
  }

  if (!showAutocomplete.value) {
    if ((event.key === 'ArrowRight' || event.key === 'End') && acceptInlineSuggestion()) {
      return false;
    }
    return true;
  }

  const hasSuggestions = autocompleteRef.value?.hasSuggestions?.() ?? false;
  const hasActiveSelection = autocompleteRef.value?.hasActiveSelection?.() ?? false;
  switch (event.key) {
    case 'Enter':
      if (hasSuggestions && hasActiveSelection && triggerAutocompleteSelection()) {
        return false;
      }
      resetAutocompleteState();
      return true;
    case 'Tab':
      if (!hasSuggestions) {
        return true;
      }
      if (triggerAutocompleteSelection()) {
        return false;
      }
      return true;
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
    case 'PageDown':
      if (!hasSuggestions) {
        return true;
      }
      autocompleteRef.value?.selectPageDown?.();
      return false;
    case 'PageUp':
      if (!hasSuggestions) {
        return true;
      }
      autocompleteRef.value?.selectPageUp?.();
      return false;
    case 'Home':
      if (!hasSuggestions) {
        return true;
      }
      autocompleteRef.value?.selectFirst?.();
      return false;
    case 'End':
      if (!hasSuggestions) {
        return true;
      }
      autocompleteRef.value?.selectLast?.();
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

const selectionAiToolbarStyle = computed<Record<string, string>>(() => {
  const toolbarWidth = 320;
  const toolbarHeight = 40;
  const viewportWidth = typeof window === 'undefined' ? toolbarWidth : window.innerWidth;
  const viewportHeight = typeof window === 'undefined' ? toolbarHeight : window.innerHeight;
  const offsetY = 14;
  const margin = 8;

  let left = selectionAiToolbar.value.x - toolbarWidth / 2;
  let top = selectionAiToolbar.value.y - toolbarHeight - offsetY;
  if (top < margin) {
    top = selectionAiToolbar.value.y + offsetY;
  }
  left = Math.max(margin, Math.min(left, viewportWidth - toolbarWidth - margin));
  top = Math.max(margin, Math.min(top, viewportHeight - toolbarHeight - margin));

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
const selectionAiToolbarTitle = computed(() => (selectionAiToolbar.value.aiEnabled ? 'AI 助手' : 'AI 助手 (未配置)'));

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
  const value = getAppearanceValue(['terminal_font_size', 'terminalFontSize'], '16');
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 16;
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

function getTerminalBackgroundImageUrl(): string {
  const enabled = parseBoolean(getAppearanceValue(['terminalBackgroundEnabled', 'terminal_background_enabled'], 'true'));
  if (!enabled) {
    return '';
  }

  const imagePath = getAppearanceValue(['terminalBackgroundImage', 'terminal_background_image'], '');
  if (!imagePath) {
    return '';
  }

  return normalizeImageUrl(imagePath);
}

function shouldUseTransparentTerminalBackground(): boolean {
  return getTerminalBackgroundImageUrl() !== '' || terminalCustomHTML.value.trim() !== '';
}

const terminalBackgroundSrcdoc = computed(() => {
  const html = terminalCustomHTML.value.trim();
  if (!html) {
    return '';
  }

  return `<!doctype html>
<html>
  <head>
    <meta charset="utf-8" />
    <style>
      html, body {
        margin: 0;
        width: 100%;
        height: 100%;
        overflow: hidden;
        background: transparent;
      }
      body {
        position: relative;
      }
      body > * {
        max-width: 100%;
      }
    </style>
  </head>
  <body>${html}</body>
</html>`;
});

const terminalContainerStyle = computed<Record<string, string>>(() => {
  const imageUrl = getTerminalBackgroundImageUrl();
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

  const transparentBackground = shouldUseTransparentTerminalBackground();
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
  return !!detail?.sessionId && detail.sessionId === currentSessionId.value;
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

function closeSelectionAiToolbar(): void {
  selectionAiToolbar.value.visible = false;
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

function resolveSelectionToolbarAnchor(): { x: number; y: number } {
  if (lastSelectionPointer.value.x > 0 && lastSelectionPointer.value.y > 0) {
    return lastSelectionPointer.value;
  }

  const rect = termRef.value?.getBoundingClientRect();
  if (!rect) {
    return { x: 0, y: 0 };
  }

  return {
    x: Math.round(rect.left + rect.width / 2),
    y: Math.round(rect.top + rect.height / 2),
  };
}

async function refreshSelectionAiToolbar(): Promise<void> {
  const selection = getSelectedText();
  if (!selection.trim()) {
    closeSelectionAiToolbar();
    return;
  }

  const serial = ++selectionAiToolbarSerial;
  await ensureAiConfigLoaded();
  if (serial !== selectionAiToolbarSerial) {
    return;
  }

  const anchor = resolveSelectionToolbarAnchor();
  selectionAiToolbar.value = {
    visible: true,
    x: anchor.x,
    y: anchor.y,
    selection,
    aiEnabled: Boolean(aiStore.hasDefaultModel),
  };
}

async function handleSelectionAiAction(action: Extract<TerminalContextMenuAction, 'ai-write' | 'ai-explain' | 'ai-optimize'>): Promise<void> {
  const storedSelection = selectionAiToolbar.value.selection;
  closeSelectionAiToolbar();

  await ensureAiConfigLoaded();
  if (!aiStore.hasDefaultModel) {
    notifications.addNotification('warning', '请先在设置中配置 AI 默认模型');
    return;
  }

  const selection = getSelectedText() || storedSelection;
  const trimmed = selection.trim();

  if (action === 'ai-write') {
    if (!trimmed) {
      notifications.addNotification('warning', '请输入描述');
      return;
    }
    openWorkspaceAiAssistant(buildAiPrompt('write', trimmed));
    return;
  }

  if (!trimmed) {
    notifications.addNotification('warning', action === 'ai-explain' ? '请先选中要解释的代码' : '请先选中要优化的代码');
    return;
  }

  openWorkspaceAiAssistant(buildAiPrompt(action === 'ai-explain' ? 'explain' : 'optimize', trimmed));
}

async function pasteClipboardToTerminal(): Promise<void> {
  if (!currentSessionId.value) {
    return;
  }
  const clipboardText = await navigator.clipboard.readText();
  if (!clipboardText) {
    return;
  }
  await sshApi.write(currentSessionId.value, btoa(clipboardText));
}

async function openTerminalContextMenu(event: MouseEvent): Promise<void> {
  closeSelectionAiToolbar();
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
  if (!currentSessionId.value) {
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
    notifications.addNotification('warning', '请先在设置中配置 AI 默认模型');
    return;
  }

  if (action === 'ai-write') {
    if (!selection.trim()) {
      notifications.addNotification('warning', '请输入描述');
      return;
    }
    openWorkspaceAiAssistant(buildAiPrompt('write', selection));
    return;
  }

  if (action === 'ai-explain') {
    if (!selection.trim()) {
      notifications.addNotification('warning', '请先选中要解释的代码');
      return;
    }
    openWorkspaceAiAssistant(buildAiPrompt('explain', selection));
    return;
  }

  if (!selection.trim()) {
    notifications.addNotification('warning', '请先选中要优化的代码');
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
    allowTransparency: true,
    theme: {
      ...activeTheme,
      background: shouldUseTransparentTerminalBackground()
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
    const selectedText = term?.getSelection();
    if (!selectedText?.trim()) {
      closeSelectionAiToolbar();
    }

    if (!isTerminalAutoCopyOnSelectEnabled()) {
      return;
    }

    if (selectedText) {
      void navigator.clipboard.writeText(selectedText).catch(() => undefined);
    }
  });
  termRef.value?.addEventListener('contextmenu', handleTerminalContextMenu);
  termRef.value?.addEventListener('mouseup', handleTerminalMouseUp);
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

  bindResizeObserver(sid);
}


async function handleTerminalContextMenu(event: MouseEvent): Promise<void> {
  if (!currentSessionId.value) {
    return;
  }

  const rightClickPasteEnabled = isTerminalRightClickPasteEnabled();
  const shouldOpenMenu = event.ctrlKey || !rightClickPasteEnabled;

  if (shouldOpenMenu) {
    event.preventDefault();
    event.stopPropagation();
    await openTerminalContextMenu(event);
    return;
  }

  closeTerminalContextMenu();
  try {
    event.preventDefault();
    event.stopPropagation();
    await pasteClipboardToTerminal();
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    notifications.addNotification('error', `右键粘贴失败：${message}`);
    await openTerminalContextMenu(event);
  }
}

function handleTerminalMouseUp(event: MouseEvent): void {
  lastSelectionPointer.value = { x: event.clientX, y: event.clientY };
  void refreshSelectionAiToolbar();
}

function handleGlobalPointerUp(event: MouseEvent): void {
  const target = event.target as Node | null;
  if (target && termRef.value?.contains(target)) {
    return;
  }
  lastSelectionPointer.value = { x: event.clientX, y: event.clientY };
  void refreshSelectionAiToolbar();
}

function handleGlobalPointerDown(event: MouseEvent): void {
  const target = event.target as HTMLElement | null;
  if (terminalContextMenu.value.visible && !target?.closest('.terminal-context-menu')) {
    closeTerminalContextMenu();
  }
  if (selectionAiToolbar.value.visible && !target?.closest('.terminal-selection-ai-toolbar')) {
    closeSelectionAiToolbar();
  }
  if (showAutocomplete.value && !target?.closest('.autocomplete-popup')) {
    resetAutocompleteState(false);
  }
}

function handleGlobalKeydown(event: KeyboardEvent): void {
  if (event.key === 'Escape') {
    closeTerminalContextMenu();
    closeSelectionAiToolbar();
    resetAutocompleteState(false);
  }
}

function bindResizeObserver(sid: string): void {
  disconnectResizeObserver();
  if (!termRef.value) {
    return;
  }

  resizeObserver = new ResizeObserver(() => {
    if (!termRef.value) {
      return;
    }
    const rect = termRef.value.getBoundingClientRect();
    if (rect.width < 1 || rect.height < 1) {
      return;
    }
    fitAddon?.fit();
    if (term) {
      const { cols, rows } = term;
      sshApi.resize(sid, cols, rows).catch(() => {});
      syncAutocompleteCursorPosition();
    }
  });
  resizeObserver.observe(termRef.value);
}

function disconnectResizeObserver(): void {
  resizeObserver?.disconnect();
  resizeObserver = null;
}

function attachGlobalListeners(): void {
  if (globalListenersAttached) {
    return;
  }
  window.addEventListener('nexus:terminal-search:update', handleSearchUpdate as EventListener);
  window.addEventListener('nexus:terminal-search:next', handleSearchNext as EventListener);
  window.addEventListener('nexus:terminal-search:previous', handleSearchPrevious as EventListener);
  window.addEventListener('nexus:terminal-search:clear', handleSearchClear as EventListener);
  window.addEventListener('mousedown', handleGlobalPointerDown);
  window.addEventListener('mouseup', handleGlobalPointerUp);
  window.addEventListener('keydown', handleGlobalKeydown);
  globalListenersAttached = true;
}

function detachGlobalListeners(): void {
  if (!globalListenersAttached) {
    return;
  }
  window.removeEventListener('nexus:terminal-search:update', handleSearchUpdate as EventListener);
  window.removeEventListener('nexus:terminal-search:next', handleSearchNext as EventListener);
  window.removeEventListener('nexus:terminal-search:previous', handleSearchPrevious as EventListener);
  window.removeEventListener('nexus:terminal-search:clear', handleSearchClear as EventListener);
  window.removeEventListener('mousedown', handleGlobalPointerDown);
  window.removeEventListener('mouseup', handleGlobalPointerUp);
  window.removeEventListener('keydown', handleGlobalKeydown);
  globalListenersAttached = false;
}

function cleanup(): void {
  unlisten?.();
  unlisten = null;

  disconnectResizeObserver();

  selectionDisposable?.dispose();
  selectionDisposable = null;
  termRef.value?.removeEventListener('contextmenu', handleTerminalContextMenu);
  termRef.value?.removeEventListener('mouseup', handleTerminalMouseUp);
  closeTerminalContextMenu();
  closeSelectionAiToolbar();
  resetAutocompleteState();
  terminalInputBuffer = '';
  terminalInputValue.value = '';
  clearInlineSuggestion();
  lastOutputSeq = -1;

  term?.dispose();
  term = null;
  fitAddon = null;
  searchAddon = null;
}

watch([currentSessionId, () => currentSession.value?.protocol], ([newSid, protocol]) => {
  if (newSid && protocol === 'SSH') {
    if (!term) {
      setTimeout(() => initTerminal(newSid), 0);
    }
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

  if (currentSessionId.value && currentSession.value?.protocol === 'SSH') {
    initTerminal(currentSessionId.value);
  }

  attachGlobalListeners();
});

onActivated(() => {
  attachGlobalListeners();
  if (currentSessionId.value && currentSession.value?.protocol === 'SSH') {
    if (!term) {
      initTerminal(currentSessionId.value);
      return;
    }
    bindResizeObserver(currentSessionId.value);
    requestAnimationFrame(() => {
      fitAddon?.fit();
      syncAutocompleteCursorPosition();
    });
  }
});

onDeactivated(() => {
  detachGlobalListeners();
  disconnectResizeObserver();
  closeTerminalContextMenu();
  resetAutocompleteState(false);
});

onBeforeUnmount(() => {
  detachGlobalListeners();

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
  position: relative;
}

.terminal-container-vnc {
  padding: 0;
}

.terminal-host {
  width: calc(100% - 12px);
  height: calc(100% - 12px);
  position: relative;
  z-index: 1;
}

.terminal-host :deep(.xterm-rows) {
  text-shadow: var(--terminal-text-shadow, none);
}

.terminal-background-html {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  border: none;
  background: transparent;
  pointer-events: none;
  z-index: 0;
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
  font-size: calc(14px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(11px + var(--ui-font-size-offset));
  font-weight: 600;
  color: var(--text-sub);
}

.terminal-selection-ai-toolbar {
  position: fixed;
  z-index: 11900;
  padding: 6px 8px;
  border-radius: 999px;
  border: 1px solid var(--border);
  background: color-mix(in srgb, var(--bg-surface0) 92%, var(--bg-mantle) 8%);
  box-shadow: 0 16px 32px color-mix(in srgb, var(--bg-base) 74%, transparent);
  display: flex;
  align-items: center;
  gap: 6px;
}

.terminal-selection-ai-toolbar-title {
  padding: 0 6px;
  font-size: calc(11px + var(--ui-font-size-offset));
  font-weight: 600;
  color: var(--text-sub);
  user-select: none;
  white-space: nowrap;
}

.terminal-selection-ai-toolbar-btn {
  border: none;
  border-radius: 999px;
  padding: 6px 10px;
  background: transparent;
  color: var(--text);
  font-size: calc(12px + var(--ui-font-size-offset));
  cursor: pointer;
  transition: background-color 0.14s ease, color 0.14s ease;
  user-select: none;
  white-space: nowrap;
}

.terminal-selection-ai-toolbar-btn:hover {
  background: var(--link-active-bg-color);
  color: var(--link-active-color);
}

.terminal-inline-suggestion {
  position: fixed;
  z-index: 11000;
  pointer-events: none;
  white-space: pre;
  overflow: hidden;
  text-overflow: ellipsis;
  font-family: Consolas, 'Courier New', monospace;
  font-size: calc(13px + var(--ui-font-size-offset));
  line-height: 1.2;
  color: color-mix(in srgb, var(--text-sub) 62%, transparent);
}
</style>


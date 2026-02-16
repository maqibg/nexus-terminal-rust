<template>
  <div class="terminal-wrapper">
    <div v-if="!sessionId" class="terminal-placeholder">
      <span class="placeholder-text">无活动会话</span>
    </div>
    <div v-else class="terminal-container" :style="terminalContainerStyle">
      <div ref="termRef" class="terminal-host"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { FitAddon } from '@xterm/addon-fit';
import { SearchAddon } from '@xterm/addon-search';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { storeToRefs } from 'pinia';
import { onSshOutput, sshApi } from '@/lib/api';
import { useSessionStore } from '@/stores/session';
import { useAppearanceStore } from '@/stores/appearance';

const sessionStore = useSessionStore();
const appearanceStore = useAppearanceStore();
const { activeSessionId: sessionId } = storeToRefs(sessionStore);
const { appearance, effectiveTerminalTheme } = storeToRefs(appearanceStore);

const termRef = ref<HTMLElement>();
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let searchAddon: SearchAddon | null = null;
let unlisten: (() => void) | null = null;
let resizeObserver: ResizeObserver | null = null;
let currentSearchTerm = '';

interface TerminalSearchEventDetail {
  sessionId?: string;
  term?: string;
}

const searchOptions = {
  caseSensitive: false,
  regex: false,
  wholeWord: false,
} as const;

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

function getTerminalFontFamily(): string {
  return getAppearanceValue(
    ['terminal_font_family', 'terminalFontFamily'],
    `Consolas, 'Courier New', monospace, 'Microsoft YaHei', '微软雅黑'`,
  );
}

function getTerminalFontSize(): number {
  const value = getAppearanceValue(['terminal_font_size', 'terminalFontSize'], '13');
  const parsed = Number.parseInt(value, 10);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 13;
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
  const enabled = parseBoolean(getAppearanceValue(['terminalBackgroundEnabled', 'terminal_background_enabled'], 'false'));
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
    const offsetX = parseNumber(getAppearanceValue(['terminalTextShadowOffsetX', 'terminal_text_shadow_offset_x'], '2'), 2);
    const offsetY = parseNumber(getAppearanceValue(['terminalTextShadowOffsetY', 'terminal_text_shadow_offset_y'], '2'), 2);
    const blur = Math.max(0, parseNumber(getAppearanceValue(['terminalTextShadowBlur', 'terminal_text_shadow_blur'], '3'), 3));
    const color = getAppearanceValue(['terminalTextShadowColor', 'terminal_text_shadow_color'], '#000000');
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
    getAppearanceValue(['terminalBackgroundEnabled', 'terminal_background_enabled'], 'false'),
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

function initTerminal(sid: string): void {
  cleanup();
  if (!termRef.value) return;

  const activeTheme = effectiveTerminalTheme.value;
  term = new Terminal({
    cursorBlink: true,
    fontSize: getTerminalFontSize(),
    fontFamily: getTerminalFontFamily(),
    theme: {
      ...activeTheme,
      background: parseBoolean(getAppearanceValue(['terminalBackgroundEnabled', 'terminal_background_enabled'], 'false'))
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
  applyTerminalAppearance();

  term.onData((data) => {
    const b64 = btoa(data);
    sshApi.write(sid, b64).catch(() => {});
  });

  onSshOutput(sid, {
    onData: (b64) => {
      term?.write(Uint8Array.from(atob(b64), (char) => char.charCodeAt(0)));
    },
    onStderr: (b64) => {
      term?.write(Uint8Array.from(atob(b64), (char) => char.charCodeAt(0)));
    },
    onExit: (code) => {
      term?.writeln(`\r\n[Process exited with code ${code}]`);
    },
    onClose: () => {
      term?.writeln('\r\n[Connection closed]');
    },
  }).then((fn) => {
    unlisten = fn;
  });

  resizeObserver = new ResizeObserver(() => {
    fitAddon?.fit();
    if (term) {
      const { cols, rows } = term;
      sshApi.resize(sid, cols, rows).catch(() => {});
    }
  });
  resizeObserver.observe(termRef.value);
}

function cleanup(): void {
  unlisten?.();
  unlisten = null;

  resizeObserver?.disconnect();
  resizeObserver = null;

  term?.dispose();
  term = null;
  fitAddon = null;
  searchAddon = null;
}

watch(sessionId, (newSid) => {
  if (newSid) {
    setTimeout(() => initTerminal(newSid), 0);
  } else {
    cleanup();
    currentSearchTerm = '';
  }
});

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
  if (!appearanceStore.loaded) {
    await appearanceStore.loadAll().catch(() => undefined);
  }

  if (sessionId.value) {
    initTerminal(sessionId.value);
  }

  window.addEventListener('nexus:terminal-search:update', handleSearchUpdate as EventListener);
  window.addEventListener('nexus:terminal-search:next', handleSearchNext as EventListener);
  window.addEventListener('nexus:terminal-search:previous', handleSearchPrevious as EventListener);
  window.addEventListener('nexus:terminal-search:clear', handleSearchClear as EventListener);
});

onBeforeUnmount(() => {
  window.removeEventListener('nexus:terminal-search:update', handleSearchUpdate as EventListener);
  window.removeEventListener('nexus:terminal-search:next', handleSearchNext as EventListener);
  window.removeEventListener('nexus:terminal-search:previous', handleSearchPrevious as EventListener);
  window.removeEventListener('nexus:terminal-search:clear', handleSearchClear as EventListener);

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
</style>

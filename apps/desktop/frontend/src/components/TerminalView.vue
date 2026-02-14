<template>
  <div class="terminal-wrapper">
    <div v-if="!sessionId" class="terminal-placeholder">
      <span class="placeholder-text">无活动会话</span>
    </div>
    <div v-else class="terminal-container">
      <div ref="termRef" class="terminal-host"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { FitAddon } from '@xterm/addon-fit';
import { SearchAddon } from '@xterm/addon-search';
import { WebLinksAddon } from '@xterm/addon-web-links';
import { Terminal } from '@xterm/xterm';
import '@xterm/xterm/css/xterm.css';
import { storeToRefs } from 'pinia';
import { onSshOutput, sshApi } from '@/lib/api';
import { useSessionStore } from '@/stores/session';

const sessionStore = useSessionStore();
const { activeSessionId: sessionId } = storeToRefs(sessionStore);

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

  term = new Terminal({
    cursorBlink: true,
    fontSize: 14,
    fontFamily: "'Cascadia Code', 'Fira Code', 'JetBrains Mono', monospace",
    theme: {
      background: '#1e1e2e',
      foreground: '#cdd6f4',
      cursor: '#f5e0dc',
      selectionBackground: '#45475a',
    },
  });

  fitAddon = new FitAddon();
  searchAddon = new SearchAddon();

  term.loadAddon(fitAddon);
  term.loadAddon(searchAddon);
  term.loadAddon(new WebLinksAddon());

  term.open(termRef.value);
  fitAddon.fit();

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

onMounted(() => {
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
}

.terminal-host {
  width: calc(100% - 12px);
  height: calc(100% - 12px);
}

.terminal-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  background: #1e1e2e;
}

.placeholder-text {
  color: var(--text-dim, #6c7086);
  font-size: 14px;
}
</style>
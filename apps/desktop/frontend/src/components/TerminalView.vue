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
import { ref, onMounted, onBeforeUnmount, watch } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import { WebLinksAddon } from '@xterm/addon-web-links';
import '@xterm/xterm/css/xterm.css';
import { sshApi, onSshOutput } from '@/lib/api';
import { useSessionStore } from '@/stores/session';
import { storeToRefs } from 'pinia';

const sessionStore = useSessionStore();
const { activeSessionId: sessionId } = storeToRefs(sessionStore);

const termRef = ref<HTMLElement>();
let term: Terminal | null = null;
let fitAddon: FitAddon | null = null;
let unlisten: (() => void) | null = null;
let resizeObserver: ResizeObserver | null = null;

function initTerminal(sid: string) {
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
  term.loadAddon(fitAddon);
  term.loadAddon(new WebLinksAddon());
  term.open(termRef.value);
  fitAddon.fit();

  term.onData((data) => {
    const b64 = btoa(data);
    sshApi.write(sid, b64).catch(() => {});
  });

  onSshOutput(sid, {
    onData: (b64) => { term?.write(Uint8Array.from(atob(b64), c => c.charCodeAt(0))); },
    onStderr: (b64) => { term?.write(Uint8Array.from(atob(b64), c => c.charCodeAt(0))); },
    onExit: (code) => { term?.writeln(`\r\n[Process exited with code ${code}]`); },
    onClose: () => { term?.writeln('\r\n[Connection closed]'); },
  }).then(fn => { unlisten = fn; });

  resizeObserver = new ResizeObserver(() => {
    fitAddon?.fit();
    if (term) {
      const { cols, rows } = term;
      sshApi.resize(sid, cols, rows).catch(() => {});
    }
  });
  resizeObserver.observe(termRef.value);
}

function cleanup() {
  unlisten?.();
  unlisten = null;
  resizeObserver?.disconnect();
  resizeObserver = null;
  term?.dispose();
  term = null;
  fitAddon = null;
}

watch(sessionId, (newSid) => {
  if (newSid) {
    // Wait for DOM update
    setTimeout(() => initTerminal(newSid), 0);
  } else {
    cleanup();
  }
});

onMounted(() => {
  if (sessionId.value) {
    initTerminal(sessionId.value);
  }
});

onBeforeUnmount(() => {
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

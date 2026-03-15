<template>
  <div class="local-terminal">
    <div ref="terminalEl" class="terminal-host"></div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';

import { localTerminalApi, onLocalTerminalOutput, type LocalTerminalOutputChunk } from '@/lib/api';

const props = defineProps<{ sessionId: string }>();

const terminalEl = ref<HTMLDivElement | null>(null);
let term: Terminal | null = null;
let fit: FitAddon | null = null;
let resizeObserver: ResizeObserver | null = null;
let unlisten: (() => void) | null = null;

function encodeUtf8Base64(value: string): string {
  const bytes = new TextEncoder().encode(value);
  let binary = '';
  bytes.forEach((b) => {
    binary += String.fromCharCode(b);
  });
  return btoa(binary);
}

function decodeUtf8Base64(base64: string): string {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i += 1) {
    bytes[i] = binary.charCodeAt(i);
  }
  return new TextDecoder().decode(bytes);
}

function writeToTerminal(base64: string, _chunk?: LocalTerminalOutputChunk) {
  term?.write(decodeUtf8Base64(base64));
}

async function applyResize() {
  if (!term) {
    return;
  }
  const cols = term.cols;
  const rows = term.rows;
  if (!cols || !rows) {
    return;
  }
  await localTerminalApi.resize(props.sessionId, cols, rows).catch(() => undefined);
}

async function init() {
  if (!terminalEl.value) {
    return;
  }

  term = new Terminal({
    convertEol: true,
    cursorBlink: true,
    fontSize: 13,
    fontFamily: 'Consolas, "Courier New", monospace',
    theme: { background: 'transparent' },
  });
  fit = new FitAddon();
  term.loadAddon(fit);
  term.open(terminalEl.value);
  fit.fit();
  term.focus();
  await applyResize();

  const backlog = await localTerminalApi.takeOutputBacklog(props.sessionId).catch(() => [] as LocalTerminalOutputChunk[]);
  backlog
    .slice()
    .sort((a, b) => a.seq - b.seq)
    .forEach((chunk) => writeToTerminal(chunk.data, chunk));

  unlisten = await onLocalTerminalOutput(props.sessionId, {
    onData: writeToTerminal,
    onStderr: writeToTerminal,
    onExit: (code) => {
      term?.writeln('');
      term?.writeln(`【进程已退出：${code}】`);
    },
    onClose: () => {
      term?.writeln('');
      term?.writeln('【会话已关闭】');
    },
  });

  term.onData((data) => {
    void localTerminalApi.write(props.sessionId, encodeUtf8Base64(data)).catch(() => undefined);
  });

  resizeObserver = new ResizeObserver(() => {
    if (!fit) {
      return;
    }
    fit.fit();
    void applyResize();
  });
  resizeObserver.observe(terminalEl.value);
}

onMounted(() => {
  void init();
});

onUnmounted(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
  unlisten?.();
  unlisten = null;
  term?.dispose();
  term = null;
  fit = null;
});
</script>

<style scoped>
.local-terminal {
  width: 100%;
  height: 100%;
  background: var(--bg-base);
}

.terminal-host {
  width: 100%;
  height: 100%;
}
</style>


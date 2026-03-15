<template>
  <div class="telnet-terminal">
    <div ref="terminalEl" class="terminal-host"></div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue';
import { Terminal } from '@xterm/xterm';
import { FitAddon } from '@xterm/addon-fit';
import '@xterm/xterm/css/xterm.css';

import { onTelnetOutput, telnetApi, type TelnetOutputChunk } from '@/lib/api';

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

function writeToTerminal(base64: string, _chunk?: TelnetOutputChunk) {
  if (!term) {
    return;
  }
  term.write(decodeUtf8Base64(base64));
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

  const backlog = await telnetApi.takeOutputBacklog(props.sessionId).catch(() => [] as TelnetOutputChunk[]);
  backlog
    .slice()
    .sort((a, b) => a.seq - b.seq)
    .forEach((chunk) => writeToTerminal(chunk.data, chunk));

  unlisten = await onTelnetOutput(props.sessionId, {
    onData: writeToTerminal,
    onClose: () => {
      term?.writeln('');
      term?.writeln('【连接已关闭】');
    },
  });

  term.onData((data) => {
    if (!term) {
      return;
    }
    void telnetApi.write(props.sessionId, encodeUtf8Base64(data)).catch(() => undefined);
  });

  resizeObserver = new ResizeObserver(() => {
    if (!fit) {
      return;
    }
    fit.fit();
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
.telnet-terminal {
  width: 100%;
  height: 100%;
  background: var(--bg-base);
}

.terminal-host {
  width: 100%;
  height: 100%;
}
</style>


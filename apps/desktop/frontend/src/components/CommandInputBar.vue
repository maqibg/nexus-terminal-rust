<template>
  <div class="command-input-bar">
    <button class="bar-btn" @click="clearTerminal" title="清空终端">
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
    />
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, onUnmounted, ref } from 'vue';
import { sshApi, historyApi } from '@/lib/api';
import { useSessionStore } from '@/stores/session';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';

const sessionStore = useSessionStore();
const focusSwitcherStore = useFocusSwitcherStore();
const inputEl = ref<HTMLInputElement>();
const command = ref('');
const history = ref<string[]>([]);
const historyIdx = ref(-1);

let unregisterCommandInput: (() => void) | null = null;
let unregisterTerminalSearch: (() => void) | null = null;

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

function focusCommandInput(): boolean | undefined {
  if (!isElementVisibleAndFocusable(inputEl.value)) {
    return undefined;
  }

  inputEl.value.focus();
  inputEl.value.select();
  return document.activeElement === inputEl.value;
}

onMounted(async () => {
  try {
    const items = await historyApi.list(100);
    history.value = items.map((h) => h.command);
  } catch {
    // ignore
  }

  await nextTick();
  unregisterCommandInput = focusSwitcherStore.registerFocusAction('commandInput', focusCommandInput);
  unregisterTerminalSearch = focusSwitcherStore.registerFocusAction('terminalSearch', focusCommandInput);
});

onUnmounted(() => {
  unregisterCommandInput?.();
  unregisterTerminalSearch?.();
  unregisterCommandInput = null;
  unregisterTerminalSearch = null;
});

function openFocusConfigurator() {
  focusSwitcherStore.toggleConfigurator(true);
}

async function send() {
  const cmd = command.value.trim();
  if (!cmd) return;
  const sid = sessionStore.activeSessionId;
  if (!sid) return;

  const data = btoa(unescape(encodeURIComponent(`${cmd}\n`)));
  await sshApi.write(sid, data);

  history.value.unshift(cmd);
  historyIdx.value = -1;
  command.value = '';

  try {
    await historyApi.add(cmd, sid, sessionStore.activeSession?.connectionId);
  } catch {
    // ignore
  }
}

function clearTerminal() {
  const sid = sessionStore.activeSessionId;
  if (!sid) return;
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
</style>
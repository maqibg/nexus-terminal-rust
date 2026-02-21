<template>
  <div v-if="visible && suggestions.length > 0" class="autocomplete-popup" :style="popupStyle">
    <div class="autocomplete-header">Tab 补全 · ↑↓ 选择 · Esc 取消</div>
    <div class="suggestions-list">
      <div
        v-for="(suggestion, index) in suggestions"
        :key="`${suggestion.type}-${suggestion.text}-${index}`"
        :class="['suggestion-item', { active: index === selectedIndex }]"
        @click="selectSuggestion(suggestion)"
        @mouseenter="selectedIndex = index"
      >
        <span class="suggestion-icon">{{ iconOf(suggestion.type) }}</span>
        <div class="suggestion-main">
          <div class="suggestion-text">
            <span class="match-part">{{ suggestion.matchPart }}</span>
            <span class="rest-part">
              {{ suggestion.displayText ? suggestion.displayText.substring(suggestion.matchPart.length) : suggestion.restPart }}
            </span>
          </div>
          <div v-if="suggestion.description || suggestion.usage" class="suggestion-meta">
            <span v-if="suggestion.usage" class="usage">{{ suggestion.usage }}</span>
            <span v-if="suggestion.usage && suggestion.description" class="meta-sep">-</span>
            <span>{{ suggestion.description }}</span>
          </div>
        </div>
        <span v-if="suggestion.usageCount" class="usage-count">{{ suggestion.usageCount }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { historyApi, quickCommandApi, sshApi, type QuickCommand } from '@/lib/api';
import { registry } from '@/utils/registry';
import type { CompletionItem, CompletionType } from '@/utils/types';
import { getRemotePathSuggestions } from '@/utils/providers/file-system';

interface Suggestion {
  text: string;
  type: CompletionType;
  matchPart: string;
  restPart: string;
  description?: string;
  usageCount?: number;
  priority?: number;
  displayText?: string;
  usage?: string;
  quickCommandId?: number;
}

interface Props {
  visible: boolean;
  input: string;
  cursorPosition: { x: number; y: number };
  sessionId: string;
}

type CompatExecResult = { success: boolean; data?: string; error?: string; exitCode?: number };
type CompatApi = { ssh: { executeCommand: (sessionId: string, command: string, timeout?: number) => Promise<CompatExecResult> } };

const props = defineProps<Props>();
const emit = defineEmits<{ select: [text: string]; close: [] }>();

const suggestions = ref<Suggestion[]>([]);
const selectedIndex = ref(0);
const hasUserSelected = ref(false);
const commandHistory = ref<string[]>([]);
const quickCommands = ref<QuickCommand[]>([]);
const quickCommandsCacheTime = ref(0);

const commonCommands = ref<string[]>([
  'ls', 'cd', 'pwd', 'cat', 'grep', 'find', 'mkdir', 'rm', 'cp', 'mv',
  'chmod', 'chown', 'tar', 'gzip', 'wget', 'curl', 'ssh', 'scp',
  'ps', 'top', 'kill', 'df', 'du', 'free', 'netstat', 'ping',
  'git', 'docker', 'npm', 'yarn', 'python', 'node', 'java', 'gcc',
  'code', 'vim', 'nano', 'gcloud', 'aws', 'kubectl',
]);

const CACHE_TTL = 5000;
const DEBOUNCE_DELAY = 150;
let lastProcessedInput = '';
let currentRequestId = 0;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

const compatApi: CompatApi = {
  ssh: {
    async executeCommand(sessionId: string, command: string, timeout = 3000): Promise<CompatExecResult> {
      try {
        const res = await sshApi.executeCommand(sessionId, command, timeout);
        return {
          success: res.exit_code === 0 || res.stdout.length > 0,
          data: res.stdout,
          error: res.stderr,
          exitCode: res.exit_code,
        };
      } catch (error: any) {
        return { success: false, error: String(error?.message || error) };
      }
    },
  },
};

const popupStyle = computed<Record<string, string>>(() => {
  const { x, y } = props.cursorPosition;
  if (x <= 0 && y <= 0) return { left: '-9999px', top: '-9999px', visibility: 'hidden' };
  const popupWidth = 380;
  const popupHeight = 250;
  const margin = 10;
  const offset = 25;
  const viewportW = window.innerWidth;
  const viewportH = window.innerHeight;
  let left = x + 20;
  if (left + popupWidth > viewportW - margin) {
    left = Math.max(margin, x - popupWidth - 10);
  }
  let top = y + offset;
  if (top + popupHeight > viewportH - margin) {
    top = Math.max(margin, y - popupHeight - 10);
  }
  return { left: `${left}px`, top: `${top}px`, visibility: 'visible' };
});

const iconOf = (type: string): string => ({
  command: 'CMD',
  subcommand: 'ARG',
  option: 'OPT',
  path: 'DIR',
  history: 'HIS',
  snippet: 'QC',
  shortcut: 'KEY',
  hint: 'TIP',
} as Record<string, string>)[type] ?? 'TIP';

async function loadCommandHistory() {
  try {
    const rows = await historyApi.list(300, 0);
    const seen = new Set<string>();
    commandHistory.value = rows
      .map((item) => item.command?.trim())
      .filter((command): command is string => Boolean(command && !seen.has(command) && seen.add(command)))
      .slice(0, 80);
  } catch {
    commandHistory.value = [];
  }
}

async function loadQuickCommands(force = false): Promise<QuickCommand[]> {
  const now = Date.now();
  if (!force && now - quickCommandsCacheTime.value < CACHE_TTL && quickCommands.value.length > 0) {
    return quickCommands.value;
  }
  try {
    quickCommands.value = await quickCommandApi.list();
    quickCommandsCacheTime.value = now;
  } catch {
    quickCommands.value = quickCommands.value ?? [];
  }
  return quickCommands.value;
}

function currentWordOf(input: string): string {
  const words = input.split(' ');
  return words[words.length - 1] ?? '';
}

async function buildRegistrySuggestions(input: string, words: string[], requestId: number): Promise<Suggestion[] | null> {
  if (words.length < 2) return null;
  const cmdName = words[0];
  const currentArgIndex = words.length - 1;
  const currentArg = words[currentArgIndex] ?? '';
  let def = registry.getCommand(cmdName);
  let depth = 0;
  for (let i = 1; i < currentArgIndex; i++) {
    if (!def?.subcommands?.[words[i]]) break;
    def = def.subcommands[words[i]];
    depth++;
  }
  if (!def) return null;

  const generated = def.generate
    ? (await def.generate({
        input,
        args: words,
        currentArgIndex,
        currentArg,
        sessionId: props.sessionId,
        electronAPI: compatApi,
      }).catch(() => [])) ?? []
    : [];

  if (requestId !== currentRequestId) return null;
  const staticMatches = (def.options ?? []).filter((opt) => {
    if (opt.type === 'subcommand') {
      if (currentArgIndex > depth + 1) return false;
      if (words.slice(1, currentArgIndex).includes(opt.text)) return false;
    }
    return opt.text.startsWith(currentArg) || (opt.displayText?.startsWith(currentArg) ?? false);
  });
  const merged = [...staticMatches, ...generated];
  if (merged.length === 0) return null;
  return merged.map((item: CompletionItem) => ({
    text: item.text,
    type: item.type,
    matchPart: item.matchPart || currentArg,
    restPart: item.restPart || (item.text.startsWith(currentArg) ? item.text.slice(currentArg.length) : item.text),
    description: item.description,
    priority: item.priority ?? 50,
    displayText: item.displayText,
    usage: item.usage,
  }));
}

async function getShortcutSuggestions(input: string): Promise<Suggestion[]> {
  const list = await loadQuickCommands();
  const keyword = input.replace(/^\//, '').trim().toLowerCase();
  return list
    .filter((item) => !keyword || item.name.toLowerCase().startsWith(keyword) || item.command.toLowerCase().startsWith(keyword))
    .slice(0, 20)
    .map((item) => ({
      text: item.command,
      type: 'shortcut' as const,
      matchPart: keyword ? `/${keyword}` : '/',
      restPart: ` → ${item.command}`,
      description: item.name,
      usageCount: item.usage_count,
      quickCommandId: item.id,
      priority: 100,
    }));
}

async function generateSuggestions() {
  const rawInput = props.input || '';
  if (rawInput === lastProcessedInput && suggestions.value.length > 0) return;
  if (!rawInput || !props.visible) {
    suggestions.value = [];
    lastProcessedInput = '';
    return;
  }
  lastProcessedInput = rawInput;
  const requestId = ++currentRequestId;
  const words = rawInput.split(' ');
  const currentWord = currentWordOf(rawInput);

  const registrySuggestions = await buildRegistrySuggestions(rawInput, words, requestId);
  if (registrySuggestions && registrySuggestions.length > 0) {
    suggestions.value = registrySuggestions;
    selectedIndex.value = 0;
    hasUserSelected.value = false;
    return;
  }

  if (currentWord.startsWith('/')) {
    suggestions.value = await getShortcutSuggestions(currentWord);
    selectedIndex.value = 0;
    hasUserSelected.value = false;
    return;
  }

  const all: Suggestion[] = [];
  if (words.length === 1) {
    all.push(...commandHistory.value.filter((v) => v.startsWith(currentWord) && v !== currentWord).slice(0, 8).map((v) => ({
      text: v, type: 'history' as const, matchPart: currentWord, restPart: v.slice(currentWord.length), priority: 100,
    })));
    all.push(...commonCommands.value.filter((v) => v.startsWith(currentWord) && v !== currentWord).map((v) => ({
      text: v, type: 'command' as const, matchPart: currentWord, restPart: v.slice(currentWord.length), priority: 80,
    })));
    all.push(...registry.getAllCommandNames().filter((v) => v.startsWith(currentWord) && v !== currentWord).map((v) => ({
      text: v, type: 'command' as const, matchPart: currentWord, restPart: v.slice(currentWord.length), priority: 86,
    })));
    all.push(...(await loadQuickCommands()).filter((v) => v.command.startsWith(currentWord) && v.command !== currentWord).slice(0, 6).map((v) => ({
      text: v.command, type: 'snippet' as const, matchPart: currentWord, restPart: v.command.slice(rawInput.length), description: v.name, usageCount: v.usage_count, priority: 20, quickCommandId: v.id,
    })));
  }

  const prevWord = words.length > 1 ? words[words.length - 2] : '';
  const isPathTrigger = currentWord.includes('/') || currentWord.startsWith('.') || currentWord.startsWith('~');
  const isCommandExpectingPath = ['cd', 'ls', 'cat', 'rm', 'cp', 'mv', 'mkdir', 'touch', 'nano', 'vim', 'vi'].includes(prevWord);
  if (props.sessionId && (isPathTrigger || isCommandExpectingPath)) {
    const lastSlash = currentWord.lastIndexOf('/');
    const dirPath = lastSlash >= 0 ? currentWord.slice(0, lastSlash + 1) : './';
    const filePrefix = lastSlash >= 0 ? currentWord.slice(lastSlash + 1) : currentWord;
    const pathItems = await getRemotePathSuggestions(props.sessionId, dirPath + filePrefix, {
      foldersOnly: prevWord === 'cd',
      electronAPI: compatApi,
    }).catch(() => []);
    if (requestId === currentRequestId) {
      all.push(...pathItems.map((item) => ({
        text: item.text, type: 'path' as const, matchPart: item.matchPart || filePrefix, restPart: item.restPart || '', description: item.description, priority: item.priority ?? 90, displayText: item.displayText,
      })));
    }
  }

  if (requestId !== currentRequestId) return;
  all.sort((a, b) => (b.priority ?? 50) - (a.priority ?? 50));
  suggestions.value = Array.from(new Map(all.map((item) => [item.text, item])).values()).slice(0, 20);
  selectedIndex.value = 0;
  hasUserSelected.value = false;
}

async function selectSuggestion(suggestion: Suggestion) {
  let text = suggestion.text;
  if (suggestion.type === 'snippet' || suggestion.type === 'history') {
    const words = props.input.split(/\s+/);
    const lastWord = words[words.length - 1];
    const prefix = props.input.slice(0, props.input.length - lastWord.length);
    if (suggestion.text.startsWith(prefix)) {
      text = suggestion.text.slice(prefix.length);
    }
  }
  if (['command', 'subcommand', 'option'].includes(suggestion.type)) text += ' ';
  if (suggestion.type === 'path' && !text.endsWith('/')) text += ' ';
  emit('select', text);
  if (suggestion.quickCommandId) {
    void quickCommandApi.use(suggestion.quickCommandId).catch(() => undefined);
  }
  suggestions.value = [];
  hasUserSelected.value = false;
}

watch(() => props.input, (newInput, oldInput) => {
  if (debounceTimer) clearTimeout(debounceTimer);
  if (!newInput || !props.visible) {
    suggestions.value = [];
    selectedIndex.value = 0;
    hasUserSelected.value = false;
    lastProcessedInput = '';
    return;
  }
  if (newInput.trim().startsWith('/')) {
    lastProcessedInput = '';
    void generateSuggestions();
    return;
  }
  if (newInput === oldInput && suggestions.value.length > 0) return;
  debounceTimer = setTimeout(() => {
    if (props.visible && props.input === newInput) {
      lastProcessedInput = '';
      void generateSuggestions();
    }
  }, DEBOUNCE_DELAY);
});

watch(() => props.visible, (newVisible) => {
  if (newVisible) {
    if (props.input && props.input !== lastProcessedInput) {
      void nextTick(() => void generateSuggestions());
    }
    return;
  }
  if (debounceTimer) clearTimeout(debounceTimer);
  suggestions.value = [];
  selectedIndex.value = 0;
  hasUserSelected.value = false;
  lastProcessedInput = '';
  currentRequestId++;
});

onMounted(() => {
  void loadCommandHistory();
  void loadQuickCommands();
});

onUnmounted(() => {
  if (debounceTimer) clearTimeout(debounceTimer);
  suggestions.value = [];
  currentRequestId++;
});

defineExpose({
  selectNext: () => {
    if (suggestions.value.length === 0) return;
    selectedIndex.value = (selectedIndex.value + 1) % suggestions.value.length;
    hasUserSelected.value = true;
  },
  selectPrevious: () => {
    if (suggestions.value.length === 0) return;
    selectedIndex.value = selectedIndex.value === 0 ? suggestions.value.length - 1 : selectedIndex.value - 1;
    hasUserSelected.value = true;
  },
  selectCurrent: () => {
    const target = suggestions.value[selectedIndex.value];
    if (target) void selectSuggestion(target);
  },
  hasSuggestions: () => suggestions.value.length > 0,
  hasActiveSelection: () => hasUserSelected.value,
  resetSelection: () => {
    hasUserSelected.value = false;
    selectedIndex.value = 0;
  },
  forceReset: () => {
    if (debounceTimer) clearTimeout(debounceTimer);
    suggestions.value = [];
    selectedIndex.value = 0;
    hasUserSelected.value = false;
    lastProcessedInput = '';
    currentRequestId++;
  },
});
</script>

<style scoped>
.autocomplete-popup {
  position: fixed;
  z-index: 13000;
  min-width: 320px;
  max-width: 520px;
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-surface0);
  box-shadow: 0 16px 32px color-mix(in srgb, var(--bg-base) 75%, transparent);
  overflow: hidden;
}

.autocomplete-header {
  padding: 7px 10px;
  font-size: 11px;
  color: var(--text-sub);
  border-bottom: 1px solid var(--border);
  background: var(--bg-mantle);
}

.suggestions-list {
  max-height: 260px;
  overflow-y: auto;
}

.suggestion-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 10px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 62%, transparent);
  cursor: pointer;
}

.suggestion-item:last-child { border-bottom: none; }
.suggestion-item.active,
.suggestion-item:hover { background: var(--link-active-bg-color); }

.suggestion-icon {
  width: 28px;
  text-align: center;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-sub);
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 2px 0;
}

.suggestion-main { flex: 1; min-width: 0; }
.suggestion-text { font-family: Consolas, 'Courier New', monospace; font-size: 12px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.match-part { color: var(--blue); font-weight: 700; }
.rest-part { color: var(--text); }
.suggestion-meta { margin-top: 2px; font-size: 11px; color: var(--text-sub); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; display: flex; align-items: center; gap: 4px; }
.usage { color: var(--blue); }
.usage-count { font-size: 11px; color: var(--text-sub); }
</style>

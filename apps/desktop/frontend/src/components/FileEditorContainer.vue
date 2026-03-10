<template>
  <div class="file-editor-container">
    <FileEditorTabs />

    <div v-if="activeFile" class="editor-header">
      <span class="editor-file-meta" :title="activeFile.path">
        正在编辑({{ activeSessionName }}): {{ activeFile.path }}
        <span v-if="activeFile.isDirty" class="modified-indicator">*</span>
      </span>

      <div class="editor-actions">
        <div v-if="!activeFile.isLoading" class="encoding-select-wrapper">
          <AppSelect
            v-model="selectedEncodingValue"
            :options="encodingOptions"
            class="encoding-select"
            :style="{ width: encodingSelectWidth }"
            title="更改文件编码"
            aria-label="更改文件编码"
            @change="handleEncodingChange"
          />
        </div>
        <span v-else class="encoding-select-placeholder">加载中...</span>

        <span v-if="activeFile.saveStatus === 'saving'" class="save-status saving">保存中...</span>
        <span v-if="activeFile.saveStatus === 'success'" class="save-status success">✅ 已保存</span>
        <span v-if="activeFile.saveStatus === 'error'" class="save-status error">❌ 保存失败: {{ activeFile.saveError || '未知错误' }}</span>

        <button
          class="save-btn"
          :disabled="activeFile.isSaving || activeFile.isLoading || !!activeFile.loadingError || !activeFile.isDirty"
          @click="save"
        >
          保存
        </button>
      </div>
    </div>

    <div v-else class="editor-header editor-header-placeholder">
      <span>未打开文件</span>
    </div>

    <div class="editor-content-area" data-focus-id="fileEditorActive">
      <div v-if="activeFile?.isLoading" class="editor-loading">文件加载中...</div>
      <div v-else-if="activeFile?.loadingError" class="editor-error">{{ activeFile.loadingError }}</div>

      <MonacoEditor
        v-else-if="activeFile"
        ref="monacoEditorRef"
        :key="activeFile.id"
        :model-value="activeFile.content"
        :language="activeFile.language"
        :font-family="editorFontFamily"
        :font-size="editorFontSize"
        theme="vs-dark"
        class="editor-instance"
        :initial-scroll-top="activeFile.scrollTop ?? 0"
        :initial-scroll-left="activeFile.scrollLeft ?? 0"
        @update:modelValue="onContentChange"
        @request-save="save"
        @update:scrollPosition="handleEditorScroll"
        @update:fontSize="handleEditorFontSizeUpdate"
      />

      <div v-else class="editor-placeholder">请从文件管理器中选择文件以开始编辑</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Buffer } from 'buffer';
import * as iconv from '@vscode/iconv-lite-umd';
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { sftpApi } from '@/lib/api';
import MonacoEditor from './MonacoEditor.vue';
import FileEditorTabs from './FileEditorTabs.vue';
import AppSelect from './AppSelect.vue';
import { useAppearanceStore } from '@/stores/appearance';
import { useFileEditorStore } from '@/stores/fileEditor';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { useSessionStore } from '@/stores/session';
import { useUINotificationStore } from '@/stores/uiNotifications';

const store = useFileEditorStore();
const notify = useUINotificationStore();
const focusSwitcherStore = useFocusSwitcherStore();
const sessionStore = useSessionStore();
const appearanceStore = useAppearanceStore();
const { activeFile, fileList, activeFileId } = storeToRefs(store);

const monacoEditorRef = ref<{ focusEditor: () => boolean } | null>(null);
const encodingSelectWidth = ref('90px');

let unregisterFocusAction: (() => void) | null = null;
let clearSaveStatusTimer: number | null = null;

const encodingOptions = [
  { value: 'utf-8', label: 'UTF-8' },
  { value: 'utf-16le', label: 'UTF-16 LE' },
  { value: 'utf-16be', label: 'UTF-16 BE' },
  { value: 'gbk', label: 'GBK' },
  { value: 'gb2312', label: 'GB2312' },
  { value: 'gb18030', label: 'GB18030' },
  { value: 'big5', label: 'Big5 (繁体中文)' },
  { value: 'shift_jis', label: 'Shift-JIS' },
  { value: 'euc-jp', label: 'EUC-JP' },
  { value: 'euc-kr', label: 'EUC-KR' },
  { value: 'iso-8859-1', label: 'ISO-8859-1 (Latin-1)' },
  { value: 'iso-8859-2', label: 'ISO-8859-2 (Latin-2)' },
  { value: 'iso-8859-5', label: 'ISO-8859-5 (Cyrillic)' },
  { value: 'iso-8859-6', label: 'ISO-8859-6 (Arabic)' },
  { value: 'iso-8859-7', label: 'ISO-8859-7 (Greek)' },
  { value: 'iso-8859-8', label: 'ISO-8859-8 (Hebrew)' },
  { value: 'iso-8859-9', label: 'ISO-8859-9 (Turkish)' },
  { value: 'cp1250', label: 'Windows-1250' },
  { value: 'cp1251', label: 'Windows-1251' },
  { value: 'cp1252', label: 'Windows-1252' },
  { value: 'cp1253', label: 'Windows-1253' },
  { value: 'cp1254', label: 'Windows-1254' },
  { value: 'cp1255', label: 'Windows-1255' },
  { value: 'cp1256', label: 'Windows-1256' },
  { value: 'cp1257', label: 'Windows-1257' },
  { value: 'cp1258', label: 'Windows-1258' },
  { value: 'koi8-r', label: 'KOI8-R' },
  { value: 'koi8-u', label: 'KOI8-U' },
  { value: 'tis-620', label: 'TIS-620' },
  { value: 'cp874', label: 'Windows-874' },
];

const selectedEncodingValue = computed({
  get: () => activeFile.value?.selectedEncoding || 'utf-8',
  set: (value: string | number | null | undefined) => {
    handleEncodingChange(value);
  },
});

const activeSessionName = computed(() => {
  const file = activeFile.value;
  if (!file) {
    return '无会话';
  }

  return sessionStore.getSession(file.sessionId)?.connectionName ?? '未知会话';
});

const editorFontFamily = computed(() =>
  appearanceStore.get('editor_font_family', 'Consolas, "Courier New", monospace'),
);

const editorFontSize = computed(() => {
  const raw = Number.parseInt(appearanceStore.get('editor_font_size', '16'), 10);
  if (Number.isNaN(raw)) {
    return 16;
  }

  return Math.min(40, Math.max(8, raw));
});

function normalizeEncoding(encoding: string): string {
  return encoding.toLowerCase().replace(/[^a-z0-9]/g, '');
}

function decodeBase64Content(rawContentBase64: string, encoding: string): string {
  const normalized = normalizeEncoding(encoding);
  const binary = Buffer.from(rawContentBase64, 'base64');

  try {
    if (normalized === 'utf8') {
      return new TextDecoder('utf-8').decode(binary);
    }

    if (normalized === 'utf16le') {
      return new TextDecoder('utf-16le').decode(binary);
    }

    if (normalized === 'utf16be') {
      return new TextDecoder('utf-16be').decode(binary);
    }

    if (iconv.encodingExists(normalized)) {
      return iconv.decode(binary, normalized);
    }
  } catch (error) {
    console.warn('[FileEditor] Decode failed with selected encoding, fallback to UTF-8.', error);
  }

  return new TextDecoder('utf-8').decode(binary);
}

function encodeContentToBase64(content: string, encoding: string): string {
  const normalized = normalizeEncoding(encoding);

  try {
    if (iconv.encodingExists(normalized)) {
      const encoded = iconv.encode(content, normalized);
      return Buffer.from(encoded).toString('base64');
    }
  } catch (error) {
    console.warn('[FileEditor] Encode failed with selected encoding, fallback to UTF-8.', error);
  }

  return Buffer.from(content, 'utf-8').toString('base64');
}

function updateEncodingSelectWidth() {
  nextTick(() => {
    const selectedEncoding = activeFile.value?.selectedEncoding || 'utf-8';
    const selectedOption = encodingOptions.find(option => option.value === selectedEncoding);
    if (!selectedOption) {
      return;
    }

    const temp = document.createElement('span');
    temp.style.position = 'absolute';
    temp.style.left = '-9999px';
    temp.style.visibility = 'hidden';
    temp.style.fontFamily = 'inherit';
    temp.style.fontSize = '0.85em';
    temp.style.fontWeight = '400';
    temp.style.whiteSpace = 'nowrap';
    temp.textContent = selectedOption.label || 'UTF-8';
    document.body.appendChild(temp);

    const width = temp.getBoundingClientRect().width;
    document.body.removeChild(temp);
    encodingSelectWidth.value = `${Math.ceil(width + 52)}px`;
  });
}

function focusActiveEditor(): boolean | undefined {
  if (!activeFile.value) {
    return undefined;
  }

  return monacoEditorRef.value?.focusEditor() ?? false;
}

function onContentChange(value: string) {
  const file = activeFile.value;
  if (!file) {
    return;
  }

  store.updateContent(file.id, value);
}

function handleEditorScroll(position: { scrollTop: number; scrollLeft: number }) {
  const file = activeFile.value;
  if (!file) {
    return;
  }

  store.updateScrollPosition(file.id, position.scrollTop, position.scrollLeft);
}

async function handleEditorFontSizeUpdate(size: number) {
  try {
    await appearanceStore.set('editor_font_size', String(size));
  } catch {
    // ignore appearance save failure
  }
}

function handleEncodingChange(value: string | number | null | undefined) {
  const file = activeFile.value;
  if (!file) {
    return;
  }

  const nextEncoding = typeof value === 'string' ? value : String(value ?? '');
  if (!nextEncoding) {
    return;
  }

  if (nextEncoding === (file.selectedEncoding || 'utf-8')) {
    return;
  }

  if (!file.rawContentBase64) {
    notify.addNotification('warning', '缺少原始文件数据，无法切换编码');
    return;
  }

  try {
    const decoded = decodeBase64Content(file.rawContentBase64, nextEncoding);
    store.setDecodedContent(file.id, decoded, nextEncoding);
    updateEncodingSelectWidth();
  } catch (error: any) {
    notify.addNotification('error', `切换编码失败: ${error?.message ?? '未知错误'}`);
  }
}

function switchTabByOffset(offset: number) {
  if (fileList.value.length <= 1 || !activeFileId.value) {
    return;
  }

  const currentIndex = fileList.value.findIndex((item) => item.id === activeFileId.value);
  if (currentIndex < 0) {
    return;
  }

  const nextIndex = (currentIndex + offset + fileList.value.length) % fileList.value.length;
  if (nextIndex !== currentIndex) {
    store.setActive(fileList.value[nextIndex].id);
  }
}

function handleKeyDown(event: KeyboardEvent) {
  if (!event.altKey || (event.key !== 'ArrowLeft' && event.key !== 'ArrowRight')) {
    return;
  }

  event.preventDefault();
  event.stopPropagation();

  switchTabByOffset(event.key === 'ArrowLeft' ? -1 : 1);
}

async function save() {
  const file = activeFile.value;
  if (!file || !file.isDirty || file.isSaving) {
    return;
  }

  store.setSaveStatus(file.id, 'saving');

  try {
    const encoded = encodeContentToBase64(file.content, file.selectedEncoding || 'utf-8');
    await sftpApi.writeFile(file.sessionId, file.path, encoded);
    store.setRawContentBase64(file.id, encoded);
    store.markSaved(file.id);
    notify.addNotification('success', '文件已保存');

    if (clearSaveStatusTimer) {
      window.clearTimeout(clearSaveStatusTimer);
    }
    clearSaveStatusTimer = window.setTimeout(() => {
      store.clearSaveStatus(file.id);
      clearSaveStatusTimer = null;
    }, 1400);
  } catch (error: any) {
    const message = error?.message ?? '未知错误';
    store.setSaveStatus(file.id, 'error', message);
    notify.addNotification('error', `保存失败: ${message}`);
  }
}

watch(
  () => activeFile.value?.id,
  () => {
    updateEncodingSelectWidth();
  },
  { immediate: true },
);

watch(
  () => activeFile.value?.selectedEncoding,
  () => {
    updateEncodingSelectWidth();
  },
);

onMounted(() => {
  unregisterFocusAction = focusSwitcherStore.registerFocusAction('fileEditorActive', focusActiveEditor);
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  unregisterFocusAction?.();
  unregisterFocusAction = null;
  window.removeEventListener('keydown', handleKeyDown);

  if (clearSaveStatusTimer) {
    window.clearTimeout(clearSaveStatusTimer);
    clearSaveStatusTimer = null;
  }
});
</script>

<style scoped>
.file-editor-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: #2d2d2d;
  color: #f0f0f0;
  overflow: hidden;
}

.editor-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 12px;
  padding: 0.5rem 1rem;
  background-color: #333;
  border-bottom: 1px solid #555;
  font-size: calc(0.9em + var(--ui-font-size-offset));
  flex-shrink: 0;
}

.editor-header-placeholder {
  justify-content: flex-start;
  color: #888;
}

.editor-file-meta {
  flex: 1 1 auto;
  min-width: 0;
  overflow-wrap: anywhere;
  word-break: break-all;
  white-space: normal;
  line-height: 1.35;
  color: #f0f0f0;
}

.modified-indicator {
  color: #ffeb3b;
  margin-left: 4px;
  font-weight: bold;
}

.editor-actions {
  display: flex;
  align-items: center;
  gap: 0.8rem;
  flex-shrink: 0;
  align-self: flex-start;
}

.encoding-select-wrapper {
  display: inline-block;
  vertical-align: middle;
}

.encoding-select :deep(.app-select-trigger) {
  background-color: var(--bg-base);
  color: var(--text);
  border: 1px solid var(--border);
  min-height: 0;
  padding: 0.3rem 0.5rem;
  border-radius: 6px;
  font-size: calc(0.85em + var(--ui-font-size-offset));
}

.encoding-select :deep(.app-select-trigger:hover) {
  background-color: var(--bg-surface1);
}

.encoding-select :deep(.app-select-trigger:focus-visible) {
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.encoding-select-placeholder {
  font-size: calc(0.85em + var(--ui-font-size-offset));
  color: var(--text-dim);
  padding: 0.3rem 0.5rem;
  display: inline-block;
  min-width: 80px;
  text-align: center;
}

.save-btn {
  background-color: #4caf50;
  color: white;
  border: none;
  padding: 0.4rem 0.8rem;
  cursor: pointer;
  border-radius: 3px;
  font-size: calc(0.9em + var(--ui-font-size-offset));
}

.save-btn:disabled {
  background-color: #aaa;
  cursor: not-allowed;
}

.save-btn:hover:not(:disabled) {
  background-color: #45a049;
}

.save-status {
  font-size: calc(0.9em + var(--ui-font-size-offset));
  padding: 0.2rem 0.5rem;
  border-radius: 3px;
  white-space: nowrap;
}

.save-status.saving {
  color: #888;
}

.save-status.success {
  color: #4caf50;
  background-color: #e8f5e9;
}

.save-status.error {
  color: #f44336;
  background-color: #ffebee;
}

.editor-content-area {
  flex-grow: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  position: relative;
}

.editor-instance {
  flex-grow: 1;
  min-height: 0;
}

.editor-loading,
.editor-error,
.editor-placeholder {
  padding: 2rem;
  text-align: center;
  font-size: calc(1.1em + var(--ui-font-size-offset));
  flex-grow: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #888;
}

.editor-error {
  color: #ff8a8a;
}

.editor-placeholder {
  color: #666;
}

@media (max-width: 768px) {
  .editor-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .editor-actions {
    width: 100%;
    justify-content: flex-start;
  }
}
</style>

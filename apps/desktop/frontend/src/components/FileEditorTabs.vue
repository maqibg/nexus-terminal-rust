<template>
  <div class="file-editor-tabs">
    <div
      v-for="tab in fileList"
      :key="tab.id"
      class="tab-item"
      :class="{ active: tab.id === activeFileId }"
      :title="tab.path"
      @click="store.setActive(tab.id)"
      @contextmenu.prevent="showContextMenu($event, tab.id)"
    >
      <span class="tab-filename">{{ tab.filename || fileName(tab.path) }}</span>
      <span v-if="tab.isDirty" class="modified-indicator">*</span>
      <button class="close-tab-btn" title="关闭标签页" @click.stop="requestClose(tab.id)">×</button>
    </div>

    <div v-if="!fileList.length" class="no-tabs-placeholder"></div>

    <Teleport to="body">
      <div v-if="contextMenuVisible" class="context-backdrop" @mousedown="closeContextMenu"></div>
      <div
        v-if="contextMenuVisible"
        ref="contextMenuRef"
        class="context-menu"
        :style="{ left: `${contextMenuPosition.x}px`, top: `${contextMenuPosition.y}px` }"
        @mousedown.stop
      >
        <button class="context-item" @click="handleContextAction('close')">关闭</button>
        <button class="context-item" :disabled="!canCloseOthers" @click="handleContextAction('close-others')">关闭其他</button>
        <button class="context-item" :disabled="!canCloseRight" @click="handleContextAction('close-right')">关闭右侧</button>
        <button class="context-item" :disabled="!canCloseLeft" @click="handleContextAction('close-left')">关闭左侧</button>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useFileEditorStore } from '@/stores/fileEditor';
import { useConfirmDialog } from '@/composables/useConfirmDialog';

const store = useFileEditorStore();
const { confirm } = useConfirmDialog();
const { fileList, activeFileId } = storeToRefs(store);

const contextMenuVisible = ref(false);
const contextMenuTargetId = ref<string | null>(null);
const contextMenuRef = ref<HTMLDivElement>();
const contextMenuPosition = reactive({ x: 0, y: 0 });

const targetTabIndex = computed(() =>
  fileList.value.findIndex((tab) => tab.id === contextMenuTargetId.value),
);

const canCloseOthers = computed(() => fileList.value.length > 1 && targetTabIndex.value >= 0);
const canCloseRight = computed(() => {
  const index = targetTabIndex.value;
  return index >= 0 && index < fileList.value.length - 1;
});
const canCloseLeft = computed(() => targetTabIndex.value > 0);

function fileName(path: string) {
  return path.split('/').pop() ?? path;
}

function showContextMenu(event: MouseEvent, tabId: string) {
  contextMenuTargetId.value = tabId;
  contextMenuPosition.x = event.clientX;
  contextMenuPosition.y = event.clientY;
  contextMenuVisible.value = true;
  void nextTick(adjustContextMenuPosition);
}

function adjustContextMenuPosition() {
  if (!contextMenuRef.value) {
    return;
  }

  const rect = contextMenuRef.value.getBoundingClientRect();
  if (contextMenuPosition.x + rect.width > window.innerWidth - 8) {
    contextMenuPosition.x = Math.max(8, window.innerWidth - rect.width - 8);
  }
  if (contextMenuPosition.y + rect.height > window.innerHeight - 8) {
    contextMenuPosition.y = Math.max(8, window.innerHeight - rect.height - 8);
  }
}

function closeContextMenu() {
  contextMenuVisible.value = false;
  contextMenuTargetId.value = null;
}

function dirtyCount(ids: string[]): number {
  return ids.filter((id) => fileList.value.find((item) => item.id === id)?.isDirty).length;
}

/** Closing a tab drops its buffer, so ask before throwing away unsaved edits. */
async function confirmDiscard(ids: string[], label: string): Promise<boolean> {
  const dirty = dirtyCount(ids);
  if (dirty === 0) {
    return true;
  }
  return confirm('未保存的修改', `${label}包含 ${dirty} 个未保存的文件，关闭后会丢失。`);
}

async function requestClose(id: string) {
  const tab = fileList.value.find((item) => item.id === id);
  const label = `「${tab?.filename ?? tab?.path ?? id}」`;
  if (!(await confirmDiscard([id], label))) {
    return;
  }
  store.closeFile(id);
}

async function handleContextAction(action: 'close' | 'close-others' | 'close-right' | 'close-left') {
  const targetId = contextMenuTargetId.value;
  if (!targetId) {
    closeContextMenu();
    return;
  }

  const index = fileList.value.findIndex((item) => item.id === targetId);
  closeContextMenu();

  switch (action) {
    case 'close':
      await requestClose(targetId);
      return;
    case 'close-others': {
      if (!canCloseOthers.value) {
        return;
      }
      const others = fileList.value.filter((item) => item.id !== targetId).map((item) => item.id);
      if (await confirmDiscard(others, '关闭其他标签')) {
        store.closeOtherFiles(targetId);
      }
      return;
    }
    case 'close-right': {
      if (!canCloseRight.value) {
        return;
      }
      const right = fileList.value.slice(index + 1).map((item) => item.id);
      if (await confirmDiscard(right, '关闭右侧标签')) {
        store.closeFilesToRight(targetId);
      }
      return;
    }
    case 'close-left': {
      if (!canCloseLeft.value) {
        return;
      }
      const left = fileList.value.slice(0, index).map((item) => item.id);
      if (await confirmDiscard(left, '关闭左侧标签')) {
        store.closeFilesToLeft(targetId);
      }
      return;
    }
    default:
      break;
  }
}
</script>

<style scoped>
.file-editor-tabs {
  position: relative;
  display: flex;
  flex-wrap: nowrap;
  overflow-x: auto;
  overflow-y: hidden;
  background: #252526;
  border-bottom: 1px solid #3f3f46;
  flex-shrink: 0;
}

.file-editor-tabs::-webkit-scrollbar {
  height: 4px;
}

.file-editor-tabs::-webkit-scrollbar-track {
  background: #252526;
}

.file-editor-tabs::-webkit-scrollbar-thumb {
  background: #555;
  border-radius: 3px;
}

.tab-item {
  display: inline-flex;
  align-items: center;
  padding: 6px 10px 6px 12px;
  border-right: 1px solid #3f3f46;
  color: #cccccc;
  background: #2d2d2d;
  white-space: nowrap;
  font-size: calc(12px + var(--ui-font-size-offset));
  cursor: pointer;
  transition: background-color 0.12s;
}

.tab-item:hover {
  background: #3e3e42;
}

.tab-item.active {
  background: #1e1e1e;
  color: #ffffff;
  border-bottom: 1px solid #1e1e1e;
  margin-bottom: -1px;
}

.tab-filename {
  max-width: 150px;
  overflow: hidden;
  text-overflow: ellipsis;
}

.modified-indicator {
  margin-left: 4px;
  color: #cccccc;
}

.tab-item.active .modified-indicator {
  color: #ffffff;
}

.close-tab-btn {
  border: none;
  background: transparent;
  color: inherit;
  font-size: calc(14px + var(--ui-font-size-offset));
  line-height: 1;
  margin-left: 6px;
  padding: 0 4px;
  border-radius: 4px;
  opacity: 0.7;
  cursor: pointer;
}

.close-tab-btn:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.12);
}

.no-tabs-placeholder {
  flex: 1;
}

.context-backdrop {
  position: fixed;
  inset: 0;
  z-index: 3490;
}

.context-menu {
  position: fixed;
  z-index: 3500;
  min-width: 152px;
  padding: 4px;
  border-radius: 8px;
  border: 1px solid var(--border, #45475a);
  background: var(--bg-surface0, #313244);
  box-shadow: 0 12px 28px rgba(0, 0, 0, 0.48);
}

.context-item {
  width: 100%;
  height: 30px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text, #cdd6f4);
  text-align: left;
  padding: 0 10px;
  font-size: calc(12px + var(--ui-font-size-offset));
  cursor: pointer;
}

.context-item:hover:not(:disabled) {
  background: rgba(137, 180, 250, 0.14);
}

.context-item:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}
</style>
<template>
  <div class="file-editor-container">
    <FileEditorTabs />
    <div class="editor-toolbar" v-if="activeFile">
      <span class="file-path">{{ activeFile.path }}</span>
      <div class="toolbar-actions">
        <button class="btn" @click="save" :disabled="!activeFile.isDirty">保存</button>
        <button class="btn" @click="store.closeFile(activeFile.id)">关闭</button>
      </div>
    </div>
    <div class="editor-body">
      <MonacoEditor
        v-if="activeFile"
        :modelValue="activeFile.content"
        :language="activeFile.language"
        @update:modelValue="onContentChange"
      />
      <div v-else class="empty-state">无打开的文件</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useFileEditorStore } from '@/stores/fileEditor';
import { useUINotificationStore } from '@/stores/uiNotifications';
import { sftpApi } from '@/lib/api';
import FileEditorTabs from './FileEditorTabs.vue';
import MonacoEditor from './MonacoEditor.vue';

const store = useFileEditorStore();
const notify = useUINotificationStore();
const { activeFile } = storeToRefs(store);

function onContentChange(value: string) {
  if (activeFile.value) store.updateContent(activeFile.value.id, value);
}

async function save() {
  const f = activeFile.value;
  if (!f || !f.isDirty) return;
  try {
    const encoded = btoa(unescape(encodeURIComponent(f.content)));
    await sftpApi.writeFile(f.sessionId, f.path, encoded);
    store.markSaved(f.id);
    notify.addNotification('success', '文件已保存');
  } catch (e: any) {
    notify.addNotification('error', `保存失败: ${e.message}`);
  }
}
</script>

<style scoped>
.file-editor-container { display: flex; flex-direction: column; height: 100%; }
.editor-toolbar { display: flex; align-items: center; justify-content: space-between; padding: 4px 8px; background: var(--bg-surface0); border-bottom: 1px solid var(--border); }
.file-path { font-size: 12px; color: var(--text-dim); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.toolbar-actions { display: flex; gap: 4px; }
.btn { padding: 3px 10px; border-radius: 4px; border: 1px solid var(--border); background: var(--bg-surface1); color: var(--text); cursor: pointer; font-size: 12px; }
.btn:hover { background: var(--blue); color: var(--bg-base); }
.btn:disabled { opacity: 0.4; cursor: default; }
.editor-body { flex: 1; overflow: hidden; }
.empty-state { display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-dim); font-size: 13px; }
</style>

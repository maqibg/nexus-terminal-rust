<template>
  <div class="file-editor-tabs">
    <div
      v-for="f in fileList"
      :key="f.id"
      class="tab"
      :class="{ active: f.id === activeFileId }"
      @click="store.setActive(f.id)"
    >
      <span class="tab-name">{{ fileName(f.path) }}</span>
      <span v-if="f.isDirty" class="dirty-dot"></span>
      <span class="tab-close" @click.stop="store.closeFile(f.id)">×</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useFileEditorStore } from '@/stores/fileEditor';

const store = useFileEditorStore();
const { fileList, activeFileId } = storeToRefs(store);

function fileName(path: string) {
  return path.split('/').pop() ?? path;
}
</script>

<style scoped>
.file-editor-tabs { display: flex; gap: 2px; background: var(--bg-mantle); padding: 4px 4px 0; overflow-x: auto; }
.tab { display: flex; align-items: center; gap: 4px; padding: 4px 10px; font-size: 12px; color: var(--text-dim); cursor: pointer; border-radius: 4px 4px 0 0; white-space: nowrap; }
.tab:hover { color: var(--text-sub); background: var(--bg-surface0); }
.tab.active { color: var(--text); background: var(--bg-base); }
.dirty-dot { width: 6px; height: 6px; border-radius: 50%; background: var(--yellow); }
.tab-close { font-size: 14px; opacity: 0.5; }
.tab-close:hover { opacity: 1; color: var(--red); }
</style>

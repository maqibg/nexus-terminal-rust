import { defineStore } from 'pinia';
import { ref, computed } from 'vue';

export interface FileTab {
  id: string;
  sessionId: string;
  path: string;
  content: string;
  originalContent: string;
  isDirty: boolean;
  language: string;
}

export const useFileEditorStore = defineStore('fileEditor', () => {
  const openFiles = ref<Map<string, FileTab>>(new Map());
  const activeFileId = ref<string | null>(null);

  const activeFile = computed(() =>
    activeFileId.value ? openFiles.value.get(activeFileId.value) : undefined
  );

  const fileList = computed(() => Array.from(openFiles.value.values()));

  function openFile(tab: FileTab) {
    openFiles.value.set(tab.id, tab);
    activeFileId.value = tab.id;
  }

  function closeFile(id: string) {
    openFiles.value.delete(id);
    if (activeFileId.value === id) {
      const remaining = Array.from(openFiles.value.keys());
      activeFileId.value = remaining.length > 0 ? remaining[remaining.length - 1] : null;
    }
  }

  function setActive(id: string) {
    if (openFiles.value.has(id)) activeFileId.value = id;
  }

  function updateContent(id: string, content: string) {
    const tab = openFiles.value.get(id);
    if (tab) {
      tab.content = content;
      tab.isDirty = content !== tab.originalContent;
    }
  }

  function markSaved(id: string) {
    const tab = openFiles.value.get(id);
    if (tab) {
      tab.originalContent = tab.content;
      tab.isDirty = false;
    }
  }

  return { openFiles, activeFileId, activeFile, fileList, openFile, closeFile, setActive, updateContent, markSaved };
});

import { defineStore } from 'pinia';
import { computed, ref } from 'vue';

export type FileSaveStatus = 'idle' | 'saving' | 'success' | 'error';

export interface FileTab {
  id: string;
  sessionId: string;
  path: string;
  filename?: string;
  content: string;
  originalContent: string;
  isDirty: boolean;
  language: string;
  rawContentBase64?: string | null;
  selectedEncoding?: string;
  isLoading?: boolean;
  loadingError?: string | null;
  isSaving?: boolean;
  saveStatus?: FileSaveStatus;
  saveError?: string | null;
  scrollTop?: number;
  scrollLeft?: number;
}

export const useFileEditorStore = defineStore('fileEditor', () => {
  const openFiles = ref<Map<string, FileTab>>(new Map());
  const activeFileId = ref<string | null>(null);

  const activeFile = computed(() =>
    activeFileId.value ? openFiles.value.get(activeFileId.value) : undefined,
  );

  const fileList = computed(() => Array.from(openFiles.value.values()));

  function normalizeTab(tab: FileTab): FileTab {
    const filename = tab.filename || tab.path.split('/').pop() || tab.path;
    return {
      ...tab,
      filename,
      rawContentBase64: tab.rawContentBase64 ?? null,
      selectedEncoding: tab.selectedEncoding ?? 'utf-8',
      isLoading: tab.isLoading ?? false,
      loadingError: tab.loadingError ?? null,
      isSaving: tab.isSaving ?? false,
      saveStatus: tab.saveStatus ?? 'idle',
      saveError: tab.saveError ?? null,
      scrollTop: tab.scrollTop ?? 0,
      scrollLeft: tab.scrollLeft ?? 0,
      isDirty: tab.isDirty ?? tab.content !== tab.originalContent,
    };
  }

  function ensureActiveFile() {
    if (activeFileId.value && openFiles.value.has(activeFileId.value)) {
      return;
    }

    const remaining = Array.from(openFiles.value.keys());
    activeFileId.value = remaining.length ? remaining[remaining.length - 1] : null;
  }

  function openFile(tab: FileTab) {
    if (openFiles.value.has(tab.id)) {
      activeFileId.value = tab.id;
      return;
    }

    openFiles.value.set(tab.id, normalizeTab(tab));
    activeFileId.value = tab.id;
  }

  function closeFile(id: string) {
    openFiles.value.delete(id);
    if (activeFileId.value === id) {
      ensureActiveFile();
    }
  }

  function closeOtherFiles(id: string) {
    const target = openFiles.value.get(id);
    if (!target) {
      return;
    }

    openFiles.value = new Map([[id, target]]);
    activeFileId.value = id;
  }

  function closeFilesToRight(id: string) {
    const items = fileList.value;
    const index = items.findIndex((item) => item.id === id);
    if (index < 0 || index >= items.length - 1) {
      return;
    }

    for (let i = index + 1; i < items.length; i += 1) {
      openFiles.value.delete(items[i].id);
    }

    ensureActiveFile();
  }

  function closeFilesToLeft(id: string) {
    const items = fileList.value;
    const index = items.findIndex((item) => item.id === id);
    if (index <= 0) {
      return;
    }

    for (let i = 0; i < index; i += 1) {
      openFiles.value.delete(items[i].id);
    }

    ensureActiveFile();
  }

  function setActive(id: string) {
    if (openFiles.value.has(id)) {
      activeFileId.value = id;
    }
  }

  function patchFile(id: string, patch: Partial<FileTab>) {
    const tab = openFiles.value.get(id);
    if (!tab) {
      return;
    }

    openFiles.value.set(id, {
      ...tab,
      ...patch,
    });
  }

  function updateContent(id: string, content: string) {
    const tab = openFiles.value.get(id);
    if (!tab) {
      return;
    }

    patchFile(id, {
      content,
      isDirty: content !== tab.originalContent,
      saveStatus: tab.saveStatus === 'error' ? 'idle' : tab.saveStatus,
      saveError: null,
    });
  }

  function setDecodedContent(id: string, content: string, encoding: string) {
    const tab = openFiles.value.get(id);
    if (!tab) {
      return;
    }

    const wasDirty = tab.isDirty;

    patchFile(id, {
      content,
      originalContent: wasDirty ? tab.originalContent : content,
      selectedEncoding: encoding,
      isDirty: wasDirty ? content !== tab.originalContent : false,
      loadingError: null,
      saveError: null,
      saveStatus: tab.saveStatus === 'error' ? 'idle' : tab.saveStatus,
    });
  }

  function setRawContentBase64(id: string, rawContentBase64: string | null) {
    patchFile(id, { rawContentBase64 });
  }

  function setSaveStatus(id: string, saveStatus: FileSaveStatus, saveError: string | null = null) {
    patchFile(id, {
      isSaving: saveStatus === 'saving',
      saveStatus,
      saveError,
    });
  }

  function clearSaveStatus(id: string) {
    const tab = openFiles.value.get(id);
    if (!tab || tab.saveStatus === 'saving') {
      return;
    }

    patchFile(id, {
      saveStatus: 'idle',
      saveError: null,
    });
  }

  function markSaved(id: string) {
    const tab = openFiles.value.get(id);
    if (!tab) {
      return;
    }

    patchFile(id, {
      originalContent: tab.content,
      isDirty: false,
      isSaving: false,
      saveStatus: 'success',
      saveError: null,
    });
  }

  function setLoadingState(id: string, isLoading: boolean, loadingError: string | null = null) {
    patchFile(id, {
      isLoading,
      loadingError,
    });
  }

  function updateScrollPosition(id: string, scrollTop: number, scrollLeft: number) {
    patchFile(id, {
      scrollTop,
      scrollLeft,
    });
  }

  return {
    openFiles,
    activeFileId,
    activeFile,
    fileList,
    openFile,
    closeFile,
    closeOtherFiles,
    closeFilesToRight,
    closeFilesToLeft,
    setActive,
    updateContent,
    setDecodedContent,
    setRawContentBase64,
    setSaveStatus,
    clearSaveStatus,
    markSaved,
    setLoadingState,
    updateScrollPosition,
  };
});
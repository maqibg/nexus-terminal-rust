import { onMounted, onUnmounted, ref, type Ref } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { DragDropEvent as TauriDragDropEvent } from '@tauri-apps/api/webview';
import type { FileEntry } from '@/lib/api-sftp';

const SCROLL_ZONE_HEIGHT = 56;
const SCROLL_STEP = 14;
const SCROLL_INTERVAL_MS = 24;

type DropRowTarget = FileEntry | '..';

interface UseSftpBrowserDragAndDropOptions {
  isConnected: Ref<boolean>;
  currentPath: Ref<string>;
  selectedEntryPaths: Ref<Set<string>>;
  fileListContainerRef: Ref<HTMLDivElement | null>;
  getParentPath: (path: string) => string;
  uploadLocalPaths: (paths: string[]) => Promise<void>;
  moveEntriesToDirectory: (sourcePaths: string[], targetDirectory: string) => Promise<void>;
}

function hasFileDrag(event: DragEvent): boolean {
  return event.dataTransfer?.types.includes('Files') ?? false;
}

function rowKey(target: DropRowTarget): string {
  return target === '..' ? '..' : target.path;
}

function extractDroppedPaths(event: DragEvent): string[] {
  const paths = new Set<string>();
  const files = event.dataTransfer?.files ? Array.from(event.dataTransfer.files) : [];
  for (const file of files) {
    const candidate = (file as File & { path?: string }).path;
    if (candidate) {
      paths.add(candidate);
    }
  }

  const items = event.dataTransfer?.items ? Array.from(event.dataTransfer.items) : [];
  for (const item of items) {
    const file = item.getAsFile() as (File & { path?: string }) | null;
    if (file?.path) {
      paths.add(file.path);
    }
  }

  return [...paths];
}

export function useSftpBrowserDragAndDrop(options: UseSftpBrowserDragAndDropOptions) {
  const showExternalDropOverlay = ref(false);
  const dragOverTarget = ref<string | null>(null);
  const draggedEntryPath = ref<string | null>(null);
  const dragDepth = ref(0);
  const scrollIntervalId = ref<number | null>(null);
  let unlistenWindowDragDrop: (() => void) | null = null;

  function isPointInFileList(position: { x: number; y: number }): boolean {
    const rect = options.fileListContainerRef.value?.getBoundingClientRect();
    return !!rect
      && position.x >= rect.left
      && position.x <= rect.right
      && position.y >= rect.top
      && position.y <= rect.bottom;
  }

  function stopAutoScroll(): void {
    if (scrollIntervalId.value == null) {
      return;
    }
    window.clearInterval(scrollIntervalId.value);
    scrollIntervalId.value = null;
  }

  function maybeAutoScroll(clientY: number): void {
    const container = options.fileListContainerRef.value;
    if (!container || !dragOverTarget.value) {
      stopAutoScroll();
      return;
    }

    const rect = container.getBoundingClientRect();
    const offsetY = clientY - rect.top;
    let delta = 0;
    if (offsetY < SCROLL_ZONE_HEIGHT) {
      delta = -SCROLL_STEP;
    } else if (offsetY > rect.height - SCROLL_ZONE_HEIGHT) {
      delta = SCROLL_STEP;
    }

    if (!delta) {
      stopAutoScroll();
      return;
    }

    if (scrollIntervalId.value != null) {
      return;
    }

    scrollIntervalId.value = window.setInterval(() => {
      container.scrollTop += delta;
      if (
        (delta < 0 && container.scrollTop <= 0)
        || (delta > 0 && container.scrollTop >= container.scrollHeight - container.clientHeight)
      ) {
        stopAutoScroll();
      }
    }, SCROLL_INTERVAL_MS);
  }

  function resetExternalDrop(): void {
    dragDepth.value = 0;
    showExternalDropOverlay.value = false;
  }

  function resetAllDragState(): void {
    resetExternalDrop();
    dragOverTarget.value = null;
    draggedEntryPath.value = null;
    stopAutoScroll();
  }

  function isValidInternalTarget(target: DropRowTarget): boolean {
    if (!draggedEntryPath.value) {
      return false;
    }
    if (target === '..') {
      return options.currentPath.value !== '/';
    }
    return target.is_dir && target.path !== draggedEntryPath.value;
  }

  function resolveTargetDirectory(target: DropRowTarget): string {
    return target === '..'
      ? options.getParentPath(options.currentPath.value)
      : target.path;
  }

  function handleDragEnter(event: DragEvent): void {
    if (draggedEntryPath.value) {
      return;
    }
    if (!options.isConnected.value || !hasFileDrag(event)) {
      return;
    }
    dragDepth.value += 1;
    showExternalDropOverlay.value = true;
  }

  function handleDragOver(event: DragEvent): void {
    if (draggedEntryPath.value) {
      event.preventDefault();
      const targetElement = event.target instanceof Element ? event.target : null;
      const targetRow = targetElement?.closest('[data-drop-row]');
      if (!targetRow) {
        dragOverTarget.value = null;
      }
      maybeAutoScroll(event.clientY);
      if (event.dataTransfer) {
        event.dataTransfer.dropEffect = dragOverTarget.value ? 'move' : 'none';
      }
      return;
    }

    if (!options.isConnected.value || !hasFileDrag(event)) {
      return;
    }
    event.preventDefault();
    showExternalDropOverlay.value = true;
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = 'copy';
    }
  }

  function handleDragLeave(event: DragEvent): void {
    const currentTarget = event.currentTarget as Node | null;
    const relatedTarget = event.relatedTarget as Node | null;
    if (currentTarget && relatedTarget && currentTarget.contains(relatedTarget)) {
      return;
    }

    if (!draggedEntryPath.value) {
      dragDepth.value = Math.max(0, dragDepth.value - 1);
      if (dragDepth.value === 0) {
        showExternalDropOverlay.value = false;
      }
      return;
    }

    const relatedRow = relatedTarget instanceof HTMLElement
      ? relatedTarget.closest('[data-drop-row]')
      : null;
    if (!relatedRow) {
      dragOverTarget.value = null;
      stopAutoScroll();
    }
  }

  function handleDrop(event: DragEvent): void {
    event.preventDefault();
    resetAllDragState();
  }

  async function handleOverlayDrop(event: DragEvent): Promise<void> {
    event.preventDefault();
    const paths = extractDroppedPaths(event);
    resetAllDragState();
    if (!paths.length) {
      return;
    }
    await options.uploadLocalPaths(paths);
  }

  function handleDragStart(entry: FileEntry): void {
    draggedEntryPath.value = entry.path;
  }

  function handleDragEnd(): void {
    resetAllDragState();
  }

  function handleDragOverRow(target: DropRowTarget, event: DragEvent): void {
    event.preventDefault();
    event.stopPropagation();

    const allowed = isValidInternalTarget(target);
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = allowed ? 'move' : 'none';
    }
    if (!allowed) {
      dragOverTarget.value = null;
      stopAutoScroll();
      return;
    }

    dragOverTarget.value = rowKey(target);
    maybeAutoScroll(event.clientY);
  }

  function handleDragLeaveRow(target: DropRowTarget): void {
    if (dragOverTarget.value === rowKey(target)) {
      dragOverTarget.value = null;
      stopAutoScroll();
    }
  }

  async function handleDropOnRow(target: DropRowTarget, event: DragEvent): Promise<void> {
    event.preventDefault();
    event.stopPropagation();

    const sourcePath = draggedEntryPath.value;
    const allowed = isValidInternalTarget(target);
    resetAllDragState();
    if (!sourcePath || !allowed) {
      return;
    }

    const sourcePaths = options.selectedEntryPaths.value.has(sourcePath)
      ? Array.from(options.selectedEntryPaths.value)
      : [sourcePath];
    await options.moveEntriesToDirectory(sourcePaths, resolveTargetDirectory(target));
  }

  async function handleWindowDragDropEvent(event: TauriDragDropEvent): Promise<void> {
    if (!options.isConnected.value) {
      resetAllDragState();
      return;
    }

    if (event.type === 'leave') {
      resetExternalDrop();
      return;
    }

    if (event.type === 'enter' || event.type === 'over') {
      showExternalDropOverlay.value = isPointInFileList(event.position);
      return;
    }

    if (event.type !== 'drop') {
      return;
    }

    const shouldHandle = isPointInFileList(event.position);
    const droppedPaths = event.paths ?? [];
    resetAllDragState();
    if (!shouldHandle || !droppedPaths.length) {
      return;
    }
    await options.uploadLocalPaths(droppedPaths);
  }

  onMounted(() => {
    void getCurrentWindow()
      .onDragDropEvent((event) => {
        void handleWindowDragDropEvent(event.payload);
      })
      .then((unlisten) => {
        unlistenWindowDragDrop = unlisten;
      });
  });

  onUnmounted(() => {
    unlistenWindowDragDrop?.();
    unlistenWindowDragDrop = null;
    resetAllDragState();
  });

  return {
    showExternalDropOverlay,
    dragOverTarget,
    handleDragEnter,
    handleDragOver,
    handleDragLeave,
    handleDrop,
    handleOverlayDrop,
    handleDragStart,
    handleDragEnd,
    handleDragOverRow,
    handleDragLeaveRow,
    handleDropOnRow,
  };
}

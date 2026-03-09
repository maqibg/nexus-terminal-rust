import { computed, onUnmounted, ref } from 'vue';

export type SftpColumnKey = 'type' | 'name' | 'size' | 'permissions' | 'modified';
export type SftpColumnWidths = Record<SftpColumnKey, number>;

const STORAGE_KEY = 'sftp_file_list_columns_v2';
const MIN_WIDTHS: SftpColumnWidths = {
  type: 52,
  name: 220,
  size: 96,
  permissions: 92,
  modified: 164,
};
const DEFAULT_WIDTHS: SftpColumnWidths = {
  type: 56,
  name: 360,
  size: 116,
  permissions: 108,
  modified: 188,
};

function readWidths(): SftpColumnWidths {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return { ...DEFAULT_WIDTHS };
  }

  try {
    const parsed = JSON.parse(raw) as Partial<SftpColumnWidths>;
    return {
      type: Math.max(MIN_WIDTHS.type, parsed.type ?? DEFAULT_WIDTHS.type),
      name: Math.max(MIN_WIDTHS.name, parsed.name ?? DEFAULT_WIDTHS.name),
      size: Math.max(MIN_WIDTHS.size, parsed.size ?? DEFAULT_WIDTHS.size),
      permissions: Math.max(MIN_WIDTHS.permissions, parsed.permissions ?? DEFAULT_WIDTHS.permissions),
      modified: Math.max(MIN_WIDTHS.modified, parsed.modified ?? DEFAULT_WIDTHS.modified),
    };
  } catch {
    return { ...DEFAULT_WIDTHS };
  }
}

export function useSftpFileListColumns() {
  const columnWidths = ref<SftpColumnWidths>(readWidths());
  const resizingKey = ref<SftpColumnKey | null>(null);
  const startX = ref(0);
  const startWidth = ref(0);

  const gridTemplate = computed(() =>
    `${columnWidths.value.type}px minmax(${columnWidths.value.name}px, 1fr) ${columnWidths.value.size}px ${columnWidths.value.permissions}px ${columnWidths.value.modified}px`,
  );

  function persistWidths(): void {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(columnWidths.value));
  }

  function handleResize(event: MouseEvent): void {
    if (!resizingKey.value) {
      return;
    }

    const nextWidth = Math.max(
      MIN_WIDTHS[resizingKey.value],
      startWidth.value + event.clientX - startX.value,
    );
    columnWidths.value = {
      ...columnWidths.value,
      [resizingKey.value]: nextWidth,
    };
  }

  function stopResize(): void {
    if (!resizingKey.value) {
      return;
    }

    resizingKey.value = null;
    document.removeEventListener('mousemove', handleResize);
    document.removeEventListener('mouseup', stopResize);
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    persistWidths();
  }

  function startResize(event: MouseEvent, key: SftpColumnKey): void {
    event.preventDefault();
    event.stopPropagation();
    resizingKey.value = key;
    startX.value = event.clientX;
    startWidth.value = columnWidths.value[key];
    document.addEventListener('mousemove', handleResize);
    document.addEventListener('mouseup', stopResize);
    document.body.style.cursor = 'col-resize';
    document.body.style.userSelect = 'none';
  }

  onUnmounted(() => {
    stopResize();
  });

  return {
    columnWidths,
    gridTemplate,
    startResize,
  };
}

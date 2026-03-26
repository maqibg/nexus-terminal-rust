import { computed, nextTick, onMounted, onUnmounted, ref, watch, type ComputedRef, type Ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { pathHistoryApi, sftpApi, sshApi, statusApi, type FileEntry, type PathHistory } from '@/lib/api';
import { save } from '@tauri-apps/plugin-dialog';
import { toAppError } from '@/lib/errors';
import { createUploadTasksFromLocalPaths } from '@/lib/local-upload';
import { useFileEditorStore } from '@/stores/fileEditor';
import { useUINotificationStore } from '@/stores/uiNotifications';
import { useSessionStore } from '@/stores/session';
import { useSettingsStore } from '@/stores/settings';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { useConfirmDialog } from './useConfirmDialog';
import { usePromptDialog } from './usePromptDialog';

const VIEWPORT_PADDING = 8;
const CONTEXT_SUBMENU_MIN_WIDTH = 190;
const SUPPORTED_ARCHIVE_EXTENSIONS = ['.zip', '.tar.gz', '.tgz', '.tar.bz2', '.tbz2'];

function decodeUtf8Base64(base64: string): string {
  const bytes = Uint8Array.from(atob(base64), (char) => char.charCodeAt(0));
  return new TextDecoder('utf-8').decode(bytes);
}

type SftpSortKey = 'type' | 'name' | 'size' | 'modified';
type SftpSortDirection = 'asc' | 'desc';

export interface BrowserContextMenuItem {
  key: string;
  label?: string;
  icon?: string;
  danger?: boolean;
  disabled?: boolean;
  separator?: boolean;
  submenu?: BrowserContextMenuItem[];
  onClick?: () => void | Promise<void>;
}

export interface BrowserClipboardState {
  operation: 'copy' | 'cut';
  sessionId: string;
  entries: FileEntry[];
}

interface TransferStatusEvent {
  task_id: string;
  file_name?: string;
  kind?: 'upload' | 'download';
  status: 'active' | 'completed' | 'failed' | 'cancelled';
  error?: string | null;
}

export function useSftpBrowser(
  sftpSessionId: ComputedRef<string | null>,
  sshSessionId: Ref<string | null>,
  connectionId: ComputedRef<number | undefined>,
) {
  const sessionStore = useSessionStore();
  const settingsStore = useSettingsStore();
  const focusSwitcherStore = useFocusSwitcherStore();
  const notify = useUINotificationStore();
  const fileEditorStore = useFileEditorStore();
  const { confirm } = useConfirmDialog();
  const { prompt } = usePromptDialog();

  const currentPath = ref('/');
  const pathInput = ref('/');
  const pathInputRef = ref<HTMLInputElement>();
  const pathInputWrapperRef = ref<HTMLDivElement>();

  const searchQuery = ref('');
  const searchInputRef = ref<HTMLInputElement>();
  const isSearchActive = ref(false);

  const entries = ref<FileEntry[]>([]);
  const sortKey = ref<SftpSortKey>('name');
  const sortDirection = ref<SftpSortDirection>('asc');
  const filteredEntries = computed(() => {
    const query = searchQuery.value.trim().toLowerCase();
    const baseEntries = query
      ? entries.value.filter((entry) => entry.name.toLowerCase().includes(query))
      : entries.value;

    const compareText = (left: string, right: string) => {
      const lowerLeft = left.toLowerCase();
      const lowerRight = right.toLowerCase();
      if (lowerLeft < lowerRight) {
        return -1;
      }
      if (lowerLeft > lowerRight) {
        return 1;
      }
      return 0;
    };

    const direction = sortDirection.value === 'asc' ? 1 : -1;
    return [...baseEntries].sort((left, right) => {
      if (sortKey.value !== 'type') {
        if (left.is_dir && !right.is_dir) {
          return -1;
        }
        if (!left.is_dir && right.is_dir) {
          return 1;
        }
      }

      let leftValue: string | number;
      let rightValue: string | number;
      switch (sortKey.value) {
        case 'type':
          leftValue = left.is_dir ? 0 : 2;
          rightValue = right.is_dir ? 0 : 2;
          break;
        case 'size':
          leftValue = left.is_dir ? -1 : (left.size ?? 0);
          rightValue = right.is_dir ? -1 : (right.size ?? 0);
          break;
        case 'modified':
          leftValue = left.modified ?? 0;
          rightValue = right.modified ?? 0;
          break;
        case 'name':
        default:
          leftValue = left.name;
          rightValue = right.name;
          break;
      }

      let result = 0;
      if (typeof leftValue === 'string' && typeof rightValue === 'string') {
        result = compareText(leftValue, rightValue);
      } else if (leftValue < rightValue) {
        result = -1;
      } else if (leftValue > rightValue) {
        result = 1;
      }

      if (result !== 0) {
        return result * direction;
      }

      if (sortKey.value !== 'name') {
        return left.name.localeCompare(right.name);
      }

      return 0;
    });
  });

  const loading = ref(false);
  const error = ref('');
  const showUpload = ref(false);
  const showSendFile = ref(false);
  const sendFileTarget = ref<string | null>(null);
  const fileManagerShowDeleteConfirmation = computed(() => settingsStore.getBoolean('fileManagerShowDeleteConfirmation', true));
  const showPopupFileEditor = computed(() => settingsStore.getBoolean('showPopupFileEditor', false));

  const ctxVisible = ref(false);
  const ctxEntry = ref<FileEntry | null>(null);
  const ctxPos = ref({ x: 0, y: 0 });
  const ctxMenuRef = ref<HTMLDivElement>();
  const ctxNearRight = ref(false);
  const ctxSubmenuKey = ref<string | null>(null);
  const clipboardState = ref<BrowserClipboardState | null>(null);
  const selectedEntryPaths = ref<Set<string>>(new Set());
  const lastSelectedPath = ref<string | null>(null);
  const showMkdir = ref(false);
  const mkdirName = ref('');
  const mkdirSubmitting = ref(false);
  const showRename = ref(false);
  const renameName = ref('');
  const renameSubmitting = ref(false);
  const renameError = ref('');
  const renameTargetEntry = ref<FileEntry | null>(null);
  const renameInputRef = ref<HTMLInputElement>();
  const showNewFile = ref(false);
  const newFileName = ref('');
  const newFileSubmitting = ref(false);
  const rootModeEnabled = ref(false);
  const rootModeSwitching = ref(false);
  const showRootModeDialog = ref(false);
  const rootModeUsername = ref('root');
  const rootModePassword = ref('');

  const isEditingPath = ref(false);
  const showPathHistoryDropdown = ref(false);
  const pathHistoryItems = ref<PathHistory[]>([]);
  const pathHistoryLoading = ref(false);
  const pathHistorySelectedIndex = ref(-1);
  const filteredPathHistory = computed(() => {
    const query = pathInput.value.trim().toLowerCase();
    if (!query) {
      return pathHistoryItems.value;
    }
    return pathHistoryItems.value.filter((item) => item.path.toLowerCase().includes(query));
  });

  const favoriteButtonRef = ref<HTMLButtonElement>();
  const favoritePopoverRef = ref<HTMLDivElement>();
  const showFavoritePathsPopover = ref(false);
  const isFavoriteDialogOpen = ref(false);
  const favoritePopoverStyle = ref<Record<string, string>>({ left: '0px', top: '0px' });

  const defaultDownloadDir = ref('');
  let unlistenTransferStatus: UnlistenFn | null = null;
  const pendingUploadRefreshes = new Map<string, string>();
  let uploadRefreshTimer: ReturnType<typeof setTimeout> | null = null;

  function isVisibleInput(input: HTMLInputElement | undefined): input is HTMLInputElement {
    if (!input || !input.isConnected || input.disabled) {
      return false;
    }
    const style = window.getComputedStyle(input);
    if (style.display === 'none' || style.visibility === 'hidden') {
      return false;
    }
    const rect = input.getBoundingClientRect();
    return rect.width > 0 && rect.height > 0;
  }

  function normalizePath(path: string): string {
    const trimmed = path.trim();
    if (!trimmed) {
      return '/';
    }

    let normalized = trimmed.replace(/\\/g, '/');
    if (!normalized.startsWith('/')) {
      normalized = `/${normalized}`;
    }

    normalized = normalized.replace(/\/{2,}/g, '/');
    return normalized || '/';
  }

  function joinRemotePath(basePath: string, childName: string): string {
    const safeChild = childName.trim().replace(/^\/+/, '');
    const normalizedBase = basePath === '/' ? '/' : basePath.replace(/\/+$/, '');
    return normalizedBase === '/' ? `/${safeChild}` : `${normalizedBase}/${safeChild}`;
  }

  function joinLocalPath(basePath: string, childName: string): string {
    const normalizedBase = basePath.trim().replace(/[\\/]+$/, '');
    const safeChild = childName.trim().replace(/^[\\/]+/, '');
    if (!normalizedBase) {
      return safeChild || childName;
    }
    const separator = normalizedBase.includes('\\') ? '\\' : '/';
    return `${normalizedBase}${separator}${safeChild}`;
  }

  async function resolveDownloadDefaultPath(fileName: string): Promise<string> {
    if (!defaultDownloadDir.value) {
      try {
        const runtimePaths = await statusApi.getRuntimePaths();
        defaultDownloadDir.value = runtimePaths.downloadDir;
      } catch {
        defaultDownloadDir.value = '';
      }
    }
    return joinLocalPath(defaultDownloadDir.value, fileName);
  }

  function getPathName(path: string): string {
    const normalized = path.replace(/\/+$/, '');
    if (!normalized || normalized === '/') {
      return '';
    }
    const index = normalized.lastIndexOf('/');
    return index >= 0 ? normalized.slice(index + 1) : normalized;
  }

  function getPathDir(path: string): string {
    const normalized = path.replace(/\/+$/, '');
    if (!normalized || normalized === '/') {
      return '/';
    }
    const index = normalized.lastIndexOf('/');
    return index <= 0 ? '/' : normalized.slice(0, index);
  }

  function splitNameExt(name: string): { base: string; ext: string } {
    const index = name.lastIndexOf('.');
    if (index > 0) {
      return { base: name.slice(0, index), ext: name.slice(index) };
    }
    return { base: name, ext: '' };
  }

  function buildDuplicateName(name: string, index: number): string {
    const { base, ext } = splitNameExt(name);
    if (index <= 1) {
      return `${base}_copy${ext}`;
    }
    return `${base}_copy_${index}${ext}`;
  }

  function validateNewEntryName(name: string, kind: '文件夹' | '文件', ignorePath?: string): string | null {
    if (!name) {
      return `请输入${kind}名称`;
    }
    if (name === '.' || name === '..') {
      return `${kind}名称不能为 ${name}`;
    }
    if (name.includes('/')) {
      return `${kind}名称不能包含 /`;
    }
    const existingEntry = entries.value.find((entry) => entry.name === name && entry.path !== ignorePath);
    if (existingEntry) {
      const existingKind = existingEntry.is_dir ? '文件夹' : '文件';
      return `当前目录已存在同名${existingKind}：${name}`;
    }
    return null;
  }

  async function statTargetIfExists(sessionId: string, targetPath: string): Promise<FileEntry | null> {
    try {
      return await sftpApi.stat(sessionId, targetPath);
    } catch {
      return null;
    }
  }

  function resetBrowserState() {
    entries.value = [];
    currentPath.value = '/';
    pathInput.value = '/';
    error.value = '';
    searchQuery.value = '';
    isSearchActive.value = false;
    isEditingPath.value = false;
    showPathHistoryDropdown.value = false;
    pathHistorySelectedIndex.value = -1;
    pathHistoryItems.value = [];
    showFavoritePathsPopover.value = false;
    showMkdir.value = false;
    showNewFile.value = false;
    showRename.value = false;
    mkdirName.value = '';
    newFileName.value = '';
    renameName.value = '';
    mkdirSubmitting.value = false;
    newFileSubmitting.value = false;
    renameSubmitting.value = false;
    renameError.value = '';
    renameTargetEntry.value = null;
    rootModeEnabled.value = false;
    rootModeSwitching.value = false;
    showRootModeDialog.value = false;
    rootModePassword.value = '';
    ctxVisible.value = false;
    ctxEntry.value = null;
    ctxSubmenuKey.value = null;
    clipboardState.value = null;
    sortKey.value = 'name';
    sortDirection.value = 'asc';
    clearSelection();
  }

  async function ensureSftpSession(sshSid: string): Promise<string> {
    const session = sessionStore.getSession(sshSid);
    if (!session) {
      throw new Error('会话不存在');
    }
    if (session.protocol !== 'SSH') {
      throw new Error('当前会话不支持 SFTP');
    }
    if (session.sftpSessionId) {
      return session.sftpSessionId;
    }

    const sftpSessionId = await sftpApi.open(session.connectionId);
    sessionStore.setSftpSession(sshSid, sftpSessionId);
    return sftpSessionId;
  }

  function closeRootModeDialog(): void {
    if (rootModeSwitching.value) {
      return;
    }
    showRootModeDialog.value = false;
    rootModePassword.value = '';
  }

  async function handleRootModeButtonClick(): Promise<void> {
    if (!connectionId.value || rootModeSwitching.value) {
      return;
    }

    if (rootModeEnabled.value) {
      await disableRootMode();
      return;
    }

    rootModeUsername.value = rootModeUsername.value || 'root';
    rootModePassword.value = '';
    showRootModeDialog.value = true;
  }

  async function enableRootMode(): Promise<void> {
    if (!connectionId.value || !sshSessionId.value || rootModeSwitching.value) {
      return;
    }

    const username = rootModeUsername.value.trim() || 'root';
    const password = rootModePassword.value;
    if (!password) {
      notify.addNotification('warning', '请输入 Root 密码');
      return;
    }

    rootModeSwitching.value = true;
    const previousSessionId = sftpSessionId.value;

    try {
      const rootSftpSessionId = await sftpApi.openOverride(connectionId.value, {
        username,
        authMethod: 'password',
        password,
      });

      sessionStore.setSftpSession(sshSessionId.value, rootSftpSessionId);
      rootModeEnabled.value = true;
      showRootModeDialog.value = false;
      rootModePassword.value = '';

      if (previousSessionId && previousSessionId !== rootSftpSessionId) {
        void sftpApi.close(previousSessionId).catch(() => undefined);
      }

      notify.addNotification('success', `已进入 Root 模式（${username}）`);
      await navigateTo('/root');
    } catch (e: unknown) {
      const message = String(toAppError(e).message || '');
      if (message.includes('authentication rejected') || message.includes('认证被拒绝')) {
        notify.addNotification(
          'error',
          'Root 认证被拒绝：请检查目标机 SSH 是否允许 root 登录，或改用密钥认证。'
        );
      } else {
        notify.addNotification('error', message || '进入 Root 模式失败');
      }
    } finally {
      rootModeSwitching.value = false;
    }
  }

  async function disableRootMode(): Promise<void> {
    if (!connectionId.value || !sshSessionId.value || rootModeSwitching.value) {
      return;
    }

    rootModeSwitching.value = true;
    const previousSessionId = sftpSessionId.value;

    try {
      const normalSftpSessionId = await sftpApi.open(connectionId.value);
      sessionStore.setSftpSession(sshSessionId.value, normalSftpSessionId);
      rootModeEnabled.value = false;

      if (previousSessionId && previousSessionId !== normalSftpSessionId) {
        void sftpApi.close(previousSessionId).catch(() => undefined);
      }

      notify.addNotification('success', '已退出 Root 模式');
      await navigateTo(currentPath.value || '/');
    } catch (e: unknown) {
      notify.addNotification('error', toAppError(e).message || '退出 Root 模式失败');
    } finally {
      rootModeSwitching.value = false;
    }
  }

  async function addPathToHistory(path: string): Promise<void> {
    try {
      await pathHistoryApi.add(path, connectionId.value);
    } catch {
      // ignore path history persistence failures
    }
  }

  async function loadPathHistory(): Promise<void> {
    pathHistoryLoading.value = true;
    try {
      pathHistoryItems.value = await pathHistoryApi.list(connectionId.value, 80);
    } catch {
      pathHistoryItems.value = [];
    } finally {
      pathHistoryLoading.value = false;
    }
  }

  function openPathHistory(): void {
    showPathHistoryDropdown.value = true;
    pathHistorySelectedIndex.value = -1;
    void loadPathHistory();
  }

  function closePathHistory(): void {
    showPathHistoryDropdown.value = false;
    pathHistorySelectedIndex.value = -1;
  }

  async function navigateTo(path: string) {
    const sid = sftpSessionId.value;
    if (!sid) return;

    const targetPath = normalizePath(path);
    const previousPath = currentPath.value;

    loading.value = true;
    error.value = '';
    try {
      const list = await sftpApi.listDir(sid, targetPath);
      entries.value = list;
      clearSelection();
      closeCtxMenu();
      currentPath.value = targetPath;
      pathInput.value = targetPath;

      if (sshSessionId.value) {
        sessionStore.setCurrentPath(sshSessionId.value, targetPath);
      }

      if (targetPath !== previousPath) {
        void addPathToHistory(targetPath);
      }
    } catch (e: unknown) {
      error.value = toAppError(e).message || '加载失败';
    } finally {
      loading.value = false;
    }
  }

  function goUp() {
    const normalized = currentPath.value.replace(/\/+$/, '');
    if (!normalized || normalized === '/') {
      return;
    }
    const index = normalized.lastIndexOf('/');
    const parent = index <= 0 ? '/' : normalized.slice(0, index);
    void navigateTo(parent);
  }

  function refresh() {
    void navigateTo(currentPath.value);
  }

  function activateSearch(): void {
    isSearchActive.value = true;
    void nextTick(() => {
      searchInputRef.value?.focus();
      searchInputRef.value?.select();
    });
  }

  function deactivateSearch(): void {
    isSearchActive.value = false;
  }

  function cancelSearch(): void {
    searchQuery.value = '';
    isSearchActive.value = false;
  }

  function updateFavoritePopoverPosition(): void {
    if (!showFavoritePathsPopover.value || !favoriteButtonRef.value || !favoritePopoverRef.value) {
      return;
    }

    const triggerRect = favoriteButtonRef.value.getBoundingClientRect();
    const popoverRect = favoritePopoverRef.value.getBoundingClientRect();

    let left = triggerRect.left;
    let top = triggerRect.bottom + 2;

    const maxLeft = window.innerWidth - popoverRect.width - VIEWPORT_PADDING;
    if (left > maxLeft) {
      left = maxLeft;
    }
    left = Math.max(VIEWPORT_PADDING, left);

    if (top + popoverRect.height + VIEWPORT_PADDING > window.innerHeight) {
      const topAbove = triggerRect.top - popoverRect.height - 2;
      top = topAbove >= VIEWPORT_PADDING
        ? topAbove
        : Math.max(VIEWPORT_PADDING, window.innerHeight - popoverRect.height - VIEWPORT_PADDING);
    }

    favoritePopoverStyle.value = {
      left: `${left}px`,
      top: `${top}px`,
    };
  }

  function toggleFavoritePopover(): void {
    if (showFavoritePathsPopover.value) {
      showFavoritePathsPopover.value = false;
      isFavoriteDialogOpen.value = false;
      return;
    }

    showFavoritePathsPopover.value = true;
    void nextTick(updateFavoritePopoverPosition);
  }

  function navigateFromFavorite(path: string): void {
    showFavoritePathsPopover.value = false;
    isFavoriteDialogOpen.value = false;
    void navigateTo(path);
  }

  function handleFavoriteDialogVisibility(visible: boolean): void {
    isFavoriteDialogOpen.value = visible;
  }

  function startPathEdit(): void {
    if (!sftpSessionId.value) {
      return;
    }

    pathInput.value = currentPath.value;
    isEditingPath.value = true;
    openPathHistory();

    void nextTick(() => {
      pathInputRef.value?.focus();
      pathInputRef.value?.select();
    });
  }

  function handlePathInputFocus(): void {
    openPathHistory();
  }

  function handlePathInputChange(): void {
    pathHistorySelectedIndex.value = -1;
  }

  async function applyPathFromInput(target: string): Promise<void> {
    await navigateTo(target);
    isEditingPath.value = false;
    closePathHistory();
  }

  function handlePathInputKeydown(event: KeyboardEvent): void {
    const list = filteredPathHistory.value;

    switch (event.key) {
      case 'ArrowDown': {
        event.preventDefault();
        if (!list.length) {
          return;
        }
        pathHistorySelectedIndex.value =
          pathHistorySelectedIndex.value >= list.length - 1 ? 0 : pathHistorySelectedIndex.value + 1;
        break;
      }
      case 'ArrowUp': {
        event.preventDefault();
        if (!list.length) {
          return;
        }
        pathHistorySelectedIndex.value =
          pathHistorySelectedIndex.value <= 0 ? list.length - 1 : pathHistorySelectedIndex.value - 1;
        break;
      }
      case 'Enter': {
        event.preventDefault();
        const selected =
          pathHistorySelectedIndex.value >= 0 ? list[pathHistorySelectedIndex.value]?.path : pathInput.value;
        void applyPathFromInput(selected || pathInput.value);
        break;
      }
      case 'Escape': {
        event.preventDefault();
        pathInput.value = currentPath.value;
        isEditingPath.value = false;
        closePathHistory();
        break;
      }
      default:
        break;
    }
  }

  function handlePathInputBlur(): void {
    window.setTimeout(() => {
      const activeEl = document.activeElement as Node | null;
      const wrapper = pathInputWrapperRef.value;
      if (!wrapper || !activeEl || !wrapper.contains(activeEl)) {
        isEditingPath.value = false;
        closePathHistory();
      }
    }, 120);
  }

  function selectPathHistory(path: string): void {
    void applyPathFromInput(path);
  }

  async function sendCdCommandToTerminal(): Promise<void> {
    const sid = sshSessionId.value;
    if (!sid || !currentPath.value) {
      return;
    }

    const escapedPath = currentPath.value.replace(/["\\$`]/g, '\\$&');
    const command = `cd "${escapedPath}"\n`;

    try {
      const data = btoa(unescape(encodeURIComponent(command)));
      await sshApi.write(sid, data);
    } catch (e: unknown) {
      notify.addNotification('error', `发送目录切换命令失败: ${toAppError(e).message || String(e)}`);
    }
  }

  const selectedEntries = computed<FileEntry[]>(() =>
    entries.value.filter((entry) => selectedEntryPaths.value.has(entry.path)),
  );

  function isArchiveFile(filename: string): boolean {
    const lower = filename.toLowerCase();
    return SUPPORTED_ARCHIVE_EXTENSIONS.some((ext) => lower.endsWith(ext));
  }

  function isEntrySelected(entry: FileEntry): boolean {
    return selectedEntryPaths.value.has(entry.path);
  }

  function clearSelection(): void {
    if (selectedEntryPaths.value.size) {
      selectedEntryPaths.value = new Set();
    }
    lastSelectedPath.value = null;
  }

  function setSingleSelection(entry: FileEntry): void {
    selectedEntryPaths.value = new Set([entry.path]);
    lastSelectedPath.value = entry.path;
  }

  function toggleEntrySelection(entry: FileEntry): void {
    const next = new Set(selectedEntryPaths.value);
    if (next.has(entry.path)) {
      next.delete(entry.path);
    } else {
      next.add(entry.path);
    }
    selectedEntryPaths.value = next;
    lastSelectedPath.value = entry.path;
  }

  function selectRangeTo(entry: FileEntry): void {
    const allVisibleEntries = filteredEntries.value;
    if (!allVisibleEntries.length) {
      setSingleSelection(entry);
      return;
    }

    const targetIndex = allVisibleEntries.findIndex((item) => item.path === entry.path);
    const anchorPath = lastSelectedPath.value ?? entry.path;
    const anchorIndex = allVisibleEntries.findIndex((item) => item.path === anchorPath);

    if (targetIndex === -1 || anchorIndex === -1) {
      setSingleSelection(entry);
      return;
    }

    const start = Math.min(targetIndex, anchorIndex);
    const end = Math.max(targetIndex, anchorIndex);
    const next = new Set(selectedEntryPaths.value);
    for (let i = start; i <= end; i += 1) {
      next.add(allVisibleEntries[i].path);
    }
    selectedEntryPaths.value = next;
    lastSelectedPath.value = entry.path;
  }

  function handleEntryClick(entry: FileEntry, event: MouseEvent): void {
    if (event.shiftKey) {
      selectRangeTo(entry);
      return;
    }

    if (event.ctrlKey || event.metaKey) {
      toggleEntrySelection(entry);
      return;
    }

    setSingleSelection(entry);
    if (entry.is_dir) {
      void navigateTo(entry.path);
    } else {
      void openEditor(entry);
    }
  }

  function closeCtxMenu(): void {
    ctxVisible.value = false;
    ctxEntry.value = null;
    ctxSubmenuKey.value = null;
    ctxNearRight.value = false;
  }

  function adjustContextMenuPosition(): void {
    const menu = ctxMenuRef.value;
    if (!menu) {
      return;
    }

    const rect = menu.getBoundingClientRect();
    let x = ctxPos.value.x;
    let y = ctxPos.value.y;

    const maxX = window.innerWidth - rect.width - VIEWPORT_PADDING;
    const maxY = window.innerHeight - rect.height - VIEWPORT_PADDING;

    if (x > maxX) {
      x = maxX;
    }
    if (y > maxY) {
      y = maxY;
    }

    x = Math.max(VIEWPORT_PADDING, x);
    y = Math.max(VIEWPORT_PADDING, y);

    ctxPos.value = { x, y };

    const remainingRight = window.innerWidth - (x + rect.width);
    ctxNearRight.value = remainingRight < CONTEXT_SUBMENU_MIN_WIDTH + VIEWPORT_PADDING;
  }

  function showCtx(e: MouseEvent, entry: FileEntry | null = null): void {
    e.preventDefault();
    if (!entry) {
      clearSelection();
    }
    ctxEntry.value = entry;
    ctxPos.value = { x: e.clientX, y: e.clientY };
    ctxSubmenuKey.value = null;
    ctxVisible.value = true;
    void nextTick(adjustContextMenuPosition);
  }

  function handleEntryContextMenu(e: MouseEvent, entry: FileEntry): void {
    const isSelected = selectedEntryPaths.value.has(entry.path);
    const hasModifiers = e.ctrlKey || e.metaKey || e.shiftKey;

    if (!isSelected && !hasModifiers) {
      setSingleSelection(entry);
    } else if (!isSelected && hasModifiers) {
      const next = new Set(selectedEntryPaths.value);
      next.add(entry.path);
      selectedEntryPaths.value = next;
      lastSelectedPath.value = entry.path;
    }

    showCtx(e, entry);
  }

  function runContextAction(action: () => void | Promise<void>): void {
    try {
      const pending = Promise.resolve(action());
      closeCtxMenu();
      void pending;
    } catch (error) {
      closeCtxMenu();
      throw error;
    }
  }

  function getContextEntries(): FileEntry[] {
    const entry = ctxEntry.value;
    if (!entry) {
      return [];
    }

    if (selectedEntryPaths.value.has(entry.path) && selectedEntries.value.length > 1) {
      return [...selectedEntries.value];
    }

    return [entry];
  }

  function buildCompressSubmenu(prefix: string, targets: FileEntry[], disabled: boolean): BrowserContextMenuItem[] {
    return [
      {
        key: `${prefix}-zip`,
        label: '压缩为 ZIP',
        disabled,
        onClick: () => triggerCompress('zip', targets),
      },
      {
        key: `${prefix}-targz`,
        label: '压缩为 TAR.GZ',
        disabled,
        onClick: () => triggerCompress('tar.gz', targets),
      },
      {
        key: `${prefix}-tarbz2`,
        label: '压缩为 TAR.BZ2',
        disabled,
        onClick: () => triggerCompress('tar.bz2', targets),
      },
    ];
  }

  const contextMenuItems = computed<BrowserContextMenuItem[]>(() => {
    const entry = ctxEntry.value;
    const canUseSftp = !!sftpSessionId.value;
    const hasClipboard = !!clipboardState.value?.entries.length;
    const items: BrowserContextMenuItem[] = [];
    const targetEntries = getContextEntries();
    const isMultiContext = targetEntries.length > 1 && !!entry && selectedEntryPaths.value.has(entry.path);

    if (isMultiContext) {
      const allFilesSelected = targetEntries.every((item) => !item.is_dir);

      items.push({
        key: 'cut-multi',
        label: '剪切',
        icon: 'fas fa-cut',
        disabled: !canUseSftp,
        onClick: () => queueClipboardAction('cut', targetEntries),
      });
      items.push({
        key: 'copy-multi',
        label: '复制',
        icon: 'fas fa-copy',
        disabled: !canUseSftp,
        onClick: () => queueClipboardAction('copy', targetEntries),
      });

      if (allFilesSelected) {
        items.push({
          key: 'download-multi',
          label: `下载 ${targetEntries.length} 个文件`,
          icon: 'fas fa-download',
          disabled: !canUseSftp,
          onClick: () => downloadMultipleFiles(targetEntries),
        });
      }

      items.push({
        key: 'compress-multi',
        label: '压缩',
        icon: 'fas fa-file-archive',
        submenu: buildCompressSubmenu('compress-multi', targetEntries, !canUseSftp),
      });
      items.push({
        key: 'send-to-multi',
        label: '发送到...',
        icon: 'fas fa-share',
        disabled: !canUseSftp,
        onClick: () => openSendModal(targetEntries[0]?.path),
      });

      items.push({ key: 'sep-multi-1', separator: true });
      items.push({
        key: 'delete-multi',
        label: `删除 ${targetEntries.length} 项`,
        icon: 'fas fa-trash-alt',
        danger: true,
        disabled: !canUseSftp,
        onClick: () => handleDeleteEntries(targetEntries),
      });
      items.push({
        key: 'refresh-multi',
        label: '刷新',
        icon: 'fas fa-sync-alt',
        disabled: !canUseSftp,
        onClick: refresh,
      });

      return items;
    }

    if (entry && entry.name !== '..') {
      if (entry.is_dir) {
        items.push({
          key: 'download-folder',
          label: '下载文件夹',
          icon: 'fas fa-file-archive',
          disabled: !canUseSftp,
          onClick: () => downloadDirectory(entry),
        });
      } else {
        items.push({
          key: 'download-file',
          label: '下载文件',
          icon: 'fas fa-download',
          disabled: !canUseSftp,
          onClick: () => downloadFile(entry),
        });
      }

      items.push({
        key: 'cut',
        label: '剪切',
        icon: 'fas fa-cut',
        disabled: !canUseSftp,
        onClick: () => queueClipboardAction('cut', targetEntries),
      });
      items.push({
        key: 'copy',
        label: '复制',
        icon: 'fas fa-copy',
        disabled: !canUseSftp,
        onClick: () => queueClipboardAction('copy', targetEntries),
      });

      if (entry.is_dir) {
        items.push({
          key: 'paste',
          label: '粘贴',
          icon: 'fas fa-paste',
          disabled: !canUseSftp || !hasClipboard,
          onClick: () => pasteFromClipboard(entry.path),
        });
      }

      items.push({
        key: 'copy-path',
        label: '复制路径',
        icon: 'fas fa-link',
        disabled: !canUseSftp,
        onClick: () => copyContextPath(entry.path),
      });

      items.push({ key: 'sep-single-1', separator: true });

      items.push({
        key: 'delete',
        label: '删除',
        icon: 'fas fa-trash-alt',
        danger: true,
        disabled: !canUseSftp,
        onClick: () => handleDelete(entry),
      });
      items.push({
        key: 'rename',
        label: '重命名',
        icon: 'fas fa-i-cursor',
        disabled: !canUseSftp,
        onClick: () => handleRename(entry),
      });

      items.push({ key: 'sep-single-2', separator: true });

      const compressTargets = [entry];
      items.push({
        key: 'compress',
        label: '压缩',
        icon: 'fas fa-file-archive',
        submenu: buildCompressSubmenu('compress', compressTargets, !canUseSftp),
      });

      if (!entry.is_dir && isArchiveFile(entry.name)) {
        items.push({
          key: 'decompress',
          label: '解压',
          icon: 'fas fa-box-open',
          disabled: !canUseSftp,
          onClick: () => triggerDecompress(entry),
        });
      }

      items.push({
        key: 'send-to',
        label: '发送到...',
        icon: 'fas fa-share',
        disabled: !canUseSftp,
        onClick: () => openSendModal(entry.path),
      });

      items.push({ key: 'sep-single-3', separator: true });
      items.push({
        key: 'new-folder',
        label: '新建文件夹',
        icon: 'fas fa-folder-plus',
        disabled: !canUseSftp,
        onClick: () => {
          showMkdir.value = true;
        },
      });
      items.push({
        key: 'new-file',
        label: '新建文件',
        icon: 'far fa-file-alt',
        disabled: !canUseSftp,
        onClick: () => {
          showNewFile.value = true;
        },
      });
      items.push({
        key: 'upload',
        label: '上传',
        icon: 'fas fa-upload',
        disabled: !canUseSftp,
        onClick: openUpload,
      });

      items.push({ key: 'sep-single-4', separator: true });
      items.push({
        key: 'chmod',
        label: '修改权限',
        icon: 'fas fa-lock',
        disabled: !canUseSftp,
        onClick: () => handleChmod(entry),
      });
      items.push({
        key: 'refresh',
        label: '刷新',
        icon: 'fas fa-sync-alt',
        disabled: !canUseSftp,
        onClick: refresh,
      });

      return items;
    }

    if (!entry) {
      items.push({
        key: 'paste-empty',
        label: '粘贴',
        icon: 'fas fa-paste',
        disabled: !canUseSftp || !hasClipboard,
        onClick: () => pasteFromClipboard(currentPath.value),
      });
      items.push({ key: 'sep-empty-1', separator: true });
      items.push({
        key: 'new-folder-empty',
        label: '新建文件夹',
        icon: 'fas fa-folder-plus',
        disabled: !canUseSftp,
        onClick: () => {
          showMkdir.value = true;
        },
      });
      items.push({
        key: 'new-file-empty',
        label: '新建文件',
        icon: 'far fa-file-alt',
        disabled: !canUseSftp,
        onClick: () => {
          showNewFile.value = true;
        },
      });
      items.push({
        key: 'upload-empty',
        label: '上传',
        icon: 'fas fa-upload',
        disabled: !canUseSftp,
        onClick: openUpload,
      });
      items.push({ key: 'sep-empty-2', separator: true });
      items.push({
        key: 'refresh-empty',
        label: '刷新',
        icon: 'fas fa-sync-alt',
        disabled: !canUseSftp,
        onClick: refresh,
      });

      return items;
    }

    items.push({
      key: 'paste-parent',
      label: '粘贴',
      icon: 'fas fa-paste',
      disabled: !canUseSftp || !hasClipboard,
      onClick: () => pasteFromClipboard(currentPath.value),
    });
    items.push({
      key: 'refresh-parent',
      label: '刷新',
      icon: 'fas fa-sync-alt',
      disabled: !canUseSftp,
      onClick: refresh,
    });

    return items;
  });

  function handleCtxItemClick(item: BrowserContextMenuItem, fromSubmenu = false): void {
    if (item.disabled) {
      return;
    }

    if (item.submenu && item.submenu.length && !fromSubmenu) {
      ctxSubmenuKey.value = ctxSubmenuKey.value === item.key ? null : item.key;
      return;
    }

    if (item.onClick) {
      runContextAction(item.onClick);
    }
  }

  function queueClipboardAction(operation: 'copy' | 'cut', targetEntriesOverride?: FileEntry[]): void {
    const sid = sftpSessionId.value;
    const targetEntries = targetEntriesOverride?.length ? targetEntriesOverride : getContextEntries();
    if (!sid || !targetEntries.length) {
      return;
    }

    clipboardState.value = {
      operation,
      sessionId: sid,
      entries: targetEntries.map((entry) => ({ ...entry })),
    };

    if (targetEntries.length === 1) {
      notify.addNotification('success', operation === 'copy' ? `已复制 ${targetEntries[0].name}` : `已剪切 ${targetEntries[0].name}`);
    } else {
      notify.addNotification('success', operation === 'copy' ? `已复制 ${targetEntries.length} 项` : `已剪切 ${targetEntries.length} 项`);
    }
  }

  async function cloneEntryRecursive(sessionId: string, source: FileEntry, targetPath: string): Promise<void> {
    await sftpApi.copyEntry(sessionId, source.path, targetPath);
  }

  async function pasteFromClipboard(targetDirOverride?: string): Promise<void> {
    const sid = sftpSessionId.value;
    const clipboard = clipboardState.value;
    if (!sid || !clipboard || !clipboard.entries.length) {
      return;
    }

    if (clipboard.sessionId !== sid) {
      notify.addNotification('warning', '暂不支持跨会话粘贴');
      return;
    }

    const targetDir = targetDirOverride ?? (ctxEntry.value?.is_dir ? ctxEntry.value.path : currentPath.value);

    try {
      const existingNames = new Set((await sftpApi.listDir(sid, targetDir)).map((item) => item.name));
      let movedCount = 0;
      let copiedCount = 0;
      let skippedCount = 0;
      let failedCount = 0;

      for (const sourceEntry of clipboard.entries) {
        const sourceName = getPathName(sourceEntry.path) || sourceEntry.name;
        if (!sourceName) {
          failedCount += 1;
          continue;
        }

        const sourceDir = getPathDir(sourceEntry.path);
        if (clipboard.operation === 'cut' && sourceDir === targetDir) {
          skippedCount += 1;
          continue;
        }

        let targetName = sourceName;
        let index = 1;
        while (existingNames.has(targetName)) {
          targetName = buildDuplicateName(sourceName, index);
          index += 1;
        }
        existingNames.add(targetName);

        const targetPath = joinRemotePath(targetDir, targetName);

        try {
          if (clipboard.operation === 'cut') {
            await sftpApi.rename(sid, sourceEntry.path, targetPath);
            movedCount += 1;
          } else {
            await cloneEntryRecursive(sid, sourceEntry, targetPath);
            copiedCount += 1;
          }
        } catch {
          failedCount += 1;
        }
      }

      if (clipboard.operation === 'cut' && (movedCount > 0 || skippedCount === clipboard.entries.length)) {
        clipboardState.value = null;
      }

      if (clipboard.operation === 'cut' && movedCount > 0) {
        notify.addNotification('success', `已移动 ${movedCount} 项`);
      } else if (clipboard.operation === 'copy' && copiedCount > 0) {
        notify.addNotification('success', `已复制 ${copiedCount} 项`);
      } else if (skippedCount > 0 && failedCount === 0) {
        notify.addNotification('info', '源目录与目标目录一致，无需粘贴');
      }

      if (failedCount > 0) {
        notify.addNotification('warning', `有 ${failedCount} 项粘贴失败`);
      }

      await navigateTo(currentPath.value);
    } catch (e: unknown) {
      notify.addNotification('error', toAppError(e).message || '粘贴失败');
    }
  }

  async function copyContextPath(targetPath?: string): Promise<void> {
    const path = targetPath ?? ctxEntry.value?.path;
    if (!path) {
      return;
    }

    try {
      await navigator.clipboard.writeText(path);
      notify.addNotification('success', '路径已复制到剪贴板');
    } catch {
      notify.addNotification('error', '复制路径失败，请检查剪贴板权限');
    }
  }

  function triggerCompress(format: 'zip' | 'tar.gz' | 'tar.bz2', targetEntries: FileEntry[] = getContextEntries()): void {
    if (!targetEntries.length) {
      return;
    }

    const formatLabel = format.toUpperCase();
    const label = targetEntries.length === 1 ? targetEntries[0].name : `${targetEntries.length} 项`;
    notify.addNotification('info', `${label} 的 ${formatLabel} 压缩能力即将接入，当前可先使用下载功能`);
  }

  function triggerDecompress(entry: FileEntry): void {
    notify.addNotification('info', `${entry.name} 的解压能力即将接入，当前可先下载后在本地解压`);
  }

  async function downloadMultipleFiles(targetEntries: FileEntry[]): Promise<void> {
    const fileEntries = targetEntries.filter((entry) => !entry.is_dir);
    if (!fileEntries.length) {
      notify.addNotification('warning', '批量下载仅支持文件');
      return;
    }

    for (const fileEntry of fileEntries) {
      await downloadFile(fileEntry);
    }
  }

  async function handleDeleteEntries(targetEntries: FileEntry[]): Promise<void> {
    const sid = sftpSessionId.value;
    if (!sid || !targetEntries.length) {
      return;
    }

    const containsDirectory = targetEntries.some((entry) => entry.is_dir);
    const label = targetEntries.length === 1 ? `"${targetEntries[0].name}"` : `${targetEntries.length} 项`;
    const shouldConfirm = fileManagerShowDeleteConfirmation.value || containsDirectory;
    if (shouldConfirm) {
      let message = `确定删除 ${label} 吗？`;
      if (targetEntries.length === 1 && targetEntries[0].is_dir) {
        message = `确定删除文件夹 ${label} 及其所有内容吗？`;
      } else if (containsDirectory) {
        message = `确定删除 ${label} 吗？其中包含文件夹，文件夹内容也会被一并删除。`;
      }

      const confirmed = await confirm('确认删除', message);
      if (!confirmed) {
        return;
      }
    }

    let successCount = 0;
    let failedCount = 0;

    // 文件优先，目录按深度从深到浅删除，降低目录删除失败概率
    const sortedTargets = [...targetEntries].sort((a, b) => {
      if (a.is_dir !== b.is_dir) {
        return a.is_dir ? 1 : -1;
      }

      if (!a.is_dir && !b.is_dir) {
        return 0;
      }

      const depthA = a.path.split('/').length;
      const depthB = b.path.split('/').length;
      return depthB - depthA;
    });

    for (const target of sortedTargets) {
      try {
        if (target.is_dir) {
          await sftpApi.rmdir(sid, target.path);
        } else {
          await sftpApi.removeFile(sid, target.path);
        }
        successCount += 1;
      } catch {
        failedCount += 1;
      }
    }

    if (successCount > 0) {
      notify.addNotification('success', `已删除 ${successCount} 项`);
    }
    if (failedCount > 0) {
      notify.addNotification('warning', `${failedCount} 项删除失败`);
    }

    if (successCount > 0) {
      if (targetEntries.every((item) => selectedEntryPaths.value.has(item.path))) {
        clearSelection();
      }
      await navigateTo(currentPath.value);
    }
  }

  function openUpload() {
    if (!sftpSessionId.value) {
      notify.addNotification('error', 'SFTP 未就绪，无法上传');
      return;
    }
    showUpload.value = true;
  }

  function toggleSort(nextKey: SftpSortKey): void {
    if (sortKey.value === nextKey) {
      sortDirection.value = sortDirection.value === 'asc' ? 'desc' : 'asc';
    } else {
      sortKey.value = nextKey;
      sortDirection.value = 'asc';
    }
  }

  async function uploadLocalPaths(paths: string[]): Promise<void> {
    const sid = sftpSessionId.value;
    if (!sid || !paths.length) {
      return;
    }

    try {
      const basePath = currentPath.value;
      const { taskIds } = await createUploadTasksFromLocalPaths({
        sessionId: sid,
        paths,
        remoteBasePath: basePath,
        joinRemotePath,
        onTaskCreated: announceTransfer,
      });

      if (!taskIds.length) {
        notify.addNotification('warning', '未找到可上传的文件');
        return;
      }

      registerUploadTasksForRefresh(taskIds, basePath);
      notify.addNotification('success', `已创建 ${taskIds.length} 个上传任务`);
    } catch (e: unknown) {
      notify.addNotification('error', `上传失败: ${toAppError(e).message}`);
    }
  }

  async function moveEntriesToDirectory(
    sourcePaths: string[],
    targetDirectory: string,
  ): Promise<void> {
    const sid = sftpSessionId.value;
    if (!sid || !sourcePaths.length) {
      return;
    }

    const normalizedTargetDir = normalizePath(targetDirectory);
    let movedCount = 0;
    let skippedCount = 0;
    let failedCount = 0;

    for (const sourcePath of sourcePaths) {
      const normalizedSourcePath = normalizePath(sourcePath);
      const entryName = getPathName(normalizedSourcePath);
      if (!entryName) {
        skippedCount += 1;
        continue;
      }

      const nextPath = joinRemotePath(normalizedTargetDir, entryName);
      const movingIntoSelf = normalizedTargetDir === normalizedSourcePath
        || normalizedTargetDir.startsWith(`${normalizedSourcePath}/`);
      if (normalizedSourcePath === nextPath || movingIntoSelf) {
        skippedCount += 1;
        continue;
      }

      try {
        await sftpApi.rename(sid, normalizedSourcePath, nextPath);
        movedCount += 1;
      } catch {
        failedCount += 1;
      }
    }

    if (movedCount > 0) {
      clearSelection();
      notify.addNotification('success', `已移动 ${movedCount} 项`);
      await navigateTo(currentPath.value);
    }
    if (skippedCount > 0) {
      notify.addNotification('info', `${skippedCount} 项位置未变化，已跳过`);
    }
    if (failedCount > 0) {
      notify.addNotification('warning', `${failedCount} 项移动失败`);
    }
  }

  function scheduleRefreshAfterUpload(): void {
    if (uploadRefreshTimer) {
      clearTimeout(uploadRefreshTimer);
    }

    uploadRefreshTimer = setTimeout(() => {
      uploadRefreshTimer = null;
      void navigateTo(currentPath.value);
    }, 250);
  }

  function registerUploadTasksForRefresh(taskIds: string[], remoteDir: string): void {
    const normalizedDir = normalizePath(remoteDir);
    for (const taskId of taskIds) {
      pendingUploadRefreshes.set(taskId, normalizedDir);
    }
  }

  function handleTransferStatusEvent(payload: TransferStatusEvent): void {
    if (payload.kind !== 'upload') {
      return;
    }

    const trackedPath = pendingUploadRefreshes.get(payload.task_id);
    if (!trackedPath) {
      return;
    }

    if (payload.status === 'completed') {
      if (trackedPath === currentPath.value) {
        scheduleRefreshAfterUpload();
      }
      pendingUploadRefreshes.delete(payload.task_id);
      return;
    }

    if (payload.status === 'failed' || payload.status === 'cancelled') {
      pendingUploadRefreshes.delete(payload.task_id);
    }
  }

  function openSendModal(path?: string) {
    if (!sftpSessionId.value) {
      notify.addNotification('error', 'SFTP 未就绪，无法发起传输');
      return;
    }
    sendFileTarget.value = path ?? null;
    showSendFile.value = true;
  }

  async function handleDelete(entry: FileEntry) {
    await handleDeleteEntries([entry]);
  }

  function closeRenameDialog(force = false): void {
    if (renameSubmitting.value && !force) {
      return;
    }
    showRename.value = false;
    renameName.value = '';
    renameError.value = '';
    renameTargetEntry.value = null;
  }

  function handleRename(entry: FileEntry): void {
    renameTargetEntry.value = entry;
    renameName.value = entry.name;
    renameError.value = '';
    showRename.value = true;
    void nextTick(() => {
      renameInputRef.value?.focus();
      renameInputRef.value?.select();
    });
  }

  async function submitRename(): Promise<void> {
    const sid = sftpSessionId.value;
    const targetEntry = renameTargetEntry.value;
    if (!sid || !targetEntry || renameSubmitting.value) {
      return;
    }

    const nextName = renameName.value.trim();
    if (!nextName || nextName === targetEntry.name) {
      closeRenameDialog();
      return;
    }

    const validationMessage = validateNewEntryName(
      nextName,
      targetEntry.is_dir ? '文件夹' : '文件',
      targetEntry.path,
    );
    if (validationMessage) {
      renameError.value = validationMessage;
      return;
    }

    renameSubmitting.value = true;
    renameError.value = '';
    const nextPath = joinRemotePath(getPathDir(targetEntry.path), nextName);

    try {
      await sftpApi.rename(sid, targetEntry.path, nextPath);
      renameSubmitting.value = false;
      closeRenameDialog(true);
      await navigateTo(currentPath.value);
    } catch (e: unknown) {
      const existingTarget = await statTargetIfExists(sid, nextPath);
      if (existingTarget && existingTarget.path !== targetEntry.path) {
        renameError.value = existingTarget.is_dir
          ? `当前目录已存在同名文件夹：${nextName}`
          : `当前目录已存在同名文件：${nextName}`;
      } else {
        renameError.value = toAppError(e).message || '重命名失败';
      }
    } finally {
      renameSubmitting.value = false;
    }
  }

  async function handleChmod(entry: FileEntry) {
    const sid = sftpSessionId.value;
    if (!sid) return;

    const current = entry.permissions != null ? (entry.permissions & 0o7777).toString(8) : '644';
    const input = await prompt({
      title: `修改${entry.is_dir ? '文件夹' : '文件'}权限`,
      message: `正在修改 “${entry.name}” 的八进制权限值`,
      initialValue: current,
      placeholder: '例如 644 或 0755',
      confirmText: '保存',
      validate: (value) => {
        const trimmed = value.trim();
        if (!trimmed) {
          return '请输入权限值';
        }
        if (!/^[0-7]{3,4}$/.test(trimmed)) {
          return '请输入 3 到 4 位八进制权限值';
        }
        return null;
      },
    });
    if (!input) return;

    const mode = parseInt(input.trim(), 8);
    if (Number.isNaN(mode)) {
      notify.addNotification('error', '无效的权限值');
      return;
    }

    try {
      await sftpApi.chmod(sid, entry.path, mode);
      notify.addNotification('success', '权限已修改');
      refresh();
    } catch (e: unknown) {
      notify.addNotification('error', toAppError(e).message);
    }
  }

  function openPopupEditor() {
    const sid = sftpSessionId.value;
    if (!sid) {
      notify.addNotification('warning', '没有活动会话，无法打开编辑器');
      return;
    }

    fileEditorStore.triggerPopup('', sid);
  }

  async function openEditor(entry: FileEntry) {
    const sid = sftpSessionId.value;
    if (!sid) return;

    const popupEditorEnabled = settingsStore.getBoolean('showPopupFileEditor', false);
    const shareFileEditorTabs = settingsStore.getBoolean('shareFileEditorTabs', true);
    const tabScope = connectionId.value != null ? String(connectionId.value) : 'global';
    const tabId = shareFileEditorTabs ? `shared:${tabScope}:${entry.path}` : `${sid}:${entry.path}`;

    if (popupEditorEnabled) {
      fileEditorStore.triggerPopup(entry.path, sid);
    }

    if (fileEditorStore.openFiles.has(tabId)) {
      fileEditorStore.updateFileSession(tabId, sid);
      fileEditorStore.setActive(tabId);
      return;
    }

    try {
      const base64 = await sftpApi.readFile(sid, entry.path);
      const content = decodeUtf8Base64(base64);
      const ext = entry.name.split('.').pop()?.toLowerCase() ?? '';
      const langMap: Record<string, string> = {
        js: 'javascript',
        ts: 'typescript',
        py: 'python',
        rs: 'rust',
        json: 'json',
        yaml: 'yaml',
        yml: 'yaml',
        md: 'markdown',
        html: 'html',
        css: 'css',
        sh: 'shell',
        bash: 'shell',
        xml: 'xml',
        sql: 'sql',
        toml: 'toml',
        conf: 'ini',
        ini: 'ini',
      };

      fileEditorStore.openFile({
        id: tabId,
        sessionId: sid,
        path: entry.path,
        filename: entry.name,
        content,
        originalContent: content,
        rawContentBase64: base64,
        selectedEncoding: 'utf-8',
        isDirty: false,
        isLoading: false,
        loadingError: null,
        isSaving: false,
        saveStatus: 'idle',
        saveError: null,
        scrollTop: 0,
        scrollLeft: 0,
        language: langMap[ext] ?? 'plaintext',
      });

    } catch (e: unknown) {
      notify.addNotification('error', `打开失败: ${toAppError(e).message}`);
    }
  }

  function announceTransfer(taskId: string) {
    window.dispatchEvent(new CustomEvent('transfer-created', { detail: { taskId } }));
  }

  async function downloadFile(entry: FileEntry) {
    const sid = sftpSessionId.value;
    if (!sid) return;

    try {
      const defaultPath = await resolveDownloadDefaultPath(entry.name || 'file');
      const localPath = await save({ defaultPath });
      if (!localPath) return;

      const taskId = await sftpApi.downloadToDisk(sid, entry.path, localPath);
      announceTransfer(taskId);
      notify.addNotification('success', `已开始下载 ${entry.name}`);
    } catch (e: unknown) {
      notify.addNotification('error', `下载失败: ${toAppError(e).message}`);
    }
  }

  async function downloadDirectory(entry: FileEntry) {
    const sid = sftpSessionId.value;
    if (!sid) return;

    try {
      const defaultName = `${entry.name || 'directory'}.zip`;
      const defaultPath = await resolveDownloadDefaultPath(defaultName);
      const localPath = await save({
        defaultPath,
        filters: [{ name: 'Zip Archive', extensions: ['zip'] }],
      });
      if (!localPath) return;

      const taskId = await sftpApi.downloadDirectoryToDisk(sid, entry.path, localPath);
      announceTransfer(taskId);
      notify.addNotification('success', `已开始打包下载 ${entry.name}`);
    } catch (e: unknown) {
      notify.addNotification('error', `目录下载失败: ${toAppError(e).message}`);
    }
  }

  function onSendCreated(taskId: string) {
    showSendFile.value = false;
    announceTransfer(taskId);
  }

  function formatPerms(p: number): string {
    return (p & 0o7777).toString(8).padStart(4, '0');
  }

  async function doMkdir() {
    const sid = sftpSessionId.value;
    const folderName = mkdirName.value.trim();
    if (!sid || mkdirSubmitting.value) return;

    const validationMessage = validateNewEntryName(folderName, '文件夹');
    if (validationMessage) {
      notify.addNotification('warning', validationMessage);
      return;
    }

    const targetPath = joinRemotePath(currentPath.value, folderName);
    mkdirSubmitting.value = true;

    try {
      await sftpApi.mkdir(sid, targetPath);
      showMkdir.value = false;
      mkdirName.value = '';
      await navigateTo(currentPath.value);
    } catch (e: unknown) {
      const existingTarget = await statTargetIfExists(sid, targetPath);
      if (existingTarget?.is_dir) {
        showMkdir.value = false;
        mkdirName.value = '';
        await navigateTo(currentPath.value);
        notify.addNotification('warning', `当前目录已存在同名文件夹：${folderName}`);
        return;
      }
      if (existingTarget) {
        notify.addNotification('error', `创建文件夹失败：当前目录已存在同名文件 ${folderName}`);
        return;
      }
      notify.addNotification('error', toAppError(e).message || '创建文件夹失败');
    } finally {
      mkdirSubmitting.value = false;
    }
  }

  async function doCreateFile() {
    const sid = sftpSessionId.value;
    const fileName = newFileName.value.trim();
    if (!sid || newFileSubmitting.value) return;

    const validationMessage = validateNewEntryName(fileName, '文件');
    if (validationMessage) {
      notify.addNotification('warning', validationMessage);
      return;
    }

    const targetPath = joinRemotePath(currentPath.value, fileName);
    newFileSubmitting.value = true;

    try {
      await sftpApi.writeFile(sid, targetPath, '');
      showNewFile.value = false;
      newFileName.value = '';
      await navigateTo(currentPath.value);
    } catch (e: unknown) {
      const existingTarget = await statTargetIfExists(sid, targetPath);
      if (existingTarget?.is_dir) {
        notify.addNotification('error', `创建文件失败：当前目录已存在同名文件夹 ${fileName}`);
        return;
      }
      if (existingTarget) {
        showNewFile.value = false;
        newFileName.value = '';
        await navigateTo(currentPath.value);
        notify.addNotification('warning', `当前目录已存在同名文件：${fileName}`);
        return;
      }
      notify.addNotification('error', toAppError(e).message || '创建文件失败');
    } finally {
      newFileSubmitting.value = false;
    }
  }

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
    return `${(bytes / 1073741824).toFixed(1)} GB`;
  }

  function formatModifiedTime(modified?: number): string {
    if (!modified) {
      return '';
    }

    const timestamp = modified > 1000000000000 ? modified : modified * 1000;
    const date = new Date(timestamp);
    if (Number.isNaN(date.getTime())) {
      return '';
    }

    return date.toLocaleString();
  }

  function handleDocumentMouseDown(event: MouseEvent): void {
    const target = event.target as Node | null;
    if (!target) {
      return;
    }

    if (showFavoritePathsPopover.value) {
      const clickInFavoriteButton = favoriteButtonRef.value?.contains(target);
      const clickInFavoritePopover = favoritePopoverRef.value?.contains(target);
      if (!clickInFavoriteButton && !clickInFavoritePopover && !isFavoriteDialogOpen.value) {
        showFavoritePathsPopover.value = false;
        isFavoriteDialogOpen.value = false;
      }
    }

    if (isEditingPath.value || showPathHistoryDropdown.value) {
      const clickInPathWrapper = pathInputWrapperRef.value?.contains(target);
      if (!clickInPathWrapper) {
        isEditingPath.value = false;
        closePathHistory();
      }
    }

    if (ctxVisible.value) {
      const clickInCtxMenu = ctxMenuRef.value?.contains(target);
      if (!clickInCtxMenu) {
        closeCtxMenu();
      }
    }
  }

  function handleWindowResize(): void {
    if (showFavoritePathsPopover.value) {
      updateFavoritePopoverPosition();
    }
    if (ctxVisible.value) {
      adjustContextMenuPosition();
    }
  }

  watch(showFavoritePathsPopover, (visible) => {
    if (visible) {
      void nextTick(updateFavoritePopoverPosition);
      return;
    }

    isFavoriteDialogOpen.value = false;
  });

  watch(ctxVisible, (visible) => {
    if (visible) {
      void nextTick(adjustContextMenuPosition);
    }
  });

  async function focusPathInput(): Promise<boolean | undefined> {
    if (!isEditingPath.value) {
      startPathEdit();
      await nextTick();
    }

    if (!isVisibleInput(pathInputRef.value)) {
      return undefined;
    }

    pathInputRef.value.focus();
    pathInputRef.value.select();
    return document.activeElement === pathInputRef.value;
  }

  async function focusSearchInput(): Promise<boolean | undefined> {
    if (!isSearchActive.value) {
      activateSearch();
      await nextTick();
    }

    if (!isVisibleInput(searchInputRef.value)) {
      return undefined;
    }

    searchInputRef.value.focus();
    searchInputRef.value.select();
    return document.activeElement === searchInputRef.value;
  }

  watch(sortKey, () => {
    clearSelection();
  });

  watch(sortDirection, () => {
    clearSelection();
  });

  onMounted(() => {
    void settingsStore.loadAll().catch(() => undefined);
    const unregisterSearch = focusSwitcherStore.registerFocusAction('fileManagerSearch', focusSearchInput);
    const unregisterPathInput = focusSwitcherStore.registerFocusAction('fileManagerPathInput', focusPathInput);
    document.addEventListener('mousedown', handleDocumentMouseDown);
    window.addEventListener('resize', handleWindowResize);
    void listen<TransferStatusEvent>('transfers/status', (event) => {
      handleTransferStatusEvent(event.payload);
    }).then((unlisten) => {
      unlistenTransferStatus = unlisten;
    });
    onUnmounted(() => {
      unregisterSearch();
      unregisterPathInput();
      document.removeEventListener('mousedown', handleDocumentMouseDown);
      window.removeEventListener('resize', handleWindowResize);
      unlistenTransferStatus?.();
      unlistenTransferStatus = null;
      if (uploadRefreshTimer) {
        clearTimeout(uploadRefreshTimer);
        uploadRefreshTimer = null;
      }
      pendingUploadRefreshes.clear();
    });
  });

  watch(
    sshSessionId,
    async (newSid) => {
      showUpload.value = false;
      showSendFile.value = false;
      sendFileTarget.value = null;
      resetBrowserState();

      if (!newSid || connectionId.value == null) {
        return;
      }

      try {
        await ensureSftpSession(newSid);
        const targetPath = sessionStore.getSession(newSid)?.currentPath || '/';
        await navigateTo(targetPath);
      } catch (e: unknown) {
        error.value = toAppError(e).message || 'SFTP 初始化失败';
      }
    },
    { immediate: true },
  );

  return {
    entries,
    currentPath,
    pathInput,
    loading,
    error,
    searchQuery,
    selectedEntryPaths,
    lastSelectedPath,
    clipboardState,
    rootModeEnabled,
    rootModeSwitching,
    showRootModeDialog,
    rootModeUsername,
    rootModePassword,
    pathHistoryItems,
    pathHistoryLoading,
    pathHistorySelectedIndex,
    isSearchActive,
    isEditingPath,
    showPathHistoryDropdown,
    ctxVisible,
    ctxPos,
    ctxNearRight,
    ctxSubmenuKey,
    ctxEntry,
    showFavoritePathsPopover,
    isFavoriteDialogOpen,
    favoritePopoverStyle,
    showUpload,
    showMkdir,
    mkdirName,
    mkdirSubmitting,
    showRename,
    renameName,
    renameSubmitting,
    renameError,
    renameTargetEntry,
    showNewFile,
    newFileName,
    newFileSubmitting,
    showSendFile,
    sendFileTarget,
    defaultDownloadDir,
    filteredEntries,
    sortKey,
    sortDirection,
    filteredPathHistory,
    selectedEntries,
    contextMenuItems,
    fileManagerShowDeleteConfirmation,
    showPopupFileEditor,
    pathInputRef,
    pathInputWrapperRef,
    searchInputRef,
    renameInputRef,
    ctxMenuRef,
    favoriteButtonRef,
    favoritePopoverRef,
    isVisibleInput,
    normalizePath,
    joinRemotePath,
    joinLocalPath,
    resolveDownloadDefaultPath,
    getPathName,
    getPathDir,
    splitNameExt,
    buildDuplicateName,
    isArchiveFile,
    resetBrowserState,
    ensureSftpSession,
    navigateTo,
    goUp,
    refresh,
    handleRootModeButtonClick,
    enableRootMode,
    disableRootMode,
    closeRootModeDialog,
    addPathToHistory,
    loadPathHistory,
    openPathHistory,
    closePathHistory,
    activateSearch,
    deactivateSearch,
    cancelSearch,
    updateFavoritePopoverPosition,
    toggleFavoritePopover,
    navigateFromFavorite,
    handleFavoriteDialogVisibility,
    startPathEdit,
    handlePathInputFocus,
    handlePathInputChange,
    applyPathFromInput,
    handlePathInputKeydown,
    handlePathInputBlur,
    selectPathHistory,
    sendCdCommandToTerminal,
    isEntrySelected,
    clearSelection,
    setSingleSelection,
    toggleEntrySelection,
    selectRangeTo,
    handleEntryClick,
    closeCtxMenu,
    adjustContextMenuPosition,
    showCtx,
    handleEntryContextMenu,
    runContextAction,
    getContextEntries,
    buildCompressSubmenu,
    handleCtxItemClick,
    queueClipboardAction,
    cloneEntryRecursive,
    pasteFromClipboard,
    copyContextPath,
    triggerCompress,
    triggerDecompress,
    downloadMultipleFiles,
    handleDeleteEntries,
    openUpload,
    toggleSort,
    uploadLocalPaths,
    moveEntriesToDirectory,
    openSendModal,
    handleDelete,
    handleRename,
    closeRenameDialog,
    submitRename,
    handleChmod,
    openPopupEditor,
    openEditor,
    announceTransfer,
    registerUploadTasksForRefresh,
    downloadFile,
    downloadDirectory,
    onSendCreated,
    doMkdir,
    doCreateFile,
    formatSize,
    formatModifiedTime,
    formatPerms,
    handleDocumentMouseDown,
    handleWindowResize,
    focusPathInput,
    focusSearchInput,
  };
}

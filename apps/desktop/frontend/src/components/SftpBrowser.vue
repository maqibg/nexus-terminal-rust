<template>
  <div class="sftp-browser">
    <template v-if="!sshSessionId || !isSshSession">
      <div class="sftp-placeholder">
        <i class="fas fa-folder-open placeholder-icon"></i>
        <span class="placeholder-text">当前会话不支持文件管理</span>
      </div>
    </template>
    <template v-else>
      <div class="toolbar">
        <div class="toolbar-main">
          <div class="toolbar-icon-group">
            <button
              class="icon-btn"
              @click="sendCdCommandToTerminal"
              :disabled="!activeSftpSessionId || isEditingPath"
              title="切换终端目录到当前路径"
            >
              <i class="fas fa-terminal"></i>
            </button>
            <button
              class="icon-btn"
              @click="refresh"
              :disabled="!activeSftpSessionId || isEditingPath"
              title="刷新"
            >
              <i class="fas fa-sync-alt"></i>
            </button>
            <button
              class="icon-btn"
              @click="goUp"
              :disabled="!activeSftpSessionId || currentPath === '/' || isEditingPath"
              title="上级目录"
            >
              <i class="fas fa-arrow-up"></i>
            </button>

            <div class="search-zone">
              <button
                v-if="!isSearchActive"
                class="icon-btn"
                @click.stop="activateSearch"
                :disabled="!activeSftpSessionId"
                title="搜索文件"
              >
                <i class="fas fa-search"></i>
              </button>
              <div v-else class="search-inline">
                <i class="fas fa-search search-inline-icon"></i>
                <input
                  ref="searchInputRef"
                  v-model="searchQuery"
                  class="search-input"
                  data-focus-id="fileManagerSearch"
                  placeholder="搜索文件..."
                  @blur="deactivateSearch"
                  @keydown.esc.prevent="cancelSearch"
                />
              </div>
            </div>

            <div class="favorite-zone">
              <button
                ref="favoriteButtonRef"
                class="icon-btn"
                @click.stop="toggleFavoritePopover"
                title="收藏路径"
              >
                <i class="fas fa-star"></i>
              </button>
            </div>
          </div>

          <div
            ref="pathInputWrapperRef"
            class="path-wrapper"
            :class="{ 'path-wrapper-expanded': isEditingPath || showPathHistoryDropdown }"
          >
            <span v-show="!isEditingPath && !showPathHistoryDropdown" class="path-label" @click="startPathEdit">
              <strong title="编辑当前路径">{{ currentPath }}</strong>
            </span>
            <input
              v-show="isEditingPath || showPathHistoryDropdown"
              ref="pathInputRef"
              v-model="pathInput"
              class="path-input"
              data-focus-id="fileManagerPathInput"
              placeholder="/"
              @focus="handlePathInputFocus"
              @input="handlePathInputChange"
              @keydown="handlePathInputKeydown"
              @blur="handlePathInputBlur"
            />

            <div v-if="showPathHistoryDropdown" class="path-history-dropdown">
              <div v-if="pathHistoryLoading && !filteredPathHistory.length" class="path-history-status">
                <i class="fas fa-spinner fa-spin"></i>
                <span>加载路径历史...</span>
              </div>

              <button
                v-for="(item, index) in filteredPathHistory"
                :key="item.id"
                class="path-history-item"
                :class="{ 'is-active': index === pathHistorySelectedIndex }"
                :title="item.path"
                @mousedown.prevent
                @click="selectPathHistory(item.path)"
              >
                <span>{{ item.path }}</span>
              </button>

              <div v-if="!pathHistoryLoading && !filteredPathHistory.length" class="path-history-status">
                <i class="fas fa-history"></i>
                <span>没有路径历史记录</span>
              </div>
            </div>
          </div>
        </div>

        <div class="toolbar-actions">
          <button
            v-if="showPopupFileEditor"
            class="action-btn"
            @click="openPopupEditor"
            :disabled="!activeSftpSessionId"
            title="打开弹窗编辑器"
          >
            <i class="far fa-edit"></i>
            <span>打开编辑器</span>
          </button>
          <button
            class="action-btn"
            @click="openUpload"
            :disabled="!activeSftpSessionId"
            title="上传文件"
          >
            <i class="fas fa-upload"></i>
            <span>上传</span>
          </button>
          <button
            class="action-btn"
            @click="showMkdir = true"
            :disabled="!activeSftpSessionId"
            title="新建文件夹"
          >
            <i class="fas fa-folder-plus"></i>
            <span>新建文件夹</span>
          </button>
          <button
            class="action-btn"
            @click="showNewFile = true"
            :disabled="!activeSftpSessionId"
            title="新建文件"
          >
            <i class="far fa-file-alt"></i>
            <span>新建文件</span>
          </button>
        </div>
      </div>

      <Teleport to="body">
        <div
          v-if="showFavoritePathsPopover"
          ref="favoritePopoverRef"
          class="favorite-popover"
          :style="favoritePopoverStyle"
        >
          <FavoritePaths
            :connection-id="connectionId"
            @navigate="navigateFromFavorite"
            @close="showFavoritePathsPopover = false"
            @modal-visibility-change="handleFavoriteDialogVisibility"
          />
        </div>
      </Teleport>

      <div v-if="loading" class="status-msg">
        <i class="fas fa-spinner fa-spin"></i>
        <span>加载中...</span>
      </div>
      <div v-else-if="error" class="status-msg error">
        <i class="fas fa-exclamation-triangle"></i>
        <span>{{ error }}</span>
      </div>
      <div v-else class="file-list" @click.self="clearSelection" @contextmenu.prevent="showCtx($event, null)">
        <div class="file-header">
          <span class="fh-icon">类型</span>
          <span class="fh-name">名称</span>
          <span class="fh-size">大小</span>
          <span class="fh-perms">权限</span>
          <span class="fh-modified">修改时间</span>
        </div>
        <div
          v-for="entry in filteredEntries"
          :key="entry.path"
          class="file-item"
          :class="{ dir: entry.is_dir, selected: isEntrySelected(entry) }"
          @click.stop="handleEntryClick(entry, $event)"
          @contextmenu.prevent.stop="handleEntryContextMenu($event, entry)"
        >
          <span class="file-icon">
            <i v-if="entry.is_dir" class="fas fa-folder folder-color"></i>
            <i v-else class="fas fa-file file-color"></i>
          </span>
          <span class="file-name">{{ entry.name }}</span>
          <span class="file-size">{{ entry.is_dir ? '' : formatSize(entry.size) }}</span>
          <span class="file-perms">{{ entry.permissions != null ? formatPerms(entry.permissions) : '' }}</span>
          <span class="file-modified">{{ formatModifiedTime(entry.modified) }}</span>
        </div>
        <div v-if="!filteredEntries.length" class="empty-dir">
          <i class="fas fa-folder-open"></i>
          <span>{{ searchQuery.trim() ? '没有匹配的文件或目录' : '空目录' }}</span>
        </div>
      </div>

      <Teleport to="body">
        <div
          v-if="ctxVisible"
          ref="ctxMenuRef"
          class="ctx-menu"
          :class="{ 'menu-near-right': ctxNearRight }"
          :style="{ left: `${ctxPos.x}px`, top: `${ctxPos.y}px` }"
          @mouseleave="ctxSubmenuKey = null"
        >
          <template v-for="item in contextMenuItems" :key="item.key">
            <div v-if="item.separator" class="ctx-divider"></div>
            <div
              v-else
              class="ctx-item"
              :class="{ danger: item.danger, disabled: item.disabled, 'ctx-item-submenu': item.submenu && item.submenu.length > 0 }"
              @mouseenter="item.submenu && item.submenu.length ? (ctxSubmenuKey = item.key) : null"
              @click.stop="handleCtxItemClick(item)"
            >
              <i v-if="item.icon" :class="[item.icon, 'ctx-icon']"></i>
              <span class="ctx-label">{{ item.label }}</span>
              <i v-if="item.submenu && item.submenu.length" class="fas fa-chevron-right ctx-submenu-arrow"></i>

              <div
                v-if="item.submenu && item.submenu.length && ctxSubmenuKey === item.key"
                class="ctx-submenu"
                @mouseenter="ctxSubmenuKey = item.key"
                @mouseleave="ctxSubmenuKey = null"
              >
                <div
                  v-for="subItem in item.submenu"
                  :key="`${item.key}-${subItem.key}`"
                  class="ctx-item"
                  :class="{ danger: subItem.danger, disabled: subItem.disabled }"
                  @click.stop="handleCtxItemClick(subItem, true)"
                >
                  <i v-if="subItem.icon" :class="[subItem.icon, 'ctx-icon']"></i>
                  <span class="ctx-label">{{ subItem.label }}</span>
                </div>
              </div>
            </div>
          </template>
        </div>
      </Teleport>

      <div v-if="showMkdir" class="mini-dialog-backdrop" @click.self="showMkdir = false">
        <div class="mini-dialog">
          <div class="mini-dialog-title">
            <i class="fas fa-folder-plus"></i>
            <span>新建文件夹</span>
          </div>
          <input class="mini-dialog-input" v-model="mkdirName" @keydown.enter="doMkdir" placeholder="输入文件夹名称..." />
          <div class="mini-actions">
            <button class="btn-cancel" @click="showMkdir = false">取消</button>
            <button class="btn-save" @click="doMkdir">创建</button>
          </div>
        </div>
      </div>

      <div v-if="showNewFile" class="mini-dialog-backdrop" @click.self="showNewFile = false">
        <div class="mini-dialog">
          <div class="mini-dialog-title">
            <i class="far fa-file-alt"></i>
            <span>新建文件</span>
          </div>
          <input class="mini-dialog-input" v-model="newFileName" @keydown.enter="doCreateFile" placeholder="输入文件名称..." />
          <div class="mini-actions">
            <button class="btn-cancel" @click="showNewFile = false">取消</button>
            <button class="btn-save" @click="doCreateFile">创建</button>
          </div>
        </div>
      </div>

      <FileUploadPopup
        :visible="showUpload"
        :session-id="activeSftpSessionId || ''"
        :remote-path="currentPath"
        @uploaded="showUpload = false; refresh()"
        @cancel="showUpload = false"
      />

      <SendFilesModal
        :visible="showSendFile"
        :session-id="activeSftpSessionId || ''"
        :current-file="sendFileTarget ?? undefined"
        @cancel="showSendFile = false"
        @sent="onSendCreated"
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { pathHistoryApi, sftpApi, sshApi, type FileEntry, type PathHistory } from '@/lib/api';
import { save } from '@tauri-apps/plugin-dialog';
import { useFileEditorStore } from '@/stores/fileEditor';
import { useUINotificationStore } from '@/stores/uiNotifications';
import { useSessionStore } from '@/stores/session';
import { useSettingsStore } from '@/stores/settings';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import FavoritePaths from '@/components/FavoritePaths.vue';
import FileUploadPopup from '@/components/FileUploadPopup.vue';
import SendFilesModal from '@/components/SendFilesModal.vue';

const sessionStore = useSessionStore();
const settingsStore = useSettingsStore();
const { activeSessionId: sshSessionId, activeSession } = storeToRefs(sessionStore);
const isSshSession = computed(() => activeSession.value?.protocol === 'SSH');
const connectionId = computed(() => (activeSession.value?.protocol === 'SSH' ? activeSession.value.connectionId : undefined));
const activeSftpSessionId = computed(() => (activeSession.value?.protocol === 'SSH' ? activeSession.value?.sftpSessionId ?? null : null));

const fileEditorStore = useFileEditorStore();
const notify = useUINotificationStore();
const focusSwitcherStore = useFocusSwitcherStore();

const currentPath = ref('/');
const pathInput = ref('/');
const pathInputRef = ref<HTMLInputElement>();
const pathInputWrapperRef = ref<HTMLDivElement>();

const searchQuery = ref('');
const searchInputRef = ref<HTMLInputElement>();
const isSearchActive = ref(false);

const entries = ref<FileEntry[]>([]);
const filteredEntries = computed(() => {
  const query = searchQuery.value.trim().toLowerCase();
  if (!query) {
    return entries.value;
  }
  return entries.value.filter((entry) => entry.name.toLowerCase().includes(query));
});

const loading = ref(false);
const error = ref('');
const showUpload = ref(false);
const showSendFile = ref(false);
const sendFileTarget = ref<string | null>(null);
const fileManagerShowDeleteConfirmation = computed(() => settingsStore.getBoolean('fileManagerShowDeleteConfirmation', true));
const showPopupFileEditor = computed(() => settingsStore.getBoolean('showPopupFileEditor', false));

interface BrowserContextMenuItem {
  key: string;
  label?: string;
  icon?: string;
  danger?: boolean;
  disabled?: boolean;
  separator?: boolean;
  submenu?: BrowserContextMenuItem[];
  onClick?: () => void | Promise<void>;
}

interface BrowserClipboardState {
  operation: 'copy' | 'cut';
  sessionId: string;
  entries: FileEntry[];
}

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
const showNewFile = ref(false);
const newFileName = ref('');

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

const VIEWPORT_PADDING = 8;
const CONTEXT_SUBMENU_MIN_WIDTH = 190;

let unregisterFileManagerSearch: (() => void) | null = null;
let unregisterFileManagerPathInput: (() => void) | null = null;

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
  mkdirName.value = '';
  newFileName.value = '';
  ctxVisible.value = false;
  ctxEntry.value = null;
  ctxSubmenuKey.value = null;
  clipboardState.value = null;
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
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  const targetPath = normalizePath(path);
  const previousPath = currentPath.value;

  loading.value = true;
  error.value = '';
  try {
    const list = await sftpApi.listDir(sid, targetPath);
    entries.value = list.sort((a, b) => {
      if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
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
  } catch (e: any) {
    error.value = e.message || '加载失败';
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
  if (!activeSftpSessionId.value) {
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
  } catch (e: any) {
    notify.addNotification('error', `发送目录切换命令失败: ${e.message || String(e)}`);
  }
}

const selectedEntries = computed<FileEntry[]>(() =>
  entries.value.filter((entry) => selectedEntryPaths.value.has(entry.path)),
);

const SUPPORTED_ARCHIVE_EXTENSIONS = ['.zip', '.tar.gz', '.tgz', '.tar.bz2', '.tbz2'];

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
  closeCtxMenu();
  void Promise.resolve(action());
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
  const canUseSftp = !!activeSftpSessionId.value;
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
      onClick: () => queueClipboardAction('cut'),
    });
    items.push({
      key: 'copy-multi',
      label: '复制',
      icon: 'fas fa-copy',
      disabled: !canUseSftp,
      onClick: () => queueClipboardAction('copy'),
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
      onClick: () => queueClipboardAction('cut'),
    });
    items.push({
      key: 'copy',
      label: '复制',
      icon: 'fas fa-copy',
      disabled: !canUseSftp,
      onClick: () => queueClipboardAction('copy'),
    });

    if (entry.is_dir) {
      items.push({
        key: 'paste',
        label: '粘贴',
        icon: 'fas fa-paste',
        disabled: !canUseSftp || !hasClipboard,
        onClick: pasteFromClipboard,
      });
    }

    items.push({
      key: 'copy-path',
      label: '复制路径',
      icon: 'fas fa-link',
      disabled: !canUseSftp,
      onClick: copyContextPath,
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
      onClick: pasteFromClipboard,
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
    onClick: pasteFromClipboard,
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

function queueClipboardAction(operation: 'copy' | 'cut'): void {
  const sid = activeSftpSessionId.value;
  const targetEntries = getContextEntries();
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
  if (source.is_dir) {
    await sftpApi.mkdir(sessionId, targetPath);
    const children = await sftpApi.listDir(sessionId, source.path);
    for (const child of children) {
      if (child.name === '.' || child.name === '..') {
        continue;
      }
      const childTargetPath = joinRemotePath(targetPath, child.name);
      await cloneEntryRecursive(sessionId, child, childTargetPath);
    }
    return;
  }

  const content = await sftpApi.readFile(sessionId, source.path);
  await sftpApi.writeFile(sessionId, targetPath, content);
}

async function pasteFromClipboard(): Promise<void> {
  const sid = activeSftpSessionId.value;
  const clipboard = clipboardState.value;
  if (!sid || !clipboard || !clipboard.entries.length) {
    return;
  }

  if (clipboard.sessionId !== sid) {
    notify.addNotification('warning', '暂不支持跨会话粘贴');
    return;
  }

  const targetDir = ctxEntry.value?.is_dir ? ctxEntry.value.path : currentPath.value;

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
  } catch (e: any) {
    notify.addNotification('error', e.message || '粘贴失败');
  }
}

async function copyContextPath(): Promise<void> {
  const entry = ctxEntry.value;
  if (!entry) {
    return;
  }

  try {
    await navigator.clipboard.writeText(entry.path);
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
  const sid = activeSftpSessionId.value;
  if (!sid || !targetEntries.length) {
    return;
  }

  const label = targetEntries.length === 1 ? `"${targetEntries[0].name}"` : `${targetEntries.length} 项`;
  if (fileManagerShowDeleteConfirmation.value && !confirm(`确定删除 ${label} 吗？`)) {
    return;
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
  if (!activeSftpSessionId.value) {
    notify.addNotification('error', 'SFTP 未就绪，无法上传');
    return;
  }
  showUpload.value = true;
}

function openSendModal(path?: string) {
  if (!activeSftpSessionId.value) {
    notify.addNotification('error', 'SFTP 未就绪，无法发起传输');
    return;
  }
  sendFileTarget.value = path ?? null;
  showSendFile.value = true;
}

async function handleDelete(entry: FileEntry) {
  await handleDeleteEntries([entry]);
}

async function handleRename(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  const newName = prompt('新名称:', entry.name);
  if (!newName || newName === entry.name) return;

  const nextPath = joinRemotePath(currentPath.value, newName);
  try {
    await sftpApi.rename(sid, entry.path, nextPath);
    refresh();
  } catch (e: any) {
    error.value = e.message;
  }
}

async function handleChmod(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  const current = entry.permissions != null ? (entry.permissions & 0o7777).toString(8) : '644';
  const input = prompt('权限 (八进制):', current);
  if (!input) return;

  const mode = parseInt(input, 8);
  if (Number.isNaN(mode)) {
    notify.addNotification('error', '无效的权限值');
    return;
  }

  try {
    await sftpApi.chmod(sid, entry.path, mode);
    notify.addNotification('success', '权限已修改');
    refresh();
  } catch (e: any) {
    notify.addNotification('error', e.message);
  }
}

function openPopupEditor() {
  const sid = activeSftpSessionId.value;
  if (!sid) {
    notify.addNotification('warning', '没有活动会话，无法打开编辑器');
    return;
  }

  fileEditorStore.triggerPopup('', sid);
}

async function openEditor(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
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
    const content = decodeURIComponent(escape(atob(base64)));
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

  } catch (e: any) {
    notify.addNotification('error', `打开失败: ${e.message}`);
  }
}

function announceTransfer(taskId: string) {
  window.dispatchEvent(new CustomEvent('transfer-created', { detail: { taskId } }));
}

async function downloadFile(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  try {
    const localPath = await save({ defaultPath: entry.name });
    if (!localPath) return;

    const taskId = await sftpApi.downloadToDisk(sid, entry.path, localPath);
    announceTransfer(taskId);
    notify.addNotification('success', `已开始下载 ${entry.name}`);
  } catch (e: any) {
    notify.addNotification('error', `下载失败: ${e.message}`);
  }
}

async function downloadDirectory(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  try {
    const defaultName = `${entry.name || 'directory'}.zip`;
    const localPath = await save({
      defaultPath: defaultName,
      filters: [{ name: 'Zip Archive', extensions: ['zip'] }],
    });
    if (!localPath) return;

    const taskId = await sftpApi.downloadDirectoryToDisk(sid, entry.path, localPath);
    announceTransfer(taskId);
    notify.addNotification('success', `已开始打包下载 ${entry.name}`);
  } catch (e: any) {
    notify.addNotification('error', `目录下载失败: ${e.message}`);
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
  const sid = activeSftpSessionId.value;
  const folderName = mkdirName.value.trim();
  if (!sid || !folderName) return;

  try {
    await sftpApi.mkdir(sid, joinRemotePath(currentPath.value, folderName));
    showMkdir.value = false;
    mkdirName.value = '';
    refresh();
  } catch (e: any) {
    notify.addNotification('error', e.message || '创建文件夹失败');
  }
}

async function doCreateFile() {
  const sid = activeSftpSessionId.value;
  const fileName = newFileName.value.trim();
  if (!sid || !fileName) return;

  try {
    await sftpApi.writeFile(sid, joinRemotePath(currentPath.value, fileName), '');
    showNewFile.value = false;
    newFileName.value = '';
    refresh();
  } catch (e: any) {
    notify.addNotification('error', e.message || '创建文件失败');
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

onMounted(() => {
  void settingsStore.loadAll().catch(() => undefined);
  unregisterFileManagerSearch = focusSwitcherStore.registerFocusAction('fileManagerSearch', focusSearchInput);
  unregisterFileManagerPathInput = focusSwitcherStore.registerFocusAction('fileManagerPathInput', focusPathInput);
  document.addEventListener('mousedown', handleDocumentMouseDown);
  window.addEventListener('resize', handleWindowResize);
});

onUnmounted(() => {
  unregisterFileManagerSearch?.();
  unregisterFileManagerPathInput?.();
  unregisterFileManagerSearch = null;
  unregisterFileManagerPathInput = null;
  document.removeEventListener('mousedown', handleDocumentMouseDown);
  window.removeEventListener('resize', handleWindowResize);
});

watch(
  sshSessionId,
  async (newSid) => {
    showUpload.value = false;
    showSendFile.value = false;
    sendFileTarget.value = null;
    resetBrowserState();

    if (!newSid || !isSshSession.value) {
      return;
    }

    try {
      await ensureSftpSession(newSid);
      const targetPath = sessionStore.getSession(newSid)?.currentPath || '/';
      await navigateTo(targetPath);
    } catch (e: any) {
      error.value = e.message || 'SFTP 初始化失败';
    }
  },
  { immediate: true },
);
</script>

<style scoped>
.sftp-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  overflow: hidden;
}

.sftp-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 100%;
  gap: 10px;
  color: var(--text-dim, #6c7086);
}

.placeholder-icon {
  font-size: 28px;
  opacity: 0.5;
}

.placeholder-text {
  font-size: 13px;
}

.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 8px;
  padding: 8px;
  border-bottom: 1px solid var(--border, #313244);
  background: var(--bg-mantle, #181825);
  flex-shrink: 0;
}

.toolbar-main {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.toolbar-icon-group {
  display: flex;
  align-items: center;
  flex-shrink: 0;
}

.icon-btn {
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  transition: background 0.15s, color 0.15s, opacity 0.15s;
}

.icon-btn:hover {
  background: rgba(255, 255, 255, 0.08);
  color: var(--text, #cdd6f4);
}

.icon-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.icon-btn:disabled:hover {
  background: transparent;
  color: var(--text-sub, #a6adc8);
}

.search-zone {
  display: flex;
  align-items: center;
}

.search-inline {
  position: relative;
  width: 180px;
  height: 28px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: var(--bg-base, #1e1e2e);
  overflow: hidden;
}

.search-inline-icon {
  position: absolute;
  left: 8px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-sub, #a6adc8);
  font-size: 12px;
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text, #cdd6f4);
  font-size: 13px;
  padding: 0 8px 0 24px;
}

.search-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.search-input:focus {
  outline: none;
}

.favorite-zone {
  position: relative;
}

.favorite-popover {
  position: fixed;
  width: min(320px, 72vw);
  max-height: 320px;
  overflow: hidden;
  background: var(--bg-surface0, #313244);
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.45);
  z-index: 3300;
}

.favorite-popover :deep(.favorite-paths-dropdown) {
  max-height: 320px;
}

.path-wrapper {
  position: relative;
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 180px;
  min-height: 30px;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: var(--bg-base, #1e1e2e);
  padding: 2px 6px;
  overflow: visible;
}

.path-wrapper-expanded {
  flex: 1;
}

.path-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: text;
  color: var(--text-sub, #a6adc8);
  padding-right: 8px;
}

.path-label strong {
  font-weight: 500;
  color: var(--blue, #89b4fa);
  border-radius: 4px;
  padding: 1px 3px;
  transition: background 0.15s;
}

.path-label strong:hover {
  background: rgba(137, 180, 250, 0.15);
}

.path-input {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--text, #cdd6f4);
  font-size: 12px;
  min-width: 80px;
  font-family: 'Cascadia Mono', 'Consolas', 'SFMono-Regular', monospace;
}

.path-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.path-input:focus {
  outline: none;
}

.path-history-dropdown {
  position: absolute;
  left: 0;
  right: 0;
  top: calc(100% + 5px);
  border: 1px solid var(--border, #45475a);
  border-radius: 8px;
  background: var(--bg-surface0, #313244);
  box-shadow: 0 10px 24px rgba(0, 0, 0, 0.45);
  max-height: 220px;
  overflow-y: auto;
  z-index: 150;
}

.path-history-status {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 10px 12px;
  font-size: 12px;
  color: var(--text-sub, #a6adc8);
}

.path-history-item {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--text, #cdd6f4);
  display: flex;
  align-items: center;
  text-align: left;
  padding: 6px 10px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.12s;
}

.path-history-item span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.path-history-item:hover,
.path-history-item.is-active {
  background: rgba(137, 180, 250, 0.16);
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-shrink: 0;
}

.action-btn {
  display: flex;
  align-items: center;
  gap: 5px;
  border-radius: 6px;
  border: 1px solid var(--border, #45475a);
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  padding: 4px 10px;
  font-size: 12px;
  cursor: pointer;
  transition: background 0.15s, border-color 0.15s, color 0.15s;
}
.action-btn:hover {
  background: var(--bg-surface0, #313244);
  border-color: var(--blue, #89b4fa);
  color: var(--blue, #89b4fa);
}

.action-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

.action-btn:disabled:hover {
  background: var(--bg-base, #1e1e2e);
  border-color: var(--border, #45475a);
  color: var(--text, #cdd6f4);
}

.action-btn span {
  white-space: nowrap;
}

.status-msg {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px;
  color: var(--text-dim, #6c7086);
  font-size: 12px;
}

.status-msg.error {
  color: var(--red, #f38ba8);
}

.status-msg i {
  font-size: 14px;
}

.file-list {
  flex: 1;
  overflow-y: auto;
  font-family: 'Segoe UI Variable Text', 'Segoe UI', 'Microsoft YaHei UI', 'Microsoft YaHei', sans-serif;
  text-rendering: geometricPrecision;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.file-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 7px 12px;
  font-size: 12px;
  font-weight: 700;
  color: #d3dcf8;
  border-bottom: 1px solid var(--border, #313244);
  background: var(--bg-mantle, #181825);
  position: sticky;
  top: 0;
  z-index: 1;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.fh-icon {
  width: 44px;
  flex-shrink: 0;
  text-align: center;
}

.fh-name {
  flex: 1;
}

.fh-size {
  min-width: 84px;
  text-align: right;
  flex-shrink: 0;
}

.fh-perms {
  min-width: 68px;
  flex-shrink: 0;
  font-family: monospace;
  text-align: center;
}

.fh-modified {
  min-width: 142px;
  flex-shrink: 0;
}

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  font-size: 13px;
  line-height: 1.35;
  cursor: default;
  border-bottom: 1px solid rgba(49, 50, 68, 0.5);
  transition: background 0.12s;
}

.file-item:hover {
  background: rgba(137, 180, 250, 0.08);
}

.file-item.dir {
  cursor: pointer;
}

.file-item.dir:hover {
  background: rgba(137, 180, 250, 0.12);
}

.file-item.selected {
  background: rgba(137, 180, 250, 0.24);
}

.file-item.selected:hover {
  background: rgba(137, 180, 250, 0.3);
}

.file-item.selected .file-size,
.file-item.selected .file-perms,
.file-item.selected .file-modified {
  color: var(--text, #cdd6f4);
}

.file-icon {
  font-size: 14px;
  flex-shrink: 0;
  width: 44px;
  text-align: center;
}

.folder-color {
  color: var(--yellow, #f9e2af);
}

.file-color {
  color: var(--text-dim, #6c7086);
}

.file-name {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: #e5ecff;
  font-size: 13px;
  font-weight: 550;
}

.file-item.dir .file-name {
  font-weight: 500;
}

.file-size {
  color: #aeb8d8;
  font-size: 12px;
  flex-shrink: 0;
  min-width: 84px;
  text-align: right;
  font-family: 'Cascadia Mono', 'Consolas', monospace;
}

.file-perms {
  color: #aeb8d8;
  font-size: 12px;
  font-family: 'Cascadia Mono', 'Consolas', monospace;
  flex-shrink: 0;
  min-width: 68px;
  text-align: center;
}

.file-modified {
  color: #aeb8d8;
  font-size: 12px;
  flex-shrink: 0;
  min-width: 142px;
  text-align: left;
  white-space: nowrap;
}

.empty-dir {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 20px;
  color: var(--text-dim, #6c7086);
  font-size: 12px;
  gap: 8px;
}

.empty-dir i {
  font-size: 24px;
  opacity: 0.5;
}

.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 3190;
}

.ctx-menu {
  position: fixed;
  z-index: 3200;
  background: var(--bg-surface0, #313244);
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 2px 8px rgba(0, 0, 0, 0.3);
  min-width: 160px;
  border: 1px solid var(--border, #45475a);
}

.ctx-item {
  position: relative;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text, #cdd6f4);
  transition: background 0.12s;
}

.ctx-item:hover {
  background: rgba(137, 180, 250, 0.12);
}

.ctx-item.disabled {
  opacity: 0.45;
  cursor: not-allowed;
  pointer-events: none;
}

.ctx-item.danger {
  color: var(--red, #f38ba8);
}

.ctx-item.danger:hover {
  background: rgba(243, 139, 168, 0.12);
}

.ctx-item-submenu {
  justify-content: space-between;
}

.ctx-label {
  flex: 1;
  white-space: nowrap;
}

.ctx-submenu-arrow {
  font-size: 10px;
  opacity: 0.65;
}

.ctx-submenu {
  position: absolute;
  top: -4px;
  left: calc(100% + 6px);
  background: var(--bg-surface0, #313244);
  border-radius: 8px;
  padding: 4px;
  border: 1px solid var(--border, #45475a);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.45);
  min-width: 180px;
  z-index: 102;
}

.ctx-menu.menu-near-right .ctx-submenu {
  left: auto;
  right: calc(100% + 6px);
}

.ctx-submenu .ctx-item {
  margin: 0;
}

.ctx-icon {
  width: 14px;
  text-align: center;
  font-size: 11px;
  opacity: 0.7;
}

.ctx-divider {
  height: 1px;
  background: var(--border, #45475a);
  margin: 3px 8px;
}

.mini-dialog-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.mini-dialog {
  background: var(--bg-surface0, #313244);
  border-radius: 10px;
  padding: 16px 20px;
  min-width: 320px;
  border: 1px solid var(--border, #45475a);
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
}

.mini-dialog-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  font-weight: 600;
  color: var(--text, #cdd6f4);
  margin-bottom: 12px;
}

.mini-dialog-title i {
  color: var(--blue, #89b4fa);
}

.mini-dialog-input {
  width: 100%;
  padding: 8px 12px;
  border-radius: 6px;
  border: 1px solid var(--border, #45475a);
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  font-size: 13px;
  outline: none;
  box-sizing: border-box;
  transition: border-color 0.2s, box-shadow 0.2s;
}

.mini-dialog-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

.mini-dialog-input::placeholder {
  color: var(--text-dim, #6c7086);
}

.mini-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 14px;
}

.btn-cancel {
  padding: 6px 14px;
  border-radius: 6px;
  border: 1px solid var(--border, #45475a);
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  font-size: 12px;
  transition: all 0.15s;
}

.btn-cancel:hover {
  background: var(--bg-surface1, #45475a);
  color: var(--text, #cdd6f4);
}

.btn-save {
  padding: 6px 14px;
  border-radius: 6px;
  border: none;
  background: var(--blue, #89b4fa);
  color: var(--bg-base, #1e1e2e);
  cursor: pointer;
  font-weight: 600;
  font-size: 12px;
  transition: opacity 0.15s;
}

.btn-save:hover {
  opacity: 0.85;
}

@media (max-width: 1040px) {
  .toolbar {
    gap: 6px;
  }

  .action-btn {
    padding: 4px 8px;
  }

  .action-btn span {
    display: none;
  }
}

@media (max-width: 760px) {
  .toolbar {
    flex-direction: column;
    align-items: stretch;
  }

  .toolbar-actions {
    justify-content: flex-end;
  }

  .search-inline {
    width: 145px;
  }

  .favorite-popover {
    width: min(320px, 92vw);
  }
}
</style>


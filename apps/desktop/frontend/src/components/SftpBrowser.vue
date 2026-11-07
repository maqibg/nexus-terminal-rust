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
              class="icon-btn root-mode-btn"
              :class="{ active: rootModeEnabled }"
              @click="handleRootModeButtonClick"
              :disabled="!connectionId || rootModeSwitching"
              :title="rootModeEnabled ? '退出 Root 模式' : '进入 Root 模式'"
            >
              <i class="fas fa-user-shield"></i>
            </button>
            <button
              class="icon-btn"
              :class="{ active: !showHiddenFiles }"
              @click="showHiddenFiles = !showHiddenFiles"
              :disabled="!activeSftpSessionId"
              :title="showHiddenFiles ? '隐藏以点开头的文件' : '显示隐藏文件'"
            >
              <i class="fas" :class="showHiddenFiles ? 'fa-eye' : 'fa-eye-slash'"></i>
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
            <nav
              v-show="!isEditingPath && !showPathHistoryDropdown"
              class="path-breadcrumbs"
              title="点击跳转，双击编辑完整路径"
            >
              <template v-for="(segment, index) in pathSegments" :key="segment.path">
                <span v-if="index > 0" class="path-sep">/</span>
                <button
                  class="path-crumb"
                  :class="{ 'is-last': index === pathSegments.length - 1 }"
                  :title="segment.path"
                  @click="navigateTo(segment.path)"
                  @dblclick="startPathEdit"
                >
                  {{ segment.label }}
                </button>
              </template>
            </nav>
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
            title="上传文件或文件夹"
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
      <div
        v-else
        ref="fileListContainerRef"
        class="file-list"
        @scroll.passive="handleListScroll"
        @click.self="clearSelection"
        @contextmenu.prevent="showCtx($event, null)"
        @dragenter.prevent="handleDragEnter"
        @dragover.prevent="handleDragOver"
        @dragleave="handleDragLeave"
        @drop.prevent="handleDrop"
      >
        <div
          v-if="showExternalDropOverlay"
          class="drag-upload-overlay"
          @dragover.prevent
          @dragleave.prevent="handleDragLeave"
          @drop.prevent="handleOverlayDrop"
        >
          <div class="drag-upload-card">
            <i class="fas fa-cloud-upload-alt"></i>
            <div class="drag-upload-title">松手即可上传到当前目录</div>
            <div class="drag-upload-path">{{ currentPath }}</div>
            <div class="drag-upload-desc">支持直接拖拽文件或文件夹</div>
          </div>
        </div>
        <div class="file-list-shell" :class="{ 'overlay-active': showExternalDropOverlay }">
          <div class="file-header" :style="{ gridTemplateColumns: gridTemplate }">
            <button class="file-header-btn" :class="{ active: sortKey === 'type' }" @click="toggleSort('type')">
              <span>类型</span>
              <span v-if="sortKey === 'type'" class="sort-indicator">{{ sortDirection === 'asc' ? '▲' : '▼' }}</span>
              <span class="resize-handle" @mousedown.prevent.stop="startResize($event, 'type')"></span>
            </button>
            <button class="file-header-btn" :class="{ active: sortKey === 'name' }" @click="toggleSort('name')">
              <span>名称</span>
              <span v-if="sortKey === 'name'" class="sort-indicator">{{ sortDirection === 'asc' ? '▲' : '▼' }}</span>
              <span class="resize-handle" @mousedown.prevent.stop="startResize($event, 'name')"></span>
            </button>
            <button class="file-header-btn align-right" :class="{ active: sortKey === 'size' }" @click="toggleSort('size')">
              <span>大小</span>
              <span v-if="sortKey === 'size'" class="sort-indicator">{{ sortDirection === 'asc' ? '▲' : '▼' }}</span>
              <span class="resize-handle" @mousedown.prevent.stop="startResize($event, 'size')"></span>
            </button>
            <div class="file-header-static">
              <span>权限</span>
              <span class="resize-handle" @mousedown.prevent.stop="startResize($event, 'permissions')"></span>
            </div>
            <button class="file-header-btn" :class="{ active: sortKey === 'modified' }" @click="toggleSort('modified')">
              <span>修改时间</span>
              <span v-if="sortKey === 'modified'" class="sort-indicator">{{ sortDirection === 'asc' ? '▲' : '▼' }}</span>
            </button>
          </div>

          <div class="file-body" tabindex="0" @keydown="handleListKeydown">
            <div class="virtual-pad" :style="{ height: `${topPad}px` }"></div>

            <div
              v-for="item in visibleRows"
              :key="item.row.isParent ? '__parent__' : item.row.entry.path"
              class="file-row"
              :class="{
                'up-row': item.row.isParent,
                dir: !item.row.isParent && item.row.entry.is_dir,
                selected: !item.row.isParent && isEntrySelected(item.row.entry),
                'drop-target':
                  item.row.isParent
                    ? dragOverTarget === '..'
                    : item.row.entry.is_dir && dragOverTarget === item.row.entry.path,
                'kbd-active': keyboardIndex === item.index,
              }"
              :style="{ gridTemplateColumns: gridTemplate }"
              :data-drop-row="item.row.isParent ? '..' : item.row.entry.path"
              :draggable="!item.row.isParent"
              @dragstart="!item.row.isParent && handleDragStart(item.row.entry)"
              @dragend="handleDragEnd"
              @dragover.prevent="handleDragOverRow(item.row.isParent ? '..' : item.row.entry, $event)"
              @dragleave="handleDragLeaveRow(item.row.isParent ? '..' : item.row.entry)"
              @drop.prevent="handleDropOnRow(item.row.isParent ? '..' : item.row.entry, $event)"
              @click.stop="item.row.isParent ? goUp() : handleEntryClick(item.row.entry, $event)"
              @contextmenu.prevent.stop="!item.row.isParent && handleEntryContextMenu($event, item.row.entry)"
            >
              <span class="file-cell file-icon">
                <i v-if="item.row.isParent" class="fas fa-level-up-alt folder-color"></i>
                <i v-else-if="item.row.entry.is_dir" class="fas fa-folder folder-color"></i>
                <i v-else class="fas fa-file file-color"></i>
              </span>
              <span class="file-cell file-name">{{ item.row.isParent ? '..' : item.row.entry.name }}</span>
              <span class="file-cell file-size">{{ item.row.sizeText }}</span>
              <span class="file-cell file-perms">{{ item.row.permsText }}</span>
              <span class="file-cell file-modified">{{ item.row.timeText }}</span>
            </div>

            <div class="virtual-pad" :style="{ height: `${bottomPad}px` }"></div>
          </div>
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

      <div v-if="showMkdir" class="mini-dialog-backdrop">
        <div class="mini-dialog">
          <div class="mini-dialog-title">
            <i class="fas fa-folder-plus"></i>
            <span>新建文件夹</span>
          </div>
          <input class="mini-dialog-input" v-model="mkdirName" :disabled="mkdirSubmitting" @keydown.enter.prevent="doMkdir" placeholder="输入文件夹名称..." />
          <div class="mini-actions">
            <button class="btn-cancel" :disabled="mkdirSubmitting" @click="showMkdir = false">取消</button>
            <button class="btn-save" :disabled="mkdirSubmitting" @click="doMkdir">{{ mkdirSubmitting ? '创建中...' : '创建' }}</button>
          </div>
        </div>
      </div>

      <div v-if="showRename" class="mini-dialog-backdrop">
        <div class="mini-dialog rename-dialog">
          <div class="mini-dialog-title">
            <i :class="renameTargetEntry?.is_dir ? 'fas fa-folder' : 'far fa-file-alt'"></i>
            <span>重命名{{ renameTargetEntry?.is_dir ? '文件夹' : '文件' }}</span>
          </div>
          <div v-if="renameTargetEntry" class="mini-dialog-note">
            <span class="mini-dialog-note-label">当前名称</span>
            <span class="mini-dialog-note-value" :title="renameTargetEntry.name">{{ renameTargetEntry.name }}</span>
          </div>
          <input
            ref="renameInputRef"
            class="mini-dialog-input"
            v-model="renameName"
            :disabled="renameSubmitting"
            @keydown.enter.prevent="submitRename"
            placeholder="输入新的名称..."
          />
          <div v-if="renameError" class="mini-dialog-error">{{ renameError }}</div>
          <div class="mini-actions">
            <button class="btn-cancel" :disabled="renameSubmitting" @click="() => closeRenameDialog()">取消</button>
            <button class="btn-save" :disabled="renameSubmitting" @click="submitRename">{{ renameSubmitting ? '重命名中...' : '确定' }}</button>
          </div>
        </div>
      </div>

      <div v-if="showRootModeDialog" class="mini-dialog-backdrop">
        <div class="mini-dialog">
          <div class="mini-dialog-title">
            <i class="fas fa-user-shield"></i>
            <span>Root 模式</span>
          </div>
          <div class="root-mode-hint">
            <span>将使用独立 SFTP 连接访问 root 目录。</span>
          </div>
          <input
            class="mini-dialog-input"
            v-model.trim="rootModeUsername"
            placeholder="用户名（默认 root）"
            :disabled="rootModeSwitching"
          />
          <input
            class="mini-dialog-input"
            v-model="rootModePassword"
            type="password"
            placeholder="Root 密码"
            :disabled="rootModeSwitching"
            @keydown.enter="enableRootMode"
          />
          <div class="mini-actions">
            <button class="btn-cancel" :disabled="rootModeSwitching" @click="closeRootModeDialog">取消</button>
            <button class="btn-save" :disabled="rootModeSwitching" @click="enableRootMode">
              {{ rootModeSwitching ? '切换中...' : '进入 Root 模式' }}
            </button>
          </div>
        </div>
      </div>

      <div v-if="showNewFile" class="mini-dialog-backdrop">
        <div class="mini-dialog">
          <div class="mini-dialog-title">
            <i class="far fa-file-alt"></i>
            <span>新建文件</span>
          </div>
          <input class="mini-dialog-input" v-model="newFileName" :disabled="newFileSubmitting" @keydown.enter.prevent="doCreateFile" placeholder="输入文件名称..." />
          <div class="mini-actions">
            <button class="btn-cancel" :disabled="newFileSubmitting" @click="showNewFile = false">取消</button>
            <button class="btn-save" :disabled="newFileSubmitting" @click="doCreateFile">{{ newFileSubmitting ? '创建中...' : '创建' }}</button>
          </div>
        </div>
      </div>

      <PermissionEditor
        v-if="showPermissionEditor"
        visible
        v-model:mode="permissionMode"
        :target-name="permissionTarget?.name ?? ''"
        @cancel="closePermissionEditor"
        @save="savePermission"
      />

      <FileUploadPopup
        :visible="showUpload"
        :session-id="activeSftpSessionId || ''"
        :remote-path="currentPath"
        @uploaded="handleUploadTasksCreated"
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
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useSessionStore } from '@/stores/session';
import type { FileEntry } from '@/lib/api';
import FavoritePaths from '@/components/FavoritePaths.vue';
import PermissionEditor from '@/components/PermissionEditor.vue';
import FileUploadPopup from '@/components/FileUploadPopup.vue';
import SendFilesModal from '@/components/SendFilesModal.vue';
import { useSftpBrowser } from '@/composables/useSftpBrowser';
import { useSftpBrowserDragAndDrop } from '@/composables/useSftpBrowserDragAndDrop';
import { useSftpFileListColumns } from '@/composables/useSftpFileListColumns';

const sessionStore = useSessionStore();
const { activeSessionId: sshSessionId, activeSession } = storeToRefs(sessionStore);
const isSshSession = computed(() => activeSession.value?.protocol === 'SSH');
const connectionId = computed(() => (activeSession.value?.protocol === 'SSH' ? activeSession.value.connectionId : undefined));
const activeSftpSessionId = computed(() => (activeSession.value?.protocol === 'SSH' ? activeSession.value?.sftpSessionId ?? null : null));
const fileListContainerRef = ref<HTMLDivElement | null>(null);

const {
  // state
  currentPath, pathInput, entries, filteredEntries, loading, error, searchQuery, showHiddenFiles,
  showPermissionEditor, permissionTarget, permissionMode,
  selectedEntryPaths, clipboardState,
  rootModeEnabled, rootModeSwitching, showRootModeDialog, rootModeUsername, rootModePassword,
  pathHistoryItems, pathHistoryLoading, filteredPathHistory, pathHistorySelectedIndex,
  isSearchActive, isEditingPath, showPathHistoryDropdown,
  ctxVisible, ctxPos, ctxNearRight, ctxSubmenuKey, ctxEntry,
  showFavoritePathsPopover, isFavoriteDialogOpen, favoritePopoverStyle,
  showUpload, showMkdir, mkdirName, mkdirSubmitting, showRename, renameName, renameSubmitting, renameError, renameTargetEntry, showNewFile, newFileName, newFileSubmitting, showSendFile, sendFileTarget,
  contextMenuItems, showPopupFileEditor, sortKey, sortDirection,
  // DOM refs
  pathInputRef, pathInputWrapperRef, searchInputRef, renameInputRef, ctxMenuRef, favoriteButtonRef, favoritePopoverRef,
  // actions
  navigateTo, goUp, refresh,
  handleRootModeButtonClick, enableRootMode, disableRootMode, closeRootModeDialog,
  handleEntryClick, handleEntryContextMenu, showCtx, closeCtxMenu, handleCtxItemClick,
  startPathEdit, handlePathInputFocus, handlePathInputChange, handlePathInputKeydown, handlePathInputBlur, selectPathHistory,
  activateSearch, deactivateSearch, cancelSearch,
  toggleFavoritePopover, navigateFromFavorite, handleFavoriteDialogVisibility,
  openUpload, openSendModal, onSendCreated,
  registerUploadTasksForRefresh,
  toggleSort, uploadLocalPaths, moveEntriesToDirectory,
  doMkdir, closeRenameDialog, submitRename, doCreateFile, openPopupEditor, sendCdCommandToTerminal,
  closePermissionEditor, savePermission,
  isEntrySelected, clearSelection,
  formatSize, formatModifiedTime, formatPerms,
  getPathDir,
} = useSftpBrowser(activeSftpSessionId, sshSessionId, connectionId);

const { gridTemplate, startResize } = useSftpFileListColumns();

// Keyboard navigation over the visible rows: the parent row (when present)
// occupies index 0, entries follow.
const keyboardIndex = ref(-1);
const hasParentRow = computed(() => currentPath.value !== '/');
const rowCount = computed(() => filteredEntries.value.length + (hasParentRow.value ? 1 : 0));

// Pre-format the display columns once per data change. Formatting inside the
// template rebuilt an Intl formatter for every row on every render, which is
// what made large directories feel heavy while scrolling.
const displayRows = computed(() =>
  filteredEntries.value.map((entry) => ({
    entry,
    sizeText: entry.is_dir ? '' : formatSize(entry.size),
    permsText: entry.permissions != null ? formatPerms(entry.permissions) : '',
    timeText: formatModifiedTime(entry.modified),
  })),
);

// Breadcrumb segments, each carrying the absolute path it navigates to.
const pathSegments = computed(() => {
  const items = [{ label: '/', path: '/' }];
  let accumulated = '';
  for (const segment of currentPath.value.split('/').filter(Boolean)) {
    accumulated += `/${segment}`;
    items.push({ label: segment, path: accumulated });
  }
  return items;
});

// ── Virtual list ─────────────────────────────────────────────────────────────
// Rendering every row of a 50k-entry directory freezes the UI, so only the
// visible window (plus a small overscan) is mounted. Rows are a fixed 37px tall;
// .file-row enforces that height so the offsets stay exact.
interface VirtualRow {
  isParent: boolean;
  entry: FileEntry;
  sizeText: string;
  permsText: string;
  timeText: string;
}

const ROW_HEIGHT = 37;
const OVERSCAN = 10;

const scrollTop = ref(0);
const viewportHeight = ref(800);

const PARENT_ROW: VirtualRow = {
  isParent: true,
  entry: { name: '..', path: '..', is_dir: true, size: 0 },
  sizeText: '',
  permsText: '',
  timeText: '',
};

const virtualRows = computed<VirtualRow[]>(() => {
  const rows: VirtualRow[] = [];
  if (hasParentRow.value) {
    rows.push(PARENT_ROW);
  }
  for (const row of displayRows.value) {
    rows.push({ isParent: false, ...row });
  }
  return rows;
});

const rangeStart = computed(() =>
  Math.max(0, Math.floor(scrollTop.value / ROW_HEIGHT) - OVERSCAN),
);
const rangeEnd = computed(() =>
  Math.min(
    virtualRows.value.length,
    Math.ceil((scrollTop.value + viewportHeight.value) / ROW_HEIGHT) + OVERSCAN,
  ),
);
const visibleRows = computed(() =>
  virtualRows.value
    .slice(rangeStart.value, rangeEnd.value)
    .map((row, offset) => ({ row, index: rangeStart.value + offset })),
);
const topPad = computed(() => rangeStart.value * ROW_HEIGHT);
const bottomPad = computed(() =>
  Math.max(0, (virtualRows.value.length - rangeEnd.value) * ROW_HEIGHT),
);

let scrollFrame = 0;
function handleListScroll(event: Event) {
  const element = event.currentTarget as HTMLElement;
  if (scrollFrame) {
    return;
  }
  scrollFrame = requestAnimationFrame(() => {
    scrollFrame = 0;
    scrollTop.value = element.scrollTop;
    viewportHeight.value = element.clientHeight || viewportHeight.value;
  });
}

watch(currentPath, () => {
  scrollTop.value = 0;
  if (fileListContainerRef.value) {
    fileListContainerRef.value.scrollTop = 0;
  }
});

onMounted(() => {
  if (fileListContainerRef.value) {
    viewportHeight.value = fileListContainerRef.value.clientHeight || 800;
  }
});

async function handleListKeydown(event: KeyboardEvent) {
  const total = rowCount.value;
  if (total === 0) {
    return;
  }

  if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
    event.preventDefault();
    const delta = event.key === 'ArrowDown' ? 1 : -1;
    keyboardIndex.value = (keyboardIndex.value + delta + total) % total;
  } else if (event.key === 'Enter') {
    const index = keyboardIndex.value;
    if (index < 0) {
      return;
    }
    event.preventDefault();
    if (hasParentRow.value && index === 0) {
      void goUp();
    } else {
      const entry = filteredEntries.value[index - (hasParentRow.value ? 1 : 0)];
      if (entry) {
        void handleEntryClick(entry, event as unknown as MouseEvent);
      }
    }
  } else if (event.key === 'Escape') {
    keyboardIndex.value = -1;
  } else {
    return;
  }

  await nextTick();
  scrollToRow(keyboardIndex.value);
}

// Keeps the active row inside the viewport without relying on DOM nodes that
// may not be mounted (the virtual list only renders the visible window).
function scrollToRow(index: number) {
  const scroller = fileListContainerRef.value;
  if (!scroller || index < 0) {
    return;
  }

  const rowTop = index * ROW_HEIGHT;
  const rowBottom = rowTop + ROW_HEIGHT;
  const viewTop = scroller.scrollTop;
  const viewBottom = viewTop + scroller.clientHeight;

  if (rowTop < viewTop) {
    scroller.scrollTop = rowTop;
  } else if (rowBottom > viewBottom) {
    scroller.scrollTop = rowBottom - scroller.clientHeight;
  }

  scrollTop.value = scroller.scrollTop;
}

watch([currentPath, searchQuery, sortKey, sortDirection], () => {
  keyboardIndex.value = -1;
});
const {
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
} = useSftpBrowserDragAndDrop({
  isConnected: computed(() => !!activeSftpSessionId.value),
  currentPath,
  selectedEntryPaths,
  fileListContainerRef,
  getParentPath: getPathDir,
  uploadLocalPaths,
  moveEntriesToDirectory,
});

function handleUploadTasksCreated(taskIds: string[]): void {
  showUpload.value = false;
  registerUploadTasksForRefresh(taskIds, currentPath.value);
}
</script>

<style scoped src="./SftpBrowser.styles.css"></style>


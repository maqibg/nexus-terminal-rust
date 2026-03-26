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

          <div class="file-body">
            <div
              v-if="currentPath !== '/'"
              class="file-row up-row"
              :class="{ 'drop-target': dragOverTarget === '..' }"
              :style="{ gridTemplateColumns: gridTemplate }"
              data-drop-row=".."
              @click.stop="goUp"
              @dragover.prevent="handleDragOverRow('..', $event)"
              @dragleave="handleDragLeaveRow('..')"
              @drop.prevent="handleDropOnRow('..', $event)"
            >
              <span class="file-cell file-icon">
                <i class="fas fa-level-up-alt folder-color"></i>
              </span>
              <span class="file-cell file-name">..</span>
              <span class="file-cell file-size"></span>
              <span class="file-cell file-perms"></span>
              <span class="file-cell file-modified"></span>
            </div>

            <div
              v-for="entry in filteredEntries"
              :key="entry.path"
              class="file-row"
              :class="{
                dir: entry.is_dir,
                selected: isEntrySelected(entry),
                'drop-target': entry.is_dir && dragOverTarget === entry.path,
              }"
              :style="{ gridTemplateColumns: gridTemplate }"
              :data-drop-row="entry.path"
              draggable="true"
              @dragstart="handleDragStart(entry)"
              @dragend="handleDragEnd"
              @dragover.prevent="handleDragOverRow(entry, $event)"
              @dragleave="handleDragLeaveRow(entry)"
              @drop.prevent="handleDropOnRow(entry, $event)"
              @click.stop="handleEntryClick(entry, $event)"
              @contextmenu.prevent.stop="handleEntryContextMenu($event, entry)"
            >
              <span class="file-cell file-icon">
                <i v-if="entry.is_dir" class="fas fa-folder folder-color"></i>
                <i v-else class="fas fa-file file-color"></i>
              </span>
              <span class="file-cell file-name">{{ entry.name }}</span>
              <span class="file-cell file-size">{{ entry.is_dir ? '' : formatSize(entry.size) }}</span>
              <span class="file-cell file-perms">{{ entry.permissions != null ? formatPerms(entry.permissions) : '' }}</span>
              <span class="file-cell file-modified">{{ formatModifiedTime(entry.modified) }}</span>
            </div>
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

      <div v-if="showMkdir" class="mini-dialog-backdrop" @click.self="showMkdir = false">
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

      <div v-if="showRename" class="mini-dialog-backdrop" @click.self="() => closeRenameDialog()">
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

      <div v-if="showRootModeDialog" class="mini-dialog-backdrop" @click.self="closeRootModeDialog">
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

      <div v-if="showNewFile" class="mini-dialog-backdrop" @click.self="showNewFile = false">
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
import { computed, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { useSessionStore } from '@/stores/session';
import FavoritePaths from '@/components/FavoritePaths.vue';
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
  currentPath, pathInput, entries, filteredEntries, loading, error, searchQuery,
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
  isEntrySelected, clearSelection,
  formatSize, formatModifiedTime, formatPerms,
  getPathDir,
} = useSftpBrowser(activeSftpSessionId, sshSessionId, connectionId);

const { gridTemplate, startResize } = useSftpFileListColumns();
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

<style scoped>
.sftp-browser {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  overflow: hidden;
  position: relative;
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
  font-size: calc(28px + var(--ui-font-size-offset));
  opacity: 0.5;
}

.placeholder-text {
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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

.root-mode-btn.active {
  color: var(--yellow, #f9e2af);
  background: rgba(249, 226, 175, 0.14);
}

.root-mode-btn.active:hover {
  background: rgba(249, 226, 175, 0.2);
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
  font-size: calc(12px + var(--ui-font-size-offset));
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 100%;
  border: none;
  background: transparent;
  color: var(--text, #cdd6f4);
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
}

.status-msg.error {
  color: var(--red, #f38ba8);
}

.status-msg i {
  font-size: calc(14px + var(--ui-font-size-offset));
}

.file-list {
  position: relative;
  flex: 1;
  overflow: auto;
  font-family: 'Segoe UI Variable Text', 'Segoe UI', 'Microsoft YaHei UI', 'Microsoft YaHei', sans-serif;
  text-rendering: geometricPrecision;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

.file-list-shell {
  position: relative;
  min-width: max-content;
}

.file-list-shell.overlay-active {
  pointer-events: none;
}

.file-header {
  display: grid;
  align-items: center;
  position: sticky;
  top: 0;
  z-index: 2;
  padding: 0 12px;
  min-width: max-content;
  font-size: calc(12px + var(--ui-font-size-offset));
  font-weight: 600;
  color: var(--text-sub, #6b7280);
  border-bottom: 1px solid var(--border, #313244);
  background: var(--bg-mantle, #181825);
}

.file-header-btn {
  position: relative;
  border: none;
  background: transparent;
  color: inherit;
  font: inherit;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  justify-content: flex-start;
  width: 100%;
  height: 36px;
  padding: 0 14px 0 0;
  cursor: pointer;
  text-align: left;
}

.file-header-btn:hover {
  color: var(--text, #cdd6f4);
}

.file-header-btn.active {
  color: var(--blue, #89b4fa);
}

.file-header-static {
  position: relative;
  display: inline-flex;
  align-items: center;
  width: 100%;
  height: 36px;
  padding: 0 14px 0 0;
  color: inherit;
}

.file-header-btn.align-right {
  justify-content: flex-end;
}

.sort-indicator {
  font-size: calc(10px + var(--ui-font-size-offset));
  line-height: 1;
}

.resize-handle {
  position: absolute;
  top: 0;
  right: -6px;
  width: 12px;
  height: 100%;
  cursor: col-resize;
  pointer-events: auto;
}

.resize-handle:hover {
  background: color-mix(in srgb, var(--blue, #89b4fa) 20%, transparent);
}

.file-body {
  min-width: max-content;
}

.drag-upload-overlay {
  position: absolute;
  inset: 0;
  z-index: 8;
  display: flex;
  align-items: center;
  justify-content: center;
  background: color-mix(in srgb, var(--bg-base, #1e1e2e) 72%, transparent);
  backdrop-filter: blur(4px);
}

.drag-upload-card {
  min-width: 320px;
  max-width: 460px;
  padding: 24px 28px;
  border: 1px dashed var(--blue, #89b4fa);
  border-radius: 14px;
  background: color-mix(in srgb, var(--bg-surface0, #313244) 92%, transparent);
  color: var(--text, #cdd6f4);
  text-align: center;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.35);
}

.drag-upload-card i {
  font-size: calc(32px + var(--ui-font-size-offset));
  color: var(--blue, #89b4fa);
  margin-bottom: 12px;
}

.drag-upload-title {
  font-size: calc(15px + var(--ui-font-size-offset));
  font-weight: 600;
}

.drag-upload-path {
  margin-top: 8px;
  font-size: calc(13px + var(--ui-font-size-offset));
  word-break: break-all;
}

.drag-upload-desc {
  margin-top: 6px;
  font-size: calc(12px + var(--ui-font-size-offset));
  color: var(--text-sub, #6b7280);
}

.file-row {
  display: grid;
  align-items: center;
  min-width: max-content;
  min-height: 36px;
  padding: 0 12px;
  border-bottom: 1px solid color-mix(in srgb, var(--border, #d1d5db) 78%, transparent);
  transition: background 0.12s, outline-color 0.12s;
  user-select: none;
  cursor: default;
}

.file-row:hover {
  background: rgba(137, 180, 250, 0.08);
}

.file-row.dir,
.file-row.up-row {
  cursor: pointer;
}

.file-row.dir:hover,
.file-row.up-row:hover {
  background: rgba(137, 180, 250, 0.12);
}

.file-row.selected {
  background: rgba(137, 180, 250, 0.24);
}

.file-row.selected:hover {
  background: rgba(137, 180, 250, 0.3);
}

.file-row.drop-target {
  outline: 2px dashed var(--blue, #89b4fa);
  outline-offset: -2px;
  background: rgba(137, 180, 250, 0.16);
}

.file-cell {
  padding: 8px 14px 8px 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.file-row.selected .file-size,
.file-row.selected .file-perms,
.file-row.selected .file-modified,
.file-row.selected .file-name {
  color: var(--text, #cdd6f4);
}

.file-icon {
  text-align: center;
  font-size: calc(14px + var(--ui-font-size-offset));
}

.folder-color {
  color: var(--yellow, #f9e2af);
}

.file-color {
  color: var(--text-sub, #6b7280);
}

.file-name {
  color: var(--text, #cdd6f4);
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 500;
}

.file-row.dir .file-name,
.file-row.up-row .file-name {
  font-weight: 600;
}

.file-size {
  color: var(--text-sub, #6b7280);
  font-size: calc(12px + var(--ui-font-size-offset));
  text-align: right;
  font-family: 'Cascadia Mono', 'Consolas', monospace;
}

.file-perms {
  color: var(--text-sub, #6b7280);
  font-size: calc(12px + var(--ui-font-size-offset));
  font-family: 'Cascadia Mono', 'Consolas', monospace;
  text-align: center;
}

.file-modified {
  color: var(--text-sub, #6b7280);
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
  gap: 8px;
}

.empty-dir i {
  font-size: calc(24px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(10px + var(--ui-font-size-offset));
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
  font-size: calc(11px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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

.rename-dialog {
  min-width: 360px;
}

.mini-dialog-note {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin: -2px 0 10px;
  padding: 10px 12px;
  border-radius: 8px;
  background: rgba(137, 180, 250, 0.08);
  border: 1px solid rgba(137, 180, 250, 0.18);
}

.mini-dialog-note-label {
  font-size: calc(11px + var(--ui-font-size-offset));
  color: var(--text-dim, #6c7086);
}

.mini-dialog-note-value {
  color: var(--text, #cdd6f4);
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-dialog-error {
  margin-top: 8px;
  color: var(--red, #f38ba8);
  font-size: calc(12px + var(--ui-font-size-offset));
  line-height: 1.4;
}

.root-mode-hint {
  margin: -2px 0 10px;
  font-size: calc(12px + var(--ui-font-size-offset));
  color: var(--text-sub, #a6adc8);
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
  transition: opacity 0.15s;
}

.btn-save:hover {
  opacity: 0.85;
}

.btn-cancel:disabled,
.btn-save:disabled,
.mini-dialog-input:disabled {
  opacity: 0.65;
  cursor: not-allowed;
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


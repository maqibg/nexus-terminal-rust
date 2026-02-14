<template>
  <div class="sftp-browser">
    <template v-if="!sshSessionId">
      <div class="sftp-placeholder">
        <i class="fas fa-folder-open placeholder-icon"></i>
        <span class="placeholder-text">无活动会话</span>
      </div>
    </template>
    <template v-else>
      <FavoritePaths :connection-id="connectionId" @navigate="navigateTo" />
      <div class="toolbar">
        <button class="tb-btn" @click="goUp" :disabled="currentPath === '/'" title="上级目录">
          <i class="fas fa-level-up-alt"></i>
        </button>
        <button class="tb-btn" @click="refresh" title="刷新">
          <i class="fas fa-sync-alt"></i>
        </button>
        <input ref="pathInputRef" class="path-input" v-model="pathInput" data-focus-id="fileManagerPathInput" @keydown.enter="navigateTo(pathInput)" placeholder="/" />
        <button class="tb-btn" @click="showMkdir = true" title="新建文件夹">
          <i class="fas fa-folder-plus"></i>
        </button>
        <button class="tb-btn" @click="openUpload" :disabled="!activeSftpSessionId" title="上传">
          <i class="fas fa-upload"></i>
        </button>
        <button class="tb-btn" @click="openSendModal()" :disabled="!activeSftpSessionId" title="发送到远程">
          <i class="fas fa-exchange-alt"></i>
        </button>
        <PathHistoryDropdown :connection-id="connectionId" @navigate="navigateTo" />
      </div>

      <div v-if="loading" class="status-msg">
        <i class="fas fa-spinner fa-spin"></i>
        <span>加载中...</span>
      </div>
      <div v-else-if="error" class="status-msg error">
        <i class="fas fa-exclamation-triangle"></i>
        <span>{{ error }}</span>
      </div>
      <div v-else class="file-list">
        <!-- 表头 -->
        <div class="file-header">
          <span class="fh-icon"></span>
          <span class="fh-name">名称</span>
          <span class="fh-size">大小</span>
          <span class="fh-perms">权限</span>
        </div>
        <div
          v-for="entry in entries" :key="entry.path"
          class="file-item" :class="{ dir: entry.is_dir }"
          @dblclick="entry.is_dir ? navigateTo(entry.path) : openEditor(entry)"
          @contextmenu.prevent="showCtx($event, entry)"
        >
          <span class="file-icon">
            <i v-if="entry.is_dir" class="fas fa-folder folder-color"></i>
            <i v-else class="fas fa-file file-color"></i>
          </span>
          <span class="file-name">{{ entry.name }}</span>
          <span class="file-size">{{ entry.is_dir ? '--' : formatSize(entry.size) }}</span>
          <span class="file-perms">{{ entry.permissions != null ? formatPerms(entry.permissions) : '' }}</span>
        </div>
        <div v-if="!entries.length" class="empty-dir">
          <i class="fas fa-folder-open"></i>
          <span>空目录</span>
        </div>
      </div>

      <!-- Context menu -->
      <div v-if="ctxEntry" class="ctx-backdrop" @click="ctxEntry = null"></div>
      <div v-if="ctxEntry" class="ctx-menu" :style="{ left: ctxPos.x + 'px', top: ctxPos.y + 'px' }">
        <div v-if="!ctxEntry.is_dir" class="ctx-item" @click="openEditor(ctxEntry!)">
          <i class="fas fa-edit ctx-icon"></i>编辑
        </div>
        <div v-if="!ctxEntry.is_dir" class="ctx-item" @click="downloadFile(ctxEntry!)">
          <i class="fas fa-download ctx-icon"></i>下载
        </div>
        <div v-if="ctxEntry.is_dir" class="ctx-item" @click="downloadDirectory(ctxEntry!)">
          <i class="fas fa-file-archive ctx-icon"></i>下载目录
        </div>
        <div v-if="!ctxEntry.is_dir" class="ctx-item" @click="openSendModal(ctxEntry!.path); ctxEntry = null">
          <i class="fas fa-share ctx-icon"></i>发送到远程
        </div>
        <div class="ctx-divider"></div>
        <div class="ctx-item" @click="handleChmod(ctxEntry!)">
          <i class="fas fa-lock ctx-icon"></i>修改权限
        </div>
        <div class="ctx-item" @click="handleRename(ctxEntry!)">
          <i class="fas fa-i-cursor ctx-icon"></i>重命名
        </div>
        <div class="ctx-divider"></div>
        <div class="ctx-item danger" @click="handleDelete(ctxEntry!)">
          <i class="fas fa-trash-alt ctx-icon"></i>删除
        </div>
      </div>

      <!-- Mkdir dialog -->
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

      <!-- Upload popup -->
      <FileUploadPopup
        :visible="showUpload"
        :session-id="activeSftpSessionId || ''"
        :remote-path="currentPath"
        @uploaded="showUpload = false; refresh()"
        @cancel="showUpload = false"
      />

      <!-- Send file to remote -->
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
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { storeToRefs } from 'pinia';
import { sftpApi, type FileEntry } from '@/lib/api';
import { save } from '@tauri-apps/plugin-dialog';
import { useFileEditorStore } from '@/stores/fileEditor';
import { useUINotificationStore } from '@/stores/uiNotifications';
import { useSessionStore } from '@/stores/session';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import FavoritePaths from '@/components/FavoritePaths.vue';
import FileUploadPopup from '@/components/FileUploadPopup.vue';
import PathHistoryDropdown from '@/components/PathHistoryDropdown.vue';
import SendFilesModal from '@/components/SendFilesModal.vue';

const sessionStore = useSessionStore();
const { activeSessionId: sshSessionId, activeSession } = storeToRefs(sessionStore);
const connectionId = computed(() => activeSession.value?.connectionId);
const activeSftpSessionId = computed(() => activeSession.value?.sftpSessionId ?? null);

const fileEditorStore = useFileEditorStore();
const notify = useUINotificationStore();
const focusSwitcherStore = useFocusSwitcherStore();

const currentPath = ref('/');
const pathInput = ref('/');
const pathInputRef = ref<HTMLInputElement>();
const entries = ref<FileEntry[]>([]);
const loading = ref(false);
const error = ref('');
const showUpload = ref(false);
const showSendFile = ref(false);
const sendFileTarget = ref<string | null>(null);

const ctxEntry = ref<FileEntry | null>(null);
const ctxPos = ref({ x: 0, y: 0 });
const showMkdir = ref(false);
const mkdirName = ref('');

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

function focusPathInput(): boolean | undefined {
  if (!isVisibleInput(pathInputRef.value)) {
    return undefined;
  }
  pathInputRef.value.focus();
  pathInputRef.value.select();
  return document.activeElement === pathInputRef.value;
}

function resetBrowserState() {
  entries.value = [];
  currentPath.value = '/';
  pathInput.value = '/';
  error.value = '';
}

async function ensureSftpSession(sshSid: string): Promise<string> {
  const session = sessionStore.getSession(sshSid);
  if (!session) {
    throw new Error('会话不存在');
  }
  if (session.sftpSessionId) {
    return session.sftpSessionId;
  }

  const sftpSessionId = await sftpApi.open(session.connectionId);
  sessionStore.setSftpSession(sshSid, sftpSessionId);
  return sftpSessionId;
}

async function navigateTo(path: string) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  loading.value = true;
  error.value = '';
  try {
    const list = await sftpApi.listDir(sid, path);
    entries.value = list.sort((a, b) => {
      if (a.is_dir !== b.is_dir) return a.is_dir ? -1 : 1;
      return a.name.localeCompare(b.name);
    });
    currentPath.value = path;
    pathInput.value = path;

    if (sshSessionId.value) {
      sessionStore.setCurrentPath(sshSessionId.value, path);
    }
  } catch (e: any) {
    error.value = e.message || '加载失败';
  } finally {
    loading.value = false;
  }
}

function goUp() {
  const parts = currentPath.value.replace(/\/+$/, '').split('/');
  parts.pop();
  void navigateTo(parts.join('/') || '/');
}

function refresh() {
  void navigateTo(currentPath.value);
}

function showCtx(e: MouseEvent, entry: FileEntry) {
  ctxEntry.value = entry;
  ctxPos.value = { x: e.clientX, y: e.clientY };
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
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  ctxEntry.value = null;
  if (!confirm(`确定删除 "${entry.name}"？`)) return;
  try {
    if (entry.is_dir) await sftpApi.rmdir(sid, entry.path);
    else await sftpApi.removeFile(sid, entry.path);
    refresh();
  } catch (e: any) {
    error.value = e.message;
  }
}

async function handleRename(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  ctxEntry.value = null;
  const newName = prompt('新名称:', entry.name);
  if (!newName || newName === entry.name) return;

  const dir = currentPath.value.endsWith('/') ? currentPath.value : `${currentPath.value}/`;
  try {
    await sftpApi.rename(sid, entry.path, dir + newName);
    refresh();
  } catch (e: any) {
    error.value = e.message;
  }
}

async function handleChmod(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  ctxEntry.value = null;
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

async function openEditor(entry: FileEntry) {
  const sid = activeSftpSessionId.value;
  if (!sid) return;

  ctxEntry.value = null;
  try {
    const base64 = await sftpApi.readFile(sid, entry.path);
    const content = decodeURIComponent(escape(atob(base64)));
    const ext = entry.name.split('.').pop() ?? '';
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
      id: `${sid}:${entry.path}`,
      sessionId: sid,
      path: entry.path,
      content,
      originalContent: content,
      isDirty: false,
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

  ctxEntry.value = null;
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

  ctxEntry.value = null;
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
  if (!sid || !mkdirName.value) return;

  const dir = currentPath.value.endsWith('/') ? currentPath.value : `${currentPath.value}/`;
  try {
    await sftpApi.mkdir(sid, dir + mkdirName.value);
    showMkdir.value = false;
    mkdirName.value = '';
    refresh();
  } catch (e: any) {
    error.value = e.message;
  }
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
  return `${(bytes / 1073741824).toFixed(1)} GB`;
}

onMounted(() => {
  unregisterFileManagerSearch = focusSwitcherStore.registerFocusAction('fileManagerSearch', focusPathInput);
  unregisterFileManagerPathInput = focusSwitcherStore.registerFocusAction('fileManagerPathInput', focusPathInput);
});

onUnmounted(() => {
  unregisterFileManagerSearch?.();
  unregisterFileManagerPathInput?.();
  unregisterFileManagerSearch = null;
  unregisterFileManagerPathInput = null;
});

watch(
  sshSessionId,
  async (newSid) => {
    showUpload.value = false;
    showSendFile.value = false;
    sendFileTarget.value = null;
    resetBrowserState();

    if (!newSid) {
      return;
    }

    try {
      await ensureSftpSession(newSid);
      await navigateTo('/');
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

/* Placeholder / empty session */
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

/* Toolbar */
.toolbar {
  display: flex;
  gap: 4px;
  padding: 6px 8px;
  border-bottom: 1px solid var(--border, #313244);
  align-items: center;
  flex-shrink: 0;
}

.tb-btn {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px solid var(--border, #45475a);
  border-radius: 6px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  flex-shrink: 0;
  font-size: 12px;
  transition: all 0.15s;
}
.tb-btn:hover {
  background: var(--bg-surface1, #45475a);
  color: var(--text, #cdd6f4);
  border-color: var(--text-dim, #6c7086);
}
.tb-btn:disabled {
  opacity: 0.35;
  cursor: not-allowed;
}
.tb-btn:disabled:hover {
  background: transparent;
  color: var(--text-sub, #a6adc8);
  border-color: var(--border, #45475a);
}

.path-input {
  flex: 1;
  padding: 5px 10px;
  border-radius: 6px;
  border: 1px solid var(--border, #45475a);
  background: var(--bg-surface0, #313244);
  color: var(--text, #cdd6f4);
  font-size: 12px;
  font-family: 'Fira Code', 'Cascadia Code', monospace;
  outline: none;
  min-width: 0;
  transition: border-color 0.2s, box-shadow 0.2s;
  box-sizing: border-box;
}
.path-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}

/* Status messages */
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

/* File list */
.file-list {
  flex: 1;
  overflow-y: auto;
}

.file-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-sub, #a6adc8);
  border-bottom: 1px solid var(--border, #313244);
  background: var(--bg-mantle, #181825);
  position: sticky;
  top: 0;
  z-index: 1;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.fh-icon { width: 24px; flex-shrink: 0; text-align: center; }
.fh-name { flex: 1; }
.fh-size { min-width: 72px; text-align: right; flex-shrink: 0; }
.fh-perms { min-width: 48px; flex-shrink: 0; font-family: monospace; text-align: center; }

.file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 5px 12px;
  font-size: 12px;
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

.file-icon {
  font-size: 14px;
  flex-shrink: 0;
  width: 24px;
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
  color: var(--text, #cdd6f4);
}
.file-item.dir .file-name {
  font-weight: 500;
}

.file-size {
  color: var(--text-dim, #6c7086);
  font-size: 11px;
  flex-shrink: 0;
  min-width: 72px;
  text-align: right;
  font-family: monospace;
}
.file-perms {
  color: var(--text-dim, #6c7086);
  font-size: 11px;
  font-family: monospace;
  flex-shrink: 0;
  min-width: 48px;
  text-align: center;
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

/* Context menu */
.ctx-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}
.ctx-menu {
  position: fixed;
  z-index: 100;
  background: var(--bg-surface0, #313244);
  border-radius: 8px;
  padding: 4px;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5), 0 2px 8px rgba(0, 0, 0, 0.3);
  min-width: 160px;
  border: 1px solid var(--border, #45475a);
}
.ctx-item {
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
.ctx-item.danger {
  color: var(--red, #f38ba8);
}
.ctx-item.danger:hover {
  background: rgba(243, 139, 168, 0.12);
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

/* Mkdir dialog */
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
</style>





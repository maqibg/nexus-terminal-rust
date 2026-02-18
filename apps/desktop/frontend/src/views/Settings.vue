<template>
  <div class="settings-page">
    <div class="settings-container">
      <div class="tabs-bar">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          class="tab-btn"
          :class="{ active: activeTab === tab.key, warn: tab.key === 'about' && isUpdateAvailable }"
          @click="activeTab = tab.key"
        >
          {{ tab.label }}
        </button>
      </div>

      <div v-if="settingsError" class="banner error">{{ settingsError }}</div>
      <div v-if="loadingSettings" class="banner">加载设置中...</div>

      <div v-else class="tab-content">
        <section v-if="activeTab === 'workspace'" class="settings-card">
          <h2 class="card-title">工作区与终端</h2>
          <div class="card-body">
            <div class="settings-section-content">
              <h3 class="section-heading">弹窗文件编辑器</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('showPopupFileEditor', workspaceForm.showPopupFileEditor, 'popupEditor', '弹窗编辑器设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-show-popup-editor" v-model="workspaceForm.showPopupFileEditor" class="checkbox-input" type="checkbox">
                  <label for="workspace-show-popup-editor">打开文件时显示弹窗编辑器</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.popupEditor?.message" :class="['feedback-msg', feedback.popupEditor.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.popupEditor.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">弹窗文件管理器</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('showPopupFileManager', workspaceForm.showPopupFileManager, 'popupFileManager', '弹窗文件管理器设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-show-popup-file-manager" v-model="workspaceForm.showPopupFileManager" class="checkbox-input" type="checkbox">
                  <label for="workspace-show-popup-file-manager">启用弹窗文件管理器</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.popupFileManager?.message" :class="['feedback-msg', feedback.popupFileManager.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.popupFileManager.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">编辑器标签页</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('shareFileEditorTabs', workspaceForm.shareFileEditorTabs, 'shareTabs', '共享编辑器标签页设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-share-tabs" v-model="workspaceForm.shareFileEditorTabs" class="checkbox-input" type="checkbox">
                  <label for="workspace-share-tabs">在所有会话间共享编辑器标签页</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.shareTabs?.message" :class="['feedback-msg', feedback.shareTabs.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.shareTabs.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">终端自动复制</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('autoCopyOnSelect', workspaceForm.autoCopyOnSelect, 'autoCopy', '自动复制设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-auto-copy" v-model="workspaceForm.autoCopyOnSelect" class="checkbox-input" type="checkbox">
                  <label for="workspace-auto-copy">松开鼠标时自动复制选中文本</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.autoCopy?.message" :class="['feedback-msg', feedback.autoCopy.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.autoCopy.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">侧边栏行为</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('workspaceSidebarPersistent', workspaceForm.workspaceSidebarPersistent, 'sidebarPersistent', '侧边栏固定设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-sidebar-persistent" v-model="workspaceForm.workspaceSidebarPersistent" class="checkbox-input" type="checkbox">
                  <label for="workspace-sidebar-persistent">弹出后固定侧边栏 (不自动收回)</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.sidebarPersistent?.message" :class="['feedback-msg', feedback.sidebarPersistent.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.sidebarPersistent.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">命令输入同步</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceText('commandInputSyncTarget', workspaceForm.commandInputSyncTarget, 'commandSync', '命令输入同步目标已保存')">
                <div class="form-field">
                  <label class="form-label" for="workspace-command-sync">同步目标</label>
                  <select id="workspace-command-sync" v-model="workspaceForm.commandInputSyncTarget" class="form-control select-control">
                    <option value="none">无</option>
                    <option value="quickCommands">快捷指令</option>
                    <option value="commandHistory">命令历史</option>
                  </select>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.commandSync?.message" :class="['feedback-msg', feedback.commandSync.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.commandSync.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">显示连接标签</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('showConnectionTags', workspaceForm.showConnectionTags, 'connectionTags', '连接标签显示设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-show-connection-tags" v-model="workspaceForm.showConnectionTags" class="checkbox-input" type="checkbox">
                  <label for="workspace-show-connection-tags">在连接列表中显示标签</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.connectionTags?.message" :class="['feedback-msg', feedback.connectionTags.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.connectionTags.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">显示快捷指令标签</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('showQuickCommandTags', workspaceForm.showQuickCommandTags, 'quickCommandTags', '快捷命令标签显示设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-show-quick-command-tags" v-model="workspaceForm.showQuickCommandTags" class="checkbox-input" type="checkbox">
                  <label for="workspace-show-quick-command-tags">在快捷指令列表中显示标签</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.quickCommandTags?.message" :class="['feedback-msg', feedback.quickCommandTags.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.quickCommandTags.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">终端回滚行数</h3>
              <form class="section-form" @submit.prevent="saveTerminalScrollbackLimit">
                <div class="form-field">
                  <label class="form-label" for="workspace-scrollback-limit">最大行数</label>
                  <input id="workspace-scrollback-limit" v-model.number="workspaceForm.terminalScrollbackLimit" class="form-control" type="number" min="0" step="1">
                  <small class="section-desc">设置终端保留的最大输出行数。0 表示无限制 (使用默认值 5000)。此设置将在下次打开终端时生效。</small>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.scrollbackLimit?.message" :class="['feedback-msg', feedback.scrollbackLimit.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.scrollbackLimit.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">文件管理器删除确认</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('fileManagerShowDeleteConfirmation', workspaceForm.fileManagerShowDeleteConfirmation, 'fileDeleteConfirm', '文件删除确认设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-file-delete-confirm" v-model="workspaceForm.fileManagerShowDeleteConfirmation" class="checkbox-input" type="checkbox">
                  <label for="workspace-file-delete-confirm">删除文件或文件夹时显示确认提示框</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.fileDeleteConfirm?.message" :class="['feedback-msg', feedback.fileDeleteConfirm.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.fileDeleteConfirm.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">终端右键粘贴</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('terminalEnableRightClickPaste', workspaceForm.terminalEnableRightClickPaste, 'rightClickPaste', '终端右键粘贴设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-right-click-paste" v-model="workspaceForm.terminalEnableRightClickPaste" class="checkbox-input" type="checkbox">
                  <label for="workspace-right-click-paste">启用终端右键粘贴</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.rightClickPaste?.message" :class="['feedback-msg', feedback.rightClickPaste.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.rightClickPaste.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">状态监视器显示IP地址</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('showStatusMonitorIpAddress', workspaceForm.showStatusMonitorIpAddress, 'statusMonitorIp', '状态监视器 IP 显示设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-status-monitor-ip" v-model="workspaceForm.showStatusMonitorIpAddress" class="checkbox-input" type="checkbox">
                  <label for="workspace-status-monitor-ip">在状态监视器中显示IP地址</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.statusMonitorIp?.message" :class="['feedback-msg', feedback.statusMonitorIp.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.statusMonitorIp.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">状态监控设置</h3>
              <form class="section-form" @submit.prevent="saveStatusMonitorInterval">
                <div class="form-field">
                  <label class="form-label" for="workspace-status-monitor-interval">状态刷新间隔 (秒):</label>
                  <input id="workspace-status-monitor-interval" v-model.number="workspaceForm.statusMonitorIntervalSeconds" class="form-control" type="number" min="1" step="1">
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.statusMonitorInterval?.message" :class="['feedback-msg', feedback.statusMonitorInterval.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.statusMonitorInterval.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">Docker 管理器设置</h3>
              <form class="section-form" @submit.prevent="saveDockerSettings">
                <div class="form-field">
                  <label class="form-label" for="workspace-docker-interval">状态刷新间隔 (秒):</label>
                  <input id="workspace-docker-interval" v-model.number="workspaceForm.dockerStatusIntervalSeconds" class="form-control" type="number" min="1" step="1">
                </div>
                <div class="checkbox-row">
                  <input id="workspace-docker-expand" v-model="workspaceForm.dockerDefaultExpand" class="checkbox-input" type="checkbox">
                  <label for="workspace-docker-expand">默认展开容器详情</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.docker?.message" :class="['feedback-msg', feedback.docker.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.docker.message }}</p>
                </div>
              </form>
            </div>
          </div>
        </section>
        <section v-if="activeTab === 'system'" class="settings-card">
          <h2 class="card-title">系统设置</h2>
          <div class="card-body">
            <div class="settings-section-content">
              <h3 class="section-heading">语言设置</h3>
              <form class="section-form" @submit.prevent="saveSystemLanguage">
                <div class="form-field">
                  <label class="form-label" for="system-language">界面语言:</label>
                  <select id="system-language" v-model="systemForm.language" class="form-control select-control">
                    <option v-for="locale in availableLocales" :key="locale" :value="locale">{{ languageNames[locale] ?? locale }}</option>
                  </select>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存语言</button>
                  <p v-if="feedback.language?.message" :class="['feedback-msg', feedback.language.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.language.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">时区设置</h3>
              <form class="section-form" @submit.prevent="saveSystemTimezone">
                <div class="form-field">
                  <label class="form-label" for="system-timezone">选择时区:</label>
                  <select id="system-timezone" v-model="systemForm.timezone" class="form-control select-control">
                    <option v-for="timezone in commonTimezones" :key="timezone" :value="timezone">{{ timezone }}</option>
                  </select>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.timezone?.message" :class="['feedback-msg', feedback.timezone.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.timezone.message }}</p>
                </div>
              </form>
            </div>
          </div>
        </section>
        <section v-if="activeTab === 'security'" class="settings-card">
          <h2 class="card-title">安全设置</h2>
          <div class="card-body">
            <div class="settings-section-content">
              <h3 class="section-heading">修改密码</h3>
              <form class="section-form" @submit.prevent="changePassword">
                <div class="form-field">
                  <label class="form-label" for="security-current-password">当前密码:</label>
                  <input id="security-current-password" v-model="passwordForm.currentPassword" class="form-control" type="password" autocomplete="current-password" required>
                </div>
                <div class="form-field">
                  <label class="form-label" for="security-new-password">新密码:</label>
                  <input id="security-new-password" v-model="passwordForm.newPassword" class="form-control" type="password" autocomplete="new-password" required>
                </div>
                <div class="form-field">
                  <label class="form-label" for="security-confirm-password">确认新密码:</label>
                  <input id="security-confirm-password" v-model="passwordForm.confirmPassword" class="form-control" type="password" autocomplete="new-password" required>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary" :disabled="passwordLoading">{{ passwordLoading ? '处理中...' : '确认修改' }}</button>
                  <p v-if="feedback.changePassword?.message" :class="['feedback-msg', feedback.changePassword.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.changePassword.message }}</p>
                </div>
              </form>
            </div>

          </div>
        </section>
        <section v-if="activeTab === 'dataManagement'" class="settings-card">
          <h2 class="card-title">数据管理</h2>
          <div class="card-body">
            <div class="settings-section-content">
              <h3 class="section-heading">导出连接数据</h3>
              <form class="section-form" @submit.prevent="exportConnections">
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary" :disabled="exportConnectionsLoading">{{ exportConnectionsLoading ? '导出中...' : '开始导出' }}</button>
                  <p v-if="exportConnectionsMessage" :class="['feedback-msg', exportConnectionsSuccess ? 'feedback-ok' : 'feedback-error']">{{ exportConnectionsMessage }}</p>
                </div>
              </form>
            </div>
          </div>
        </section>
        <section v-if="activeTab === 'appearance'" class="settings-card">
          <h2 class="card-title">外观设置</h2>
          <div class="card-body">
            <div class="settings-section-content">
              <h3 class="section-heading">外观设置</h3>
              <p class="section-desc">自定义应用程序的视觉主题和背景。</p>
              <button type="button" class="btn btn-primary appearance-customize-btn" @click="appearanceStore.toggleStyleCustomizer(true)">自定义外观</button>
            </div>
          </div>
        </section>
        <section v-if="activeTab === 'about'" class="settings-card">
          <h2 class="card-title">关于</h2>
          <div class="card-body">
            <div class="settings-section-content">
              <div class="about-inline">
                <span class="about-item">版本：{{ appVersion }}</span>
                <span v-if="isCheckingVersion" class="status-pill info">检查更新中...</span>
                <span v-else-if="versionCheckError" class="status-pill danger" :title="versionCheckError">检查失败</span>
                <span v-else-if="latestVersion && !isUpdateAvailable" class="status-pill success">已是最新版本</span>
                <a v-else-if="latestVersion && isUpdateAvailable" class="status-pill warning" :href="latestReleaseUrl" target="_blank" rel="noopener noreferrer">发现新版本 {{ latestVersion }}</a>
                <span class="about-sep">|</span>
                <a class="about-link" href="https://github.com/Heavrnl/nexus-terminal" target="_blank" rel="noopener noreferrer">Heavrnl/nexus-terminal</a>
                <span class="about-sep">|</span>
                <a class="about-link" href="https://ko-fi.com/0heavrnl" target="_blank" rel="noopener noreferrer">Ko-fi</a>
              </div>
            </div>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue';
import { storeToRefs } from 'pinia';
import { authApi, connectionsApi } from '@/lib/api';
import { useUiNotificationsStore } from '@/stores/uiNotifications';
import { useAppearanceStore } from '@/stores/appearance';
import { useSettingsStore } from '@/stores/settings';

type AppVersionHost = {
  __APP_VERSION__?: unknown;
};

type TabKey = 'workspace' | 'system' | 'security' | 'dataManagement' | 'appearance' | 'about';


const tabs = computed<Array<{ key: TabKey; label: string }>>(() => {
  if (locale.value === 'zh-CN') {
    return [
      { key: 'workspace', label: '工作区' },
      { key: 'system', label: '系统' },
      { key: 'security', label: '安全' },
      { key: 'dataManagement', label: '数据管理' },
      { key: 'appearance', label: '外观' },
      { key: 'about', label: '关于' },
    ];
  }

  if (locale.value === 'ja-JP') {
    return [
      { key: 'workspace', label: 'ワークスペース' },
      { key: 'system', label: 'システム' },
      { key: 'security', label: 'セキュリティ' },
      { key: 'dataManagement', label: 'データ管理' },
      { key: 'appearance', label: '外観' },
      { key: 'about', label: '情報' },
    ];
  }

  return [
    { key: 'workspace', label: 'Workspace' },
    { key: 'system', label: 'System' },
    { key: 'security', label: 'Security' },
    { key: 'dataManagement', label: 'Data' },
    { key: 'appearance', label: 'Appearance' },
    { key: 'about', label: 'About' },
  ];
});

const availableLocales = ['en-US', 'zh-CN', 'ja-JP'];
const languageNames: Record<string, string> = {
  'en-US': 'English',
  'zh-CN': '中文',
  'ja-JP': '日本語',
};

const commonTimezones = [
  'UTC',
  'Etc/GMT+12', 'Pacific/Midway', 'Pacific/Honolulu', 'America/Anchorage',
  'America/Los_Angeles', 'America/Denver', 'America/Chicago', 'America/New_York',
  'America/Caracas', 'America/Halifax', 'America/Sao_Paulo', 'Atlantic/Azores',
  'Europe/London', 'Europe/Paris', 'Europe/Berlin', 'Europe/Moscow',
  'Asia/Dubai', 'Asia/Karachi', 'Asia/Dhaka', 'Asia/Bangkok',
  'Asia/Shanghai', 'Asia/Tokyo', 'Australia/Sydney', 'Pacific/Auckland',
  'Etc/GMT-14',
];

const notifications = useUiNotificationsStore();
const appearanceStore = useAppearanceStore();
const settingsStore = useSettingsStore();
const { locale, settings: runtimeSettings } = storeToRefs(settingsStore);

const activeTab = ref<TabKey>('workspace');
const loadingSettings = ref(false);
const settingsError = ref('');
const settingsMap = ref<Record<string, string>>({});

const feedback = reactive<Record<string, { message: string; success: boolean }>>({});

const workspaceForm = reactive({
  showPopupFileEditor: false,
  showPopupFileManager: false,
  shareFileEditorTabs: true,
  autoCopyOnSelect: false,
  workspaceSidebarPersistent: false,
  commandInputSyncTarget: 'none' as 'none' | 'quickCommands' | 'commandHistory',
  showConnectionTags: true,
  showQuickCommandTags: true,
  terminalScrollbackLimit: 5000,
  fileManagerShowDeleteConfirmation: true,
  terminalEnableRightClickPaste: true,
  showStatusMonitorIpAddress: false,
  statusMonitorIntervalSeconds: 3,
  dockerStatusIntervalSeconds: 2,
  dockerDefaultExpand: false,
});

const systemForm = reactive({
  language: 'en-US',
  timezone: 'Asia/Shanghai',
});

const passwordForm = reactive({
  currentPassword: '',
  newPassword: '',
  confirmPassword: '',
});
const passwordLoading = ref(false);

const exportConnectionsLoading = ref(false);
const exportConnectionsMessage = ref('');
const exportConnectionsSuccess = ref(false);

const appVersion = (() => {
  const maybeVersion = (globalThis as AppVersionHost).__APP_VERSION__;
  if (typeof maybeVersion === 'string' && maybeVersion.trim()) {
    return maybeVersion.replace(/^v/i, '');
  }
  return '0.1.0';
})();
const latestVersion = ref('');
const isCheckingVersion = ref(false);
const versionCheckError = ref('');

const isUpdateAvailable = computed(() => {
  if (!latestVersion.value) {
    return false;
  }
  return compareVersion(latestVersion.value, appVersion) > 0;
});

const latestReleaseUrl = computed(() => {
  if (!latestVersion.value) {
    return 'https://github.com/Heavrnl/nexus-terminal/releases';
  }
  return `https://github.com/Heavrnl/nexus-terminal/releases/tag/${latestVersion.value}`;
});


function setFeedback(key: string, message: string, success: boolean) {
  feedback[key] = { message, success };
}

function normalizeError(error: unknown, fallback = '操作失败'): string {
  if (error instanceof Error && error.message) {
    return error.message;
  }
  if (typeof error === 'string' && error.trim()) {
    return error;
  }
  return fallback;
}

function toBool(value: string | undefined, fallback = false): boolean {
  if (value == null) {
    return fallback;
  }
  const normalized = value.trim().toLowerCase();
  if (['1', 'true', 'yes', 'on'].includes(normalized)) {
    return true;
  }
  if (['0', 'false', 'no', 'off'].includes(normalized)) {
    return false;
  }
  return fallback;
}

function toInt(value: string | undefined, fallback: number): number {
  if (value == null || value.trim() === '') {
    return fallback;
  }
  const parsed = Number.parseInt(value, 10);
  if (!Number.isFinite(parsed)) {
    return fallback;
  }
  return parsed;
}


function hydrateFormsFromSettings() {
  const map = settingsMap.value;

  workspaceForm.showPopupFileEditor = toBool(map.showPopupFileEditor, false);
  workspaceForm.showPopupFileManager = toBool(map.showPopupFileManager, false);
  workspaceForm.shareFileEditorTabs = toBool(map.shareFileEditorTabs, true);
  workspaceForm.autoCopyOnSelect = toBool(map.autoCopyOnSelect, false);
  workspaceForm.workspaceSidebarPersistent = toBool(map.workspaceSidebarPersistent, false);

  const syncTarget = map.commandInputSyncTarget ?? 'none';
  workspaceForm.commandInputSyncTarget = syncTarget === 'quickCommands' || syncTarget === 'commandHistory' ? syncTarget : 'none';

  workspaceForm.showConnectionTags = toBool(map.showConnectionTags, true);
  workspaceForm.showQuickCommandTags = toBool(map.showQuickCommandTags, true);
  workspaceForm.terminalScrollbackLimit = toInt(map.terminalScrollbackLimit, 5000);
  workspaceForm.fileManagerShowDeleteConfirmation = toBool(map.fileManagerShowDeleteConfirmation, true);
  workspaceForm.terminalEnableRightClickPaste = toBool(map.terminalEnableRightClickPaste, true);
  workspaceForm.showStatusMonitorIpAddress = toBool(map.showStatusMonitorIpAddress, false);
  workspaceForm.statusMonitorIntervalSeconds = toInt(map.statusMonitorIntervalSeconds, 3);
  workspaceForm.dockerStatusIntervalSeconds = toInt(map.dockerStatusIntervalSeconds, 2);
  workspaceForm.dockerDefaultExpand = toBool(map.dockerDefaultExpand, false);

  const languageFromSettings = map.language?.trim();
  if (languageFromSettings && availableLocales.includes(languageFromSettings)) {
    systemForm.language = languageFromSettings;
  } else {
    const navigatorLocale = typeof navigator !== 'undefined' ? navigator.language : '';
    const navigatorLanguage = navigatorLocale?.split('-')[0] ?? '';
    const matchedLocale = availableLocales.find((locale) => locale === navigatorLocale || locale.split('-')[0] === navigatorLanguage);
    systemForm.language = matchedLocale || 'en-US';
  }
  systemForm.timezone = map.timezone || 'Asia/Shanghai';
  settingsStore.setRuntimeLocale(systemForm.language);
}

async function saveSetting(key: string, value: string) {
  await settingsStore.set(key, value);
  settingsMap.value[key] = settingsStore.get(key, value);
}

async function saveSettingsBatch(entries: Array<[string, string]>) {
  for (const [key, value] of entries) {
    await saveSetting(key, value);
  }
}

async function loadSettings() {
  loadingSettings.value = true;
  settingsError.value = '';

  try {
    await settingsStore.loadAll();
    settingsMap.value = { ...runtimeSettings.value };
    hydrateFormsFromSettings();
  } catch (error) {
    settingsError.value = normalizeError(error, '加载设置失败');
  } finally {
    loadingSettings.value = false;
  }
}

async function saveWorkspaceBoolean(settingKey: string, value: boolean, feedbackKey: string, successMessage: string) {
  try {
    await saveSetting(settingKey, value ? 'true' : 'false');
    setFeedback(feedbackKey, successMessage, true);
  } catch (error) {
    setFeedback(feedbackKey, normalizeError(error, '保存失败'), false);
  }
}

async function saveWorkspaceNumber(settingKey: string, value: number, feedbackKey: string, successMessage: string, min = 1) {
  try {
    if (!Number.isInteger(value) || value < min) {
      throw new Error(`请输入不小于 ${min} 的整数`);
    }
    await saveSetting(settingKey, String(value));
    setFeedback(feedbackKey, successMessage, true);
  } catch (error) {
    setFeedback(feedbackKey, normalizeError(error, '保存失败'), false);
  }
}

async function saveWorkspaceText(settingKey: string, value: string, feedbackKey: string, successMessage: string) {
  try {
    await saveSetting(settingKey, value);
    setFeedback(feedbackKey, successMessage, true);
  } catch (error) {
    setFeedback(feedbackKey, normalizeError(error, '保存失败'), false);
  }
}

async function saveTerminalScrollbackLimit() {
  try {
    const value = workspaceForm.terminalScrollbackLimit;
    const hasValue = value !== null && value !== undefined && String(value).trim() !== '';
    const normalizedValue = hasValue ? Number(value) : 5000;

    if (!Number.isInteger(normalizedValue) || normalizedValue < 0) {
      throw new Error('请输入不小于 0 的整数');
    }

    await saveSetting('terminalScrollbackLimit', String(normalizedValue));
    workspaceForm.terminalScrollbackLimit = normalizedValue;
    setFeedback('scrollbackLimit', '终端回滚行数设置已保存', true);
  } catch (error) {
    setFeedback('scrollbackLimit', normalizeError(error, '保存失败'), false);
  }
}

async function saveStatusMonitorInterval() {
  await saveWorkspaceNumber(
    'statusMonitorIntervalSeconds',
    workspaceForm.statusMonitorIntervalSeconds,
    'statusMonitorInterval',
    '状态监视器刷新间隔设置已保存',
    1,
  );
}

async function saveDockerSettings() {
  try {
    if (!Number.isInteger(workspaceForm.dockerStatusIntervalSeconds) || workspaceForm.dockerStatusIntervalSeconds < 1) {
      throw new Error('刷新间隔必须为不小于 1 的整数');
    }

    await saveSettingsBatch([
      ['dockerStatusIntervalSeconds', String(workspaceForm.dockerStatusIntervalSeconds)],
      ['dockerDefaultExpand', workspaceForm.dockerDefaultExpand ? 'true' : 'false'],
    ]);

    setFeedback('docker', 'Docker 设置已保存', true);
  } catch (error) {
    setFeedback('docker', normalizeError(error, '保存失败'), false);
  }
}

async function saveSystemLanguage() {
  try {
    await saveSetting('language', systemForm.language);
    const appliedLocale = settingsStore.setRuntimeLocale(systemForm.language);
    systemForm.language = appliedLocale;
    settingsMap.value.language = appliedLocale;
    setFeedback('language', '语言设置已保存', true);
  } catch (error) {
    setFeedback('language', normalizeError(error, '保存失败'), false);
  }
}

async function saveSystemTimezone() {
  try {
    await saveSetting('timezone', systemForm.timezone);
    setFeedback('timezone', '时区设置已保存', true);
  } catch (error) {
    setFeedback('timezone', normalizeError(error, '保存失败'), false);
  }
}

async function changePassword() {
  setFeedback('changePassword', '', false);

  if (!passwordForm.currentPassword || !passwordForm.newPassword || !passwordForm.confirmPassword) {
    setFeedback('changePassword', '请填写所有字段', false);
    return;
  }

  if (passwordForm.newPassword !== passwordForm.confirmPassword) {
    setFeedback('changePassword', '两次输入的新密码不一致', false);
    return;
  }

  passwordLoading.value = true;
  try {
    await authApi.changePassword(passwordForm.currentPassword, passwordForm.newPassword);
    passwordForm.currentPassword = '';
    passwordForm.newPassword = '';
    passwordForm.confirmPassword = '';
    setFeedback('changePassword', '密码修改成功', true);
  } catch (error) {
    setFeedback('changePassword', normalizeError(error, '密码修改失败'), false);
  } finally {
    passwordLoading.value = false;
  }
}


async function exportConnections() {
  exportConnectionsLoading.value = true;
  exportConnectionsMessage.value = '';
  exportConnectionsSuccess.value = false;

  try {
    const payload = await connectionsApi.export();
    const blob = new Blob([payload], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = 'nexus_connections_export.json';
    anchor.click();
    URL.revokeObjectURL(url);

    exportConnectionsMessage.value = '导出成功。文件已开始下载。';
    exportConnectionsSuccess.value = true;
    notifications.addNotification({ type: 'success', message: '连接数据导出成功' });
  } catch (error) {
    exportConnectionsMessage.value = normalizeError(error, '导出失败');
    exportConnectionsSuccess.value = false;
    notifications.addNotification({ type: 'error', message: exportConnectionsMessage.value });
  } finally {
    exportConnectionsLoading.value = false;
  }
}

function normalizeVersion(version: string): string {
  return version.replace(/^v/i, '').trim();
}

function versionToParts(version: string): number[] {
  const cleaned = normalizeVersion(version).split('-')[0];
  return cleaned.split('.').map((segment) => {
    const value = Number.parseInt(segment, 10);
    return Number.isFinite(value) ? value : 0;
  });
}

function compareVersion(left: string, right: string): number {
  const l = versionToParts(left);
  const r = versionToParts(right);
  const len = Math.max(l.length, r.length);

  for (let index = 0; index < len; index += 1) {
    const lv = l[index] ?? 0;
    const rv = r[index] ?? 0;
    if (lv > rv) {
      return 1;
    }
    if (lv < rv) {
      return -1;
    }
  }

  return 0;
}


async function checkLatestVersion() {
  isCheckingVersion.value = true;
  versionCheckError.value = '';

  try {
    const response = await fetch('https://api.github.com/repos/Heavrnl/nexus-terminal/releases/latest', {
      headers: {
        Accept: 'application/vnd.github+json',
      },
    });

    if (!response.ok) {
      throw new Error(`GitHub API 请求失败 (${response.status})`);
    }

    const data = await response.json() as { tag_name?: string };
    latestVersion.value = normalizeVersion(data.tag_name ?? '');
  } catch (error) {
    versionCheckError.value = normalizeError(error, '检查版本失败');
  } finally {
    isCheckingVersion.value = false;
  }
}

onMounted(async () => {
  await loadSettings();
  await checkLatestVersion();
});
</script>

<style scoped>
.settings-page {
  height: 100%;
  padding: 20px 24px;
  overflow-y: auto;
  color: var(--text);
}

.settings-container {
  max-width: 1220px;
  margin: 0 auto;
}

.tabs-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
  padding: 8px 0;
  margin-bottom: 16px;
}

.tab-btn {
  height: 34px;
  padding: 0 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  background: transparent;
  color: var(--text-sub);
  font-size: 13px;
  font-weight: 500;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  background: var(--bg-surface1);
  color: var(--text);
}

.tab-btn.active {
  background: var(--blue);
  color: #fff;
}

.tab-btn.warn:not(.active) {
  color: var(--color-warning);
}

.banner {
  margin-bottom: 12px;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-surface0);
  color: var(--text-sub);
  padding: 10px 12px;
  font-size: 13px;
}

.banner.error {
  border-color: color-mix(in srgb, var(--red) 40%, var(--border));
  background: color-mix(in srgb, var(--red) 12%, var(--bg-surface0));
  color: var(--red);
}

.tab-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.settings-card {
  border: 1px solid var(--border);
  border-radius: 10px;
  overflow: hidden;
  background: var(--bg-surface0);
  box-shadow: 0 8px 18px rgba(0, 0, 0, 0.15);
}

.card-title {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text);
  padding: 14px 20px;
  border-bottom: 1px solid var(--border);
  background: color-mix(in srgb, var(--header-bg-color) 80%, transparent);
}

.card-body {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 18px;
}

.settings-section-content {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-heading {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.section-heading-inline {
  margin-bottom: 0;
}

.section-desc {
  margin: 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--text-sub);
}

.section-form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-divider {
  margin: 0;
  border: none;
  border-top: 1px solid var(--border);
}

.section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.form-grid-two {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-label {
  font-size: 13px;
  color: var(--text-sub);
}

.form-control {
  width: 100%;
  min-height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  padding: 6px 10px;
  font-size: 13px;
  box-sizing: border-box;
}

.form-control:focus {
  outline: none;
  border-color: var(--blue);
}

.select-control {
  appearance: none;
  background-image: linear-gradient(45deg, transparent 50%, var(--text-sub) 50%), linear-gradient(135deg, var(--text-sub) 50%, transparent 50%);
  background-position: calc(100% - 16px) 14px, calc(100% - 11px) 14px;
  background-size: 5px 5px, 5px 5px;
  background-repeat: no-repeat;
  padding-right: 28px;
}

.mono-textarea {
  min-height: 120px;
  resize: vertical;
  font-family: 'Cascadia Mono', Consolas, 'Courier New', monospace;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--text);
}

.checkbox-input {
  width: 14px;
  height: 14px;
  accent-color: var(--blue);
}

.form-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.btn {
  height: 34px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0 14px;
  font-size: 13px;
  cursor: pointer;
  background: var(--header-bg-color);
  color: var(--text);
  transition: all 0.15s ease;
}

.btn:hover {
  background: var(--bg-surface1);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-primary {
  border-color: var(--button-bg-color);
  background: var(--button-bg-color);
  color: var(--button-text-color);
}

.btn-primary:hover {
  background: var(--button-hover-bg-color);
}

.appearance-customize-btn {
  align-self: flex-start;
  width: auto;
}

.btn-muted {
  border-color: var(--border);
  background: transparent;
  color: var(--text-sub);
}

.btn-danger {
  border-color: var(--red);
  background: var(--red);
  color: #fff;
}

.btn-danger:hover {
  opacity: 0.9;
}

.btn-xs {
  height: 28px;
  padding: 0 10px;
  font-size: 12px;
}

.feedback-msg {
  margin: 0;
  font-size: 13px;
}

.feedback-ok {
  color: var(--green);
}

.feedback-error {
  color: var(--red);
}

.status-pill {
  display: inline-flex;
  align-items: center;
  min-height: 24px;
  border-radius: 999px;
  padding: 0 10px;
  font-size: 12px;
}

.status-pill.success {
  background: color-mix(in srgb, var(--green) 24%, transparent);
  color: var(--green);
}

.status-pill.info {
  background: color-mix(in srgb, var(--blue) 24%, transparent);
  color: var(--blue);
}

.status-pill.warning {
  background: color-mix(in srgb, var(--color-warning) 24%, transparent);
  color: var(--color-warning);
  text-decoration: none;
}

.status-pill.danger {
  background: color-mix(in srgb, var(--red) 24%, transparent);
  color: var(--red);
}

.twofactor-setup {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--header-bg-color) 80%, transparent);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.twofactor-qr {
  width: 180px;
  height: 180px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: #fff;
  padding: 4px;
}

.twofactor-secret {
  display: inline-block;
  border-radius: 4px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  padding: 2px 8px;
  font-family: 'Cascadia Mono', Consolas, 'Courier New', monospace;
  font-size: 12px;
}

.switch-btn {
  width: 46px;
  height: 24px;
  border: none;
  border-radius: 999px;
  background: var(--border);
  position: relative;
  cursor: pointer;
  transition: all 0.2s ease;
}

.switch-dot {
  position: absolute;
  left: 2px;
  top: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #fff;
  transition: all 0.2s ease;
}

.switch-btn.enabled {
  background: var(--blue);
}

.switch-btn.enabled .switch-dot {
  left: 24px;
}

.table-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text);
}

.ip-table-wrap {
  overflow-x: auto;
  border: 1px solid var(--border);
  border-radius: 8px;
}

.ip-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 13px;
}

.ip-table th,
.ip-table td {
  border-bottom: 1px solid var(--border);
  padding: 8px 10px;
  text-align: left;
  white-space: nowrap;
}

.ip-table th {
  background: var(--header-bg-color);
  color: var(--text-sub);
  font-weight: 500;
}

.ip-table tr:last-child td {
  border-bottom: none;
}

.empty-state {
  margin: 0;
  border: 1px dashed var(--border);
  border-radius: 8px;
  padding: 12px;
  text-align: center;
  color: var(--text-sub);
  font-size: 13px;
}

.warn-text {
  color: var(--color-warning);
  font-weight: 600;
}

.about-inline {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  color: var(--text-sub);
  font-size: 13px;
}

.about-item {
  color: var(--text);
  font-weight: 500;
}

.about-sep {
  opacity: 0.5;
}

.about-link {
  color: var(--blue);
  text-decoration: none;
}

.about-link:hover {
  text-decoration: underline;
}

@media (max-width: 980px) {
  .settings-page {
    padding: 16px;
  }

  .card-body {
    padding: 14px;
  }

  .form-grid-two {
    grid-template-columns: 1fr;
  }
}
</style>


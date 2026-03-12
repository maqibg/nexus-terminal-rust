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
                <small class="section-desc">如果启用，所有 SSH 会话将共享同一组打开的文件编辑器标签页。如果禁用，每个会话将拥有自己独立的一组标签页。</small>
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
                <small class="section-desc">开启后，点击侧边栏外部区域不会自动收回侧边栏。</small>
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
                  <label class="form-label" for="workspace-command-sync">同步目标:</label>
                  <div class="command-sync-select-wrap" ref="commandSyncSelectRef" :class="{ open: commandSyncMenuOpen }">
                    <button
                      id="workspace-command-sync"
                      type="button"
                      class="form-control command-sync-trigger"
                      :aria-expanded="commandSyncMenuOpen ? 'true' : 'false'"
                      aria-haspopup="listbox"
                      @click="toggleCommandSyncMenu"
                    >
                      <span>{{ commandSyncTargetLabel }}</span>
                      <i class="fas fa-chevron-down"></i>
                    </button>
                    <div v-if="commandSyncMenuOpen" class="command-sync-menu" role="listbox">
                      <button
                        v-for="option in commandSyncTargetOptions"
                        :key="option.value"
                        type="button"
                        class="command-sync-option"
                        :class="{ active: workspaceForm.commandInputSyncTarget === option.value }"
                        @click="selectCommandSyncTarget(option.value)"
                      >
                        {{ option.label }}
                      </button>
                    </div>
                  </div>
                  <small class="section-desc">将命令输入栏的内容实时同步到所选面板的搜索框。键盘上下选中后使用 Enter 使用指令</small>
                </div>
                <div class="form-actions command-sync-actions">
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
                <small class="section-desc">关闭后将隐藏连接列表中的标签，并从搜索中排除标签。</small>
                <div class="form-actions form-actions-top-padding">
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
                <small class="section-desc">关闭后将隐藏快捷指令列表中的标签，并从搜索中排除标签。</small>
                <div class="form-actions form-actions-top-padding">
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
              <h3 class="section-heading">关闭会话确认</h3>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('terminalShowSessionCloseConfirmation', workspaceForm.terminalShowSessionCloseConfirmation, 'sessionCloseConfirm', '关闭会话确认设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-session-close-confirm" v-model="workspaceForm.terminalShowSessionCloseConfirmation" class="checkbox-input" type="checkbox">
                  <label for="workspace-session-close-confirm">关闭会话 (标签页) 时显示确认提示框</label>
                </div>
                <small class="section-desc">开启后，关闭单个会话或批量关闭会话前都会提示确认，避免误操作导致终端状态丢失。</small>
                <div class="form-actions form-actions-top-padding">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.sessionCloseConfirm?.message" :class="['feedback-msg', feedback.sessionCloseConfirm.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.sessionCloseConfirm.message }}</p>
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
                <small class="section-desc">开启后：右键=粘贴，Ctrl+右键=菜单；关闭后：右键=菜单。</small>
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
              <form class="section-form" @submit.prevent="saveStatusMonitorEnabled">
                <div class="checkbox-row">
                  <input id="workspace-status-monitor-enabled" v-model="workspaceForm.statusMonitorEnabled" class="checkbox-input" type="checkbox">
                  <label for="workspace-status-monitor-enabled">启用状态监控（关闭后不再采集远端状态）</label>
                </div>
                <small class="section-desc">关闭后将停止状态监控采集与事件推送；已连接的 SSH 会话也会立即停止采集。</small>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.statusMonitorEnabled?.message" :class="['feedback-msg', feedback.statusMonitorEnabled.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.statusMonitorEnabled.message }}</p>
                </div>
              </form>
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

            <div id="docker-settings" ref="dockerSettingsSectionRef" class="settings-section-content">
              <h3 class="section-heading">Docker 管理器</h3>
              <form class="section-form" @submit.prevent="saveDockerStatusInterval">
                <div class="form-field">
                  <label class="form-label" for="workspace-docker-interval">Docker 刷新间隔 (秒):</label>
                  <input id="workspace-docker-interval" v-model.number="workspaceForm.dockerStatusIntervalSeconds" class="form-control" type="number" min="1" step="1">
                  <small class="section-desc">原项目默认周期性轮询远端 Docker 状态；这里保持同类行为。</small>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.dockerStatusInterval?.message" :class="['feedback-msg', feedback.dockerStatusInterval.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.dockerStatusInterval.message }}</p>
                </div>
              </form>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('dockerDefaultExpand', workspaceForm.dockerDefaultExpand, 'dockerDefaultExpand', 'Docker 默认展开设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-docker-default-expand" v-model="workspaceForm.dockerDefaultExpand" class="checkbox-input" type="checkbox">
                  <label for="workspace-docker-default-expand">默认展开所有容器详情</label>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.dockerDefaultExpand?.message" :class="['feedback-msg', feedback.dockerDefaultExpand.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.dockerDefaultExpand.message }}</p>
                </div>
              </form>
              <form class="section-form" @submit.prevent="saveWorkspaceBoolean('dockerUseSudo', workspaceForm.dockerUseSudo, 'dockerUseSudo', 'Docker sudo 设置已保存')">
                <div class="checkbox-row">
                  <input id="workspace-docker-use-sudo" v-model="workspaceForm.dockerUseSudo" class="checkbox-input" type="checkbox">
                  <label for="workspace-docker-use-sudo">Docker 尝试使用 sudo</label>
                </div>
                <small class="section-desc">如果 SSH 登录账户无使用 Docker 的权限，请开启 sudo 获取。</small>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <p v-if="feedback.dockerUseSudo?.message" :class="['feedback-msg', feedback.dockerUseSudo.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.dockerUseSudo.message }}</p>
                </div>
              </form>
            </div>

          </div>
        </section>
        <section v-if="activeTab === 'ai'" class="settings-card">
          <h2 class="card-title">AI 助手</h2>
          <div class="card-body">
            <AISettingsPanel />
          </div>
        </section>
        <section v-if="activeTab === 'system'" class="settings-card">
          <h2 class="card-title">系统设置</h2>
          <div class="card-body">
            <div class="settings-section-content">
              <h3 class="section-heading">时区设置</h3>
              <form class="section-form" @submit.prevent="saveSystemTimezone">
                <div class="form-field">
                  <label class="form-label" for="system-timezone">选择时区:</label>
                  <AppSelect
                    id="system-timezone"
                    v-model="systemForm.timezone"
                    class="timezone-select"
                    :options="timezoneOptions"
                    aria-label="选择时区"
                  />
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
            <!-- 完整备份导出 -->
            <div class="settings-section-content">
              <h3 class="section-heading">完整备份导出</h3>
              <p class="section-desc">导出全部数据（连接、SSH 密钥名称存根、代理、快捷命令、收藏路径、终端主题、设置及通知渠道）。加密密钥材料不会被包含在内。</p>
              <div class="form-actions">
                <button type="button" class="btn btn-primary" :disabled="appExportLoading" @click="appExport">
                  {{ appExportLoading ? '导出中...' : '导出完整备份' }}
                </button>
                <p v-if="appExportMessage" :class="['feedback-msg', appExportSuccess ? 'feedback-ok' : 'feedback-error']">{{ appExportMessage }}</p>
              </div>
            </div>

            <hr class="section-divider">

            <!-- 完整备份导入 -->
            <div class="settings-section-content">
              <h3 class="section-heading">完整备份导入</h3>
              <p class="section-desc">从备份文件恢复数据。导入在单个数据库事务中执行，失败时自动回滚。SSH 私钥不包含在备份中，导入后需重新录入。</p>
              <input
                ref="appImportFileInput"
                type="file"
                accept=".json"
                style="display: none"
                @change="appImport"
              >
              <div class="form-actions">
                <button type="button" class="btn btn-primary" :disabled="appImportLoading" @click="triggerAppImport">
                  {{ appImportLoading ? '导入中...' : '选择备份文件' }}
                </button>
                <p v-if="appImportMessage" :class="['feedback-msg', appImportSuccess ? 'feedback-ok' : 'feedback-error']">{{ appImportMessage }}</p>
              </div>
              <div v-if="appImportResult" class="import-result-grid">
                <div class="import-result-item"><span class="label">连接</span><span class="value">{{ appImportResult.connections }}</span></div>
                <div class="import-result-item"><span class="label">代理</span><span class="value">{{ appImportResult.proxies }}</span></div>
                <div class="import-result-item"><span class="label">SSH 密钥</span><span class="value">{{ appImportResult.ssh_keys }}</span></div>
                <div class="import-result-item"><span class="label">快捷命令</span><span class="value">{{ appImportResult.quick_commands }}</span></div>
                <div class="import-result-item"><span class="label">收藏路径</span><span class="value">{{ appImportResult.favorite_paths }}</span></div>
                <div class="import-result-item"><span class="label">终端主题</span><span class="value">{{ appImportResult.terminal_themes }}</span></div>
                <div class="import-result-item"><span class="label">设置项</span><span class="value">{{ appImportResult.settings }}</span></div>
                <div class="import-result-item"><span class="label">通知渠道</span><span class="value">{{ appImportResult.notification_channels }}</span></div>
              </div>
            </div>

            <hr class="section-divider">

            <!-- 旧版：仅导出连接数据 -->
            <div class="settings-section-content">
              <h3 class="section-heading">仅导出连接数据（旧版）</h3>
              <p class="section-desc">仅导出连接列表（不含密码），用于与旧版本兼容或跨设备迁移连接配置。</p>
              <form class="section-form" @submit.prevent="exportConnections">
                <div class="form-actions">
                  <button type="submit" class="btn btn-muted" :disabled="exportConnectionsLoading">{{ exportConnectionsLoading ? '导出中...' : '导出连接' }}</button>
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
              <h3 class="section-heading">界面字体</h3>
              <form class="section-form" @submit.prevent="saveUiTypography">
                <div class="form-field">
                  <label class="form-label" for="appearance-font-family">界面字体</label>
                  <AppSelect
                    id="appearance-font-family"
                    v-model="appearanceForm.uiFontFamily"
                    class="timezone-select"
                    :options="uiFontFamilyOptions"
                    aria-label="选择界面字体"
                  />
                </div>
                <div class="form-field">
                  <label class="form-label" for="appearance-font-size">当前字号 (px)</label>
                  <input
                    id="appearance-font-size"
                    v-model.number="appearanceForm.uiFontSize"
                    class="form-control"
                    type="number"
                    min="8"
                    max="32"
                    step="1"
                  >
                  <small class="section-desc">默认字号为 {{ DEFAULT_UI_FONT_SIZE_BASE + DEFAULT_UI_FONT_SIZE_OFFSET }}px。这里会作用到整个界面的默认文本和显式字号规则。</small>
                </div>
                <div class="form-actions">
                  <button type="submit" class="btn btn-primary">保存</button>
                  <button type="button" class="btn btn-muted" @click="resetUiTypography">重置默认字体</button>
                  <p v-if="feedback.uiTypography?.message" :class="['feedback-msg', feedback.uiTypography.success ? 'feedback-ok' : 'feedback-error']">{{ feedback.uiTypography.message }}</p>
                </div>
              </form>
            </div>

            <hr class="section-divider">

            <div class="settings-section-content">
              <h3 class="section-heading">主题与背景</h3>
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
import { computed, nextTick, onMounted, onUnmounted, reactive, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { useRoute } from 'vue-router';
import AppSelect from '@/components/AppSelect.vue';
import AISettingsPanel from '@/components/AI/AISettingsPanel.vue';
import { authApi, connectionsApi, statusApi } from '@/lib/api';
import type { ImportResult } from '@/lib/api';
import { useUiNotificationsStore } from '@/stores/uiNotifications';
import {
  DEFAULT_UI_FONT_FAMILY,
  DEFAULT_UI_FONT_SIZE_BASE,
  DEFAULT_UI_FONT_SIZE_OFFSET,
  useAppearanceStore,
} from '@/stores/appearance';
import { useSettingsStore } from '@/stores/settings';

type AppVersionHost = {
  __APP_VERSION__?: unknown;
};

type TabKey = 'workspace' | 'ai' | 'system' | 'security' | 'dataManagement' | 'appearance' | 'about';
const SETTINGS_TAB_STORAGE_KEY = 'settings_active_tab';
const SETTINGS_FOCUS_SECTION_STORAGE_KEY = 'settings_focus_section';
const RELEASES_BASE_URL = 'https://github.com/maqibg/nexus-terminal-rust/releases';
const LATEST_RELEASE_API_URL = 'https://api.github.com/repos/maqibg/nexus-terminal-rust/releases/latest';

const tabs = computed<Array<{ key: TabKey; label: string }>>(() => [
  { key: 'workspace', label: '工作区' },
  { key: 'ai', label: 'AI 助手' },
  { key: 'system', label: '系统' },
  { key: 'security', label: '安全' },
  { key: 'dataManagement', label: '数据管理' },
  { key: 'appearance', label: '外观' },
  { key: 'about', label: '关于' },
]);

function getInitialActiveTab(): TabKey {
  const raw = localStorage.getItem(SETTINGS_TAB_STORAGE_KEY);
  if (raw && tabs.value.some((tab) => tab.key === raw)) {
    return raw as TabKey;
  }
  return 'workspace';
}

const notifications = useUiNotificationsStore();
const appearanceStore = useAppearanceStore();
const settingsStore = useSettingsStore();
const route = useRoute();
const { settings: runtimeSettings } = storeToRefs(settingsStore);
const { currentUiFontFamily, currentUiFontSize } = storeToRefs(appearanceStore);

const activeTab = ref<TabKey>(getInitialActiveTab());

watch(activeTab, (value) => {
  localStorage.setItem(SETTINGS_TAB_STORAGE_KEY, value);
  void maybeFocusRequestedSection();
});

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

const timezoneOptions = computed(() => commonTimezones.map(timezone => ({ value: timezone, label: timezone })));
const uiFontFamilyOptions = [
  { value: `'Segoe UI Variable Text', 'Segoe UI', 'Microsoft YaHei UI', 'Microsoft YaHei', sans-serif`, label: 'Segoe UI / 微软雅黑' },
  { value: `'Microsoft YaHei UI', 'Microsoft YaHei', 'Segoe UI', sans-serif`, label: '微软雅黑 UI' },
  { value: `'PingFang SC', 'Microsoft YaHei UI', 'Segoe UI', sans-serif`, label: '苹方 / 微软雅黑' },
  { value: `'Noto Sans SC', 'Segoe UI', 'Microsoft YaHei UI', sans-serif`, label: 'Noto Sans SC' },
  { value: `system-ui, 'Segoe UI', 'Microsoft YaHei UI', sans-serif`, label: '系统默认' },
];

const feedback = reactive<Record<string, { message: string; success: boolean }>>({});

const commandSyncTargetOptions = [
  { value: 'none', label: '无' },
  { value: 'quickCommands', label: '快捷指令' },
  { value: 'commandHistory', label: '命令历史' },
] as const;

const commandSyncMenuOpen = ref(false);
const commandSyncSelectRef = ref<HTMLElement | null>(null);
const dockerSettingsSectionRef = ref<HTMLElement | null>(null);
const loadingSettings = ref(false);
const settingsError = ref('');
const settingsMap = ref<Record<string, string>>({});

const commandSyncTargetLabel = computed(() => {
  const item = commandSyncTargetOptions.find((option) => option.value === workspaceForm.commandInputSyncTarget);
  return item?.label ?? '无';
});

function toggleCommandSyncMenu() {
  commandSyncMenuOpen.value = !commandSyncMenuOpen.value;
}

function selectCommandSyncTarget(value: 'none' | 'quickCommands' | 'commandHistory') {
  workspaceForm.commandInputSyncTarget = value;
  commandSyncMenuOpen.value = false;
}

function handleCommandSyncOutsideClick(event: MouseEvent) {
  if (!commandSyncMenuOpen.value) {
    return;
  }

  const target = event.target as Node | null;
  if (target && commandSyncSelectRef.value && !commandSyncSelectRef.value.contains(target)) {
    commandSyncMenuOpen.value = false;
  }
}

function handleCommandSyncEscape(event: KeyboardEvent) {
  if (event.key === 'Escape' && commandSyncMenuOpen.value) {
    commandSyncMenuOpen.value = false;
  }
}

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
  terminalShowSessionCloseConfirmation: true,
  fileManagerShowDeleteConfirmation: true,
  terminalEnableRightClickPaste: true,
  showStatusMonitorIpAddress: false,
  statusMonitorEnabled: true,
  statusMonitorIntervalSeconds: 3,
  dockerStatusIntervalSeconds: 5,
  dockerDefaultExpand: false,
  dockerUseSudo: false,
});
const appearanceForm = reactive({
  uiFontFamily: '',
  uiFontSize: DEFAULT_UI_FONT_SIZE_BASE + DEFAULT_UI_FONT_SIZE_OFFSET,
});

const systemForm = reactive({
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

// Full backup
const appExportLoading = ref(false);
const appExportMessage = ref('');
const appExportSuccess = ref(false);
const appImportLoading = ref(false);
const appImportMessage = ref('');
const appImportSuccess = ref(false);
const appImportResult = ref<ImportResult | null>(null);
const appImportFileInput = ref<HTMLInputElement | null>(null);

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
    return RELEASES_BASE_URL;
  }
  return `${RELEASES_BASE_URL}/tag/${latestVersion.value}`;
});


function setFeedback(key: string, message: string, success: boolean) {
  feedback[key] = { message, success };
}

async function maybeFocusRequestedSection() {
  if (activeTab.value !== 'workspace') {
    return;
  }

  const requested = localStorage.getItem(SETTINGS_FOCUS_SECTION_STORAGE_KEY)
    || (route.hash.startsWith('#') ? route.hash.slice(1) : '');
  if (requested !== 'docker') {
    return;
  }

  await nextTick();
  dockerSettingsSectionRef.value?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  localStorage.removeItem(SETTINGS_FOCUS_SECTION_STORAGE_KEY);
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
  workspaceForm.terminalShowSessionCloseConfirmation = toBool(map.terminalShowSessionCloseConfirmation, true);
  workspaceForm.fileManagerShowDeleteConfirmation = toBool(map.fileManagerShowDeleteConfirmation, true);
  workspaceForm.terminalEnableRightClickPaste = toBool(map.terminalEnableRightClickPaste, true);
  workspaceForm.showStatusMonitorIpAddress = toBool(map.showStatusMonitorIpAddress, false);
  workspaceForm.statusMonitorEnabled = toBool(map.statusMonitorEnabled, true);
  workspaceForm.statusMonitorIntervalSeconds = toInt(map.statusMonitorIntervalSeconds, 3);
  workspaceForm.dockerStatusIntervalSeconds = toInt(map.dockerStatusIntervalSeconds, 5);
  workspaceForm.dockerDefaultExpand = toBool(map.dockerDefaultExpand, false);
  workspaceForm.dockerUseSudo = toBool(map.dockerUseSudo, false);

  systemForm.timezone = map.timezone || 'Asia/Shanghai';
  appearanceForm.uiFontFamily = currentUiFontFamily.value;
  appearanceForm.uiFontSize = currentUiFontSize.value;
  settingsStore.setRuntimeLocale('zh-CN');
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
    await maybeFocusRequestedSection();
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

async function saveStatusMonitorEnabled() {
  try {
    await saveSetting('statusMonitorEnabled', workspaceForm.statusMonitorEnabled ? 'true' : 'false');
    await statusApi.setStatusMonitorEnabled(workspaceForm.statusMonitorEnabled);
    setFeedback('statusMonitorEnabled', '状态监控开关已保存', true);
  } catch (error) {
    setFeedback('statusMonitorEnabled', normalizeError(error, '保存失败'), false);
  }
}

async function saveDockerStatusInterval() {
  await saveWorkspaceNumber(
    'dockerStatusIntervalSeconds',
    workspaceForm.dockerStatusIntervalSeconds,
    'dockerStatusInterval',
    'Docker 刷新间隔设置已保存',
    1,
  );
}

async function saveUiTypography() {
  try {
    const fontSize = Number(appearanceForm.uiFontSize);
    if (!Number.isInteger(fontSize) || fontSize < 8 || fontSize > 32) {
      throw new Error('当前字号请输入 8 到 32 之间的整数');
    }

    await appearanceStore.setUiFontFamily(appearanceForm.uiFontFamily);
    await appearanceStore.setUiFontSizeOffset(fontSize - DEFAULT_UI_FONT_SIZE_BASE);
    appearanceForm.uiFontFamily = currentUiFontFamily.value;
    appearanceForm.uiFontSize = currentUiFontSize.value;
    setFeedback('uiTypography', '界面字体设置已保存', true);
  } catch (error) {
    setFeedback('uiTypography', normalizeError(error, '保存失败'), false);
  }
}

async function resetUiTypography() {
  try {
    await appearanceStore.setUiFontFamily(DEFAULT_UI_FONT_FAMILY);
    await appearanceStore.setUiFontSizeOffset(DEFAULT_UI_FONT_SIZE_OFFSET);
    appearanceForm.uiFontFamily = currentUiFontFamily.value;
    appearanceForm.uiFontSize = currentUiFontSize.value;
    setFeedback('uiTypography', '已恢复默认字体设置', true);
  } catch (error) {
    setFeedback('uiTypography', normalizeError(error, '重置失败'), false);
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

function triggerAppImport() {
  appImportFileInput.value?.click();
}

async function appExport() {
  appExportLoading.value = true;
  appExportMessage.value = '';
  appExportSuccess.value = false;
  try {
    const payload = await connectionsApi.appExport();
    const blob = new Blob([payload], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = `nexus_backup_${new Date().toISOString().slice(0, 10)}.json`;
    anchor.click();
    URL.revokeObjectURL(url);
    appExportMessage.value = '完整备份导出成功，文件已开始下载。';
    appExportSuccess.value = true;
    notifications.addNotification({ type: 'success', message: '完整备份导出成功' });
  } catch (error) {
    appExportMessage.value = normalizeError(error, '导出失败');
    appExportSuccess.value = false;
    notifications.addNotification({ type: 'error', message: appExportMessage.value });
  } finally {
    appExportLoading.value = false;
  }
}

async function appImport(event: Event) {
  const input = event.target as HTMLInputElement;
  const file = input.files?.[0];
  if (!file) return;
  // Reset so same file can be re-selected next time
  input.value = '';
  appImportLoading.value = true;
  appImportMessage.value = '';
  appImportSuccess.value = false;
  appImportResult.value = null;
  try {
    const json = await file.text();
    const result = await connectionsApi.appImport(json);
    appImportResult.value = result;
    appImportMessage.value = '导入成功。';
    appImportSuccess.value = true;
    notifications.addNotification({ type: 'success', message: '完整备份导入成功' });
  } catch (error) {
    appImportMessage.value = normalizeError(error, '导入失败');
    appImportSuccess.value = false;
    notifications.addNotification({ type: 'error', message: appImportMessage.value });
  } finally {
    appImportLoading.value = false;
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
    const response = await fetch(LATEST_RELEASE_API_URL, {
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
  document.addEventListener('mousedown', handleCommandSyncOutsideClick);
  window.addEventListener('keydown', handleCommandSyncEscape);
  await appearanceStore.loadAll().catch(() => undefined);
  await loadSettings();
  await checkLatestVersion();
});

onUnmounted(() => {
  document.removeEventListener('mousedown', handleCommandSyncOutsideClick);
  window.removeEventListener('keydown', handleCommandSyncEscape);
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
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 500;
  transition: all 0.15s ease;
}

.tab-btn:hover {
  background: var(--bg-surface1);
  color: var(--text);
}

.tab-btn.active {
  background: var(--blue);
  color: var(--button-text-color);
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
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(18px + var(--ui-font-size-offset));
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
  font-size: calc(16px + var(--ui-font-size-offset));
  font-weight: 600;
  color: var(--text);
}

.section-heading-inline {
  margin-bottom: 0;
}

.section-desc {
  margin: 0;
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
  box-sizing: border-box;
}

.form-control:focus {
  outline: none;
  border-color: var(--blue);
}

.command-sync-select-wrap {
  position: relative;
}

.command-sync-trigger {
  display: flex;
  align-items: center;
  justify-content: space-between;
  text-align: left;
  cursor: pointer;
  padding-right: 12px;
}

.command-sync-trigger .fa-chevron-down {
  color: var(--text-sub);
  font-size: calc(11px + var(--ui-font-size-offset));
  transition: transform 0.15s ease;
}

.command-sync-select-wrap.open .fa-chevron-down {
  transform: rotate(180deg);
}

.command-sync-menu {
  position: absolute;
  top: calc(100% + 2px);
  left: 0;
  right: 0;
  border: 1px solid var(--border);
  border-radius: 0 0 10px 10px;
  background: var(--bg-base);
  box-shadow: 0 12px 24px rgba(0, 0, 0, 0.35);
  overflow: hidden;
  z-index: 80;
}

.command-sync-option {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--text);
  text-align: left;
  padding: 5px 10px;
  font-size: calc(13px + var(--ui-font-size-offset));
  line-height: 1.2;
  cursor: pointer;
}

.command-sync-option:hover {
  background: color-mix(in srgb, var(--blue) 20%, var(--bg-base));
}

.command-sync-option.active {
  background: var(--blue);
  color: var(--button-text-color);
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
  font-size: calc(13px + var(--ui-font-size-offset));
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

.form-actions-top-padding {
  padding-top: 2px;
}

.command-sync-actions {
  justify-content: space-between;
}

.command-sync-actions .feedback-msg {
  margin-left: auto;
}

.btn {
  height: 34px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0 14px;
  font-size: calc(13px + var(--ui-font-size-offset));
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
  color: var(--button-text-color);
}

.btn-danger:hover {
  opacity: 0.9;
}

.btn-xs {
  height: 28px;
  padding: 0 10px;
  font-size: calc(12px + var(--ui-font-size-offset));
}

.feedback-msg {
  margin: 0;
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  font-size: calc(12px + var(--ui-font-size-offset));
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
  background: var(--ui-switch-knob);
  transition: all 0.2s ease;
}

.switch-btn.enabled {
  background: var(--blue);
}

.switch-btn.enabled .switch-dot {
  left: 24px;
}

.table-title {
  font-size: calc(14px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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
  font-size: calc(13px + var(--ui-font-size-offset));
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

.import-result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(120px, 1fr));
  gap: 8px;
  margin-top: 12px;
}

.import-result-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-surface1);
  font-size: calc(13px + var(--ui-font-size-offset));
}

.import-result-item .label {
  color: var(--text-sub);
  font-size: calc(11px + var(--ui-font-size-offset));
  margin-bottom: 2px;
}

.import-result-item .value {
  color: var(--text-main);
  font-weight: 600;
  font-size: calc(18px + var(--ui-font-size-offset));
}
</style>


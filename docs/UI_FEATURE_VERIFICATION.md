# UI 功能逐项验证记录

> 本文件对应 `docs/UI_FEATURE_INVENTORY.md` 的同编号条目：按顺序逐项核对“代码是否实现 + 前端是否可用”。

## 0. 验证口径

- 静态验证：能从 UI 入口追到实际事件处理/调用链（store/composable/api/tauri command），不依赖“猜测”。
- 可用性验证：至少完成 `pnpm --dir apps/desktop/frontend build`；若存在测试则尽量跑 `pnpm --dir apps/desktop/frontend test`；Rust 侧至少 `cargo test --workspace` 一轮（结果写入本文件）。

## 1. 全局构建/测试结果（待填）

- `pnpm --dir apps/desktop/frontend build`：✅ 通过（vite build 成功，含 chunk size warning）
- `pnpm --dir apps/desktop/frontend test`：✅ 通过（2 files / 11 tests）
- `cargo test --workspace`：✅ 通过（含 `crates/proxy-core` 与 `crates/storage-sqlite` tests）

## 2. 条目验证表（按清单顺序）

状态：
- ✅ 已实现（静态链路闭合；必要时附带构建/测试支撑）
- ⚠️ 部分实现/有条件可用（写清前置条件/限制）
- ❌ 未实现或链路断裂（UI 存在但无后端/无调用/不可达）
- ➖ 未验证（尚未轮到或需要运行态验证）

| ID | 状态 | 验证方式 | 证据（最短链路） | 备注 |
|---:|:---:|---|---|---|
| F001 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（router-link）+ `apps/desktop/frontend/src/main.ts`（routes） |  |
| F002 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（固定 href 外链） |  |
| F003 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（showGlobalAiPanel + TerminalAIChatPanel） | 需登录；AI 渠道/模型需配置 |
| F004 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（toggleStyleCustomizer）+ `apps/desktop/frontend/src/stores/appearance.ts` |  |
| F005 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（handleLogout）+ `apps/desktop/frontend/src/stores/auth.ts` |  |
| F006 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（minimizeWindow -> appWindow.minimize） |  |
| F007 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（toggleMaximize -> appWindow.toggleMaximize） |  |
| F008 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（closeWindow -> appWindow.close） |  |
| F009 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（checkBackendStartup）+ `apps/desktop/frontend/src/lib/api-status.ts`（get_backend_health） |  |
| F010 | ✅ | 静态 | `apps/desktop/frontend/src/components/UINotificationDisplay.vue` + `apps/desktop/frontend/src/stores/uiNotifications.ts` |  |
| F011 | ✅ | 静态 | `apps/desktop/frontend/src/components/GlobalAlertDialog.vue` + `apps/desktop/frontend/src/stores/dialog.ts` |  |
| F012 | ✅ | 静态 | `apps/desktop/frontend/src/components/GlobalConfirmDialog.vue` + `apps/desktop/frontend/src/stores/dialog.ts` |  |
| F013 | ✅ | 静态 | `apps/desktop/frontend/src/components/SshKeyConfirmModal.vue`（sshApi.acceptHostKey）+ `apps/desktop/frontend/src/lib/api-ssh.ts` |  |
| F014 | ✅ | 静态 | `apps/desktop/frontend/src/components/FocusSwitcherConfigurator.vue` + `apps/desktop/frontend/src/stores/focusSwitcher.ts` |  |
| F015 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（contextmenu prevent） |  |
| F020 | ✅ | 静态 | `apps/desktop/frontend/src/views/Setup.vue`（auth.setup）+ `apps/desktop/frontend/src/stores/auth.ts` |  |
| F021 | ✅ | 静态 | `apps/desktop/frontend/src/views/Login.vue`（auth.login）+ `apps/desktop/frontend/src/stores/auth.ts` |  |
| F022 | ✅ | 静态 | `apps/desktop/frontend/src/views/Login.vue`（auth.verify2fa）+ `apps/desktop/frontend/src/lib/api-auth.ts`（auth_verify_2fa） |  |
| F023 | ✅ | 静态 | `apps/desktop/frontend/src/main.ts`（beforeEach guard） |  |
| F030 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（onMounted store.fetch）+ `apps/desktop/frontend/src/stores/connections.ts`（connectionsApi.list/tagList） |  |
| F031 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（toggleBatchEditMode） |  |
| F032 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（searchQuery + filteredAndSortedConnections） |  |
| F033 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（tag dropdown + handleCreateTagFromFilter/handleDeleteTagFromFilter）+ `apps/desktop/frontend/src/lib/api-connections.ts`（tag_create/tag_delete） |  |
| F034 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（sortOptions/localSortBy） |  |
| F035 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（toggleSortOrder） |  |
| F036 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（openAddConnectionForm）+ `apps/desktop/frontend/src/components/AddConnectionForm.vue` |  |
| F037 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（handleTestAllFilteredConnections -> connectionsApi.test） |  |
| F038 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（handleConnectAllFilteredConnections -> connectTo -> connectConnection）+ `apps/desktop/frontend/src/composables/useSessionLifecycle.ts` |  |
| F039 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（selectedConnectionIdsForBatch 操作） |  |
| F040 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（openBatchEditModal）+ `apps/desktop/frontend/src/components/BatchEditConnectionForm.vue`（connectionsApi.update） |  |
| F041 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（useConfirmDialog + 批量 delete） |  |
| F042 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（handleTestSingleConnection -> connectionsApi.test） |  |
| F043 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（openEditConnectionForm） |  |
| F044 | ✅ | 静态 | `apps/desktop/frontend/src/views/Connections.vue`（connectTo -> router.push('/workspace')）+ `apps/desktop/frontend/src/composables/useSessionLifecycle.ts` |  |
| F045 | ✅ | 静态 | `apps/desktop/frontend/src/components/AddConnectionFormBasicInfo.vue` |  |
| F046 | ✅ | 静态 | `apps/desktop/frontend/src/components/AddConnectionFormAuth.vue` + `apps/desktop/frontend/src/lib/api-connections.ts`（ssh_key_list 等） |  |
| F047 | ✅ | 静态 | `apps/desktop/frontend/src/components/AddConnectionFormAdvanced.vue` |  |
| F048 | ✅ | 静态 | `apps/desktop/frontend/src/components/AddConnectionForm.vue`（script mode）+ `apps/desktop/frontend/src/composables/useAddConnectionForm.ts` |  |
| F049 | ✅ | 静态 | `apps/desktop/frontend/src/components/AddConnectionForm.vue`（handleTestConnection）+ `apps/desktop/frontend/src/composables/useAddConnectionForm.ts`（connectionsApi.testUnsaved） |  |
| F050 | ✅ | 静态 | `apps/desktop/frontend/src/composables/useAddConnectionForm.ts`（connectionsApi.create/update） |  |
| F051 | ✅ | 静态 | `apps/desktop/frontend/src/components/AddConnectionForm.vue`（handleDeleteConnection）+ `apps/desktop/frontend/src/composables/useAddConnectionForm.ts`（connectionsApi.delete） |  |
| F052 | ✅ | 静态 | `apps/desktop/frontend/src/composables/useAddConnectionForm.ts`（handleCreateTag/handleDeleteTag -> tagsStore） |  |
| F053 | ✅ | 静态 | `apps/desktop/frontend/src/components/BatchEditConnectionForm.vue`（connectionsApi.update） |  |
| F060 | ✅ | 静态 | `apps/desktop/frontend/src/views/Proxies.vue`（store.fetchAll）+ `apps/desktop/frontend/src/stores/proxies.ts`（proxyList） |  |
| F061 | ✅ | 静态 | `apps/desktop/frontend/src/views/Proxies.vue`（showForm）+ `apps/desktop/frontend/src/components/AddProxyForm.vue` |  |
| F062 | ✅ | 静态 | `apps/desktop/frontend/src/views/Proxies.vue`（confirm + store.remove）+ `apps/desktop/frontend/src/stores/proxies.ts`（proxyDelete） |  |
| F063 | ✅ | 静态 | `apps/desktop/frontend/src/components/AddProxyForm.vue`（store.create）+ `apps/desktop/frontend/src/stores/proxies.ts` |  |
| F070 | ✅ | 静态 | `apps/desktop/frontend/src/components/TerminalTabBar.vue`（emit activate/close/closeOthers/...）+ `apps/desktop/frontend/src/views/Workspace.vue`（handlers） |  |
| F071 | ✅ | 静态 | `apps/desktop/frontend/src/views/Workspace.vue`（toggleLeftToolPane） |  |
| F072 | ✅ | 静态 | `apps/desktop/frontend/src/views/Workspace.vue`（closeLeftToolPane） |  |
| F073 | ✅ | 静态 | `apps/desktop/frontend/src/components/WorkspaceConnectionList.vue`（@select -> connectConnection）+ `apps/desktop/frontend/src/composables/useSessionLifecycle.ts` |  |
| F074 | ⚠️ | 静态 | `apps/desktop/frontend/src/components/DockerManager.vue` + `apps/desktop/frontend/src/composables/useDockerManager.ts`（sshApi.exec docker） | 依赖 SSH 会话；且 `docs/README.md` 标注为桌面端排除项 |
| F075 | ✅ | 静态 | `apps/desktop/frontend/src/views/Workspace.vue`（Splitpanes + resize 持久化）+ `apps/desktop/frontend/src/stores/layout.ts` |  |
| F076 | ✅ | 静态 | `apps/desktop/frontend/src/components/LayoutRenderer.vue`（pane -> componentMap） |  |
| F077 | ✅ | 静态 | `apps/desktop/frontend/src/views/Workspace.vue`（TerminalAIChatPanel 右侧面板） | 需 AI 配置 |
| F078 | ✅ | 静态 | `apps/desktop/frontend/src/views/Workspace.vue`（showConnList + WorkspaceConnectionList） |  |
| F079 | ✅ | 静态 | `apps/desktop/frontend/src/views/Workspace.vue`（SftpBrowser popup）+ `apps/desktop/frontend/src/composables/useSftpBrowser.ts`（sftpApi.*） |  |
| F080 | ✅ | 静态 | `apps/desktop/frontend/src/components/FileEditorOverlay.vue` + `apps/desktop/frontend/src/stores/fileEditor.ts` |  |
| F081 | ✅ | 静态 | `apps/desktop/frontend/src/composables/useTransferProgress.ts`（transferApi.list/pause/resume/cancel/cleanup） |  |
| F082 | ✅ | 静态 | `apps/desktop/frontend/src/components/LayoutConfigurator.vue` + `apps/desktop/frontend/src/stores/layout.ts`（saveLayout/resetLayout） |  |
| F083 | ✅ | 静态 | `apps/desktop/frontend/src/stores/layout.ts`（settingsStore.set: layoutConfig/leftSize/...） |  |
| F084 | ✅ | 静态 | `apps/desktop/frontend/src/components/TerminalView.vue`（KeepAlive SessionTerminalView） |  |
| F085 | ✅ | 静态 | `apps/desktop/frontend/src/components/SessionTerminalView.vue`（sshApi.write/onSshOutput）+ `apps/desktop/frontend/src/lib/api-ssh.ts` |  |
| F086 | ✅ | 静态 | `apps/desktop/frontend/src/components/SessionTerminalView.vue`（terminalBackgroundSrcdoc）+ `apps/desktop/frontend/src/stores/appearance.ts`（terminalCustomHTML） |  |
| F087 | ✅ | 静态 | `apps/desktop/frontend/src/components/SessionTerminalView.vue`（context menu actions） |  |
| F088 | ✅ | 静态 | `apps/desktop/frontend/src/components/SessionTerminalView.vue`（ai-* actions -> 触发全局/终端 AI）+ `apps/desktop/frontend/src/stores/ai.ts` |  |
| F089 | ✅ | 静态 | `apps/desktop/frontend/src/components/CommandAutocomplete.vue`（historyApi/quickCommandApi） |  |
| F090 | ✅ | 静态 | `apps/desktop/frontend/src/utils/inline-suggest.ts` + `apps/desktop/frontend/src/components/SessionTerminalView.vue` |  |
| F091 | ⚠️ | 静态 | `apps/desktop/frontend/src/composables/useSessionLifecycle.ts`（desktopApi.openVncConnection）+ `apps/desktop/src-tauri/src/commands/desktop.rs`（desktop_open_vnc*） | `docs/README.md` 标注排除；可用性依赖 noVNC/WS 端口 |
| F092 | ✅ | 静态 | `apps/desktop/frontend/src/components/CommandInputBar.vue`（sshApi.write/historyApi.add + search events） |  |
| F093 | ✅ | 静态 | `apps/desktop/frontend/src/components/StatusMonitor.vue` + `apps/desktop/frontend/src/composables/useStatusMonitor.ts`（statusApi.getConnectionRuntimeStatus） |  |
| F094 | ✅ | 静态 | `apps/desktop/frontend/src/components/CommandHistoryPanel.vue`（useCommandHistoryStore）+ `apps/desktop/frontend/src/stores/commandHistory.ts`（historyApi.*） |  |
| F095 | ✅ | 静态 | `apps/desktop/frontend/src/components/QuickCommandsPanel.vue`（useQuickCommandsStore）+ `apps/desktop/frontend/src/stores/quickCommands.ts`（quickCommandApi.*） |  |
| F096 | ✅ | 静态 | `apps/desktop/frontend/src/components/FileEditorContainer.vue` + `apps/desktop/frontend/src/stores/fileEditor.ts` |  |
| F097 | ✅ | 静态 | `apps/desktop/frontend/src/composables/useSftpBrowser.ts`（open/list/read/write/rename/delete/chmod/download/upload）+ `apps/desktop/frontend/src/lib/api-sftp.ts` |  |
| F098 | ✅ | 静态 | `apps/desktop/frontend/src/components/FavoritePaths.vue` + `apps/desktop/frontend/src/lib/api-auxiliary.ts`（favoritePathApi.*） |  |
| F099 | ✅ | 静态 | `apps/desktop/frontend/src/components/PathHistoryDropdown.vue` + `apps/desktop/frontend/src/lib/api-auxiliary.ts`（pathHistoryApi.*） |  |
| F100 | ✅ | 静态 | `apps/desktop/frontend/src/components/FileUploadPopup.vue` + `apps/desktop/frontend/src/lib/api-sftp.ts`（upload*） |  |
| F101 | ✅ | 静态 | `apps/desktop/frontend/src/components/SuspendedSshSessionsModal.vue` + `apps/desktop/frontend/src/lib/api-ssh-suspend.ts`（ssh_suspend_*） |  |
| F102 | ✅ | 静态 | `apps/desktop/frontend/src/App.vue`（Alt 焦点切换）+ `apps/desktop/frontend/src/stores/focusSwitcher.ts` |  |
| F110 | ✅ | 静态 | `apps/desktop/frontend/src/views/Statistics.vue`（timeRangeOptions + 计算属性） |  |
| F111 | ✅ | 静态 | `apps/desktop/frontend/src/views/Statistics.vue`（AppSelect v-model displayCurrency） |  |
| F112 | ✅ | 静态 | `apps/desktop/frontend/src/views/Statistics.vue`（refreshData） |  |
| F113 | ✅ | 静态 | `apps/desktop/frontend/src/views/Statistics.vue`（showSettingsDialog + save/reset） |  |
| F114 | ✅ | 静态 | `apps/desktop/frontend/src/views/Statistics.vue`（summary-grid） |  |
| F115 | ✅ | 静态 | `apps/desktop/frontend/src/views/Statistics.vue`（panel-grid + visibleComponents） |  |
| F130 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（tabs + activeTab） |  |
| F131 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（saveWorkspaceBoolean showPopupFileEditor）+ `apps/desktop/frontend/src/components/CommandInputBar.vue`（showPopupFileEditor） |  |
| F132 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（saveWorkspaceBoolean showPopupFileManager）+ `apps/desktop/frontend/src/components/CommandInputBar.vue`（showPopupFileManager） |  |
| F133 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（shareFileEditorTabs）+ `apps/desktop/frontend/src/composables/useSftpBrowser.ts`（tabId 作用域） |  |
| F134 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（autoCopyOnSelect）+ `apps/desktop/frontend/src/components/SessionTerminalView.vue`（getSettingBoolean） |  |
| F135 | ✅ | 静态 | `apps/desktop/frontend/src/stores/layout.ts`（left/right/header/locked 持久化） |  |
| F136 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（commandInputSyncTarget）+ `apps/desktop/frontend/src/components/CommandInputBar.vue`（dispatch nexus:*:set-search） |  |
| F137 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（showConnectionTags）+ `apps/desktop/frontend/src/views/Connections.vue`（v-if showConnectionTags） |  |
| F138 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（showQuickCommandTags）+ `apps/desktop/frontend/src/components/QuickCommandsPanel.vue` |  |
| F139 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（terminalScrollbackLimit）+ `apps/desktop/frontend/src/components/SessionTerminalView.vue`（getSettingInteger） |  |
| F140 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（terminalShowSessionCloseConfirmation）+ `apps/desktop/frontend/src/views/Workspace.vue`（closeSession 前 confirm） |  |
| F141 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（fileManagerShowDeleteConfirmation）+ `apps/desktop/frontend/src/composables/useSftpBrowser.ts`（shouldConfirm） |  |
| F142 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（terminalEnableRightClickPaste）+ `apps/desktop/frontend/src/components/SessionTerminalView.vue`（getSettingBoolean） |  |
| F143 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（showStatusMonitorIpAddress）+ `apps/desktop/frontend/src/components/StatusMonitor.vue` |  |
| F144 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（statusMonitorIntervalSeconds）+ `apps/desktop/frontend/src/composables/useStatusMonitor.ts` |  |
| F145 | ⚠️ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（docker* settings）+ `apps/desktop/frontend/src/composables/useDockerManager.ts` | 同 F074：文档标注排除；依赖 SSH/docker 环境 |
| F146 | ✅ | 静态 | `apps/desktop/frontend/src/components/AI/AISettingsPanel.vue`（aiStore.loadAll/add/update/verify/fetchModels）+ `apps/desktop/frontend/src/lib/api-ai.ts` |  |
| F147 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（timezoneOptions + saveSetting('timezone')） |  |
| F148 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（authApi.changePassword）+ `apps/desktop/frontend/src/lib/api-auth.ts` |  |
| F149 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（connectionsApi.appExport）+ `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F150 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（connectionsApi.appImport）+ `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F151 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（connectionsApi.export/import）+ `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F152 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（UI 字体）+ `apps/desktop/frontend/src/stores/appearance.ts`（setUiFontFamily/size） |  |
| F153 | ✅ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（toggleStyleCustomizer）+ `apps/desktop/frontend/src/stores/appearance.ts`（主题/背景/HTML） |  |
| F154 | ⚠️ | 静态 | `apps/desktop/frontend/src/views/Settings.vue`（LATEST_RELEASE_API_URL + RELEASES_BASE_URL） | 需联网访问 GitHub API |
| F173 | ⚠️ | 静态 | `apps/desktop/frontend/src/views/Notifications.vue` + `apps/desktop/frontend/src/lib/api-settings.ts` | 未路由接入 |

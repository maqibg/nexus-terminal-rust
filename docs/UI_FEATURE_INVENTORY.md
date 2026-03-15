# UI 控件与功能清单（盘点）

> 目标：把当前桌面端前端“可见控件/可触发功能”做成可逐项验证的清单；每条都给到入口与代码定位，供后续逐条核对。

## 1. 盘点范围

- 路由接入（来自 `apps/desktop/frontend/src/main.ts`）：`/setup`、`/login`、`/connections`、`/workspace`、`/databases`、`/tools`、`/proxies`、`/statistics`、`/settings`
- 全局壳（来自 `apps/desktop/frontend/src/App.vue`）：顶部导航、窗口控制、全局 AI/外观/焦点切换、全局对话框与通知
- 工作区布局 pane（来自 `apps/desktop/frontend/src/components/LayoutRenderer.vue` + `apps/desktop/frontend/src/stores/layout.ts`）：`connections/dockerManager/terminal/fileManager/editor/commandBar/statusMonitor/commandHistory/quickCommands`
- 未接入路由但仍在代码树内的 `views/*`：单独列出（不保证可从 UI 访问）

## 2. 清单（按顺序编号）

说明：
- “控件/入口”尽量按 UI 可见文本/图标描述；“代码定位”给最短可追踪路径（页面/组件/关键 store 或 API）。
- 本清单只做“盘点”，实现与可用性在 `docs/UI_FEATURE_VERIFICATION.md` 逐项验证。

| ID | 页面/模块 | 控件/入口 | 功能 | 代码定位 | 备注 |
|---:|---|---|---|---|---|
| F001 | 全局 | 顶部导航：连接/工作区/数据库/工具箱/代理/统计/设置 | 页面切换 | `apps/desktop/frontend/src/App.vue` + `apps/desktop/frontend/src/main.ts` |  |
| F002 | 全局 | GitHub 图标按钮 | 打开仓库链接 | `apps/desktop/frontend/src/App.vue` | 外链 |
| F003 | 全局 | 机器人图标按钮 | 打开/关闭全局 AI 侧栏 | `apps/desktop/frontend/src/App.vue` + `apps/desktop/frontend/src/components/AI/TerminalAIChatPanel.vue` |  |
| F004 | 全局 | 画笔图标按钮 | 打开外观定制器（StyleCustomizer） | `apps/desktop/frontend/src/App.vue` + `apps/desktop/frontend/src/stores/appearance.ts` |  |
| F005 | 全局 | “退出登录” | 退出并跳转登录页 | `apps/desktop/frontend/src/App.vue` + `apps/desktop/frontend/src/stores/auth.ts` |  |
| F006 | 全局 | 窗口按钮：最小化 | 最小化窗口 | `apps/desktop/frontend/src/App.vue` | Tauri window API |
| F007 | 全局 | 窗口按钮：最大化/还原 | 切换最大化 | `apps/desktop/frontend/src/App.vue` |  |
| F008 | 全局 | 窗口按钮：关闭 | 关闭窗口 | `apps/desktop/frontend/src/App.vue` |  |
| F009 | 全局 | 后端启动状态横幅 | 展示 starting/error 状态与错误信息 | `apps/desktop/frontend/src/App.vue` + `apps/desktop/frontend/src/lib/api-status.ts` |  |
| F010 | 全局 | UI 通知浮层 | 展示 success/error/info 通知 | `apps/desktop/frontend/src/components/UINotificationDisplay.vue` + `apps/desktop/frontend/src/stores/uiNotifications.ts` |  |
| F011 | 全局 | 全局 AlertDialog | 错误/提示弹窗 | `apps/desktop/frontend/src/components/GlobalAlertDialog.vue` + `apps/desktop/frontend/src/stores/dialog.ts` |  |
| F012 | 全局 | 全局 ConfirmDialog | 二次确认弹窗 | `apps/desktop/frontend/src/components/GlobalConfirmDialog.vue` + `apps/desktop/frontend/src/stores/dialog.ts` |  |
| F013 | 全局 | SSH 主机指纹确认弹窗 | 新主机指纹确认/接受 | `apps/desktop/frontend/src/components/SshKeyConfirmModal.vue` + `apps/desktop/frontend/src/lib/api-ssh.ts` |  |
| F014 | 全局 | FocusSwitcherConfigurator | 配置焦点切换器 | `apps/desktop/frontend/src/components/FocusSwitcherConfigurator.vue` + `apps/desktop/frontend/src/stores/focusSwitcher.ts` |  |
| F015 | 全局 | 禁止浏览器右键菜单 | 屏蔽默认 contextmenu | `apps/desktop/frontend/src/App.vue` |  |

| F020 | 认证 | `/setup` 初始设置表单 | 创建管理员账户 | `apps/desktop/frontend/src/views/Setup.vue` + `apps/desktop/frontend/src/stores/auth.ts` |  |
| F021 | 认证 | `/login` 登录表单 | 用户名/密码登录 | `apps/desktop/frontend/src/views/Login.vue` + `apps/desktop/frontend/src/stores/auth.ts` |  |
| F022 | 认证 | `/login` 2FA 输入 | 2FA 验证并登录 | `apps/desktop/frontend/src/views/Login.vue` + `apps/desktop/frontend/src/lib/api-auth.ts` |  |
| F023 | 认证 | 路由守卫 | NeedsSetup/未登录重定向 | `apps/desktop/frontend/src/main.ts` |  |

| F030 | 连接管理 | 连接列表加载 | 拉取连接与标签 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/stores/connections.ts` |  |
| F031 | 连接管理 | “批量修改”开关 | 进入/退出批量选择模式 | `apps/desktop/frontend/src/views/Connections.vue` |  |
| F032 | 连接管理 | 搜索框 | 按关键字过滤连接 | `apps/desktop/frontend/src/views/Connections.vue` |  |
| F033 | 连接管理 | 标签筛选下拉 | 按标签过滤连接；可在下拉内新建/删除标签 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` | 受设置项控制 |
| F034 | 连接管理 | 排序字段下拉 | 切换排序字段 | `apps/desktop/frontend/src/views/Connections.vue` |  |
| F035 | 连接管理 | 升/降序按钮 | 切换排序方向 | `apps/desktop/frontend/src/views/Connections.vue` |  |
| F036 | 连接管理 | “+”新建按钮 | 打开新建连接表单 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/components/AddConnectionForm.vue` |  |
| F037 | 连接管理 | “测试全部” | 并发测试筛选后的 SSH 连接 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F038 | 连接管理 | “连接全部” | 并发连接筛选后的 SSH 连接 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/composables/useSessionLifecycle.ts` |  |
| F039 | 连接管理 | 批量：全选/取消/反选 | 批量选择连接 | `apps/desktop/frontend/src/views/Connections.vue` |  |
| F040 | 连接管理 | 批量：“编辑选中” | 打开批量编辑表单 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/components/BatchEditConnectionForm.vue` |  |
| F041 | 连接管理 | 批量：“删除选中” | 确认后批量删除 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/composables/useConfirmDialog.ts` |  |
| F042 | 连接管理 | 单条：SSH 测试 | 测试单条 SSH 连接并显示结果 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F043 | 连接管理 | 单条：编辑 | 打开编辑连接表单 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/components/AddConnectionForm.vue` |  |
| F044 | 连接管理 | 单条：连接 | 创建会话并进入工作区 | `apps/desktop/frontend/src/views/Connections.vue` + `apps/desktop/frontend/src/composables/useSessionLifecycle.ts` |  |
| F045 | 连接表单 | 基本信息 | 名称/类型/主机/端口等 | `apps/desktop/frontend/src/components/AddConnectionFormBasicInfo.vue` |  |
| F046 | 连接表单 | 认证信息 | 密码/Key/Key 选择等 | `apps/desktop/frontend/src/components/AddConnectionFormAuth.vue` + `apps/desktop/frontend/src/stores/sshKeys.ts` |  |
| F047 | 连接表单 | 高级设置 | 代理/跳板链/标签/备注/RDP/VNC 等 | `apps/desktop/frontend/src/components/AddConnectionFormAdvanced.vue` |  |
| F048 | 连接表单 | 脚本模式开关 + 文本域 | 多行批量创建连接 | `apps/desktop/frontend/src/components/AddConnectionForm.vue` + `apps/desktop/frontend/src/composables/useAddConnectionForm.ts` |  |
| F049 | 连接表单 | “测试连接” | 测试未保存 SSH 连接 | `apps/desktop/frontend/src/components/AddConnectionForm.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F050 | 连接表单 | “保存/创建” | 创建或更新连接 | `apps/desktop/frontend/src/composables/useAddConnectionForm.ts` + `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F051 | 连接表单 | “删除” | 删除连接 | `apps/desktop/frontend/src/components/AddConnectionForm.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F052 | 连接表单 | 标签：创建/删除 | 创建/删除连接标签 | `apps/desktop/frontend/src/components/AddConnectionFormAdvanced.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F053 | 批量编辑表单 | 批量更新字段 | 保存/取消批量修改 | `apps/desktop/frontend/src/components/BatchEditConnectionForm.vue` |  |

| F060 | 代理管理 | 代理列表 | 拉取/展示代理 | `apps/desktop/frontend/src/views/Proxies.vue` + `apps/desktop/frontend/src/stores/proxies.ts` |  |
| F061 | 代理管理 | “新建” | 打开 AddProxyForm | `apps/desktop/frontend/src/views/Proxies.vue` + `apps/desktop/frontend/src/components/AddProxyForm.vue` |  |
| F062 | 代理管理 | “删除” | 确认后删除代理 | `apps/desktop/frontend/src/views/Proxies.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` |  |
| F063 | 代理表单 | 添加代理 | 新建代理并保存 | `apps/desktop/frontend/src/components/AddProxyForm.vue` |  |

| F070 | 工作区 | TerminalTabBar | 会话 tabs：激活/关闭/批量关闭/打开 AI/切换 header/打开传输/打开布局/新增会话 | `apps/desktop/frontend/src/views/Workspace.vue` + `apps/desktop/frontend/src/components/TerminalTabBar.vue` |  |
| F071 | 工作区 | 左侧工具栏按钮 | 打开连接列表/（Docker）/AI | `apps/desktop/frontend/src/views/Workspace.vue` |  |
| F072 | 工作区 | 左侧面板头部 | 面板标题 + 关闭按钮 | `apps/desktop/frontend/src/views/Workspace.vue` |  |
| F073 | 工作区 | WorkspaceConnectionList | 选择连接触发连接 | `apps/desktop/frontend/src/components/WorkspaceConnectionList.vue` + `apps/desktop/frontend/src/composables/useSessionLifecycle.ts` |  |
| F074 | 工作区 | DockerManager | Docker 管理器 UI | `apps/desktop/frontend/src/components/DockerManager.vue` + `apps/desktop/frontend/src/composables/useDockerManager.ts` | 文档标注“排除项” |
| F075 | 工作区 | Splitpanes | 主布局拖拽 resize（含锁定） | `apps/desktop/frontend/src/views/Workspace.vue` + `apps/desktop/frontend/src/stores/layout.ts` |  |
| F076 | 工作区 | LayoutRenderer | 根据布局节点渲染 pane 组件 | `apps/desktop/frontend/src/components/LayoutRenderer.vue` |  |
| F077 | 工作区 | 右侧 AI 面板 | TerminalAIChatPanel（会话关联） | `apps/desktop/frontend/src/views/Workspace.vue` + `apps/desktop/frontend/src/components/AI/TerminalAIChatPanel.vue` |  |
| F078 | 工作区 | “选择连接”弹窗 | 从弹窗选择连接并连接 | `apps/desktop/frontend/src/views/Workspace.vue` |  |
| F079 | 工作区 | 弹窗文件管理器 | 弹窗打开 SftpBrowser | `apps/desktop/frontend/src/views/Workspace.vue` + `apps/desktop/frontend/src/components/SftpBrowser.vue` |  |
| F080 | 工作区 | FileEditorOverlay | 全局文件编辑遮罩 | `apps/desktop/frontend/src/components/FileEditorOverlay.vue` + `apps/desktop/frontend/src/stores/fileEditor.ts` |  |
| F081 | 工作区 | TransferProgressModal | 传输任务列表：暂停/继续/取消/清理 | `apps/desktop/frontend/src/components/TransferProgressModal.vue` + `apps/desktop/frontend/src/lib/api-transfer.ts` |  |
| F082 | 工作区 | LayoutConfigurator | 工作区布局配置/保存/重置 | `apps/desktop/frontend/src/components/LayoutConfigurator.vue` + `apps/desktop/frontend/src/stores/layout.ts` |  |
| F083 | 工作区 | 布局持久化 | header/侧栏/锁定/布局 JSON 持久化 | `apps/desktop/frontend/src/stores/layout.ts` + `apps/desktop/frontend/src/stores/settings.ts` |  |
| F084 | 工作区 | TerminalView | 显示活动会话（KeepAlive） | `apps/desktop/frontend/src/components/TerminalView.vue` + `apps/desktop/frontend/src/stores/session.ts` |  |
| F085 | 终端 | SessionTerminalView | SSH 终端渲染/输出监听/输入写入/自适应 resize | `apps/desktop/frontend/src/components/SessionTerminalView.vue` + `apps/desktop/frontend/src/lib/api-ssh.ts` |  |
| F086 | 终端 | 终端背景 HTML | `terminal_custom_html` 渲染到 iframe | `apps/desktop/frontend/src/components/SessionTerminalView.vue` + `apps/desktop/frontend/src/stores/appearance.ts` |  |
| F087 | 终端 | 右键菜单 | 复制/粘贴/全选/清屏/AI 动作 | `apps/desktop/frontend/src/components/SessionTerminalView.vue` |  |
| F088 | 终端 | 选区 AI 浮条 | AI 撰写/解释/优化 | `apps/desktop/frontend/src/components/SessionTerminalView.vue` + `apps/desktop/frontend/src/stores/ai.ts` |  |
| F089 | 终端 | CommandAutocomplete | 命令补全下拉：选择/关闭 | `apps/desktop/frontend/src/components/CommandAutocomplete.vue` |  |
| F090 | 终端 | Inline Suggestion | 行内幽灵提示 | `apps/desktop/frontend/src/components/SessionTerminalView.vue` + `apps/desktop/frontend/src/utils/inline-suggest.ts` |  |
| F091 | 终端 | VncSessionView | VNC 会话视图 | `apps/desktop/frontend/src/components/VncSessionView.vue` + `apps/desktop/frontend/src/lib/api-desktop.ts` | 文档标注“排除项” |
| F092 | 工作区 | CommandInputBar | 发送命令/历史/清屏/搜索/焦点配置/弹窗入口/输入同步 | `apps/desktop/frontend/src/components/CommandInputBar.vue` |  |
| F093 | 工作区 | StatusMonitor | 状态监视器（CPU/内存/磁盘/网速等） | `apps/desktop/frontend/src/components/StatusMonitor.vue` + `apps/desktop/frontend/src/lib/api-status.ts` |  |
| F094 | 工作区 | CommandHistoryPanel | 命令历史面板（列表/搜索/删除/清空） | `apps/desktop/frontend/src/components/CommandHistoryPanel.vue` + `apps/desktop/frontend/src/lib/api-auxiliary.ts` |  |
| F095 | 工作区 | QuickCommandsPanel | 快捷指令面板（列表/搜索/执行/编辑/标签） | `apps/desktop/frontend/src/components/QuickCommandsPanel.vue` + `apps/desktop/frontend/src/lib/api-auxiliary.ts` |  |
| F096 | 工作区 | FileEditorContainer | 文件编辑器容器（tabs/保存） | `apps/desktop/frontend/src/components/FileEditorContainer.vue` + `apps/desktop/frontend/src/stores/fileEditor.ts` |  |
| F097 | 工作区 | SftpBrowser | 文件管理器（列表/上传/下载/复制/删除/重命名/权限） | `apps/desktop/frontend/src/components/SftpBrowser.vue` + `apps/desktop/frontend/src/lib/api-sftp.ts` |  |
| F098 | 工作区 | FavoritePaths | 常用路径（增删改） | `apps/desktop/frontend/src/components/FavoritePaths.vue` + `apps/desktop/frontend/src/lib/api-auxiliary.ts` |  |
| F099 | 工作区 | PathHistoryDropdown | 路径历史下拉 | `apps/desktop/frontend/src/components/PathHistoryDropdown.vue` + `apps/desktop/frontend/src/lib/api-auxiliary.ts` |  |
| F100 | 工作区 | 文件上传/发送文件 | 上传文件/目录并展示进度 | `apps/desktop/frontend/src/components/FileUploadPopup.vue` + `apps/desktop/frontend/src/lib/api-sftp.ts` |  |
| F101 | 工作区 | 挂起会话弹窗 | 列表/恢复/终止挂起会话 | `apps/desktop/frontend/src/components/SuspendedSshSessionsModal.vue` + `apps/desktop/frontend/src/lib/api-ssh-suspend.ts` |  |
| F102 | 工作区 | 焦点切换器 | Alt/快捷键切换焦点 | `apps/desktop/frontend/src/stores/focusSwitcher.ts` + `apps/desktop/frontend/src/App.vue` |  |

| F110 | 统计 | 时间范围切换 | 切换统计时间范围 | `apps/desktop/frontend/src/views/Statistics.vue` |  |
| F111 | 统计 | 币种选择 | 切换显示币种 | `apps/desktop/frontend/src/views/Statistics.vue` + `apps/desktop/frontend/src/components/AppSelect.vue` |  |
| F112 | 统计 | 刷新按钮 | 重新计算/拉取统计数据 | `apps/desktop/frontend/src/views/Statistics.vue` |  |
| F113 | 统计 | “显示设置”弹窗 | 勾选统计组件显示/保存/重置 | `apps/desktop/frontend/src/views/Statistics.vue` |  |
| F114 | 统计 | 概览卡片 | 总连接/费用/活跃/时长汇总 | `apps/desktop/frontend/src/views/Statistics.vue` |  |
| F115 | 统计 | 多面板图表/表格 | 地区/服务商/费用/活跃/时长/命令/流量/到期/详情 | `apps/desktop/frontend/src/views/Statistics.vue` |  |

| F130 | 设置 | Tab 切换栏 | 工作区/AI/系统/安全/数据/外观/关于 | `apps/desktop/frontend/src/views/Settings.vue` |  |
| F131 | 设置·工作区 | 弹窗文件编辑器 | 开关 `showPopupFileEditor` | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/stores/settings.ts` |  |
| F132 | 设置·工作区 | 弹窗文件管理器 | 开关 `showPopupFileManager` | `apps/desktop/frontend/src/views/Settings.vue` |  |
| F133 | 设置·工作区 | 编辑器标签页 | 文件编辑器 tabs 行为设置 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/stores/fileEditor.ts` |  |
| F134 | 设置·工作区 | 终端自动复制 | 终端选区/复制行为设置 | `apps/desktop/frontend/src/views/Settings.vue` |  |
| F135 | 设置·工作区 | 侧边栏行为 | 侧栏/布局相关设置 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/stores/layout.ts` |  |
| F136 | 设置·工作区 | 命令输入同步 | 选择同步目标（无/快捷指令/命令历史） | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/CommandInputBar.vue` |  |
| F137 | 设置·工作区 | 显示连接标签 | 控制连接列表标签显示 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/views/Connections.vue` |  |
| F138 | 设置·工作区 | 显示快捷指令标签 | 控制快捷指令标签显示 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/QuickCommandsPanel.vue` |  |
| F139 | 设置·工作区 | 终端回滚行数 | 设置 xterm scrollback | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/SessionTerminalView.vue` |  |
| F140 | 设置·工作区 | 关闭会话确认 | 关闭 tab 前二次确认 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/TerminalTabBar.vue` |  |
| F141 | 设置·工作区 | 文件删除确认 | 文件管理器删除前确认 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/SftpBrowser.vue` |  |
| F142 | 设置·工作区 | 终端右键粘贴 | 开关右键粘贴 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/SessionTerminalView.vue` |  |
| F143 | 设置·工作区 | 状态监视器显示 IP | 开关 IP 显示 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/StatusMonitor.vue` |  |
| F144 | 设置·工作区 | 状态监控设置 | 频率/指标开关等 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/composables/useStatusMonitor.ts` |  |
| F145 | 设置·工作区 | Docker 管理器设置 | Docker 相关设置项 | `apps/desktop/frontend/src/views/Settings.vue` | 文档标注“排除项” |
| F146 | 设置·AI | AISettingsPanel | 渠道/模型/参数等配置 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/components/AI/AISettingsPanel.vue` |  |
| F147 | 设置·系统 | 时区设置 | 设置时区 | `apps/desktop/frontend/src/views/Settings.vue` |  |
| F148 | 设置·安全 | 修改密码 | 旧密码/新密码提交 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/lib/api-auth.ts` |  |
| F149 | 设置·数据 | 完整备份导出 | 导出全量 JSON | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` | `appExport` |
| F150 | 设置·数据 | 完整备份导入 | 导入全量 JSON | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` | `appImport` |
| F151 | 设置·数据 | 仅导出连接（旧版） | 旧连接导出 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/lib/api-connections.ts` | `export/import` |
| F152 | 设置·外观 | 界面字体 | 字体族/字号偏移设置 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/stores/appearance.ts` |  |
| F153 | 设置·外观 | 主题与背景 | 打开 StyleCustomizer + 终端主题/背景 | `apps/desktop/frontend/src/views/Settings.vue` + `apps/desktop/frontend/src/stores/appearance.ts` |  |
| F154 | 设置·关于 | 版本/检查更新 | 打开 releases/拉取 latest release | `apps/desktop/frontend/src/views/Settings.vue` | GitHub API |

| F160 | 数据库 | 打开 SQLite | 选择本地 SQLite 文件并记录连接资产 | `apps/desktop/frontend/src/views/Databases.vue` + `apps/desktop/frontend/src/views/databases/useSqliteManager.ts` | SQLite MVP |
| F161 | 数据库 | SQLite 资产树 | 表/列列表与刷新 | `apps/desktop/frontend/src/views/Databases.vue` + `apps/desktop/frontend/src/lib/api-database.ts` + `apps/desktop/src-tauri/src/commands/database.rs` |  |
| F162 | 数据库 | SQL 编辑器 + 查询结果 | SQL 执行与结果表（columns/rows/影响行数/耗时） | `apps/desktop/frontend/src/views/Databases.vue` + `apps/desktop/frontend/src/lib/api-database.ts` + `apps/desktop/src-tauri/src/commands/database.rs` |  |

| F165 | 工具箱 | 工具集合页面 | 17 内置开发工具（JSON/Base64/URL/HTML/Unicode/Hash/JWT/Regex/Diff/UUID/Cron/QR/时间戳/密码/颜色/命名/行处理） | `apps/desktop/frontend/src/views/Tools.vue` + `apps/desktop/frontend/src/views/tools/*` |  |

| F173 | 未接入路由 | `views/Notifications.vue` | 通知渠道管理（未路由接入） | `apps/desktop/frontend/src/views/Notifications.vue` + `apps/desktop/frontend/src/lib/api-settings.ts` |  |

# Nexus Terminal Rust 重构总方案（Tauri 2 + Rust，零开始）

更新时间：2026-02-11

## 1. 目标与硬约束

### 1.1 业务目标
- 从零重建一个新项目：`D:/Code/codeSpace/nexus-terminal/nexus-terminal-rust`
- 桌面端容器采用 **Tauri 2**
- 后端核心采用 **Rust**（不引入 Go 双栈）
- 保持现有产品功能与 API 能力

### 1.2 不可妥协约束
- **不改变显示效果**：导航结构、样式、布局、交互路径与视觉层级保持一致。
- **不改用户数据格式**：沿用原 SQLite 数据库与配置存储语义，支持直接迁移。
- **不降低功能覆盖**：连接管理、终端、SFTP、通知、审计、主题、设置、SSH 挂起等能力完整保留。
- **可回退**：任一阶段失败，可回退到当前 Electron 版本发布链。

### 1.3 明确排除项（仅 Windows 桌面端，不含网页端功能）
- **Docker 容器管理**：原系统通过 SSH 远程执行 Docker 命令，桌面端不保留此功能。
- **RDP/VNC（Guacamole）**：原系统通过 guacd 代理实现远程桌面，桌面端不保留。RDP 仅保留通过 `mstsc` 本地启动的能力。
- **Captcha 验证码**：hCaptcha/reCAPTCHA 为网页端防护，桌面端不需要。
- **remote-gateway 包**：远程网关为网页端独立部署场景，桌面端不保留。
- **IP 白名单/黑名单**：网页端安全防护机制，桌面端为本地单用户使用，不保留。

### 1.3 成功标准（验收）
- 启动性能：
  - 冷启动目标：<= 2.5s（Windows 主流 SSD）
  - 热启动目标：<= 1.0s
- 资源占用（空闲态）：
  - 内存目标：较当前版本降低 >= 40%
- 功能一致性：核心场景回归通过率 100%
- 打包交付：Windows 安装包 + 便携包，自动更新链路可用

---

## 2. 现有系统能力映射（来自当前仓库）

### 2.1 后端 API 域（现状 → 保留/排除）

| 路由前缀 | 域 | 桌面端 | 说明 |
|----------|-----|--------|------|
| `/api/v1/status` | 状态检查 | 保留 | 健康检查 |
| `/api/v1/auth` | 认证 | 保留 | 登录/注册/2FA/Passkey（移除 Captcha） |
| `/api/v1/connections` | 连接管理 | 保留 | CRUD + 测试连接（移除 RDP/VNC 类型，仅保留 SSH） |
| `/api/v1/sftp` | SFTP HTTP 接口 | 保留 | 文件下载等 HTTP 端点 |
| `/api/v1/proxies` | 代理管理 | 保留 | SOCKS5/HTTP 代理 |
| `/api/v1/tags` | 连接标签 | 保留 | |
| `/api/v1/settings` | 系统设置 | 保留 | |
| `/api/v1/notifications` | 通知设置 | 保留 | Webhook/Email/Telegram |
| `/api/v1/audit-logs` | 审计日志 | 保留 | |
| `/api/v1/command-history` | 命令历史 | 保留 | |
| `/api/v1/quick-commands` | 快速命令 | 保留 | |
| `/api/v1/quick-command-tags` | 快速命令标签 | 保留 | |
| `/api/v1/terminal-themes` | 终端主题 | 保留 | |
| `/api/v1/appearance` | 外观设置 | 保留 | |
| `/api/v1/ssh-keys` | SSH 密钥管理 | 保留 | |
| `/api/v1/ssh-suspend` | SSH 挂起 | 保留 | HTTP 部分（编辑名称等） |
| `/api/v1/transfers` | 文件传输 | 保留 | |
| `/api/v1/path-history` | 路径历史 | 保留 | |
| `/api/v1/favorite-paths` | 收藏路径 | 保留 | |

### 2.2 WebSocket 消息类型（完整清单）

**SSH 域：**

| 方向 | type | 说明 |
|------|------|------|
| C→S | `ssh:connect` | 发起 SSH 连接（payload 含 connectionId） |
| C→S | `ssh:input` | 终端键盘输入 |
| C→S | `ssh:resize` | 终端窗口尺寸调整 |
| S→C | `ssh:connected` | 连接成功（payload: connectionId + sessionId） |
| S→C | `ssh:output` | 终端输出（base64 编码） |
| S→C | `ssh:disconnected` | 连接断开 |
| S→C | `ssh:error` | 连接错误 |

**SFTP 域（14 种操作 + 上传流）：**

| 方向 | type | 说明 |
|------|------|------|
| C→S | `sftp:readdir` | 读取目录 |
| C→S | `sftp:stat` | 获取文件信息 |
| C→S | `sftp:readfile` | 读取文件内容 |
| C→S | `sftp:writefile` | 写入文件 |
| C→S | `sftp:mkdir` | 创建目录 |
| C→S | `sftp:rmdir` | 删除目录 |
| C→S | `sftp:unlink` | 删除文件 |
| C→S | `sftp:rename` | 重命名 |
| C→S | `sftp:chmod` | 修改权限 |
| C→S | `sftp:realpath` | 解析真实路径 |
| C→S | `sftp:copy` | 复制 |
| C→S | `sftp:move` | 移动 |
| C→S | `sftp:compress` | 压缩（zip/targz/tarbz2） |
| C→S | `sftp:decompress` | 解压 |
| C→S | `sftp:upload:start` | 开始分块上传 |
| C→S | `sftp:upload:chunk` | 上传数据块 |
| C→S | `sftp:upload:cancel` | 取消上传 |
| S→C | `sftp:upload:progress` | 上传进度 |
| S→C | `sftp:compress:success/error` | 压缩结果 |
| S→C | `sftp:decompress:success/error` | 解压结果 |
| S→C | `sftp_ready` | SFTP 会话就绪 |

**SSH Suspend 域：**

| 方向 | type | 说明 |
|------|------|------|
| C→S | `SSH_MARK_FOR_SUSPEND` | 标记会话待挂起 |
| C→S | `SSH_UNMARK_FOR_SUSPEND` | 取消标记 |
| C→S | `SSH_SUSPEND_LIST_REQUEST` | 请求挂起列表 |
| C→S | `SSH_SUSPEND_RESUME_REQUEST` | 恢复挂起会话 |
| C→S | `SSH_SUSPEND_TERMINATE_REQUEST` | 终止挂起会话 |
| C→S | `SSH_SUSPEND_REMOVE_ENTRY` | 移除已断开条目 |
| S→C | `SSH_MARKED_FOR_SUSPEND_ACK` | 标记确认 |
| S→C | `SSH_UNMARKED_FOR_SUSPEND_ACK` | 取消标记确认 |
| S→C | `SSH_SUSPEND_LIST_RESPONSE` | 挂起列表 |
| S→C | `SSH_SUSPEND_RESUMED_NOTIF` | 恢复通知 |
| S→C | `SSH_OUTPUT_CACHED_CHUNK` | 缓存日志块 |
| S→C | `SSH_SUSPEND_TERMINATED` | 终止确认 |
| S→C | `SSH_SUSPEND_ENTRY_REMOVED` | 条目移除确认 |
| S→C | `SSH_SUSPEND_AUTO_TERMINATED` | 自动终止通知 |

**已排除的 WS 消息（桌面端不实现）：**
- `docker:get_status`、`docker:command`、`docker:get_stats`（Docker 管理）
- RDP 代理相关消息

### 2.3 数据库 Schema（18 张表，保留 16 张）

| 表名 | 保留 | 说明 |
|------|------|------|
| `users` | 是 | 用户（username, hashed_password, two_factor_secret） |
| `passkeys` | 是 | WebAuthn 凭证 |
| `connections` | 是 | 连接配置（type 字段仅保留 'SSH'） |
| `proxies` | 是 | SOCKS5/HTTP 代理 |
| `ssh_keys` | 是 | SSH 密钥（加密存储） |
| `tags` | 是 | 连接标签 |
| `connection_tags` | 是 | 连接-标签关联 |
| `settings` | 是 | 系统设置 KV |
| `appearance_settings` | 是 | 外观设置 KV |
| `terminal_themes` | 是 | 终端主题（22 个颜色字段） |
| `notification_settings` | 是 | 通知渠道配置 |
| `audit_logs` | 是 | 审计日志 |
| `command_history` | 是 | 命令历史 |
| `path_history` | 是 | 路径历史 |
| `quick_commands` | 是 | 快速命令 |
| `quick_command_tags` + `quick_command_tag_associations` | 是 | 快速命令标签 |
| `favorite_paths` | 是 | 收藏路径 |
| `ip_blacklist` | 否 | 桌面端不需要 IP 黑名单 |

### 2.4 前端能力域（现状）

**Views (11)：** Login, Setup, Connections, Workspace, Settings, Proxies, Tags, QuickCommands, AuditLog, CommandHistory, Notifications, SuspendedSshSessions, Dashboard

**Stores (16)：** auth, connections, session, settings, appearance, proxies, tags, sshKeys, quickCommands, quickCommandTags, commandHistory, pathHistory, favoritePaths, notifications, audit, fileEditor, layout, focusSwitcher, dialog, uiNotifications

**关键组件：** Terminal (xterm.js), FileManager (SFTP), MonacoEditor/CodeMirror, StatusMonitor, StyleCustomizer, SshKeyManagement, TransferProgress

**桌面端移除的前端组件：** DockerManager, VncModal, RemoteDesktopModal（Guacamole）, CaptchaSettingsForm

### 2.5 Electron 特定能力（需迁移到 Tauri）
- 自定义窗口控制（最小化/最大化/关闭）
- RDP 启动（Windows `mstsc`，通过 Tauri shell 命令）
- 文件下载桥接
- 启动状态回传（backend-startup-status）

---

## 3. 目标架构（Tauri 2 + Rust 单栈）

```text
┌─────────────────────────────────────────────────────────┐
│                    Tauri 2 Shell                        │
│  - Window / Tray / Updater / Permissions / IPC          │
└───────────────┬─────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────────┐
│                 Web Frontend (Vue3)                     │
│  - 维持现有 UI/路由/样式，不做视觉改动                    │
│  - API 调用仍走 http://127.0.0.1:{port}/api/v1/*        │
└───────────────┬─────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────────┐
│                Rust API Server (Axum)                   │
│  - REST: /api/v1/*                                      │
│  - WS: /ws                                               │
│  - Auth/Session/SFTP/SSH/Notifications/...               │
│  - SQLite + 迁移 + 审计                                  │
└───────────────┬─────────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────────────────────────┐
│           OS Integration / Native Utilities              │
│  - RDP (mstsc)  - 文件系统  - 网络  - 加密               │
└─────────────────────────────────────────────────────────┘
```

关键原则：
- 前端 API 协议保持稳定，减少前端迁移成本。
- 容器层（Tauri）仅负责窗口/权限/更新/进程编排。
- 业务核心集中 Rust 后端，避免多语言后端带来的维护分裂。

---

## 4. 仓库设计（nexus-terminal-rust）

建议目录：

```text
nexus-terminal-rust/
├── apps/
│   ├── desktop/                 # Tauri 2 应用壳
│   │   ├── src-tauri/
│   │   └── frontend/            # 迁入现有 Vue 前端（视觉不变）
│   └── backend-server/          # Rust API 可独立运行（便于压测）
├── crates/
│   ├── api-contract/            # DTO、错误码、请求响应 schema
│   ├── auth-core/               # 认证与授权（密码/2FA/Passkey，不含 Captcha）
│   ├── connection-core/         # 连接实体与生命周期（仅 SSH 类型）
│   ├── session-core/            # 会话管理与状态机
│   ├── ssh-core/                # SSH 连接、Shell 流、密钥管理
│   ├── sftp-core/               # SFTP 文件操作与传输
│   ├── ssh-suspend-core/        # SSH 挂起/恢复/临时日志
│   ├── proxy-core/              # SOCKS5/HTTP 代理管理
│   ├── tag-core/                # 连接标签 + 快速命令标签
│   ├── quick-command-core/      # 快速命令 CRUD
│   ├── notifications-core/      # 通知（Webhook/Email/Telegram）
│   ├── audit-core/              # 审计日志
│   ├── settings-core/           # 系统设置 + 外观设置 + 终端主题
│   ├── history-core/            # 命令历史 + 路径历史 + 收藏路径
│   ├── transfer-core/           # 文件传输管理
│   ├── ws-gateway/              # WebSocket 事件编排与消息路由
│   ├── storage-sqlite/          # 数据库访问、迁移、schema
│   └── shared-utils/            # 日志、加密、时间、序列化等共通组件
├── scripts/
│   ├── benchmark/
│   ├── e2e/
│   └── packaging/
├── docs/
│   ├── 01-architecture.md
│   ├── 02-api-compat.md
│   ├── 03-migration-plan.md
│   ├── 04-risk-register.md
│   └── 05-benchmark-baseline.md
└── Cargo.toml
```

> **crate 数量说明**：从原始 12 个扩展到 18 个，以覆盖现有系统全部 19 个 API 域（排除 Docker 后）。部分相近域合并：settings + appearance + terminal-themes → `settings-core`；command-history + path-history + favorite-paths → `history-core`；tags + quick-command-tags → `tag-core`。

---

## 5. 模块重建策略（从零开始，不复制旧实现）

### 5.1 API 兼容优先
- 第一阶段不改前端调用方式，后端以“兼容层”实现旧接口。
- 每个接口定义契约测试（状态码、字段名、错误格式）。

### 5.2 数据层策略
- 保留 SQLite 主库。
- 建立 Rust migration 管理（如 `sqlx`/`refinery`）。
- 迁移顺序：
  1) schema 兼容
  2) 索引与约束补齐
  3) 启动时版本校验与自动升级

### 5.3 会话与认证
- Session 模型保持 Cookie 语义，兼容前端现有行为。
- 登录/2FA/Passkey 分层实现：
  - `auth-core`（策略）
  - `api`（HTTP 处理）
  - `storage`（仓储）

### 5.4 实时链路（WS）
- 建立统一事件协议：`event_type + payload + request_id`。
- 先实现与当前前端最紧密依赖事件：
  - SSH output / connected / disconnected
  - SFTP upload/download progress
  - suspend/resume 相关事件

### 5.5 本地能力（Tauri）
- 使用 Tauri window API 复刻窗口行为。
- 使用 shell/sidecar 权限模型执行外部命令（如 `mstsc`）。
- 下载、文件对话框、路径访问统一走 capability 权限清单。

---

## 6. 分阶段实施计划（建议 6 阶段）

### Phase 0：蓝图冻结（1 周）
产出：
- 架构文档 + API 契约清单 + 风险清单
- 非功能指标（性能、内存、稳定性）明确

退出条件：
- 所有核心团队对边界与里程碑一致确认

### Phase 1：骨架搭建（1~2 周）
产出：
- Tauri 2 壳工程
- Rust backend-server 工程
- CI 基础（fmt/clippy/test/build）

退出条件：
- 可启动空壳，前后端健康检查可通

### Phase 2：API 兼容层（2~4 周）
优先域：`auth/settings/connections/status`

退出条件：
- 前端可登录并进入连接管理页面，关键请求成功

### Phase 3：实时能力迁移（2~4 周）
迁移 `ws + ssh + sftp + suspend`

退出条件：
- 终端可连接、可稳定输出、SFTP 主要操作可用

### Phase 4：全功能补齐（2~3 周）
迁移通知、审计、主题、快速命令、标签等剩余模块

退出条件：
- 功能覆盖达到旧版本同级

### Phase 5：性能与安全收敛（1~2 周）
- 启动与内存专项优化
- 权限最小化与威胁建模复查

退出条件：
- 达成既定启动/内存目标

### Phase 6：灰度发布与替换（1~2 周）
- 灰度 + 回滚机制 + 监控

退出条件：
- 灰度稳定，无 P0/P1 问题

---

## 7. 性能方案（从设计期内建）

### 7.1 启动关键路径优化
- 前端首屏先渲染，业务数据异步回填。
- Rust 后端：
  - 路由与服务按需初始化
  - 非关键后台任务延后
  - 连接池与线程池参数按平台调优

### 7.2 资源与体积优化
- 前端手动分包：`terminal`, `editor`, `settings`, `audit` 等按路由拆分
- Rust 构建：LTO + `panic=abort` + strip
- 产物最小化：仅打包必要资源与字典

### 7.3 可观测性
- 启动阶段埋点：
  - `shell_ready`
  - `frontend_first_paint`
  - `api_ready`
  - `workspace_interactive`
- 每次发布保留 AB 数据与趋势图

---

## 8. 风险清单与缓解

### 高风险 1：WS 行为不兼容
- 缓解：先做协议回放测试（录制旧版消息流，回放到新服务）

### 高风险 2：SFTP/SSH 边缘行为回归
- 缓解：引入会话状态机 + 长稳压测（长连接 8h）

### 高风险 3：Windows 权限与安全策略
- 缓解：最小权限 capability；外部命令白名单；签名与更新校验

### 高风险 4：迁移周期过长
- 缓解：按域切片发布，保持 Electron 版本持续可发

---

## 9. 质量门禁（每阶段必须达成）

- 单测：核心域覆盖 >= 80%
- 契约测试：关键 API 100% 覆盖
- E2E：登录、建连、终端、SFTP、设置、关闭流程全绿
- 性能：
  - 冷启动、热启动、内存均有基线和回归阈值
- 安全：
  - 权限清单审计通过
  - 高危依赖 0 容忍

---

## 10. 我们不做的事情（防止范围蔓延）

- 不做 UI 重设计
- 不引入 Rust+Go 双后端并行生产栈
- 不在第一版追求跨平台"全部特性齐平"（优先 Windows）
- 不在第一版引入无明确收益的新协议
- **不实现 Docker 容器管理**（原系统网页端功能）
- **不实现 RDP/VNC Guacamole 代理**（桌面端仅保留 mstsc 本地启动）
- **不实现 Captcha 验证码**（桌面端无需）
- **不实现 IP 黑白名单**（桌面端本地单用户）
- **不实现 remote-gateway**（网页端独立部署场景）

---

## 11. 初始执行清单（下一步）

1. 冻结 API 契约文档（按域拆分）
2. 生成 Tauri + Rust 工作区骨架
3. 迁移前端静态资源与路由（保持视觉一致）
4. 打通 `status/auth/needs-setup` 最小链路
5. 建立 AB 基准脚本（旧版 vs 新版）
6. 进入 Phase 2 按域迁移

---

## 12. 外部资料（用于架构评估）

- Tauri Architecture: https://v2.tauri.app/concept/architecture/
- Tauri Sidecar: https://v2.tauri.app/develop/sidecar/
- Tauri Shell Plugin: https://v2.tauri.app/plugin/shell/
- Tauri Security: https://v2.tauri.app/security/
- Tauri Plugin Permissions: https://v2.tauri.app/learn/security/using-plugin-permissions/
- Electron Performance: https://electronjs.org/docs/latest/tutorial/performance
- Node SEA: https://nodejs.org/api/single-executable-applications.html
- Wails Architecture: https://wails.io/docs/howdoesitwork/
- Wails Windows/WebView2: https://wails.io/docs/guides/windows

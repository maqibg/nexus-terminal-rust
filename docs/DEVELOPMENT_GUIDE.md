# DEVELOPMENT GUIDE

> 文档类型：完整开发文档  
> 适用项目：`nexus-terminal-rust`  
> 技术基线：`Tauri 2 + Rust + Vue 3`  
> 更新时间：2026-02-11

---

## 1. 文档目标

本文件用于定义重构项目的统一技术规范，确保以下目标：

1. 从零开始重构时，技术路线和边界清晰。
2. 多人协作开发时，接口、代码风格、测试标准一致。
3. 任何阶段都可审计、可回退、可发布。
4. 在“不改变显示效果”的硬约束下，实现性能和可维护性提升。

---

## 2. 产品与技术范围

### 2.1 必须保留的业务能力

- 用户与认证（登录、登出、2FA、Passkey 兼容）
- 连接管理（新增、编辑、测试、连接，仅 SSH 类型）
- 终端交互（SSH 输出、输入、会话生命周期）
- SFTP 文件管理（上传/下载/编辑/压缩/解压）
- 代理管理（SOCKS5/HTTP）
- 标签（连接标签 + 快速命令标签）
- 快速命令
- SSH 密钥管理
- 设置、外观与终端主题
- 审计日志与通知（Webhook/Email/Telegram）
- SSH 挂起/恢复
- 命令历史、路径历史、收藏路径
- 文件传输管理
- 导入/导出连接配置
- 状态监控

### 2.2 必须保持不变的体验约束

- 页面布局结构不变
- 主视觉（颜色、间距、组件结构、图标位置）不变
- 顶部导航与右上角窗口按钮行为不变
- 主要操作路径（点击顺序）不变

### 2.3 明确排除项（桌面端不实现）

- **Docker 容器管理**：原系统通过 SSH 远程执行 Docker 命令实现容器状态/命令/统计，桌面端不保留
- **RDP/VNC（Guacamole 代理）**：原系统通过 guacd 代理实现远程桌面，桌面端不保留。RDP 仅保留通过 `mstsc` 本地启动
- **Captcha 验证码**：hCaptcha/reCAPTCHA 为网页端防护，桌面端不需要
- **remote-gateway 包**：远程网关为网页端独立部署场景，桌面端不保留
- **IP 白名单/黑名单**：网页端安全防护机制，桌面端为本地单用户使用，不保留

### 2.4 第一阶段不做

- 不做 UI 改版
- 不改核心信息架构
- 不引入 Rust+Go 双后端生产栈
- 不做跨平台齐平（先保证 Windows）

---

## 3. 总体架构

### 3.1 分层架构

```text
┌────────────────────────────────────────────────────┐
│                   Tauri Shell                      │
│   window / updater / permissions / process mgmt    │
└───────────────┬────────────────────────────────────┘
                │
                ▼
┌────────────────────────────────────────────────────┐
│               Frontend (Vue3 + Pinia)             │
│  UI保持一致 / API调用 / WebSocket事件处理           │
└───────────────┬────────────────────────────────────┘
                │ HTTP + WS
                ▼
┌────────────────────────────────────────────────────┐
│                 Rust Backend (Axum)               │
│   REST API / WS Gateway / Domain Services          │
└───────────────┬────────────────────────────────────┘
                │
                ▼
┌────────────────────────────────────────────────────┐
│          SQLite + FileSystem + Native Cmd         │
└────────────────────────────────────────────────────┘
```

### 3.2 关键设计原则

- **协议兼容优先**：前端与后端通信协议尽量保持原语义。
- **领域分层**：HTTP 层、服务层、仓储层、基础设施层明确分离。
- **最小权限**：Tauri capability 仅开启必要能力。
- **可观测**：每个关键流程必须有可测量埋点。

---

## 4. 仓库与模块设计

### 4.1 推荐目录结构

```text
nexus-terminal-rust/
├── apps/
│   ├── desktop/
│   │   ├── frontend/
│   │   └── src-tauri/
│   └── backend-server/
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
│   └── release/
└── docs/
```

### 4.2 crate 责任边界

| crate | 责任 | 对应原系统模块 |
|-------|------|---------------|
| `api-contract` | DTO、错误码、请求响应 schema | `types/*.types.ts` |
| `auth-core` | 认证与授权策略（密码/2FA/Passkey） | `auth/`、`passkey/`、`user/` |
| `connection-core` | 连接实体与连接生命周期 | `connections/` |
| `session-core` | 会话管理与状态机 | `websocket/state.ts` |
| `ssh-core` | SSH 连接、Shell 流、密钥管理 | `services/ssh.service.ts`、`ssh_keys/` |
| `sftp-core` | 文件传输与操作抽象 | `sftp/` |
| `ssh-suspend-core` | SSH 挂起/恢复/临时日志存储 | `ssh-suspend/` |
| `proxy-core` | SOCKS5/HTTP 代理管理 | `proxies/` |
| `tag-core` | 连接标签 + 快速命令标签 | `tags/`、`quick-command-tags/` |
| `quick-command-core` | 快速命令 CRUD | `quick-commands/` |
| `notifications-core` | 通知渠道与事件分发 | `notifications/` |
| `audit-core` | 审计日志记录与查询 | `audit/` |
| `settings-core` | 系统设置 + 外观 + 终端主题 | `settings/`、`appearance/`、`terminal-themes/` |
| `history-core` | 命令历史 + 路径历史 + 收藏路径 | `command-history/`、`path-history/`、`favorite-paths/` |
| `transfer-core` | 文件传输管理 | `transfers/` |
| `ws-gateway` | 事件编排与实时消息路由 | `websocket/`、`websocket/handlers/` |
| `storage-sqlite` | 数据库访问与迁移 | `database/` |
| `shared-utils` | 日志、加密、时间、序列化 | `utils/`、`config/`、`i18n.ts` |

---

## 5. API 兼容规范

### 5.1 兼容策略

- `api/v1` 前缀保持不变。
- 路径语义与字段命名保持一致。
- 错误返回格式统一（状态码 + message + 可选 details）。
- 对前端依赖强的接口先迁移（status/auth/connections/workspace）。

### 5.2 接口分级

- **P0（首批，Phase 2）**：
  - `/api/v1/status`
  - `/api/v1/auth/needs-setup`
  - `/api/v1/auth/status`
  - `/api/v1/auth/login`、`/logout`、`/setup`
  - `/api/v1/connections/*`
  - `/api/v1/settings/*`
  - `/api/v1/appearance/*`
- **P1（第二批，Phase 3）**：
  - `/api/v1/sftp/*`（HTTP 端点：下载等）
  - `/api/v1/ssh-keys/*`
  - `/api/v1/ssh-suspend/*`（HTTP 部分：编辑名称等）
  - `/api/v1/proxies/*`
  - `/api/v1/terminal-themes/*`
- **P2（第三批，Phase 4）**：
  - `/api/v1/notifications/*`
  - `/api/v1/audit-logs/*`
  - `/api/v1/tags/*`
  - `/api/v1/quick-commands/*`
  - `/api/v1/quick-command-tags/*`
  - `/api/v1/command-history/*`
  - `/api/v1/path-history/*`
  - `/api/v1/favorite-paths/*`
  - `/api/v1/transfers/*`

**已排除（桌面端不实现）**：
- Docker 相关 API（原系统无独立 REST 路由，通过 WS 实现）
- Captcha 相关接口
- IP 黑白名单相关接口

### 5.3 契约测试要求

每个 API 必须有自动化契约测试，包含：

- 路径可达性
- 状态码一致性
- 必填字段存在
- 错误场景可预期

---

## 6. WebSocket 事件规范

### 6.1 事件信封统一格式

```json
{
  "type": "event_type",
  "requestId": "optional-request-id",
  "payload": {}
}
```

### 6.2 完整事件清单

**SSH 域：**

| 方向 | type | payload 关键字段 | 说明 |
|------|------|-----------------|------|
| C→S | `ssh:connect` | `connectionId` | 发起 SSH 连接 |
| C→S | `ssh:input` | `sessionId`, `data` | 终端键盘输入 |
| C→S | `ssh:resize` | `sessionId`, `cols`, `rows` | 终端窗口尺寸调整 |
| S→C | `ssh:connected` | `connectionId`, `sessionId` | 连接成功 |
| S→C | `ssh:output` | `data`(base64), `encoding` | 终端输出 |
| S→C | `ssh:disconnected` | `sessionId` | 连接断开 |
| S→C | `ssh:error` | `sessionId`, `error` | 连接错误 |

**SFTP 域：**

| 方向 | type | 说明 |
|------|------|------|
| C→S | `sftp:readdir` | 读取目录列表 |
| C→S | `sftp:stat` | 获取文件/目录信息 |
| C→S | `sftp:readfile` | 读取文件内容 |
| C→S | `sftp:writefile` | 写入文件 |
| C→S | `sftp:mkdir` | 创建目录 |
| C→S | `sftp:rmdir` | 删除目录 |
| C→S | `sftp:unlink` | 删除文件 |
| C→S | `sftp:rename` | 重命名 |
| C→S | `sftp:chmod` | 修改权限 |
| C→S | `sftp:realpath` | 解析真实路径 |
| C→S | `sftp:copy` | 复制文件/目录 |
| C→S | `sftp:move` | 移动文件/目录 |
| C→S | `sftp:compress` | 压缩（zip/targz/tarbz2） |
| C→S | `sftp:decompress` | 解压 |
| C→S | `sftp:upload:start` | 开始分块上传 |
| C→S | `sftp:upload:chunk` | 上传数据块 |
| C→S | `sftp:upload:cancel` | 取消上传 |
| S→C | `sftp:upload:progress` | 上传进度（uploadId, bytesWritten, totalSize, progress） |
| S→C | `sftp:compress:success` / `sftp:compress:error` | 压缩结果 |
| S→C | `sftp:decompress:success` / `sftp:decompress:error` | 解压结果 |
| S→C | `sftp_ready` | SFTP 会话就绪 |

**SSH Suspend 域：**

| 方向 | type | 说明 |
|------|------|------|
| C→S | `SSH_MARK_FOR_SUSPEND` | 标记会话待挂起（含可选 initialBuffer） |
| C→S | `SSH_UNMARK_FOR_SUSPEND` | 取消标记 |
| C→S | `SSH_SUSPEND_LIST_REQUEST` | 请求挂起会话列表 |
| C→S | `SSH_SUSPEND_RESUME_REQUEST` | 恢复挂起会话 |
| C→S | `SSH_SUSPEND_TERMINATE_REQUEST` | 终止挂起会话 |
| C→S | `SSH_SUSPEND_REMOVE_ENTRY` | 移除已断开条目 |
| S→C | `SSH_MARKED_FOR_SUSPEND_ACK` | 标记确认 |
| S→C | `SSH_UNMARKED_FOR_SUSPEND_ACK` | 取消标记确认 |
| S→C | `SSH_SUSPEND_LIST_RESPONSE` | 挂起列表（含 suspendSessions 数组） |
| S→C | `SSH_SUSPEND_RESUMED_NOTIF` | 恢复通知 |
| S→C | `SSH_OUTPUT_CACHED_CHUNK` | 缓存日志块（恢复时回放） |
| S→C | `SSH_SUSPEND_TERMINATED` | 终止确认 |
| S→C | `SSH_SUSPEND_ENTRY_REMOVED` | 条目移除确认 |
| S→C | `SSH_SUSPEND_AUTO_TERMINATED` | 后端自动终止通知 |

**状态监控域：**

| 方向 | type | 说明 |
|------|------|------|
| C→S | `status_subscribe` | 前端订阅指定 SSH 会话的状态采集 |
| C→S | `status_unsubscribe` | 前端取消订阅指定 SSH 会话的状态采集 |
| S→C | `status:update:{sessionId}` | 服务器状态更新（CPU/内存/网络等，按会话隔离） |
| S→C | `status:error:{sessionId}` | 状态采集失败（包含错误信息与时间戳） |

**已排除（桌面端不实现）：**
- `docker:get_status`、`docker:command`、`docker:get_stats`
- RDP Guacamole 代理相关消息

### 6.3 稳定性要求

- 必须支持断线重连。
- 事件处理幂等。
- 长连接异常可自动恢复或显式失败。

---

## 7. 数据与迁移规范

### 7.1 数据库策略

- 主存储继续使用 SQLite。
- schema 版本必须可追踪（migration version）。
- 启动阶段只做必要迁移，禁止无差别全表扫描。

### 7.2 完整 Schema 清单（16 张表）

| 表名 | crate 归属 | 关键字段 |
|------|-----------|---------|
| `users` | auth-core | id, username, hashed_password, two_factor_secret |
| `passkeys` | auth-core | id, user_id, credential_id, public_key, counter, transports, name |
| `connections` | connection-core | id, name, type('SSH'), host, port, username, auth_method, proxy_id, ssh_key_id, jump_chain, notes |
| `proxies` | proxy-core | id, name, type('SOCKS5'/'HTTP'), host, port, username, auth_method, encrypted_password/key |
| `ssh_keys` | ssh-core | id, name, encrypted_private_key, encrypted_passphrase |
| `tags` | tag-core | id, name |
| `connection_tags` | tag-core | connection_id, tag_id |
| `settings` | settings-core | key, value |
| `appearance_settings` | settings-core | key, value |
| `terminal_themes` | settings-core | id, name, theme_type('preset'/'user'), 22 个颜色字段 |
| `notification_settings` | notifications-core | id, channel_type, name, enabled, config(JSON), enabled_events(JSON) |
| `audit_logs` | audit-core | id, timestamp, action_type, details |
| `command_history` | history-core | id, command, timestamp |
| `path_history` | history-core | id, path, timestamp |
| `quick_commands` | quick-command-core | id, name, command, usage_count, variables |
| `quick_command_tags` | tag-core | id, name |
| `quick_command_tag_associations` | tag-core | quick_command_id, tag_id |
| `favorite_paths` | history-core | id, name, path, last_used_at |

**已排除（桌面端不迁移）：**
- `ip_blacklist`（网页端 IP 封禁机制）

### 7.3 迁移流程

1. 生成 migration 脚本
2. 在测试库演练
3. 在历史数据样本库回放
4. 再进入灰度

### 7.3 数据安全

- 迁移前自动备份。
- 迁移失败自动回滚到上一个稳定版本。

---

## 8. Tauri 容器与权限规范

### 8.1 必要能力

- 窗口控制：最小化/最大化/关闭
- 文件系统访问：用户授权目录
- Shell 调用：仅白名单命令（如 `mstsc` 启动 RDP）
- 更新能力：签名校验 + 通道控制

> 注意：RDP 仅通过 Tauri shell 命令调用本地 `mstsc`，不实现 Guacamole 代理。

### 8.2 安全边界

- 前端与 Rust 命令通道必须显式声明。
- 所有外部命令参数必须校验与转义。
- 禁止前端直接传递任意系统命令。

### 8.3 配置管理

- 能力配置随环境区分：dev/staging/prod
- capability 变更需安全评审

---

## 9. 编码规范

### 9.1 Rust

- `clippy` 无 warning（除明确白名单）
- 关键模块必须写文档注释
- `Result` 错误链完整，禁止静默吞错
- 避免“一文件超大”，模块职责单一

### 9.2 Frontend

- 不做视觉层改版
- 组件命名、store 命名保持语义一致
- 避免在视图层写业务编排，业务逻辑下沉 composable/store

### 9.3 日志规范

- 启动日志必须包含阶段标识：
  - `shell_ready`
  - `frontend_ready`
  - `backend_ready`
  - `workspace_interactive`

---

## 10. 测试与质量门禁

### 10.1 测试层级

1. 单元测试（domain/service）
2. 契约测试（REST/WS）
3. 集成测试（DB + service + API）
4. E2E 测试（关键用户链路）
5. 性能回归测试（冷/热启动、内存）

### 10.2 覆盖率目标

- Rust 核心域：>= 80%
- API 契约：P0/P1 100%
- E2E：关键路径全覆盖

### 10.3 阶段放行标准

- 代码评审通过
- 自动化测试全绿
- 无 P0/P1 未解决缺陷
- 关键性能指标不回退

---

## 11. 性能目标与观测

### 11.1 启动性能目标

- 冷启动 <= 2.5s
- 热启动 <= 1.0s

### 11.2 资源目标

- 空闲内存较旧版降低 >= 40%

### 11.3 监控指标

- 启动耗时分段
- WS 建连成功率
- API P95 延迟
- 崩溃率与恢复率

---

## 12. CI/CD 与发布规范

### 12.1 CI 流程

- Rust：`fmt` -> `clippy` -> `test`
- Frontend：`type-check` -> `build`
- 集成：契约测试 + E2E smoke

### 12.2 打包与发布

- 产物：Windows 安装包 + 便携包
- 每次发布必须带：
  - 版本变更说明
  - 已知风险
  - 回滚方式

### 12.3 回滚策略

- 灰度阶段保留旧版可切回
- 数据迁移必须可逆或可恢复

---

## 13. 运维与支持规范

### 13.1 日志与诊断

- 输出统一结构日志（可 grep、可聚合）
- 故障包包含：版本、启动阶段、错误栈、关键配置摘要

### 13.2 现场问题处理

- P0：4 小时内响应
- P1：24 小时内临时修复
- 所有线上问题需补充自动化回归

---

## 14. 架构决策记录（ADR）机制

每个关键技术决策必须新增 ADR，包含：

1. 背景
2. 候选方案
3. 取舍理由
4. 结论
5. 回滚条件

建议路径：`docs/adr/ADR-XXXX-*.md`

---

## 15. 开发启动前检查

- [ ] 已阅读并确认本文件
- [ ] 已阅读 `DEVELOPMENT_PLAN.md`
- [ ] 已明确本阶段不做项
- [ ] 已配置本地开发环境与脚本
- [ ] 已建立基线性能数据

---

## 16. 附录：关键原则

- 先可用，再优化
- 先兼容，再演进
- 先可观测，再扩展
- 先可回退，再上线

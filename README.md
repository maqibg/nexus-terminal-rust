# Nexus Terminal Desktop

<div align="center">

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-4CAF50?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2.x-blue?style=flat-square&logo=tauri)](https://tauri.app)
[![Vue](https://img.shields.io/badge/Vue-3.x-42b883?style=flat-square&logo=vue.js)](https://vuejs.org)

**星枢终端桌面版** — 基于 Rust + Tauri 的跨平台 SSH 客户端

[中文](./README.md)

</div>

---

## 概述

**Nexus Terminal Desktop** 是 [nexus-terminal](https://github.com/Heavrnl/nexus-terminal) 的桌面版重写，采用 **Rust + Tauri 2** 架构重构。在保留原版完整终端、SFTP、会话管理能力的基础上，新增了 **AI 助手**、**端口转发**、**任务调度**等高级功能（功能设计参考 [Mshell](https://github.com/inspoaibox/Mshell)，代码独立实现）。

相比原版 Web 应用，桌面版通过 Rust 原生实现取代 Node.js 后端，大幅降低资源占用、提升性能。

---

## 功能特性

### 核心终端功能

- 多标签页 SSH 连接管理，支持分屏（水平 / 垂直）
- xterm.js WebGL 渲染，终端响应流畅
- 内置心跳保活，连接稳定不中断
- SSH 会话挂起与恢复，长任务不因断网中止
- 支持 SSH 密钥认证与密码认证
- 通过代理连接（HTTP、SOCKS5）
- 命令历史记录与快速命令片段库
- 标签分组与连接收藏管理

### 文件管理（SFTP）

- 双面板文件浏览器，支持跨服务器传输
- 拖拽上传 / 下载，实时传输进度跟踪
- Monaco Editor 在线编辑远程文件
- 文件操作全覆盖：创建、删除、重命名、移动、修改权限
- 收藏路径快速跳转，路径历史记录

### AI 助手（新增）

集成多 AI 服务提供商，在终端旁侧提供上下文感知的智能辅助：

**支持的提供商**

| 提供商 | 说明 |
|--------|------|
| OpenAI | GPT-4o 等系列模型 |
| Anthropic | Claude 系列模型 |
| Google Gemini | Gemini 系列模型 |
| OpenAI 兼容 | 任意兼容端点（如 Ollama、本地模型） |

**功能亮点**

- 终端集成 AI 聊天面板，无需切换窗口
- 流式响应实时输出，支持中途停止 / 重新生成
- 附加文件功能：可将服务器上的文件直接上传给 AI 分析
- 代码块渲染，支持一键复制、直接插入终端或执行
- 快速操作入口：解释命令 / 排查报错 / 生成脚本
- 每个会话独立维护聊天历史
- 可配置提示词模板（解释、优化、生成）
- Temperature、Max Tokens、请求超时全局可调

### 安全与认证

- 双因素认证（TOTP 2FA）
- Passkey 生物识别登录（WebAuthn）
- 密码 Argon2 / Bcrypt 安全存储
- 凭据 AES-GCM 加密持久化
- 会话锁定保护

### 通知与审计

- 多渠道通知：邮件、Telegram、Webhook
- 审计日志，记录所有连接操作与文件变更
- 服务器状态监控（CPU、内存、磁盘、网络）、实时图表

### 数据管理

- SQLite 本地数据库，数据完全本地存储
- 加密备份与恢复
- 配置导入 / 导出

### 数据库管理（新增）

- SQLite 本地文件：资产树（表/列）浏览
- SQL 编辑器 + 查询结果表（支持执行与影响行数）
- 已支持：Redis、MySQL/MariaDB、PostgreSQL、SQLite、SQL Server、ClickHouse、Oracle（Oracle 基于 ODBC；在当前构建中仅 Windows 可用）

### 工具箱（新增）

- 17 种内置开发工具：JSON、Base64、URL、HTML 实体、Unicode、Hash、JWT、Regex、Diff、UUID、Cron、QR 码、时间戳、密码生成、颜色转换、命名转换、行处理

---

## 架构

项目采用 **Rust Cargo Workspace** 单体仓库，后端拆分为 18 个独立 crate，前端为 Vue 3 + TypeScript 应用，通过 Tauri IPC 通信。

```
nexus-terminal-rust/
├── apps/
│   └── desktop/
│       ├── src-tauri/          # Tauri 主进程（Rust）
│       └── frontend/           # Vue 3 前端
└── crates/
    ├── api-contract/           # API 类型定义与接口契约
    ├── auth-core/              # 认证（密码 / 2FA / Passkey）
    ├── connection-core/        # 连接生命周期管理
    ├── session-core/           # 会话状态管理
    ├── ssh-core/               # SSH 客户端（基于 russh）
    ├── sftp-core/              # SFTP 文件传输
    ├── ssh-suspend-core/       # 会话挂起与恢复
    ├── proxy-core/             # 代理支持
    ├── tag-core/               # 标签与分组
    ├── quick-command-core/     # 快速命令管理
    ├── notifications-core/     # 通知系统
    ├── audit-core/             # 审计日志
    ├── settings-core/          # 应用设置
    ├── history-core/           # 命令 / 路径历史
    ├── transfer-core/          # 文件传输进度跟踪
    ├── ws-gateway/             # WebSocket 网关
    ├── storage-sqlite/         # SQLite 持久化层
    └── shared-utils/           # 共享加密 / UUID 工具
```

**核心技术栈**

| 层级 | 技术 |
|------|------|
| 桌面框架 | Tauri 2 |
| 后端语言 | Rust 1.75+ |
| 异步运行时 | Tokio |
| SSH 库 | russh 0.49 + russh-sftp 2.1 |
| 数据库 | SQLx 0.8 (SQLite) |
| 前端框架 | Vue 3.5 + TypeScript |
| 终端渲染 | xterm.js 6.0 |
| 代码编辑 | Monaco Editor |
| VNC 客户端 | noVNC 1.5 |
| 状态管理 | Pinia |
| 构建工具 | Vite 6 |

---

## 构建

### 环境依赖

- [Rust](https://rustup.rs/) 1.75+
- [Node.js](https://nodejs.org/) 20+
- [pnpm](https://pnpm.io/) 9+
- 系统依赖（Ubuntu/Debian）：

```bash
sudo apt install -y libwebkit2gtk-4.1-dev libssl-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev
```

### 开发模式

```bash
# 克隆仓库
git clone <repository-url>
cd nexus-terminal-rust

# 安装前端依赖
cd apps/desktop/frontend
pnpm install
cd ../../..

# 启动开发环境
cd apps/desktop
pnpm tauri dev
```

### 生产构建

```bash
cd apps/desktop
pnpm tauri build
```

构建产物位于 `apps/desktop/src-tauri/target/release/bundle/`。

---

## 与原版对比

| 功能 | nexus-terminal (Web) | nexus-terminal-rust (桌面) |
|------|---------------------|--------------------------|
| 部署方式 | Docker / 服务器 | 本地安装包 |
| 后端运行时 | Node.js | Rust 原生 |
| AI 助手 | 无 | 支持（多提供商） |
| 端口转发 | 无 | 支持 |
| 数据存储 | 服务器端 | 本地 SQLite |
| PWA 支持 | 支持 | 原生应用 |
| RDP/VNC | 支持（Guacamole） | VNC（noVNC） |
| 会话挂起 | 支持 | 支持 |
| 登录验证 | hCaptcha / reCAPTCHA / 2FA | 2FA + Passkey |

---

## 许可证与归属

### 开源协议

本项目采用 [GPL-3.0](LICENSE) 开源协议。

### 上游项目归属

本项目是 [nexus-terminal](https://github.com/Heavrnl/nexus-terminal)（GPL-3.0）的衍生作品，前端架构、终端功能、SFTP模块等核心组件基于原版设计重写。

### 功能设计参考

AI 助手、端口转发等高级功能的设计理念参考了 [Mshell](https://github.com/inspoaibox/Mshell)（MIT License），但所有代码均为独立实现，未使用 Mshell 的源代码。

工具箱、数据库管理等功能形态参考了 [HiKit](https://github.com/hsqbyte/hikit)（Apache-2.0），但所有实现均为独立开发，未拷贝 HiKit 源代码。

### 版权声明

详细的版权归属和上游致谢请参见 [NOTICE](NOTICE) 文件。

### 第三方依赖

本项目使用的所有第三方库的许可证信息可通过以下命令查看：

```bash
# Rust依赖
cargo license --json

# 前端依赖
cd apps/desktop/frontend && pnpm licenses list
```

---

## 致谢

- 原版 Web 端：[nexus-terminal](https://github.com/Heavrnl/nexus-terminal)
- 功能设计参考：[Mshell](https://github.com/inspoaibox/Mshell)
- 终端主题：[iTerm2-Color-Schemes](https://github.com/mbadolato/iTerm2-Color-Schemes)

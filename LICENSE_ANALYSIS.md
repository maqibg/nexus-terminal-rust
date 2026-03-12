# 许可证合规性分析报告

## 项目概况

**项目名称**: nexus-terminal-rust
**当前许可证**: GPL-3.0
**分析日期**: 2026-03-12

---

## 上游依赖关系

### 1. nexus-terminal (主要上游)
- **仓库**: https://github.com/Heavrnl/nexus-terminal
- **许可证**: GPL-3.0
- **关系**: 本项目是其桌面版重写（derived work）
- **影响范围**: 前端架构、终端功能、SFTP功能、会话管理

### 2. Mshell (功能参考)
- **仓库**: https://github.com/inspoaibox/Mshell
- **许可证**: MIT License
- **关系**: 功能设计参考（inspired by），非代码衍生
- **参考功能**: AI助手集成、端口转发、任务调度概念

---

## 许可证兼容性分析

### GPL-3.0 与 MIT 的兼容性

✅ **结论：兼容且合规**

**理由**：
1. MIT是宽松许可证，允许代码被整合到GPL项目中
2. GPL-3.0是copyleft许可证，要求衍生作品也使用GPL-3.0
3. 本项目基于GPL-3.0的nexus-terminal，必须使用GPL-3.0
4. 参考MIT项目的设计理念（而非直接复制代码）是允许的

### 代码来源验证

**AI功能实现分析**：
- `apps/desktop/src-tauri/src/commands/ai.rs`: 独立实现的AI API调用逻辑
- 前端AI组件: Vue 3独立实现，无Mshell代码痕迹
- 使用标准OpenAI/Anthropic/Gemini API模式
- 无版权声明冲突

**结论**: AI功能是独立开发，仅参考Mshell的功能设计理念，不构成代码衍生。

---

## 当前许可证文件评估

### LICENSE 文件
✅ **状态：完整且正确**
- 包含完整的GPL-3.0许可证文本
- 符合GPL-3.0要求

### NOTICE 文件
⚠️ **状态：需要优化**

**当前内容**：
```
This project incorporates design concepts and features from Mshell
```

**问题**：
- "incorporates"一词可能被误解为"包含代码"
- 未明确说明是"功能参考"而非"代码衍生"

**建议修改**：
```
This project draws inspiration from design concepts in Mshell
(功能设计参考，非代码衍生)
```

### README.md
⚠️ **状态：需要增强**

**当前表述**：
```
融合了 Mshell 的多项核心功能
```

**问题**：
- "融合"可能引起误解
- 未明确说明许可证合规性

---

## 潜在法律风险评估

### 风险等级：🟢 低风险

**理由**：
1. ✅ 主上游nexus-terminal是GPL-3.0，本项目使用GPL-3.0合规
2. ✅ Mshell是MIT许可证，允许参考设计理念
3. ✅ AI功能代码是独立实现，无直接复制
4. ⚠️ 表述不够清晰，可能引起误解（非法律风险，但需优化）

### 需要注意的GPL-3.0义务

根据GPL-3.0第5条，本项目必须：
1. ✅ 保持GPL-3.0许可证
2. ✅ 提供完整源代码（已满足，开源项目）
3. ✅ 保留上游版权声明（NOTICE文件已包含）
4. ⚠️ 标注修改内容（建议在NOTICE中补充）

---

## 优化建议

### 优先级1：修改NOTICE文件（避免误解）

**建议新内容**：
```
Nexus Terminal Desktop
Copyright (C) 2025 nexus-terminal-rust contributors

This product is a desktop rewrite derived from nexus-terminal:

  nexus-terminal — Web SSH / RDP / VNC Client
  Copyright (C) Heavrnl
  Licensed under the GNU General Public License v3.0
  https://github.com/Heavrnl/nexus-terminal

  Derived components:
  - Frontend architecture and terminal UI design
  - SSH session management and SFTP functionality
  - Connection lifecycle and authentication flow

This project draws design inspiration from Mshell:

  Mshell — Windows SSH Client
  Copyright (C) MShell Team
  Licensed under the MIT License
  https://github.com/inspoaibox/Mshell

  Inspired features (independently implemented):
  - AI assistant integration concept
  - Port forwarding workflow design
  - Task scheduling interface patterns

  Note: No source code from Mshell was copied or incorporated.
  All AI and advanced features are original implementations.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <https://www.gnu.org/licenses/>.
```

### 优先级2：优化README.md

**建议修改章节**：

#### 修改"概述"部分
```markdown
## 概述

**Nexus Terminal Desktop** 是 [nexus-terminal](https://github.com/Heavrnl/nexus-terminal) 的桌面版重写，
采用 **Rust + Tauri 2** 架构重构。在保留原版完整终端、SFTP、会话管理能力的基础上，
新增了 **AI 助手**、**端口转发**、**任务调度**等高级功能（功能设计参考 [Mshell](https://github.com/inspoaibox/Mshell)，
代码独立实现）。

相比原版 Web 应用，桌面版通过 Rust 原生实现取代 Node.js 后端，大幅降低资源占用、提升性能。
```

#### 新增"许可证与归属"章节
```markdown
## 许可证与归属

### 开源协议

本项目采用 [GPL-3.0](LICENSE) 开源协议。

### 上游项目归属

本项目是 [nexus-terminal](https://github.com/Heavrnl/nexus-terminal)（GPL-3.0）的衍生作品，
前端架构、终端功能、SFTP模块等核心组件基于原版设计重写。

### 功能设计参考

AI 助手、端口转发等高级功能的设计理念参考了 [Mshell](https://github.com/inspoaibox/Mshell)（MIT License），
但所有代码均为独立实现，未使用 Mshell 的源代码。

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
```

### 优先级3：添加文件头版权声明

**建议在关键源文件添加**：

```rust
// Copyright (C) 2025 nexus-terminal-rust contributors
//
// This file is part of Nexus Terminal Desktop.
//
// Nexus Terminal Desktop is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
```

**建议添加到**：
- `apps/desktop/src-tauri/src/main.rs`
- `apps/desktop/src-tauri/src/commands/ai.rs`
- 所有 `crates/*/src/lib.rs`

### 优先级4：创建CONTRIBUTORS文件

```markdown
# Contributors

## Core Team
- [Your Name/Team] - Project maintainer and primary developer

## Original Work Attribution

### nexus-terminal
- Original Author: Heavrnl
- Repository: https://github.com/Heavrnl/nexus-terminal
- License: GPL-3.0
- Contribution: Base architecture, terminal functionality, SFTP implementation

### Design Inspiration
- Mshell Team
- Repository: https://github.com/inspoaibox/Mshell
- License: MIT
- Contribution: AI assistant feature concept, workflow design patterns

## Third-Party Libraries
See `Cargo.toml` and `package.json` for complete dependency list.
```

---

## 合规性检查清单

### 必须完成（避免法律风险）
- [ ] 修改NOTICE文件，明确区分"衍生"和"参考"
- [ ] 优化README.md，准确描述项目关系
- [ ] 确认未直接复制Mshell代码

### 建议完成（提升专业性）
- [ ] 添加文件头版权声明
- [ ] 创建CONTRIBUTORS文件
- [ ] 添加第三方依赖许可证说明
- [ ] 在应用"关于"页面显示许可证信息

### 可选完成（最佳实践）
- [ ] 使用REUSE规范管理许可证
- [ ] 添加CLA（贡献者许可协议）
- [ ] 设置GitHub许可证检查CI

---

## 总结

### 当前状态
✅ **许可证合规**：项目使用GPL-3.0符合上游要求
⚠️ **表述不清**：NOTICE和README的措辞可能引起误解
🟢 **风险等级**：低（无法律风险，但需优化文档）

### 核心建议
1. **立即修改NOTICE文件**，明确说明与Mshell的关系是"功能参考"而非"代码衍生"
2. **优化README.md**，准确描述项目定位和上游关系
3. **添加详细归属说明**，避免潜在的版权纠纷

### 法律意见
本分析基于公开信息和代码审查，不构成正式法律意见。如需确保完全合规，
建议咨询专业开源许可证律师。

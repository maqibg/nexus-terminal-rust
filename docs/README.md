# Nexus Terminal Rust 文档总览

> 项目：`nexus-terminal-rust`（Tauri 2 + Rust，零开始重构）

本目录是重构项目的**单一事实来源（Single Source of Truth）**。  
所有设计决策、开发规范、实施计划、发布与回滚均以本文档集为准。

---

## 1. 文档结构

1. `REBUILD_PLAN_TAURI2_RUST.md`  
   初版重构蓝图（目标、架构、阶段、风险），用于立项与方向确认。

2. `DEVELOPMENT_GUIDE.md`  
   完整开发文档（架构、模块、编码规范、测试标准、安全与运维规范）。

3. `DEVELOPMENT_PLAN.md`  
   完整开发计划（WBS、里程碑、阶段目标、资源安排、风险管理、验收节奏）。

4. `DELIVERY_CHECKLIST.md`  
   执行与交付清单（按阶段可勾选），用于日常推进与发布前总复核。

---

## 2. 阅读顺序（建议）

1. 先读：`REBUILD_PLAN_TAURI2_RUST.md`
2. 再读：`DEVELOPMENT_GUIDE.md`
3. 然后：`DEVELOPMENT_PLAN.md`
4. 每日执行：`DELIVERY_CHECKLIST.md`

---

## 3. 文档维护规则

- 涉及架构、API、数据模型、权限边界的变更，必须同步更新文档。
- 文档更新与代码变更必须同批提交，禁止“代码先行、文档滞后”。
- 重大决策须记录“背景-方案-权衡-结论-回滚条件”。
- 每周固定一次文档巡检，清理过时内容。

---

## 4. 当前重构原则（冻结）

- 容器：`Tauri 2`
- 后端：`Rust` 单栈
- 前端：沿用现有 Vue 体系，**不改变显示效果**
- 协议：优先保持现有 `HTTP/WS` API 兼容
- 发布策略：并行可回退，逐步替换，不做一次性 Big Bang 上线
- 目标平台：**仅 Windows 桌面端**

## 5. 明确排除项（桌面端不实现）

以下为原系统网页端功能，桌面端重构不包含：

| 排除项 | 原因 |
|--------|------|
| Docker 容器管理 | 网页端通过 SSH 远程执行 Docker 命令，桌面端不需要 |
| RDP/VNC（Guacamole 代理） | 网页端通过 guacd 代理，桌面端仅保留 mstsc 本地启动 |
| Captcha 验证码 | hCaptcha/reCAPTCHA 为网页端防护 |
| remote-gateway 包 | 网页端独立部署场景 |
| IP 白名单/黑名单 | 网页端安全防护，桌面端本地单用户 |
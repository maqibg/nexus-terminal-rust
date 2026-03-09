# UI / Runtime Remediation Plan

## 目标

本轮修复聚焦桌面端当前主线：`App` 全局壳、`Connections`、`Workspace`、`Settings`、`TerminalView`、Tauri 命令层与 SFTP/SSH 运行链。

明确**不在本轮处理**：

- 全局禁用原生右键菜单
- 整体测试覆盖率偏低的问题本身

## 修复范围

1. **入口与状态一致性**
   - 修正认证检查失败时仍可进入受保护页面的问题
   - 为 `settings` / `connections` / `appearance` / `ai` 增加并发加载去重
   - 修正 AI 设置页共享 `loading` 状态导致的提前结束

2. **布局与交互**
   - 为工作区布局节点补稳定 `id`，避免拖拽重排后组件实例错位
   - 合并布局 resize 通知，降低终端 resize 与图表重绘风暴
   - 为工作区侧栏尺寸增加延迟持久化
   - 移除未实现的 Docker 假入口和相关设置项
   - 为 `AppSelect` 补方向键 / Enter / Home / End 操作
   - 为全局样式补 `prefers-reduced-motion` 降级策略

3. **运行链与后端**
   - 将 `get_backend_health` 从固定返回改为真实目录/数据库检查
   - 去掉启动阶段多处 `expect`，改为可传播的初始化错误
   - 补齐 `ssh_suspend_*` 命令，恢复前端已有挂起会话 UI
   - 将未使用的 `get_connection_runtime_status` 接到状态监视器首包拉取

4. **文件与备份**
   - SFTP 粘贴改为后端流式复制，避免大文件经前端内存中转
   - 文件打开的 Base64 解码统一为 `TextDecoder`
   - 完整备份补齐 SSH key、连接密码、代理凭据
   - 版本检查改为当前仓库 `maqibg/nexus-terminal-rust`
   - 自定义 `terminal_custom_html` 真正接入终端背景渲染

## 验收标准

- 认证异常时自动回到 `/login`
- 调整工作区侧栏宽度后重启仍保留
- 挂起/恢复命令可被前端调用且恢复后可重新显示会话
- 自定义 HTML 背景应用后终端可见
- 批量连接按钮在全部连接完成前保持 loading
- `pnpm --dir apps/desktop/frontend build` 与 `cargo test --workspace` 至少完成一轮验证

# 状态监控增强开发文档
> 目标：在**保留 `nexus-terminal-rust` 当前状态监控全部能力**的前提下，补齐 `Netcatty` 已有而当前项目缺失的全部监控能力；重叠功能按“实现质量更高、架构更适配 Tauri/Rust”的方案落地。

## 1. 方案结论
本次增强不应照搬 `Netcatty` 的前端轮询模型，而应采用三层融合方案：
1. 保留 `nexus-terminal-rust` 现有的 Rust 后端采集 + Tauri 事件推送架构。
2. 吸收 `Netcatty` 更丰富的指标、设备保护、展示细节和可见性经验。
3. 在保留现有独立 `StatusMonitor` 面板的同时，新增一个可选的终端内联监控摘要。
最终目标形态：
- 工作区 `StatusMonitor` 面板继续保留。
- 新增终端内联监控摘要，可独立开关。
- 指标维度达到并覆盖 `Netcatty`。
- 采集架构升级为“后端集中采集 + 前端按需订阅 + 失败保护”。

## 2. 对比结论
### 2.1 `nexus-terminal-rust` 当前已有
- 独立工作区监控面板
- 首包主动拉取 + 后续事件推送
- CPU / 内存 / Swap / 磁盘 / 网络总览
- 多磁盘列表
- CPU / 网络趋势图
- Windows 远端 PowerShell 采集
- SSH 连接、恢复、关闭、挂起时的采集生命周期控制
### 2.2 `Netcatty` 当前更多的能力
- 终端底部内联监控摘要
- CPU 核心数
- 每核心 CPU 使用率
- 内存细分：`free / buffers / cached`
- Top 10 内存占用进程
- 每网卡速率明细
- 更丰富的磁盘细节展示
- macOS 远端采集
- 网络设备保护逻辑
- 连续失败熔断 / 降噪策略
- 可见性驱动的采样约束
### 2.3 重叠能力最佳实现选择
| 能力 | 方案 |
|------|------|
| 首包获取 | 保留 `nexus-terminal-rust` 的 `get_connection_runtime_status` |
| 持续刷新 | 保留当前后端集中采集 + 事件推送 |
| CPU / 网络 delta 计算 | 保留当前 Rust 实现思路 |
| 指标维度 | 采用 `Netcatty` 的完整指标集 |
| 展示层 | 融合：保留面板，补齐内联摘要与细节 |
| 支持平台 | 取并集：Linux + Windows + macOS |
| 失败控制 | 采用增强融合方案，不直接照搬任一现状 |
### 2.4 共有功能逐项对比与取优
| 共有功能 | `nexus-terminal-rust` 实现 | `Netcatty` 实现 | 取优结论 |
|------|------|------|------|
| 监控总开关 | 工作区级开关，关闭后可直接停止后端采集 | 终端设置级开关，主要控制终端状态栏展示与轮询 | 选 `nexus-terminal-rust`，控制面更集中，也能直接作用后端 |
| 刷新间隔设置 | 独立设置项，默认 `3s`，连接与恢复时都能生效 | 终端设置项，默认 `5s`，前端轮询最小 `5s` | 选 `nexus-terminal-rust` 的设置入口，吸收 `Netcatty` 的最小间隔和可见性约束 |
| 首包获取 | 先拉一次 `get_connection_runtime_status`，避免面板空白等待 | 依赖首轮轮询结果，没有单独首包命令 | 选 `nexus-terminal-rust`，首屏体验更稳 |
| 持续刷新链路 | Rust 后端常驻任务 + Tauri 事件推送 | React hook 定时调用 Electron IPC | 选 `nexus-terminal-rust`，更适合多消费者和 Tauri 架构 |
| 连接生命周期联动 | 连接、恢复、关闭、挂起都显式接入监控启停 | 跟随终端组件状态和可见性控制轮询 | 选 `nexus-terminal-rust`，会话生命周期控制更完整 |
| CPU / 网络速率算法 | 后端保存前值，按 delta 计算 CPU 与网速 | Electron bridge 保存前值，按 delta 计算 CPU 与网速 | 选 `nexus-terminal-rust` 的 Rust 实现，类型安全更高；补入 `Netcatty` 的 macOS 分支 |
| 错误可见化 | `status:error:{sessionId}` 事件明确上送前端 | hook 内维护 `error` 状态并控制停轮询 | 选融合方案：保留当前错误事件通道，吸收 `Netcatty` 的连续失败保护 |
| 基础指标覆盖 | CPU / 内存 / Swap / 磁盘 / 网络摘要 | CPU / 内存 / 磁盘 / 网络摘要 | 保留当前字段兼容层，按 `Netcatty` 补全更细粒度字段 |

## 3. 目标能力定义
增强完成后，`nexus-terminal-rust` 必须同时支持：
1. 工作区独立 `StatusMonitor` 面板。
2. 可选终端内联监控摘要。
3. 指标覆盖：
- CPU 总使用率
- CPU 核心数
- 每核心 CPU 使用率
- 内存总量 / 已用 / 空闲 / buffers / cached
- Swap 总量 / 已用 / 空闲
- Top 10 内存进程
- 根分区摘要
- 全挂载磁盘列表
- 网络主接口摘要
- 每接口上下行速率
4. 趋势图覆盖：CPU、网络上下行、内存使用。
5. 远端采集覆盖：Linux、Windows、macOS。
6. 对网络设备或明显不支持 shell 采样的目标显式停止高级采样并给出原因。
7. 所有采样错误必须可见，不允许静默失败。

## 4. 不纳入本阶段
以下内容不属于本次增强范围：
- 本机设备电池
- 本机温度 / 风扇 / 功耗
- 本机 CPU / 内存 / 磁盘状态条
- Docker / 容器指标并入状态监控
原因：这些能力在 `nexus-terminal-rust` 与 `Netcatty` 中都未形成完整现成实现；本次目标是补齐 `Netcatty` 已有能力，而不是新建一套本机系统监控子系统。

## 5. 目标架构
### 5.1 总体模型
采用“后端集中采集，前端多视图消费”：
- Rust 后端维护每个 SSH 会话的监控 collector。
- 前端消费者包括工作区 `StatusMonitor`、终端内联摘要、未来可能的连接列表快速状态。
- 前端不直接轮询远端 shell。
- 前端只负责首包请求、事件订阅、可见性和订阅意图上报。
### 5.2 明确放弃
- 不照搬 `Netcatty` 的“每个终端组件自己 `setInterval` 调 IPC”。
- 不继续扩张当前单文件 `status_monitor.rs`。
- 不把高级采样逻辑塞进 Vue 组件。
### 5.3 模块拆分
将 `apps/desktop/src-tauri/src/status_monitor.rs` 重构为：
- `status_monitor/mod.rs`
- `status_monitor/types.rs`
- `status_monitor/service.rs`
- `status_monitor/collector/linux.rs`
- `status_monitor/collector/windows.rs`
- `status_monitor/collector/macos.rs`
- `status_monitor/parser.rs`
- `status_monitor/policy.rs`
职责边界：
- `types.rs`：payload / 原始结构
- `collector/*`：远端脚本与原始输出
- `parser.rs`：解析归一化
- `service.rs`：任务、缓存、事件
- `policy.rs`：支持性判断、失败策略、设备保护

## 6. 数据模型方案
### 6.1 扩展字段
在现有 `StatusUpdatePayload` 基础上新增：
- `cpu_cores: Option<u32>`
- `cpu_per_core: Vec<f64>`
- `mem_free: Option<u64>`
- `mem_buffers: Option<u64>`
- `mem_cached: Option<u64>`
- `top_processes: Vec<StatusProcessEntry>`
- `net_interfaces: Vec<NetInterfaceEntry>`
新增结构：
- `StatusProcessEntry { pid, mem_percent, command }`
- `NetInterfaceEntry { name, rx_total, tx_total, rx_rate, tx_rate }`
### 6.2 兼容原则
- 旧字段全部保留。
- 老前端不消费新增字段也不应回归。
- 当前 `RuntimeStatusSnapshot` 走增量扩展，不做破坏性改名。

## 7. 采样策略
### 7.1 生命周期
保留当前优点：
- SSH 建立时可启动监控
- SSH 恢复时重建监控
- SSH 关闭 / 挂起时停止监控
升级点：
- 不再“只要连接存在就持续采样”
- 改为“连接存在且至少一个前端消费者订阅时采样”
### 7.2 订阅模型
新增命令：
- `status_subscribe(sessionId, consumerId)`
- `status_unsubscribe(sessionId, consumerId)`
`consumerId` 取值：
- `workspace-pane`
- `terminal-inline:<tabId>`
后端维护每个 session 的订阅计数。
### 7.3 可见性规则
- 工作区面板只订阅当前活动 SSH 会话。
- 终端内联摘要只在对应终端可见时订阅。
- 隐藏后取消订阅，恢复可见后重新订阅。
- 恢复时网络速率先重置为 0，避免隐藏期间累计流量被误算成瞬时速率。

## 8. 支持性与失败处理
### 8.1 网络设备保护
必须引入 `Netcatty` 的保护思想：
- 对识别为网络设备的 SSH 目标默认关闭高级 shell 采样。
- 前端明确展示“不支持该主机类型的高级状态采集”。
- 禁止继续无意义重试导致远端 AAA / 审计日志膨胀。
### 8.2 连续失败策略
不直接照搬 `Netcatty` 的“3 次失败后永久熔断”，采用增强方案：
1. 连续失败 `3` 次后进入 `degraded`。
2. `degraded` 状态下降到慢轮询，例如 `30s`。
3. 错误持续对前端可见。
4. 手动刷新、重新连接、恢复会话、重新订阅时退出 `degraded`。
### 8.3 首包规则
- Linux / macOS 首包 CPU 可为空值或 fallback 百分比。
- 网络速率首包固定为 `0`。
- 第二轮事件更新必须补齐 delta 指标。

## 9. 前端展示方案
### 9.1 保留并增强现有面板
保留 `StatusMonitor.vue`，但拆成子组件：
- `StatusMonitorSummary.vue`
- `StatusMonitorCpuPanel.vue`
- `StatusMonitorMemoryPanel.vue`
- `StatusMonitorDiskPanel.vue`
- `StatusMonitorNetworkPanel.vue`
- `StatusMonitorProcessPanel.vue`
- `StatusMonitorCharts.vue`
新增展示：
- CPU 核心数与每核心网格
- 内存组成条：used / buffers / cached / free
- Swap 细节
- Top 10 进程列表
- 每网卡速率列表
- 内存折线图
### 9.2 新增终端内联摘要
新增 `StatusInlineSummary.vue`，挂在：
- `apps/desktop/frontend/src/components/SessionTerminalView.vue`
要求：
- 仅 SSH 会话显示
- 默认关闭，通过设置开启
- 展示 CPU / 内存 / 根磁盘 / 上下行速率摘要
- 支持点击或悬浮弹出细节
- 支持跳转或高亮工作区 `StatusMonitor`

## 10. 设置项
保留现有：
- `statusMonitorEnabled`
- `statusMonitorIntervalSeconds`
- `showStatusMonitorIpAddress`
- `statusMonitorShowCharts`
新增：
- `statusMonitorInlineSummaryEnabled`
- `statusMonitorShowTopProcesses`
- `statusMonitorShowPerCoreCpu`
- `statusMonitorShowInterfaceDetails`
- `statusMonitorFailureBackoffEnabled`
原则：高级项可关闭，但默认值面向信息完整；关闭项只影响展示和调度，不破坏类型兼容。

## 11. 实施拆解
### 模块 A：类型与协议
- 扩展 `StatusUpdatePayload`
- 扩展 `api-status.ts`
- 扩展前端 `SessionStatusSnapshot`
### 模块 B：collector 重构
- 拆出 Linux / Windows collector
- 新增 macOS collector
- 统一原始输出格式
### 模块 C：服务调度
- 引入订阅计数
- 引入 `degraded`
- 引入失败统计与退避
### 模块 D：前端面板增强
- 拆分 `StatusMonitor.vue`
- 接入新增字段
- 增加内存图与进程列表
### 模块 E：终端内联摘要
- 新增 `StatusInlineSummary.vue`
- 接入可见性与面板联动
### 模块 F：文档与测试
- 更新 `docs/UI_FEATURE_INVENTORY.md`
- 更新 `docs/DEVELOPMENT_GUIDE.md`
- 补齐静态检查、组件测试和后端 collector 验证

## 12. 验收标准
### 功能
- Linux：全部指标可用
- Windows：保持当前摘要指标，新增能力按系统差异部分可用
- macOS：CPU / 内存 / Swap / 磁盘 / 网络 / Top 进程可用
- 当前工作区面板不回归
- 新增终端内联摘要可独立开关
- 当前图表继续可用，并新增内存图
### 稳定性
- 隐藏终端后不再高频采样
- 恢复后网络速率不虚高
- 网络设备不会刷远端 AAA / 审计日志
- 连续失败可进入可见降级状态
### 结构
- 不再保留超长单文件 `status_monitor.rs`
- 新增模块职责边界清晰
- 前端监控组件拆分后长度受控

## 13. 开发顺序
1. 后端类型与 collector 重构
2. 前端面板字段接入
3. 补齐 `Netcatty` 细节能力
4. 实现终端内联摘要与设置
5. 补文档与测试
禁止顺序：
- 禁止先做内联 UI，再补后端字段
- 禁止继续在原 `status_monitor.rs` 上堆新逻辑

## 14. 最终判定
最优方案不是“把 `Netcatty` 监控照搬进来”，而是：
- 架构层保留 `nexus-terminal-rust` 更适合 Tauri/Rust 的后端集中采集模型
- 能力层完整吸收 `Netcatty` 的丰富指标与保护逻辑
- 交互层同时保留当前独立面板，并补齐 `Netcatty` 的终端内联监控体验
只有这样，才能同时满足：
- 保留当前项目已有功能
- 获得 `Netcatty` 全部额外监控能力
- 不把复杂度继续堆到超长单文件和超大组件里
阅读完阅读这个新文档

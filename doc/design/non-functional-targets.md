# wp-agent 非功能目标草案

## 1. 文档目的

本文档把前面讨论过的“业务优先、低资源、可退化、可恢复”收敛成可量化目标草案。

重点回答：

- 空闲态 CPU / 内存目标是多少
- 中等流量下 CPU / 内存目标是多少
- 峰值流量下允许怎样退化
- buffer 上限和 backpressure 如何定义

相关文档：

- [`target.md`](./target.md)
- [`architecture.md`](./architecture.md)
- [`self-observability.md`](./self-observability.md)
- [`agent-config-schema.md`](./agent-config-schema.md)

---

## 2. 核心结论

`wp-agent` 如果要达到“业务第一”的非功能水准，不能只写原则，必须把资源预算、退化阈值和保底能力写死成验收目标。

第一版先固定四个结论：

- 控制面、数据面、升级面必须有隔离预算
- 默认资源目标按常见边缘节点规格定义
- 峰值下优先退化观测深度，不拖垮宿主业务
- 最后保底对象必须明确，不能在压力下把审计、状态和最终结果一起丢掉

---

## 3. 度量前提

### 3.1 参考节点

第一版量化目标先以以下基线环境为准：

- Linux `x86_64`
- `2 vCPU`
- `4 GiB RAM`
- 本地 SSD

### 3.2 参考功能集

第一版非功能目标先覆盖以下能力组合：

- `wp-agentd` 常驻
- control plane 长连
- self-observability 开启
- `host_metrics`
- `process_metrics`
- `container_metrics`
- `k8s_node_pod_metrics` 或等价节点侧 Kubernetes 采集
- `prom_scrape` 基础抓取
- `otlp_metrics_receiver` 开启但受全局 budget 约束

### 3.3 流量档位

第一版先统一三档口径：

- `idle`
  只有心跳、自观测和低频发现刷新，无持续业务采集流量
- `moderate`
  平台底盘级 metrics 常态采集，约等于：
  - `20` 个 `prom_scrape` target
  - `<= 10000` metrics samples / 秒的持续输入
  - `<= 300` 个 container / pod 级被观测对象
- `peak`
  `3x moderate` 持续 `10` 分钟，且伴随网络抖动、控制面重试或 target 波动

说明：

- 日志 / traces / security 的更细量化目标后续再补
- 当前草案先把 `wp-agentd` 的公共底盘和 Batch A metrics 路径固定下来

---

## 4. 资源目标

### 4.1 `idle`

- CPU:
  - `p95 <= 1%` 单核占用
  - `p99 <= 2%`
- 内存:
  - `RSS <= 96 MiB`
  - `峰值 RSS <= 128 MiB`
- 线程 / fd:
  - `threads <= 32`
  - `open_fds <= 128`

### 4.2 `moderate`

- CPU:
  - `p95 <= 8%`
  - `p99 <= 12%`
- 内存:
  - `RSS <= 220 MiB`
  - `峰值 RSS <= 288 MiB`
- 线程 / fd:
  - `threads <= 96`
  - `open_fds <= 512`

### 4.3 `peak`

在 `peak` 档位下，不要求完全无退化，但必须满足：

- CPU:
  - `p95 <= 20%`
  - 短时尖峰允许到 `30%`，但不能持续超过 `60` 秒
- 内存:
  - `RSS <= 384 MiB`
  - 任何情况下都不得因 agent 自身无界增长触发宿主 OOM 风险
- 功能保底:
  - control plane 心跳不丢
  - 当前运行中的 action 最终结果不丢
  - 本地审计事件不丢

---

## 5. 退化等级

### 5.1 `normal`

满足常规 budget，全部已启用 integration 正常工作。

### 5.2 `degraded`

进入条件建议满足任一项：

- `CPU p95 > 15%` 持续 `3` 分钟
- `RSS > 300 MiB`
- 全局内存 buffer 使用率 `> 70%`
- 磁盘 spool 使用率 `> 80%`

退化动作建议按顺序执行：

1. 降低 discovery refresh 频率
2. 降低 self-observability 日志细度
3. 降低低优先级 `prom_scrape` 并发
4. 暂停可选的高成本 label / metadata enrich

### 5.3 `protect`

进入条件建议满足任一项：

- `CPU p95 > 25%` 持续 `1` 分钟
- `RSS > 384 MiB`
- 全局内存 buffer 使用率 `> 90%`
- 磁盘 spool 使用率 `> 95%`

保护动作建议按顺序执行：

1. 暂停低优先级 integration
2. 拒绝新的低优先级 receiver 输入
3. 停止发现新 target，只保留已有 target 稳态采集
4. 保留 control plane、状态落盘、审计和结果回传预算

### 5.4 退出条件

建议满足以下条件再退出：

- CPU、内存、水位连续 `5` 分钟回落到 `degraded` 以下
- 未出现新的 report backlog 扩大

---

## 6. Buffer 与 Backpressure 目标

### 6.1 内存 buffer

第一版建议默认上限：

- 单 integration in-memory queue: `32 MiB`
- 全局 telemetry in-memory queue: `128 MiB`
- action result/report in-memory queue: `16 MiB`

### 6.2 磁盘 spool

第一版建议默认上限：

- telemetry spool: `4 GiB`
- action/report spool: `512 MiB`

### 6.3 本地执行与回传队列

第一版建议默认上限：

- `execution_queue` 最大 `128` 项
- `reporting` 最大 `256` 项

### 6.4 Backpressure 顺序

第一版必须固定丢弃 / 限流顺序：

1. 先压缩 debug/self-observability 明细
2. 再限制低优先级 telemetry 输入
3. 再暂停低优先级 target discovery
4. 最后才允许丢弃低优先级 telemetry 样本

默认不应丢弃：

- `ActionResult`
- 审计事件
- agent 状态切换事件
- control plane 必需心跳

---

## 7. 执行与升级面的保底目标

### 7.1 远程执行

在 `normal` 或 `degraded` 模式下，建议满足：

- 已接受计划到 `wp-agent-exec` 启动：
  - `p95 <= 2s`
- cancel 请求到本地 kill / graceful stop 生效：
  - `p95 <= 3s`
- 单节点同时运行 action：
  - 第一版默认 `1`
  - 通过显式配置才能放大

### 7.2 自升级

第一版建议满足：

- 升级期间控制面状态可见
- 升级失败能自动回滚
- 升级过程与 action execution 默认互斥
- 升级不会清空本地 history / reporting 状态

---

## 8. 验收建议

第一版建议至少做三类验收：

- `24h` soak test：
  - 不进入 `protect`
  - 无内存持续爬升
- `peak` 压测：
  - agent 不触发宿主业务异常
  - 最终结果和审计链不丢
- 网络异常压测：
  - spool / backpressure 行为符合预期
  - 恢复后能渐进追平 backlog

---

## 9. 当前决定

当前阶段固定以下结论：

- 非功能目标必须量化，不能只写“低资源”
- `wp-agent` 要优先保护业务、控制面和审计链
- 退化策略必须先于实现编码
- `normal / degraded / protect` 三态应作为后续实现和验收的统一口径

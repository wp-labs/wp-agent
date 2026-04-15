# warp-insight Self-Observability 设计

## 1. 文档目的

本文档定义 `warp-insight` 自身的可观测性设计，重点覆盖：

- `warp-insightd`
- `warp-insight-exec`
- `warp-insight-upgrader`

目标是让边缘代理自身也具备可诊断、可验收、可压测的观测能力。

相关文档：

- [`agentd-architecture.md`](agentd-architecture.md)
- [`metrics-batch-a-plan.md`](../telemetry/metrics-batch-a-plan.md)
- [`error-codes.md`](error-codes.md)

---

## 2. 核心结论

`warp-insight` 必须把自观测视为一等能力，而不是上线后再补的辅助项。

第一版自观测至少要覆盖三类信息：

- agent 自身指标
- agent 自身日志
- agent 自身关键事件

一句话说：

- 没有 self-observability，就很难验证 `warp-insight` 是否真的轻、稳、可退化、可恢复

---

## 3. 设计原则

### 3.1 自观测必须分层

建议分成：

- process-level
- module-level
- workflow-level

### 3.2 自观测不能反过来拖垮 agent

第一版必须控制：

- 自身指标数量
- 自身日志量
- 事件采样与保留窗口

### 3.3 自观测优先服务工程验收

第一版重点不是“大而全”，而是能回答：

- agent 现在健康吗
- 为什么拒绝了一个计划
- queue/running/reporting 当前是什么状态
- metrics 数据面是否过载
- 是否进入 degrade / protect 模式

---

## 4. 观测对象分层

### 4.1 `warp-insightd`

重点观测：

- daemon 生命周期
- control plane 连接
- execution_queue / running / reporting 状态
- metrics collection framework 健康
- degrade / protect 状态

### 4.2 `warp-insight-exec`

重点观测：

- 进程启动和退出
- step 执行统计
- stdout/stderr 摘要
- 失败、取消、超时原因

### 4.3 `warp-insight-upgrader`

重点观测：

- 升级准备
- 校验
- 切换
- 回滚

---

## 5. Self Metrics

### 5.1 `warp-insightd` 基础指标

建议第一版至少包括：

- `agent_up`
- `agent_build_info`
- `agent_uptime_seconds`
- `agent_mode`

说明：

- `agent_mode` 可用受控枚举表示：
  - `normal`
  - `degraded`
  - `protect`
  - `upgrade_in_progress`

### 5.2 调度与执行指标

建议至少包括：

- `agent_execution_queue_size`
- `agent_running_executions`
- `agent_reporting_executions`
- `agent_plan_received_total`
- `agent_plan_rejected_total`
- `agent_plan_completed_total`
- `agent_plan_failed_total`
- `agent_plan_cancelled_total`
- `agent_plan_timed_out_total`

### 5.3 子进程指标

建议至少包括：

- `agent_exec_spawn_total`
- `agent_exec_spawn_failed_total`
- `agent_exec_exit_total`
- `agent_exec_kill_total`

### 5.4 metrics 数据面指标

建议至少包括：

- `agent_metrics_targets_total`
- `agent_metrics_scrape_total`
- `agent_metrics_scrape_failed_total`
- `agent_metrics_samples_received_total`
- `agent_metrics_samples_dropped_total`
- `agent_metrics_receiver_rejected_total`

### 5.5 资源与保护模式指标

建议至少包括：

- `agent_cpu_usage_pct`
- `agent_memory_rss_bytes`
- `agent_open_fds`
- `agent_degrade_enter_total`
- `agent_protect_enter_total`
- `agent_backpressure_events_total`

---

## 6. Self Logs

### 6.1 日志分类

建议第一版分三类：

- `system`
- `execution`
- `metrics`

### 6.2 `system`

记录：

- 启动
- 配置加载
- control plane 连接变化
- 模式切换

### 6.3 `execution`

记录：

- 计划接收
- 校验拒绝
- spawn
- cancel
- exit
- result report

### 6.4 `metrics`

记录：

- target discovery 变化
- scrape 错误摘要
- receiver 拒绝摘要
- budget 命中

### 6.5 日志约束

第一版必须避免：

- 高频全量 debug 默认打开
- 把大 payload 原样打进日志
- 把敏感 secret 写入日志

---

## 7. Self Events

建议自观测事件与 [`agentd-events.md`](agentd-events.md) 对齐。

第一版建议至少把以下事件进入本地审计或事件流：

- `PlanReceived`
- `PlanRejected`
- `SpawnRequested`
- `ProcessSpawned`
- `CancelRequested`
- `ProcessExited`
- `ResultReady`
- `ReportFailed`

metrics 数据面建议至少有：

- `DiscoveryRefreshed`
- `TargetAdded`
- `TargetRemoved`
- `ScrapeBudgetHit`
- `ReceiverRejected`

---

## 8. 指标与事件命名建议

第一版建议统一前缀：

- metrics: `agent_`
- events: `Agent*`

例如：

- `agent_execution_queue_size`
- `AgentPlanRejected`

---

## 9. Batch A 验收依赖

`Batch A` metrics 数据面是否达标，强依赖 self-observability。

至少需要依靠这些指标判断：

- scrape 是否稳定
- 样本是否被丢弃
- target 是否持续失效
- memory / cpu 是否逼近上限

也就是说：

- self-observability 不是附属功能
- 它是 metrics 数据面验收前提

---

## 10. 第一版限制

第一版不建议：

- 做复杂 tracing 链路
- 做高频全事件持久化
- 做过深的 profiling 系统

第一版先把：

- 核心 metrics
- 核心 logs
- 核心事件

做稳定即可。

---

## 11. 当前决定

当前阶段固定以下结论：

- `warp-insight` 自身必须具备自观测能力
- `warp-insightd` 的 queue / running / reporting / mode 必须有对应指标
- metrics 数据面验收必须依赖 self-observability 指标
- 自观测要受控，不能反过来拖重 agent

# wp-agentd 本地状态与模块边界设计

## 1. 文档目的

本文档把 `wp-agentd` 的两件关键前置设计固定下来：

- 本地状态模型
- 模块实现边界

它直接服务于：

- `M2 Agentd Skeleton`
- `M4 Local Edge Loop`

相关文档：

- [`agentd-architecture.md`](./agentd-architecture.md)
- [`agentd-exec-protocol.md`](./agentd-exec-protocol.md)
- [`action-plan-ir.md`](./action-plan-ir.md)
- [`roadmap.md`](./roadmap.md)

---

## 2. 核心结论

`wp-agentd` 的实现必须建立在下面三个原则上：

1. 本地状态必须分层
2. 每类状态必须有唯一写入者
3. 模块边界必须按“接收、校验、调度、执行管理、结果汇总”拆开

一句话说：

- 不允许多个模块抢写同一个状态对象
- 不允许调度模块直接改执行结果
- 不允许执行管理模块直接做校验结论

---

## 3. 本地状态分层

`wp-agentd` 第一版建议把本地状态分成五层：

- `agent_runtime_state`
- `execution_queue_state`
- `execution_state`
- `report_state`
- `history_state`

### 3.1 `agent_runtime_state`

表示 daemon 自身运行态。

主要内容：

- agent 版本
- instance id
- boot id
- 当前配置版本
- 当前策略版本
- 当前全局模式

例如：

- `normal`
- `degraded`
- `protect`
- `upgrade_in_progress`

### 3.2 `execution_queue_state`

表示等待本地调度的 execution 队列。

这里的 `execution_queue` 专指：

- 已通过本地校验
- 尚未拉起 `wp-agent-exec`
- 正在等待 `execution_scheduler` 调度

它不是：

- 数据面的 buffer / spool
- 网络消息队列
- metrics / logs / traces 事件队列

主要内容：

- 待执行 execution 列表
- 优先级
- 入队时间
- 是否可取消

### 3.3 `execution_state`

表示单个 execution 的运行时状态。

主要内容：

- 当前状态
- 当前 step
- 子进程 pid
- workdir
- deadline
- cancel 标记

### 3.4 `report_state`

表示结果上报状态。

主要内容：

- 是否已形成最终结果
- 是否已成功回传中心
- 最近一次上报时间
- 最近一次上报失败原因

### 3.5 `history_state`

表示历史归档。

主要内容：

- 最近执行摘要
- 最近拒绝摘要
- 最近失败摘要

第一版不要求长周期历史都在本地保留，但要有最小归档能力。

---

## 4. 本地目录与对象模型

建议 `wp-agentd` 采用如下目录：

```text
<agent_root>/
  run/
    actions/<execution_id>/
  state/
    agent_runtime.json
    execution_queue.json
    running/
      <execution_id>.json
    reporting/
      <execution_id>.json
    history/
      recent.json
  log/
    agentd.log
```

### 4.1 `agent_runtime.json`

保存 daemon 自身状态。

建议字段：

- `agent_id`
- `instance_id`
- `boot_id`
- `version`
- `config_version`
- `policy_version`
- `mode`
- `updated_at`

### 4.2 `execution_queue.json`

保存等待调度的 execution 队列。

建议字段：

- `items[]`

每个 `item` 建议包括：

- `execution_id`
- `action_id`
- `request_id`
- `priority`
- `queued_at`
- `deadline_at`

### 4.3 `running/<execution_id>.json`

保存运行中 execution 的控制视图。

建议字段：

- `execution_id`
- `action_id`
- `state`
- `workdir`
- `pid?`
- `started_at?`
- `deadline_at`
- `cancel_requested_at?`
- `kill_requested_at?`
- `updated_at`

### 4.4 `reporting/<execution_id>.json`

保存待上报或上报中的结果视图。

建议字段：

- `execution_id`
- `action_id`
- `final_state`
- `result_path`
- `report_attempt`
- `last_report_at?`
- `last_report_error?`

### 4.5 `history/recent.json`

保存最近执行摘要。

第一版只需保留有限窗口，例如最近 N 条。

---

## 5. 哪些状态必须落盘

第一版建议明确区分：

### 5.1 必须落盘

- `agent_runtime.json`
- `execution_queue.json`
- `running/<execution_id>.json`
- `reporting/<execution_id>.json`
- `run/actions/<execution_id>/plan.json`
- `run/actions/<execution_id>/runtime.json`
- `run/actions/<execution_id>/state.json`
- `run/actions/<execution_id>/result.json`

### 5.2 可以只保存在内存

- 临时调度索引
- 进程句柄对象
- 短生命周期 debounce 状态
- 非关键统计缓存

原则是：

- crash 后必须能恢复的状态，就落盘
- crash 后可以重算的状态，就只放内存

---

## 6. 模块唯一写入权

这是实现边界里最重要的一条。

建议唯一写入权如下：

| 状态对象 | 唯一写入模块 | 其他模块权限 |
|---|---|---|
| `agent_runtime.json` | `bootstrap` / `config_runtime` | 只读 |
| `execution_queue.json` | `execution_scheduler` | 只读 |
| `running/<execution_id>.json` | `execution_scheduler` | `executor_manager` 通过事件回传，由 scheduler 落盘 |
| `reporting/<execution_id>.json` | `result_aggregator` | 只读 |
| `history/recent.json` | `audit_logger` | 只读 |
| `run/actions/*/meta.json` | `executor_manager` | 只读 |

这里的核心原则是：

- `executor_manager` 不直接写 scheduler 的状态文件
- `result_aggregator` 不回写 queue 状态
- `plan_validator` 不直接改 running 集合

模块之间通过事件或返回对象传递，不通过“大家一起改文件”协作。

---

## 7. 模块边界与调用关系

第一版建议模块调用链固定为：

```text
control_receiver
  -> plan_validator
  -> execution_scheduler
  -> executor_manager
  -> result_aggregator
  -> audit_logger / reporting adapter
```

### 7.1 `control_receiver`

输入：

- 中心侧下发对象

输出：

- `ReceivedPlan`

它不做：

- 排队
- 状态落盘
- spawn

### 7.2 `plan_validator`

输入：

- `ReceivedPlan`

输出：

- `ValidatedPlan`
- 或 `RejectedPlan`

它不做：

- 入队
- 生成 execution workdir
- 拉起进程

### 7.3 `execution_scheduler`

输入：

- `ValidatedPlan`
- cancel request
- timeout tick
- process exit event
- result ready event

输出：

- queue 更新
- running 状态更新
- `SpawnRequest`
- `CancelRequest`

它是：

- 本地状态机拥有者
- queue/running 状态拥有者

### 7.4 `executor_manager`

输入：

- `SpawnRequest`
- `CancelRequest`

输出：

- `ProcessSpawned`
- `ProcessExited`
- `ProcessKillRequested`

它不做：

- 校验计划是否合法
- 决定队列顺序
- 解释 step 结果

### 7.5 `result_aggregator`

输入：

- `ProcessExited`
- workdir 路径

输出：

- `FinalExecutionResult`
- `ReportReady`

它拥有：

- reporting 状态写入权

它不做：

- 调度重试
- 排队
- 修改 running 集合

### 7.6 `audit_logger`

输入：

- 所有关键状态事件

输出：

- 本地审计记录
- 最近历史摘要

---

## 8. 事件驱动边界

建议第一版在进程内采用显式事件对象，而不是模块之间相互直接改状态。

建议最小事件集合：

- `PlanReceived`
- `PlanRejected`
- `PlanQueued`
- `SpawnRequested`
- `ProcessSpawned`
- `ProcessStarted`
- `CancelRequested`
- `ProcessExited`
- `ResultReady`
- `ReportSucceeded`
- `ReportFailed`

这样做的好处是：

- 更容易做单测
- 更容易做 crash 恢复
- 更容易限制模块越权写状态

---

## 9. 本地状态机细化

建议把 `agentd-architecture.md` 中的状态机进一步固定为：

```text
received
  -> validating
  -> rejected | queued

queued
  -> dispatching_local
  -> cancelled

dispatching_local
  -> running
  -> failed

running
  -> succeeded | failed | cancelled | timed_out

succeeded | failed | cancelled | timed_out
  -> reporting
  -> done
```

### 9.1 状态拥有者

- `received` / `validating` / `rejected`：
  `plan_validator`
- `queued` / `dispatching_local` / `running` / `cancelling`：
  `execution_scheduler`
- `succeeded` / `failed` / `cancelled` / `timed_out`：
  `result_aggregator` 形成最终判定，`execution_scheduler` 接收并更新运行态
- `reporting` / `done`：
  `result_aggregator`

### 9.2 单 execution 不变量

第一版建议固定以下不变量：

- 一个 `execution_id` 只能出现在一个 queue item 中
- 一个 `execution_id` 同时只能对应一个 running 文件
- 一个 `execution_id` 最终只能产生一个 `result.json`
- 一个 `execution_id` 进入 `done` 后不能重新回到 `running`

---

## 10. crash 恢复最小算法

`wp-agentd` 启动时至少执行以下恢复步骤：

1. 读取 `execution_queue.json`
2. 扫描 `state/running/*.json`
3. 扫描 `state/reporting/*.json`
4. 扫描 `run/actions/*`
5. 对每个 running execution 检查：
   - 对应 pid 是否仍存活
   - workdir 是否存在
   - `result.json` 是否已存在
6. 形成恢复结论：
   - 若结果已存在，转入 reporting
   - 若进程不存在且无结果，标记为 `failed` 或 `unknown`
   - 若进程仍存在，重新纳入 running 监控

第一版不要求复杂断点续跑，但必须做到：

- 不重复 spawn 同一 execution
- 不丢失已完成结果
- 不把孤儿执行永久留在 running 状态

---

## 11. 并发与互斥边界

建议 `execution_scheduler` 持有以下调度约束：

- `max_running_actions`
- `upgrade_mutex`
- `high_risk_action_mutex`

第一版建议最保守策略：

- `max_running_actions = 1`
- upgrade 与 action 全互斥

这样可以显著降低本地状态复杂度。

---

## 12. 模块实现建议

第一版建议把模块实现分成三类：

### 12.1 纯状态模块

- `state_store`
- `audit_logger`

特点：

- 只做状态读写和归档

### 12.2 纯决策模块

- `plan_validator`
- `execution_scheduler`

特点：

- 不直接操作外部进程
- 输入对象，输出决策

### 12.3 外部副作用模块

- `control_receiver`
- `executor_manager`
- `result_aggregator`

特点：

- 负责 IO、进程、文件、上报

这样拆分后，测试会更容易做。

---

## 13. 当前决定

当前阶段固定以下结论：

- `wp-agentd` 的本地状态必须分层
- `execution_queue` / `running` / `reporting` / `history` 必须分开
- 每类状态必须有唯一写入模块
- 模块之间通过事件和返回对象协作，不通过共享写文件协作
- `execution_scheduler` 是本地状态机拥有者
- `executor_manager` 只管进程，不拥有调度状态

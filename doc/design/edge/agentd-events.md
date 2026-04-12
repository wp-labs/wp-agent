# wp-agentd 事件对象设计

## 1. 文档目的

本文档定义 `wp-agentd` 进程内模块协作所使用的最小事件对象集合。

目标是让模块之间通过显式事件协作，而不是共享抢写状态文件。

相关文档：

- [`agentd-state-and-boundaries.md`](agentd-state-and-boundaries.md)
- [`agentd-state-schema.md`](agentd-state-schema.md)
- [`agentd-architecture.md`](agentd-architecture.md)

---

## 2. 核心结论

第一版建议 `wp-agentd` 使用“对象 + 事件”边界：

- 模块输入是结构化对象
- 模块输出是结构化事件
- 状态文件只由拥有者模块更新

第一版不要求引入复杂 event bus。

只要满足：

- 事件类型固定
- 事件字段固定
- 事件边界清楚

就足以支撑骨架实现。

---

## 3. 事件分层

建议把事件分成四类：

- 接收类
- 校验类
- 调度类
- 结果类

---

## 4. 接收类事件

### 4.1 `PlanReceived`

表示 `control_receiver` 成功接收并解析了一个 `ActionPlan`。

建议字段：

- `event_type = "PlanReceived"`
- `received_at`
- `execution_id`
- `action_id`
- `request_id`
- `plan_ref`

### 4.2 `PlanReceiveFailed`

表示接收或反序列化失败。

建议字段：

- `event_type = "PlanReceiveFailed"`
- `received_at`
- `reason_code`
- `detail?`

---

## 5. 校验类事件

### 5.1 `PlanValidated`

表示 `plan_validator` 通过本地校验。

建议字段：

- `event_type = "PlanValidated"`
- `validated_at`
- `execution_id`
- `action_id`
- `request_id`
- `deadline_at`
- `priority`

### 5.2 `PlanRejected`

表示本地校验失败。

建议字段：

- `event_type = "PlanRejected"`
- `validated_at`
- `execution_id?`
- `action_id?`
- `request_id?`
- `reason_code`
- `detail?`

---

## 6. 调度类事件

### 6.1 `PlanQueued`

表示 execution 已进入 `execution_queue`。

建议字段：

- `event_type = "PlanQueued"`
- `queued_at`
- `execution_id`
- `action_id`
- `request_id`
- `priority`

### 6.2 `SpawnRequested`

表示 `execution_scheduler` 决定启动本地执行。

建议字段：

- `event_type = "SpawnRequested"`
- `requested_at`
- `execution_id`
- `action_id`
- `request_id`
- `workdir`
- `deadline_at`

### 6.3 `ProcessSpawned`

表示 `executor_manager` 已成功拉起 `wp-agent-exec`。

建议字段：

- `event_type = "ProcessSpawned"`
- `spawned_at`
- `execution_id`
- `action_id`
- `pid`
- `workdir`

### 6.4 `CancelRequested`

表示本地对运行中 execution 发起取消。

建议字段：

- `event_type = "CancelRequested"`
- `requested_at`
- `execution_id`
- `action_id`
- `reason_code`

### 6.5 `ProcessExited`

表示 `wp-agent-exec` 已退出。

建议字段：

- `event_type = "ProcessExited"`
- `exited_at`
- `execution_id`
- `action_id`
- `pid`
- `exit_code?`
- `signal?`
- `workdir`

---

## 7. 结果类事件

### 7.1 `ResultReady`

表示 `result_aggregator` 已确认 `result.json` 可消费。

建议字段：

- `event_type = "ResultReady"`
- `ready_at`
- `execution_id`
- `action_id`
- `request_id`
- `final_state`
- `result_path`

### 7.2 `ReportSucceeded`

表示结果已成功回传中心。

建议字段：

- `event_type = "ReportSucceeded"`
- `reported_at`
- `execution_id`
- `action_id`
- `request_id`

### 7.3 `ReportFailed`

表示结果回传失败。

建议字段：

- `event_type = "ReportFailed"`
- `reported_at`
- `execution_id`
- `action_id`
- `request_id`
- `reason_code`
- `detail?`

---

## 8. 最小事件流

建议最小正常流如下：

```text
PlanReceived
  -> PlanValidated
  -> PlanQueued
  -> SpawnRequested
  -> ProcessSpawned
  -> ProcessExited
  -> ResultReady
  -> ReportSucceeded
```

校验拒绝流：

```text
PlanReceived
  -> PlanRejected
  -> ResultReady
  -> ReportSucceeded
```

取消流：

```text
PlanReceived
  -> PlanValidated
  -> PlanQueued
  -> SpawnRequested
  -> ProcessSpawned
  -> CancelRequested
  -> ProcessExited
  -> ResultReady
  -> ReportSucceeded
```

---

## 9. 第一版边界

第一版建议：

- 事件对象在进程内传递即可
- 不要求持久化所有事件
- 审计类事件由 `audit_logger` 选择性落盘

但以下事件建议必须进入审计链：

- `PlanRejected`
- `SpawnRequested`
- `CancelRequested`
- `ProcessExited`
- `ReportFailed`

---

## 10. 当前决定

当前阶段固定以下结论：

- 模块协作优先通过事件对象完成
- 状态文件更新只由拥有者模块完成
- `execution_scheduler`、`executor_manager`、`result_aggregator` 的边界通过事件明确隔开

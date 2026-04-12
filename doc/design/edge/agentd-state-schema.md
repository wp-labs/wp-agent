# wp-agentd 本地状态 Schema 草案

## 1. 文档目的

本文档把 [`agentd-state-and-boundaries.md`](agentd-state-and-boundaries.md) 中的本地状态模型进一步收敛成字段级 schema 草案。

目标是让 `M3 Edge Runtime Skeleton` 可以直接围绕这些本地状态对象实现：

- `agent_runtime.json`
- `execution_queue.json`
- `running/<execution_id>.json`
- `reporting/<execution_id>.json`
- `history/recent.json`

相关文档：

- [`agentd-state-and-boundaries.md`](agentd-state-and-boundaries.md)
- [`agentd-architecture.md`](agentd-architecture.md)
- [`agentd-exec-protocol.md`](agentd-exec-protocol.md)

---

## 2. 核心结论

第一版本地状态 schema 必须满足三点：

- 字段足够少，能直接支撑实现
- 字段稳定，避免和运行时内存对象耦合
- 状态文件之间职责不重叠

因此第一版建议：

- 每个状态文件只保留本层最小必要字段
- 通过 `execution_id` / `action_id` 关联
- 不在多个文件重复保存大对象

---

## 3. 通用约定

### 3.1 时间字段

统一使用：

- RFC3339 UTC 时间字符串

例如：

- `2026-04-12T10:00:00Z`

### 3.2 标识字段

建议固定：

- `agent_id`
- `instance_id`
- `execution_id`
- `action_id`
- `request_id`

### 3.3 文件更新策略

所有状态文件建议：

- 先写临时文件
- `fsync`
- 原子 `rename`

### 3.4 schema 版本

每个状态文件建议都有：

- `schema_version`

第一版固定：

- `v1alpha1`

---

## 4. `agent_runtime.json`

### 4.1 作用

保存 `wp-agentd` 自身运行态。

### 4.2 建议字段

```text
AgentRuntimeState {
  schema_version
  agent_id
  instance_id
  boot_id
  version
  config_version
  policy_version?
  mode
  started_at
  updated_at
}
```

### 4.3 字段说明

- `mode`
  建议枚举：
  - `normal`
  - `degraded`
  - `protect`
  - `upgrade_in_progress`

---

## 5. `execution_queue.json`

### 5.1 作用

保存等待本地调度的 execution 队列。

### 5.2 建议字段

```text
ExecutionQueueState {
  schema_version
  updated_at
  items[]
}
```

```text
ExecutionQueueItem {
  execution_id
  action_id
  plan_digest
  request_id
  priority
  queued_at
  deadline_at
  cancelable
  risk_level?
}
```

### 5.3 字段说明

- `priority`
  第一版建议整数，值越小优先级越高
- `cancelable`
  表示在进入本地运行前是否可被取消
- `plan_digest`
  用于 crash 恢复后的执行语义去重

### 5.4 第一版限制

第一版不建议在 `execution_queue.json` 中存：

- 完整 `ActionPlan`
- 完整 `constraints`
- 大段错误文本

这些内容应通过 `workdir` 文件或审计记录关联。

---

## 6. `running/<execution_id>.json`

### 6.1 作用

保存运行中 execution 的本地控制状态。

### 6.2 建议字段

```text
RunningExecutionState {
  schema_version
  execution_id
  action_id
  plan_digest
  request_id
  state
  workdir
  pid?
  started_at?
  deadline_at
  current_step_id?
  attempt?
  cancel_requested_at?
  kill_requested_at?
  updated_at
}
```

### 6.3 `state` 枚举

第一版建议：

- `validating`
- `queued`
- `dispatching_local`
- `running`
- `cancelling`

说明：

- `rejected` 不应长期保留在 `running` 目录
- `succeeded` / `failed` / `cancelled` / `timed_out` 形成最终结果后应转入 `reporting`

---

## 7. `reporting/<execution_id>.json`

### 7.1 作用

保存本地已形成最终结果、等待或进行上报的 execution 状态。

### 7.2 建议字段

```text
ReportingExecutionState {
  schema_version
  execution_id
  action_id
  plan_digest
  request_id
  final_state
  result_path
  result_digest?
  result_signature?
  report_attempt
  first_report_at?
  last_report_at?
  last_report_error?
  updated_at
}
```

### 7.3 `final_state` 枚举

第一版建议：

- `rejected`
- `succeeded`
- `failed`
- `cancelled`
- `timed_out`

---

## 8. `history/recent.json`

### 8.1 作用

保存最近执行摘要，用于本地排障和最小审计索引。

### 8.2 建议字段

```text
RecentHistoryState {
  schema_version
  updated_at
  items[]
}
```

```text
RecentHistoryItem {
  execution_id
  action_id
  plan_digest?
  request_id
  final_state
  started_at?
  finished_at?
  reason_code?
  summary?
}
```

### 8.3 第一版建议

- 本地只保留最近 N 条
- N 作为配置项

---

## 9. 与 workdir 文件的关系

本地状态文件不应替代 workdir 协议文件。

建议关系如下：

- `running/*.json`
  保存 scheduler / controller 视角的运行状态
- `run/actions/<execution_id>/state.json`
  保存 `wp-agent-exec` 视角的执行状态
- `reporting/*.json`
  保存上报流程状态
- `run/actions/<execution_id>/result.json`
  保存最终 `ActionResult`

也就是说：

- `wp-agentd` 自身状态文件和 `wp-agent-exec` workdir 文件是两层不同状态
- 不能混成一份

---

## 10. 最小示例

### 10.1 `execution_queue.json`

```json
{
  "schema_version": "v1alpha1",
  "updated_at": "2026-04-12T10:00:00Z",
  "items": [
    {
      "execution_id": "exec_01",
      "action_id": "act_01",
      "plan_digest": "sha256:abc123",
      "request_id": "req_01",
      "priority": 100,
      "queued_at": "2026-04-12T10:00:00Z",
      "deadline_at": "2026-04-12T10:05:00Z",
      "cancelable": true,
      "risk_level": "R0"
    }
  ]
}
```

### 10.2 `running/exec_01.json`

```json
{
  "schema_version": "v1alpha1",
  "execution_id": "exec_01",
  "action_id": "act_01",
  "plan_digest": "sha256:abc123",
  "request_id": "req_01",
  "state": "running",
  "workdir": "/var/lib/wp-agent/run/actions/exec_01",
  "pid": 38122,
  "started_at": "2026-04-12T10:00:01Z",
  "deadline_at": "2026-04-12T10:05:00Z",
  "current_step_id": "s2",
  "attempt": 1,
  "updated_at": "2026-04-12T10:00:02Z"
}
```

---

## 11. 当前决定

当前阶段固定以下结论：

- `execution_queue.json` 只保存待调度 execution 摘要
- `running/*.json` 只保存运行中的 controller 视角状态
- `reporting/*.json` 只保存结果回传状态
- 最终结果仍以 `workdir/result.json` 为准

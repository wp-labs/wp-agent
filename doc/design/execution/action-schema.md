# warp-insight 动作 Schema 设计

## 1. 文档目的

本文档把 [`action-dsl.md`](action-dsl.md) 中的 opcode 设计示例进一步收敛为可实现 schema。

目标是明确：

- 每个 opcode 的参数结构
- 每个 opcode 的返回结构
- 默认风险等级
- 默认限制要求
- capability 名称
- 是否建议审批

本文档是执行面和控制平面之间的协议基础之一。

---

## 2. 通用约定

### 2.1 参数命名约定

建议所有参数遵循：

- 小写蛇形或小写点风格选其一，第一版建议统一用小写蛇形
- 显式命名参数
- 不使用位置参数

### 2.2 返回约定

每个步骤返回建议统一包含：

- `ok`
- `op`
- `duration_ms`
- `data`

其中：

- `ok` 表示步骤是否执行成功
- `op` 表示 opcode 名称
- `duration_ms` 表示步骤耗时
- `data` 表示该 opcode 的结构化结果

### 2.3 通用限制

所有 opcode 都应受以下限制中的部分或全部约束：

- `timeout`
- `concurrency`
- `max_stdout`
- `max_stderr`
- `max_memory`
- `allowed_paths`
- `allowed_services`

### 2.4 通用风险等级

建议第一版按以下口径固定：

- `R0`：
  只读诊断、只读读取
- `R1`：
  agent 自检、reload、upgrade prepare/verify
- `R2`：
  service restart
- `R3`：
  第一版暂不开放

---

## 3. 诊断类

### 3.1 `process.list`

- capability：
  `process.list`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 参数 schema：
  - `name?: string`
  - `user?: string`
  - `contains_cmdline?: string`
  - `limit?: uint32`
- 返回 schema：
  - `items: ProcessSummary[]`

`ProcessSummary`

- `pid: uint32`
- `ppid: uint32`
- `name: string`
- `user?: string`
- `cmdline?: string`
- `start_time?: string`

### 3.2 `process.stat`

- capability：
  `process.stat`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 参数 schema：
  - `pid?: uint32`
  - `name?: string`
- 返回 schema：
  - `pid: uint32`
  - `cpu_pct?: float`
  - `rss_bytes?: uint64`
  - `fd_count?: uint32`
  - `thread_count?: uint32`
  - `state?: string`

### 3.3 `socket.check`

- capability：
  `socket.check`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 参数 schema：
  - `port: uint16`
  - `protocol?: string`
  - `state?: string`
- 返回 schema：
  - `open: bool`
  - `protocol?: string`
  - `port: uint16`
  - `listeners?: SocketListener[]`

`SocketListener`

- `pid?: uint32`
- `address?: string`
- `state?: string`

### 3.4 `service.status`

- capability：
  `service.status`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 参数 schema：
  - `service: string`
  - `manager?: string`
- 返回 schema：
  - `name: string`
  - `state?: string`
  - `substate?: string`
  - `enabled?: bool`
  - `pid?: uint32`

---

## 4. 读取类

### 4.1 `file.tail`

- capability：
  `file.tail`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 额外控制要求：
  必须受 `allowed_paths` 约束
- 参数 schema：
  - `path: string`
  - `lines?: uint32`
  - `max_bytes?: uint64`
- 返回 schema：
  - `path: string`
  - `lines: string[]`
  - `truncated: bool`

### 4.2 `file.read_range`

- capability：
  `file.read_range`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 额外控制要求：
  必须受 `allowed_paths` 约束
- 参数 schema：
  - `path: string`
  - `offset: uint64`
  - `length: uint64`
- 返回 schema：
  - `path: string`
  - `offset: uint64`
  - `length: uint64`
  - `content: string`
  - `truncated: bool`

### 4.3 `config.inspect`

- capability：
  `config.inspect`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 额外控制要求：
  必须受 `allowed_paths` 约束
- 参数 schema：
  - `path: string`
  - `format: string`
  - `selectors: string[]`
- 返回 schema：
  - `matched: map<string, string>`

---

## 5. Agent 控制类

### 5.1 `agent.reload`

- capability：
  `agent.reload`
- 默认风险：
  `R1`
- 默认审批：
  `role("ops")`
- 参数 schema：
  - `scope: string`
- 返回 schema：
  - `reloaded: bool`
  - `version_before?: string`
  - `version_after?: string`

### 5.2 `agent.health_check`

- capability：
  `agent.health_check`
- 默认风险：
  `R0`
- 默认审批：
  `not_required`
- 参数 schema：
  - `deep?: bool`
  - `include_buffer?: bool`
  - `include_exporter?: bool`
- 返回 schema：
  - `healthy: bool`
  - `summary?: string`
  - `checks?: HealthCheckResult[]`

`HealthCheckResult`

- `name: string`
- `ok: bool`
- `message?: string`

---

## 6. 服务控制类

### 6.1 `service.restart`

- capability：
  `service.restart`
- 默认风险：
  `R2`
- 默认审批：
  `required("team-lead")`
- 额外控制要求：
  必须受 `allowed_services` 约束
- 参数 schema：
  - `service: string`
  - `manager?: string`
  - `graceful?: bool`
  - `wait_ready?: bool`
- 返回 schema：
  - `restarted: bool`
  - `old_pid?: uint32`
  - `new_pid?: uint32`
  - `ready?: bool`

### 6.2 `service.reload`

- capability：
  `service.reload`
- 默认风险：
  `R1`
- 默认审批：
  `role("ops")`
- 额外控制要求：
  必须受 `allowed_services` 约束
- 参数 schema：
  - `service: string`
  - `manager?: string`
- 返回 schema：
  - `reloaded: bool`
  - `ready?: bool`

---

## 7. 升级辅助类

### 7.1 `upgrade.prepare`

- capability：
  `upgrade.prepare`
- 默认风险：
  `R1`
- 默认审批：
  `role("release")`
- 参数 schema：
  - `version?: string`
  - `channel?: string`
  - `artifact_ref?: string`
- 返回 schema：
  - `prepared: bool`
  - `package_verified: bool`
  - `compatible: bool`
  - `disk_ok?: bool`

### 7.2 `upgrade.verify`

- capability：
  `upgrade.verify`
- 默认风险：
  `R1`
- 默认审批：
  `role("release")`
- 参数 schema：
  - `expected_version?: string`
  - `include_buffer?: bool`
  - `include_exporter?: bool`
- 返回 schema：
  - `version_ok: bool`
  - `healthy: bool`
  - `checks?: HealthCheckResult[]`

---

## 8. 统一结果封装

建议每个步骤最终都返回如下统一结构：

```json
{
  "ok": true,
  "op": "process.list",
  "duration_ms": 12,
  "data": {
    "items": [
      { "pid": 123, "name": "nginx" }
    ]
  }
}
```

这样做的好处是：

- `expect` 逻辑统一
- 中心侧可统一归档
- `step_records` 结构稳定

---

## 9. 默认限制建议

建议第一版给每类 opcode 设默认上限：

- 诊断类：
  `timeout <= 5s`
- 读取类：
  `timeout <= 5s`
  `max_stdout <= 64kb`
- agent 控制类：
  `timeout <= 10s`
- 服务控制类：
  `timeout <= 30s`
  `concurrency <= 1`
- 升级辅助类：
  `timeout <= 60s`

这些只是默认值，实际生效仍应受 `control.wac` 和平台策略约束。

---

## 10. 与 `run.gxl` 的关系

`run.gxl` 中的每个 opcode 调用，最终都应能映射到本文件中的某个 schema。

如果出现：

- `run.gxl` 调用了不存在的 opcode
- 参数不符合 schema
- 返回不符合约定

则编译阶段必须失败。

---

## 11. 当前结论

`action-schema.md` 的作用是把远程动作从“示例设计”推进到“可实现协议”。

它与其他文档的关系是：

- [`control.wac`](action-dsl.md)
  决定动作是否被允许
- [`run.gxl`](run-gxl-subset.md)
  决定动作如何表达
- `action-schema.md`
  决定每个 opcode 的参数、返回、风险和 capability
- `ActionPlan IR`
  决定边缘最终执行输入

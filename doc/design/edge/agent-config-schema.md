# wp-agent Agent 配置 Schema 草案

## 1. 文档目的

本文档定义 `wp-agentd` 的本地总配置骨架。

目标是把当前分散的配置讨论收敛成一份统一结构，覆盖：

- daemon 基础配置
- control plane 连接
- 本地路径
- execution 限制
- logs inputs
- metrics integrations
- upgrade 配置

相关文档：

- [`metrics-config-schema.md`](../telemetry/metrics-config-schema.md)
- [`log-file-input-spec.md`](../telemetry/log-file-input-spec.md)
- [`agentd-architecture.md`](agentd-architecture.md)
- [`agentd-state-schema.md`](agentd-state-schema.md)

---

## 2. 顶层结构

```text
AgentConfig {
  schema_version
  agent
  control_plane
  paths
  execution
  resource_budget?
  buffering?
  protection?
  logs?
  metrics?
  upgrade?
}
```

第一版固定：

- `schema_version = "v1"`

---

## 3. `agent`

```text
AgentSection {
  agent_id?
  environment_id?
  instance_name?
}
```

说明：

- `agent_id` 可为空，由首次注册后固化

---

## 4. `control_plane`

```text
ControlPlaneSection {
  enabled
  endpoint?
  tls_mode?
  auth_mode?
}
```

第一版建议：

- `enabled = false` 表示 `standalone` 模式
- `enabled = true` 表示 `managed` 模式
- 当 `enabled = false` 时，`endpoint / tls_mode / auth_mode` 可为空
- 当 `enabled = true` 时，`endpoint` 为必填

也就是说：

- 是否连接中心节点，必须是显式配置
- 没有中心节点不是异常态，而是一种受支持运行模式

---

## 5. `paths`

```text
PathsSection {
  root_dir
  run_dir
  state_dir
  log_dir
}
```

这些路径要与：

- `agentd-exec-protocol.md`
- `agentd-state-schema.md`

保持一致。

---

## 6. `execution`

```text
ExecutionSection {
  max_running_actions
  cancel_grace_ms
  default_stdout_limit_bytes
  default_stderr_limit_bytes
}
```

第一版建议：

- `max_running_actions = 1`

---

## 7. `resource_budget`

```text
ResourceBudgetSection {
  idle_cpu_target_pct?
  moderate_cpu_target_pct?
  peak_cpu_limit_pct?
  idle_rss_target_bytes?
  moderate_rss_target_bytes?
  peak_rss_limit_bytes?
}
```

说明：

- 用于把 [`non-functional-targets.md`](../foundation/non-functional-targets.md) 中的量化资源目标映射到本地配置

---

## 8. `buffering`

```text
BufferingSection {
  telemetry_memory_queue_limit_bytes?
  telemetry_spool_limit_bytes?
  action_report_memory_queue_limit_bytes?
  action_report_spool_limit_bytes?
}
```

---

## 9. `protection`

```text
ProtectionSection {
  degraded_cpu_pct?
  protect_cpu_pct?
  degraded_rss_bytes?
  protect_rss_bytes?
  degraded_memory_queue_pct?
  protect_memory_queue_pct?
  degraded_spool_pct?
  protect_spool_pct?
}
```

说明：

- 用于描述 `normal / degraded / protect` 切换阈值

---

## 10. `logs`

```text
LogsSection {
  file_inputs[]?
}
```

第一版建议：

- 先固定 `logs.file_inputs[]`
- 其字段结构直接复用：
  - [`../telemetry/log-file-input-spec.md`](../telemetry/log-file-input-spec.md)
- 当前 schema 复用的是完整 `file input` 基线设计
- 但 `M4` 实现只要求其中受控子集，用于验证显式单路径 `standalone` 替代切片
- 通用 `path_patterns[]` / `exclude_path_patterns[]`、完整 watcher 策略和更完整 telemetry runtime 字段在后续 telemetry core 阶段补齐
- `syslog` / `journald` / 其他 logs receiver 后续再补

---

## 11. `metrics`

直接复用：

- [`metrics-config-schema.md`](../telemetry/metrics-config-schema.md)

即：

- `metrics.integrations[]`

---

## 12. `upgrade`

```text
UpgradeSection {
  enabled
  mutex_with_actions
}
```

第一版建议：

- `enabled = true`
- `mutex_with_actions = true`

---

## 13. 当前决定

当前阶段固定以下结论：

- `wp-agentd` 需要一份统一总配置
- logs 配置作为其子树挂入
- metrics 配置作为其子树挂入
- `resource_budget / buffering / protection` 需要承担非功能量化目标的配置落点
- execution / paths / control_plane 作为第一版必需配置段

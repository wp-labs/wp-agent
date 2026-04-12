# wp-agent Agent 配置 Schema 草案

## 1. 文档目的

本文档定义 `wp-agentd` 的本地总配置骨架。

目标是把当前分散的配置讨论收敛成一份统一结构，覆盖：

- daemon 基础配置
- control plane 连接
- 本地路径
- execution 限制
- metrics integrations
- upgrade 配置

相关文档：

- [`metrics-config-schema.md`](../telemetry/metrics-config-schema.md)
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
  metrics?
  upgrade?
}
```

第一版固定：

- `schema_version = "v1alpha1"`

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
  endpoint
  tls_mode
  auth_mode
}
```

第一版只要求最小连接信息。

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

## 10. `metrics`

直接复用：

- [`metrics-config-schema.md`](../telemetry/metrics-config-schema.md)

即：

- `metrics.integrations[]`

---

## 11. `upgrade`

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

## 12. 当前决定

当前阶段固定以下结论：

- `wp-agentd` 需要一份统一总配置
- metrics 配置作为其子树挂入
- `resource_budget / buffering / protection` 需要承担非功能量化目标的配置落点
- execution / paths / control_plane 作为第一版必需配置段

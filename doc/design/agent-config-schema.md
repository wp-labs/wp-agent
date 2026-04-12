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

- [`metrics-config-schema.md`](./metrics-config-schema.md)
- [`agentd-architecture.md`](./agentd-architecture.md)
- [`agentd-state-schema.md`](./agentd-state-schema.md)

---

## 2. 顶层结构

```text
AgentConfig {
  schema_version
  agent
  control_plane
  paths
  execution
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

## 7. `metrics`

直接复用：

- [`metrics-config-schema.md`](./metrics-config-schema.md)

即：

- `metrics.integrations[]`

---

## 8. `upgrade`

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

## 9. 当前决定

当前阶段固定以下结论：

- `wp-agentd` 需要一份统一总配置
- metrics 配置作为其子树挂入
- execution / paths / control_plane 作为第一版必需配置段

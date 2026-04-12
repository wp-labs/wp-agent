# wp-agent CapabilityReport Schema 草案

## 1. 文档目的

本文档定义 `CapabilityReport` 的字段级 schema，用于统一：

- `wp-agentd` 如何声明本机能力
- 控制平面如何筛选目标 agent
- `ActionPlan.constraints.required_capabilities` 如何在边缘校验

相关文档：

- [`control-plane.md`](../center/control-plane.md)
- [`action-plan-schema.md`](../execution/action-plan-schema.md)
- [`metrics-config-schema.md`](../telemetry/metrics-config-schema.md)

---

## 2. 核心结论

`CapabilityReport` 必须回答三件事：

- 本机能执行哪些 action opcode
- 本机能提供哪些 metrics/logs/traces/security 输入与 discovery 能力
- 本机有哪些资源边界和限制

第一版不追求复杂能力推理，只做显式声明和显式匹配。

---

## 3. 顶层结构

```text
CapabilityReport {
  schema_version
  agent_id
  instance_id
  reported_at
  exec
  metrics
  upgrade
  limits
}
```

第一版固定：

- `schema_version = "v1alpha1"`

---

## 4. `exec`

```text
ExecCapabilities {
  opcodes[]
  execution_profiles[]
}
```

字段说明：

- `opcodes`
  例如：
  - `process.list`
  - `service.status`
  - `file.read_range`
- `execution_profiles`
  例如：
  - `agent_exec_v1`

边缘校验规则：

- `ActionPlan.program.invoke.op` 必须全部包含在 `opcodes[]`
- `ActionPlan.constraints.execution_profile` 必须包含在 `execution_profiles[]`

---

## 5. `metrics`

```text
MetricsCapabilities {
  collectors[]
  scrapers[]
  receivers[]
  discovery_modes[]
}
```

字段说明：

- `collectors`
  例如：
  - `host_metrics`
  - `process_metrics`
  - `container_metrics`
- `scrapers`
  例如：
  - `prom_scrape`
  - `jmx_scrape`
- `receivers`
  例如：
  - `otlp_metrics_receiver`
  - `statsd_receiver`
- `discovery_modes`
  例如：
  - `local_runtime`
  - `static`
  - `file`
  - `k8s`

---

## 6. `upgrade`

```text
UpgradeCapabilities {
  supported
  features[]
}
```

字段说明：

- `supported`: `bool`
- `features`
  例如：
  - `prepare`
  - `verify`
  - `rollback`

---

## 7. `limits`

```text
CapabilityLimits {
  max_running_actions?
  max_stdout_bytes?
  max_stderr_bytes?
  max_memory_bytes?
  max_metrics_targets?
}
```

这些字段用于：

- 中心编译期筛选
- 边缘本地二次校验

---

## 8. 匹配规则

第一版建议固定如下：

1. 控制平面编译 `ActionPlan` 时先做 capability 预筛选
2. 边缘收到 `ActionPlan` 后再次做本机 capability 校验
3. 任一缺失都拒绝执行

匹配只做显式包含判断：

- required capability in reported capabilities

第一版不做：

- 子能力推导
- 模糊匹配
- 版本区间推理

---

## 9. 最小示例

```json
{
  "schema_version": "v1alpha1",
  "agent_id": "agent_prod_web_01",
  "instance_id": "inst_01",
  "reported_at": "2026-04-12T10:00:00Z",
  "exec": {
    "opcodes": ["file.read_range", "process.list", "service.status"],
    "execution_profiles": ["agent_exec_v1"]
  },
  "metrics": {
    "collectors": ["host_metrics", "process_metrics"],
    "scrapers": ["prom_scrape"],
    "receivers": ["otlp_metrics_receiver"],
    "discovery_modes": ["local_runtime", "static", "file", "k8s"]
  },
  "upgrade": {
    "supported": true,
    "features": ["prepare", "verify", "rollback"]
  },
  "limits": {
    "max_running_actions": 1,
    "max_stdout_bytes": 65536,
    "max_stderr_bytes": 65536
  }
}
```

---

## 10. 当前决定

当前阶段固定以下结论：

- capability 采用显式声明，不做隐式推导
- `exec` / `metrics` / `upgrade` 三类能力分开建模
- `required_capabilities` 的边缘校验走包含判断

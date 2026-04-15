# warp-insight Metrics 配置 Schema 草案

## 1. 文档目的

本文档定义 `warp-insightd` 数据面中 metrics integration 的统一配置骨架。

目标是先把公共配置模型固定下来，避免每个 integration 各写一套结构。

相关文档：

- [`metrics-integration-roadmap.md`](metrics-integration-roadmap.md)
- [`metrics-batch-a-plan.md`](metrics-batch-a-plan.md)
- [`architecture.md`](../foundation/architecture.md)

---

## 2. 核心结论

所有 metrics integration 都应尽量收敛到统一对象模型：

- `kind`
- `enabled`
- `discovery`
- `auth`
- `schedule`
- `normalize`
- `resource_mapping`
- `budget`

不同 integration 只在：

- `kind`
- `discovery`
- `auth`
- `schedule`

这几部分做差异化扩展。

---

## 3. 顶层骨架

建议统一骨架如下：

```text
metrics {
  integrations: [
    integration {
      id
      kind
      enabled
      discovery
      auth
      schedule
      normalize
      resource_mapping
      budget
    }
  ]
}
```

### 3.1 顶层字段

建议字段：

- `id`
- `kind`
- `enabled`
- `discovery`
- `auth`
- `schedule`
- `normalize`
- `resource_mapping`
- `budget`

约束：

- `id` 在本地 agent 配置内唯一
- `kind` 决定扩展字段集合
- `enabled = false` 时不得参与调度

---

## 4. 通用字段定义

### 4.1 `id`

用途：

- 本地唯一 integration 标识

建议类型：

- `string`

示例：

- `host-default`
- `prom-k8s-pods`
- `otlp-app-metrics`

### 4.2 `kind`

第一版建议枚举：

- `host_metrics`
- `process_metrics`
- `container_metrics`
- `k8s_node_pod_metrics`
- `prom_scrape`
- `otlp_metrics_receiver`

### 4.3 `enabled`

建议类型：

- `bool`

---

## 5. `discovery`

### 5.1 通用结构

建议骨架：

```text
discovery {
  mode
  static_targets?
  file_targets?
  k8s?
  selectors?
}
```

### 5.2 `mode`

建议枚举：

- `none`
- `static`
- `file`
- `k8s`
- `local_runtime`

解释：

- `none`
  不需要外部 target discovery
- `local_runtime`
  从本机运行时读取，如 host/process/container

### 5.3 `static_targets`

建议结构：

```text
static_targets: [
  {
    endpoint
    labels?
  }
]
```

### 5.4 `file_targets`

建议结构：

```text
file_targets {
  path
  refresh_interval_ms
}
```

### 5.5 `k8s`

建议结构：

```text
k8s {
  role
  namespaces?
  label_selectors?
  field_selectors?
}
```

第一版 `role` 建议枚举：

- `node`
- `pod`
- `service`
- `endpoints`

---

## 6. `auth`

### 6.1 通用结构

建议骨架：

```text
auth {
  mode
  basic?
  bearer?
  tls?
  db?
}
```

### 6.2 `mode`

建议枚举：

- `none`
- `basic`
- `bearer`
- `tls`
- `db_credentials`

### 6.3 第一版要求

- 不把明文秘密直接写死进长期配置作为唯一方案
- 应支持引用本地 secret store 或环境注入

---

## 7. `schedule`

### 7.1 通用结构

建议骨架：

```text
schedule {
  interval_ms?
  timeout_ms?
  jitter_ms?
  startup_delay_ms?
}
```

说明：

- pull 型 integration 通常有 `interval_ms`
- receiver 型 integration 可以没有 `interval_ms`

### 7.2 约束

- `timeout_ms` 必须小于或等于 `interval_ms`
- 所有数值必须受 `budget` 和全局策略上限约束

---

## 8. `normalize`

### 8.1 通用结构

建议骨架：

```text
normalize {
  metric_prefix?
  rename_rules?
  drop_labels?
  keep_labels?
  unit_overrides?
}
```

### 8.2 第一版目标

- 支持有限 rename
- 支持有限 label drop / keep
- 支持 unit 修正

第一版不建议：

- 复杂脚本化转换
- 任意表达式执行

---

## 9. `resource_mapping`

### 9.1 通用结构

建议骨架：

```text
resource_mapping {
  resource_kind
  static_attributes?
  copy_from_target_labels?
  copy_from_runtime_facts?
}
```

### 9.2 `resource_kind`

第一版建议枚举：

- `host`
- `process`
- `container`
- `k8s_pod`
- `service`

### 9.3 第一版要求

必须至少保证：

- host 指标可稳定绑定 host 资源
- pod / container 指标可稳定绑定 k8s / container 资源
- scrape target 可稳定映射到 service 或 pod

---

## 10. `budget`

### 10.1 通用结构

建议骨架：

```text
budget {
  max_concurrent_targets?
  max_samples?
  max_labels_per_series?
  max_label_value_len?
  cpu_budget_pct?
  memory_budget_bytes?
}
```

### 10.2 第一版硬约束

每个 integration 至少要有：

- `timeout_ms`
- `max_samples`
- `max_concurrent_targets`

Prometheus/OpenMetrics scrape 还必须有：

- `max_labels_per_series`
- `max_label_value_len`

---

## 11. Batch A 示例

### 11.1 `host_metrics`

```text
integration {
  id = "host-default"
  kind = "host_metrics"
  enabled = true
  discovery {
    mode = "local_runtime"
  }
  auth {
    mode = "none"
  }
  schedule {
    interval_ms = 15000
    timeout_ms = 3000
  }
  resource_mapping {
    resource_kind = "host"
  }
  budget {
    max_samples = 500
    max_concurrent_targets = 1
  }
}
```

### 11.2 `prom_scrape`

```text
integration {
  id = "prom-k8s-pods"
  kind = "prom_scrape"
  enabled = true
  discovery {
    mode = "k8s"
    k8s {
      role = "pod"
    }
  }
  auth {
    mode = "bearer"
  }
  schedule {
    interval_ms = 30000
    timeout_ms = 5000
  }
  normalize {
    drop_labels = ["pod_annotation_hash"]
  }
  resource_mapping {
    resource_kind = "k8s_pod"
  }
  budget {
    max_samples = 5000
    max_concurrent_targets = 20
    max_labels_per_series = 30
    max_label_value_len = 256
  }
}
```

---

## 12. 第一版限制

第一版不建议在 schema 中支持：

- 任意脚本
- 任意模板函数
- 任意表达式计算
- 动态下载外部配置

配置必须保持：

- 可静态校验
- 可审计
- 可被中心统一治理

---

## 13. 当前决定

当前阶段固定以下结论：

- 所有 metrics integration 尽量收敛到统一骨架
- `Batch A` 先覆盖 `host_metrics`、`process_metrics`、`container_metrics`、`k8s_node_pod_metrics`、`prom_scrape`、`otlp_metrics_receiver`
- 复杂脚本化配置不进入第一版

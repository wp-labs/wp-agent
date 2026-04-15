# warp-insight Metrics Discovery 与 Resource Mapping 设计

## 1. 文档目的

本文档定义 metrics 数据面的两个核心公共能力：

- target discovery
- resource mapping

相关文档：

- [`metrics-config-schema.md`](metrics-config-schema.md)
- [`metrics-batch-a-plan.md`](metrics-batch-a-plan.md)
- [`metrics-batch-a-specs.md`](metrics-batch-a-specs.md)

---

## 2. 核心结论

metrics integration 能否规模化，关键不在单个 collector，而在：

- target 能否被稳定发现
- 同一个 target 能否被稳定绑定到统一 resource

第一版建议：

- discovery 做成公共框架
- resource mapping 做成公共规则层
- 各 integration 只提供少量差异化字段

---

## 3. Discovery 统一模型

建议统一成：

```text
DiscoveredTarget {
  target_id
  kind
  endpoint?
  labels
  runtime_facts
  discovered_at
}
```

### 3.1 `kind`

第一版建议枚举：

- `host`
- `process`
- `container`
- `k8s_node`
- `k8s_pod`
- `service_endpoint`

### 3.2 `labels`

表示 discovery 过程得到的原始标签。

### 3.3 `runtime_facts`

表示本机运行时事实，例如：

- pid
- container id
- pod uid
- node name

---

## 4. Discovery 模式

第一版建议支持：

- `local_runtime`
- `static`
- `file`
- `k8s`

### 4.1 `local_runtime`

适用：

- host
- process
- container runtime

### 4.2 `static`

适用：

- 固定 scrape target

### 4.3 `file`

适用：

- 外部生成的 target 文件

### 4.4 `k8s`

适用：

- pod
- service
- endpoints
- node

---

## 5. Discovery 刷新与去重

第一版建议：

- 定期刷新
- target_id 稳定
- 去重逻辑固定

### 5.1 target_id 生成建议

- host: `host.id`
- process: `host.id + pid`
- container: `container.id`
- k8s pod: `pod.uid`
- service endpoint: `scheme + host + port + path`

### 5.2 去重规则

同一轮 discovery 中：

- `target_id` 相同则视为同一 target

跨轮 refresh：

- target_id 仍存在则更新 metadata
- target_id 消失则标记失效

---

## 6. Resource Mapping 统一模型

建议统一成：

```text
MappedResource {
  resource_kind
  attributes
}
```

### 6.1 `resource_kind`

第一版建议枚举：

- `host`
- `process`
- `container`
- `k8s_pod`
- `service`

### 6.2 `attributes`

统一采用 OTel 风格属性键。

例如：

- `host.name`
- `host.id`
- `process.pid`
- `container.id`
- `k8s.namespace.name`
- `k8s.pod.name`
- `service.name`

---

## 7. Mapping 优先级

第一版建议按以下优先级绑定 resource：

1. runtime stable id
2. discovered labels 中的稳定标识
3. 配置中的静态映射
4. fallback 推断

禁止：

- 直接用高基数字段作为主 resource key
- 用易变化字段生成 resource identity

---

## 8. 高基数控制

第一版必须控制：

- 全量 cmdline
- 全量 annotations
- 全量动态 query labels
- image digest 泛化扩散

原则：

- 高基数信息可进入调试或诊断链路
- 不默认进入 metrics resource attrs 或 labels

---

## 9. Batch A 最小规则

### 9.1 `host_metrics`

- 直接映射到 `host`

### 9.2 `process_metrics`

- 映射到 `process`
- 资源键优先：
  - `host.id`
  - `process.pid`

### 9.3 `prom_scrape`

- 映射到 `service` 或 `k8s_pod`
- 先用 discovery labels
- 再用静态配置覆盖

### 9.4 `otlp_metrics_receiver`

- 优先保留上游 resource attrs
- 本地只做受控补充，不盲目覆盖

---

## 10. 当前决定

当前阶段固定以下结论：

- discovery 和 resource mapping 是公共框架能力
- `target_id` 必须稳定
- resource attrs 优先对齐 OTel 风格
- 高基数信息默认不进入统一 metrics 主路径

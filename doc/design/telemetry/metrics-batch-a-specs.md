# wp-agent Metrics Batch A 规格草案

## 1. 文档目的

本文档对 `Batch A` 的四个核心 integration 先给出规格草案：

- `host_metrics`
- `process_metrics`
- `prom_scrape`
- `otlp_metrics_receiver`

这四个 integration 足以支撑第一批 `wp-agentd` 数据面框架落地。

相关文档：

- [`metrics-batch-a-plan.md`](metrics-batch-a-plan.md)
- [`metrics-config-schema.md`](metrics-config-schema.md)
- [`architecture.md`](../foundation/architecture.md)
- [`container-metrics-spec.md`](container-metrics-spec.md)
- [`k8s-node-pod-metrics-spec.md`](k8s-node-pod-metrics-spec.md)

---

## 2. `host_metrics`

### 2.1 目标

提供主机基础资源指标。

### 2.2 discovery

- `mode = local_runtime`

### 2.3 最小指标集

- CPU usage / utilization / load
- memory total / used / available / utilization
- filesystem total / used / available / utilization
- disk io read/write bytes + ops
- network rx/tx bytes + packets + errors

### 2.4 最小资源属性

- `host.id`
- `host.name`
- `os.type`
- `os.version`
- `host.arch`

### 2.5 默认 budget 建议

- `interval_ms = 15000`
- `timeout_ms = 3000`
- `max_samples = 500`

---

## 3. `process_metrics`

### 3.1 目标

提供受控的进程级基础指标。

### 3.2 discovery

- `mode = local_runtime`

### 3.3 最小发现字段

- `pid`
- `ppid`
- `name`
- `exe`
- `user`

### 3.4 最小指标集

- process count
- cpu usage
- memory rss
- thread count
- open fd count
- start time

### 3.5 label 限制

默认不暴露：

- 全量 cmdline
- env vars

### 3.6 默认 budget 建议

- `interval_ms = 15000`
- `timeout_ms = 3000`
- `max_samples = 2000`

---

## 4. `prom_scrape`

### 4.1 目标

抓取标准 Prometheus / OpenMetrics endpoint。

### 4.2 支持的 discovery

第一版建议支持：

- `static`
- `file`
- `k8s`

### 4.3 最小协议能力

- HTTP / HTTPS
- `path`
- `query`
- `headers`
- basic auth
- bearer token
- tls

### 4.4 最小 normalize 能力

- metric rename
- unit normalize
- drop labels
- keep labels
- target labels -> resource attrs

### 4.5 默认 budget 建议

- `interval_ms = 30000`
- `timeout_ms = 5000`
- `max_samples = 5000`
- `max_concurrent_targets = 20`
- `max_labels_per_series = 30`
- `max_label_value_len = 256`

### 4.6 第一版重点

- 抓得稳
- labels 可控
- resource 绑定稳定

不是第一版重点：

- 完整复刻 Prometheus 全能力

---

## 5. `otlp_metrics_receiver`

### 5.1 目标

接收已经支持 OTel 的应用或 SDK 的 metrics。

### 5.2 最小协议能力

- OTLP/gRPC
- OTLP/HTTP
- resource attributes 保留
- metric temporality 基础识别

### 5.3 最小运行要求

- 有输入上限
- 有拒绝计数
- 有错误计数
- 有资源预算

### 5.4 默认 budget 建议

- `max_samples = 10000`
- `memory_budget_bytes` 受全局 agent budget 限制

### 5.5 第一版重点

- 稳定接入
- 稳定上送
- 稳定 resource model

---

## 6. 统一验收口径

上述四个 integration 第一版都应满足：

- 可被统一配置模型描述
- 可被统一调度
- 可被统一 budget 约束
- 可稳定绑定到统一 resource model
- 可输出统一 self-observability 指标

---

## 7. 相关专门规格

本文件只展开四个公共框架型 integration。

`Batch A` 中另外两个运行时集成已拆到专门文档：

- [`container-metrics-spec.md`](container-metrics-spec.md)
- [`k8s-node-pod-metrics-spec.md`](k8s-node-pod-metrics-spec.md)

后续再扩到：

- `statsd_receiver`
- `jmx_scrape`

---

## 8. 当前决定

当前阶段把下面六个 integration 视为 `Batch A` 的起步集：

- `host_metrics`
- `process_metrics`
- `container_metrics`
- `k8s_node_pod_metrics`
- `prom_scrape`
- `otlp_metrics_receiver`

其中：

- 本文档展开四个公共框架型 integration
- `container_metrics` 与 `k8s_node_pod_metrics` 由专门规格文档单独展开

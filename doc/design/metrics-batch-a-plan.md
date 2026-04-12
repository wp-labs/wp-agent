# wp-agent Metrics Batch A 计划

## 1. 文档目的

本文档把 [`metrics-integration-roadmap.md`](./metrics-integration-roadmap.md) 中的 `Batch A` 进一步细化成可开发输入。

重点是明确：

- 第一批 metrics integration 到底包括哪些 target
- 每类 target 的最小发现范围是什么
- 每类 target 的最小指标范围是什么
- 哪些字段必须优先对齐 OTel / Prometheus 常见语义

相关文档：

- [`metrics-integration-roadmap.md`](./metrics-integration-roadmap.md)
- [`architecture.md`](./architecture.md)
- [`target.md`](./target.md)

---

## 2. Batch A 范围

`Batch A` 固定覆盖以下范围：

- host 基础资源指标
- process 基础指标
- container runtime 基础指标
- Kubernetes node / pod 基础指标
- Prometheus / OpenMetrics scrape
- OTLP metrics receiver

这一批的目标不是“把所有指标都做全”，而是：

- 建立 metrics collection framework
- 建立统一 discovery / scrape / normalize / resource binding 模型
- 先覆盖平台底盘级最常用指标

---

## 3. Batch A 设计原则

### 3.1 先做基础指标，不追求全量深度

第一批优先：

- 容量指标
- 利用率指标
- 可用性指标
- 基础错误指标

不优先：

- 过深的内核细项
- 低复用的长尾统计
- 高成本诊断型高级指标

### 3.2 优先稳定语义

第一批应优先保证：

- 指标名稳定
- unit 稳定
- resource 绑定稳定
- labels 不失控

### 3.3 labels 必须受控

第一批不应放开高基数 label。

尤其要控制：

- process cmdline 全量展开
- container image digest 全量扩散
- pod annotation 全量带入
- 动态 query param 型 label

---

## 4. Host 基础资源指标

### 4.1 discovery

host 类指标不依赖外部 target discovery。

只需要识别：

- `host.id`
- `host.name`
- `os.type`
- `os.version`
- `host.arch`

### 4.2 最小指标范围

建议第一批至少包括：

- CPU
  - usage
  - utilization
  - load
- Memory
  - total
  - used
  - available
  - utilization
- Filesystem
  - total
  - used
  - available
  - utilization
- Disk IO
  - read bytes
  - write bytes
  - read ops
  - write ops
- Network
  - rx bytes
  - tx bytes
  - rx packets
  - tx packets
  - error count

### 4.3 资源绑定

全部 host 指标必须至少绑定：

- `host.name`
- `host.id`
- `os.type`
- `service.instance.id` 或等价 host runtime id

---

## 5. Process 基础指标

### 5.1 discovery

process discovery 第一批建议支持：

- pid
- ppid
- process name
- executable path
- owner user

### 5.2 最小指标范围

建议至少包括：

- process count
- cpu usage
- memory rss
- open fd count
- thread count
- start time

### 5.3 label 控制

第一批 process metrics 默认不带：

- 全量 cmdline
- environment variables

如需关联命令行，建议通过受控归一化字段或单独诊断链路处理。

---

## 6. Container Runtime 基础指标

### 6.1 discovery

第一批建议支持发现：

- container id
- container name
- image
- runtime kind
- pod/container 关联

### 6.2 最小指标范围

建议至少包括：

- cpu usage
- memory usage
- memory limit
- network rx/tx
- restart count
- running state

### 6.3 资源绑定

必须尽量绑定：

- `container.id`
- `container.name`
- `k8s.pod.name` 若存在
- `service.name` 若可推断

---

## 7. Kubernetes Node / Pod 基础指标

### 7.1 discovery

第一批建议支持：

- node
- namespace
- pod
- container
- owner workload

### 7.2 最小指标范围

Node：

- node ready state
- node allocatable cpu/memory
- node usage cpu/memory

Pod：

- pod running / pending / failed state
- pod restart count
- pod cpu / memory usage
- pod network rx/tx

### 7.3 label 控制

第一批默认不直接展开：

- 全量 pod labels
- 全量 annotations

建议只带受控关键维度：

- `k8s.namespace.name`
- `k8s.pod.name`
- `k8s.node.name`
- `k8s.deployment.name` 或 owner ref

---

## 8. Prometheus / OpenMetrics Scrape

### 8.1 目标

这是 `Batch A` 里替代大量 exporter 的核心能力。

### 8.2 discovery

第一批建议支持：

- static targets
- file-based targets
- Kubernetes service / pod discovery
- basic relabel

### 8.3 最小能力

建议第一批必须支持：

- HTTP / HTTPS
- path
- query params
- headers
- basic auth
- bearer token
- tls
- scrape interval
- scrape timeout

### 8.4 normalize

第一批建议至少支持：

- metric rename
- unit normalize
- target label 映射为 resource attributes
- 受控 label drop / keep

### 8.5 限制

第一批必须有：

- 单目标样本上限
- label 数量上限
- label value 长度上限
- scrape 超时上限

---

## 9. OTLP Metrics Receiver

### 9.1 目标

用于直接接收已支持 OTel 的应用和 SDK 指标。

### 9.2 最小能力

建议第一批支持：

- OTLP/gRPC
- OTLP/HTTP
- resource attributes 保留
- metric temporality 识别
- 基础校验与拒绝计数

### 9.3 第一批重点

重点不是做复杂转换，而是：

- 稳定接入
- 稳定资源绑定
- 稳定上送

---

## 10. Batch A 统一 budget 要求

第一批每类 integration 都必须支持：

- `interval`
- `timeout`
- `max_samples`
- `max_labels_per_series`
- `max_concurrent_targets`

并建议统一有：

- CPU budget
- memory budget
- backpressure / drop 策略

---

## 11. Batch A 统一配置骨架

建议第一批所有 integration 都向下收敛到类似结构：

```text
integration {
  kind
  enabled
  discovery
  auth
  schedule
  normalize
  resource_mapping
  budget
}
```

其中：

- `kind` 例如 `host_metrics`、`prom_scrape`、`otlp_metrics`
- `schedule` 定义 interval / timeout
- `budget` 定义采样上限、并发上限和资源预算

---

## 12. Batch A 验收标准

要认为 `Batch A` 可交付，至少应满足：

- `wp-agentd` 可稳定输出 host 基础指标
- `wp-agentd` 可稳定输出 process 基础指标
- `wp-agentd` 可稳定 scrape Prometheus / OpenMetrics target
- `wp-agentd` 可稳定接收 OTLP metrics
- 至少完成一种 container runtime 或 Kubernetes 基础指标接入
- 所有指标都能稳定绑定到统一 resource model

---

## 13. 后续衔接

`Batch A` 完成后，再进入：

- `Batch B`
  StatsD / JMX / nginx / mysql / postgresql / redis
- `Batch C`
  kafka / elasticsearch / rabbitmq / clickhouse / kube control plane

也就是说：

- `Batch A` 建框架
- `Batch B` 做高频服务
- `Batch C` 扩平台和中间件广度

---

## 14. 当前决定

当前阶段固定以下结论：

- `Batch A` 的目标是先把平台底盘与标准接口做稳
- 先不追求“所有指标一次做全”
- 先把 discovery、normalize、resource binding、budget 这四件事做稳定

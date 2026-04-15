# warp-insight `container_metrics` 规格草案

## 1. 文档目的

本文档定义 `container_metrics` integration 的第一版规格。

目标是让 `warp-insightd` 能直接采集常见 container runtime 的基础指标，而不是默认依赖额外 exporter。

相关文档：

- [`metrics-batch-a-plan.md`](metrics-batch-a-plan.md)
- [`metrics-config-schema.md`](metrics-config-schema.md)
- [`metrics-discovery-and-resource-mapping.md`](metrics-discovery-and-resource-mapping.md)

---

## 2. 目标

`container_metrics` 第一版重点解决三件事：

- 稳定发现本机 container runtime 中的存活容器
- 采集容器级基础资源指标
- 稳定映射到 `container` / `k8s_pod` 资源模型

第一版不是要做全量容器诊断平台。

---

## 3. Discovery

### 3.1 支持模式

- `mode = local_runtime`

### 3.2 第一版支持的 runtime

优先顺序建议：

1. `containerd`
2. `docker`
3. `cri-o`

### 3.3 最小发现字段

- `container.id`
- `container.name`
- `container.runtime`
- `container.image.name`
- `container.image.tag?`
- `host.id`
- `k8s.pod.uid?`
- `k8s.namespace.name?`
- `k8s.pod.name?`
- `k8s.container.name?`
- `pid?`
- `cgroup.path?`

### 3.4 target_id

建议固定为：

- `target_id = container.id`

---

## 4. 最小指标集

第一版建议至少包括：

- CPU
  - usage
  - utilization
  - throttled time 或 throttled count
- Memory
  - usage
  - working set
  - limit
- Filesystem
  - usage
  - read bytes
  - write bytes
- Network
  - rx bytes
  - tx bytes
  - rx packets
  - tx packets
- State
  - running state
  - restart count

说明：

- 第一版允许不同 runtime 在极少数指标上存在可用性差异
- 但 CPU / memory / network / state 四类基础能力必须统一可用

---

## 5. Resource Mapping

### 5.1 主资源

默认映射到：

- `container`

### 5.2 可选附加资源上下文

如能稳定关联，允许补充：

- `host`
- `k8s_pod`
- `service`

### 5.3 最小资源属性

- `container.id`
- `container.name`
- `container.runtime`
- `host.id`
- `host.name?`
- `k8s.namespace.name?`
- `k8s.pod.uid?`
- `k8s.pod.name?`
- `k8s.container.name?`

---

## 6. 高基数控制

第一版默认不进入主指标路径：

- 全量 image digest
- 全量 image repo digest 列表
- 全量 container labels
- 全量 env vars
- 全量 cmdline

如确需诊断链路，可后续通过单独调试输出补充。

---

## 7. 预算建议

第一版默认建议：

- `schedule.interval_ms = 15000`
- `schedule.timeout_ms = 3000`
- `budget.max_targets = 2000`
- `budget.max_samples = 10000`

### 7.1 退化建议

进入 `degraded` 后建议：

- 降低 container metadata refresh 频率
- 限制高成本 filesystem 细项
- 保留 CPU / memory / state 基础指标

进入 `protect` 后建议：

- 暂停可选 enrich
- 必要时只保留存活状态和核心资源指标

---

## 8. 第一版实现约束

第一版建议：

- 通过本机 runtime API 或 cgroup 事实采集
- 不通过远程 action 临时执行命令拿容器指标
- 不要求完整复刻 cAdvisor 全部指标面

---

## 9. 验收口径

第一版至少应满足：

- 容器新增 / 删除后 `2` 个 refresh 周期内收敛
- 同一容器的 `target_id` 稳定
- 指标与 `k8s_pod` 关联在可用时稳定，不可用时不乱绑
- 高基数字段默认不扩散进 labels / resource attrs

---

## 10. 当前决定

当前阶段固定以下结论：

- `container_metrics` 属于 `warp-insightd` 内建能力
- 第一版优先覆盖 `containerd` / `docker` / `cri-o`
- 先把容器基础资源指标与稳定资源绑定做好，再谈更深诊断指标

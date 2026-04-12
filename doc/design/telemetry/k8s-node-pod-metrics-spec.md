# wp-agent `k8s_node_pod_metrics` 规格草案

## 1. 文档目的

本文档定义 `k8s_node_pod_metrics` integration 的第一版规格。

目标是让部署在 Kubernetes 节点侧的 `wp-agentd` 直接提供 node / pod 基础指标与资源映射，而不是要求额外安装专用 exporter 作为默认前提。

相关文档：

- [`metrics-batch-a-plan.md`](metrics-batch-a-plan.md)
- [`metrics-config-schema.md`](metrics-config-schema.md)
- [`metrics-discovery-and-resource-mapping.md`](metrics-discovery-and-resource-mapping.md)
- [`container-metrics-spec.md`](container-metrics-spec.md)

---

## 2. 目标

`k8s_node_pod_metrics` 第一版重点解决四件事：

- 稳定发现本节点上的 node / pod / container 对象
- 提供 node 与 pod 级基础容量、利用率和状态指标
- 绑定稳定的 Kubernetes 资源属性
- 与 `container_metrics` 形成互补，而不是重复扩散

---

## 3. Discovery

### 3.1 支持模式

第一版建议组合使用：

- `mode = local_runtime`
- `mode = k8s`

### 3.2 推荐数据来源优先级

1. 本机 kubelet 可获得的本地统计接口
2. 本机 container runtime 事实
3. Kubernetes API metadata watch

### 3.3 最小发现字段

Node:

- `k8s.node.name`
- `k8s.node.uid?`
- `host.id`
- `host.name`

Pod:

- `k8s.namespace.name`
- `k8s.pod.uid`
- `k8s.pod.name`
- `k8s.node.name`
- `k8s.owner.kind?`
- `k8s.owner.name?`
- `k8s.workload.name?`
- `k8s.qos.class?`

Container 关联字段：

- `k8s.container.name`
- `container.id?`

### 3.4 target_id

建议固定：

- node: `k8s.node.name`
- pod: `k8s.pod.uid`

---

## 4. 最小指标集

### 4.1 Node

第一版建议至少包括：

- ready state
- allocatable cpu
- allocatable memory
- cpu usage
- memory usage
- filesystem usage
- network rx / tx
- pod capacity / current pod count

### 4.2 Pod

第一版建议至少包括：

- phase / running state
- restart count
- cpu usage
- memory usage
- network rx / tx
- volume filesystem usage 若可稳定获取

### 4.3 Container 衔接

第一版原则：

- 容器更细粒度资源指标优先由 [`container-metrics-spec.md`](container-metrics-spec.md) 承担
- `k8s_node_pod_metrics` 不要求完整重复容器级指标面

---

## 5. Resource Mapping

### 5.1 主资源

默认映射到：

- `k8s_node`
- `k8s_pod`

### 5.2 最小资源属性

Node:

- `host.id`
- `host.name`
- `k8s.node.name`

Pod:

- `k8s.namespace.name`
- `k8s.pod.uid`
- `k8s.pod.name`
- `k8s.node.name`
- `k8s.owner.kind?`
- `k8s.owner.name?`

### 5.3 与 service / workload 的关系

第一版可以受控补充：

- `service.name`
- `k8s.deployment.name`
- `k8s.statefulset.name`
- `k8s.daemonset.name`

但前提是：

- 来源稳定
- 不把 owner 链无界展开

---

## 6. 高基数控制

第一版默认不直接展开：

- 全量 pod labels
- 全量 pod annotations
- 全量 ownerReferences 链
- 动态注入的临时 annotation

建议只保留低基数、稳定且对聚合有意义的关键维度。

---

## 7. 预算建议

第一版默认建议：

- stats `schedule.interval_ms = 15000`
- metadata refresh `interval_ms = 30000`
- `schedule.timeout_ms = 5000`
- `budget.max_targets = 5000`
- `budget.max_samples = 20000`

### 7.1 退化建议

进入 `degraded` 后建议：

- 降低 metadata watch 转换频率
- 暂停 owner / workload 深度映射
- 保留 node / pod 核心状态和资源指标

进入 `protect` 后建议：

- 暂停非关键 enrich
- 对超大节点优先保留 node 级与 pod phase / restart / cpu / memory

---

## 8. 第一版实现约束

第一版建议：

- 不要求依赖 `kube-state-metrics` 作为默认前提
- 不把 Kubernetes 指标采集退化成远程 action 命令执行
- 不要求完整复刻所有 Kubernetes 平台 exporter

---

## 9. 验收口径

第一版至少应满足：

- pod 生命周期变化在 `2` 个 refresh 周期内收敛
- 同一 pod 的 `target_id` 稳定
- owner / workload 映射在可得时稳定，不可得时为空而不是误绑
- 在大节点压力下仍优先保留 node / pod 核心状态与资源指标

---

## 10. 当前决定

当前阶段固定以下结论：

- `k8s_node_pod_metrics` 是 Batch A 内建能力
- 默认路线是本地统计 + Kubernetes metadata 组合
- 先把 node / pod 基础指标和资源绑定打稳，再扩展更多平台细项

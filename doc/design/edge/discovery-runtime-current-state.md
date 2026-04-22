# warp-insightd Discovery Runtime 当前实现摘要

## 1. 文档目的

本文档用一页纸回答当前实现层面的三个问题：

- 现在 `warp-insightd` 能扫描出哪些资源
- 现在本地会输出哪些状态文件
- 现在 discovery / planner / metrics runtime 已经接到什么程度

如果需要完整边界、对象模型和分阶段设计，应阅读：

- [`resource-discovery-runtime.md`](./resource-discovery-runtime.md)

---

## 2. 当前可扫描资源

当前版本已经能扫描：

- `host`
  - `host.id`
  - `host.name`
  - 单一 `host` target
- `process`
  - Linux 下扫描 `/proc`
  - 其他 Unix 下回退 `ps`
  - 当前稳定字段：
    - `process.pid`
    - `process.identity?`
    - `discovery.identity_strength?`
    - `discovery.identity_status?`
- `container`
  - 扫描本地常见 runtime task root
  - 当前覆盖：
    - `containerd`
    - `docker runtime-runc`
  - 当前稳定字段：
    - `container.id`
    - `container.name`
    - `container.runtime`
    - `container.runtime.namespace?`
    - `pid?`
    - `cgroup.path?`
    - `k8s.namespace.name?`
    - `k8s.pod.uid?`
    - `k8s.pod.name?`
    - `k8s.container.name?`

当前还没有实现：

- `k8s_node`
- `k8s_pod`
- 远程 endpoint 类 discovery
- container runtime API 驱动 enrich

---

## 3. 当前运行时接线

当前 daemon tick 中，已形成下面这条最小闭环：

```text
discovery refresh
  -> state/discovery/*.json
  -> planner bridge
  -> state/planner/*_metrics_candidates.json
  -> metrics target view
  -> state/telemetry/metrics_target_view.json
  -> metrics runtime
  -> state/telemetry/metrics_runtime_snapshot.json
  -> metrics samples
  -> state/telemetry/metrics_samples.json
```

这条链路当前已经可以验证：

- discovery 是否足够支撑数据面
- planner 是否把 `DiscoveredTarget` 正确编译成 `CandidateCollectionTarget`
- metrics runtime 是否能基于 candidate 执行最小本地 probe
- sample 文件是否能作为后续 exporter / uplink 的中间输入

---

## 4. 当前本地状态文件

### 4.1 discovery cache

```text
state/discovery/resources.json
state/discovery/targets.json
state/discovery/meta.json
```

用途：

- 保存最近一次成功 discovery snapshot
- daemon 重启后作为冷启动事实视图

### 4.2 planner candidate state

```text
state/planner/host_metrics_candidates.json
state/planner/process_metrics_candidates.json
state/planner/container_metrics_candidates.json
```

用途：

- 保存 `CandidateCollectionTarget[]`
- 显式区分“发现到了什么”和“准备怎么采”

### 4.3 metrics target view

```text
state/telemetry/metrics_target_view.json
```

用途：

- 保存 metrics runtime 实际读取的统一 target 视图
- 聚合 planner candidate，但不包含执行结果

### 4.4 metrics runtime snapshot

```text
state/telemetry/metrics_runtime_snapshot.json
```

用途：

- 保存当前 metrics tick 的执行摘要
- 记录每类 `collection_kind` 的：
  - `status`
  - `attempted_targets`
  - `succeeded_targets`
  - `failed_targets`
  - `last_error?`
  - `runtime_facts[]`
  - `sample_targets[]`

当前 `status` 语义：

- `idle`
- `succeeded`
- `partial`
- `failed`

### 4.5 metrics samples

```text
state/telemetry/metrics_samples.json
```

用途：

- 保存第一版可消费 sample 视图
- 作为后续 Prometheus text / OTel exporter 的中间输入

当前 sample 结构包含：

- `metric_name`
- `value`
- `value_type`
- `unit`
- `collection_kind`
- `target_ref`
- `resource_ref`
- `metric_attributes[]`
- `resource_attributes[]`

当前 `value_type` 示例：

- `gauge_i64`
- `gauge_f64`
- `gauge_string`

---

## 5. 当前 metrics runtime 覆盖范围

### 5.1 `host_metrics`

当前已是本地真实 probe。

当前可产出：

- `host.target.count`
- `host.loadavg.1m`
- `host.loadavg.5m`
- `host.loadavg.15m`
- `host.uptime.seconds`
- `host.memory.total_kb`
- `host.memory.available_kb`

### 5.2 `process_metrics`

当前已是本地真实 probe。

Linux 下优先读取 `/proc/<pid>/stat`，其他 Unix 下回退 `ps`。

当前可产出：

- `process.state`
- `process.cpu.user_ticks`
- `process.cpu.system_ticks`
- `process.memory.rss_pages` 或 `process.memory.rss_kb`

### 5.3 `container_metrics`

当前是基于 discovery hint 的轻量 probe。

当前优先复用：

- `container.runtime`
- `container.runtime.namespace`
- `cgroup.path`
- `k8s.*`
- `pid`

若存在 `pid`，会复用 process probe 补：

- `process.state`
- `process.cpu.user_ticks`
- `process.cpu.system_ticks`
- `process.memory.rss_pages` 或 `process.memory.rss_kb`

---

## 6. 当前实现边界

当前实现已经具备最小闭环，但仍然不是最终态：

- 还没有真正 exporter
- 还没有 metrics uplink
- 还没有统一 interval / timeout / budget 调度
- 还没有完整 filesystem / disk io / network / fd / thread / restart count 指标
- `container_metrics` 仍以 discovery hint 为主，不依赖 runtime API
- `metrics_samples.json` 仍是本地中间态，不是最终对外协议

当前实现更适合定位为：

- 可验证 discovery 是否足够支撑数据面
- 可验证 planner -> runtime -> sample 的状态边界
- 可作为后续 exporter / uplink / 调度器的稳定输入

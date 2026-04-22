# warp-insightd Discovery 当前输出样例

## 1. 文档目的

本文档回答当前实现层面的两个具体问题：

- 现在 `warp-insightd` 可以发现哪些资源与 target
- 当前 `state/discovery/*.json` 以及下游相关状态大致长什么样

本文档只描述当前实现现状，不扩展未来设计。

相关文档：

- [`resource-discovery-runtime.md`](./resource-discovery-runtime.md)
- [`discovery-runtime-current-state.md`](./discovery-runtime-current-state.md)

---

## 2. 当前 discovery 覆盖范围

当前 `warp-insightd` 已接入并实际执行的 discovery probe 只有：

- `host`
- `process`
- `container`

当前还没有接入主循环的 discovery 类型：

- `k8s_node`
- `k8s_pod`
- `service`
- `service_endpoint`
- `log_file`

因此当前 `state/discovery/*.json` 中实际出现的 `kind` 主要是：

- resource:
  - `host`
  - `process`
  - `container`
- target:
  - `host`
  - `process`
  - `container`

---

## 3. 当前能扫描出的对象

### 3.1 `host`

当前每轮至少会发现：

- 一个 `host` resource
- 一个 `host` target

当前稳定字段：

- `host.id`
- `host.name`

### 3.2 `process`

当前会扫描本机进程：

- Linux 下读取 `/proc`
- 其他 Unix 下回退 `ps`

每个进程当前会形成：

- 一个 `process` resource
- 一个 `process` target

当前常见字段：

- `process.pid`
- `process.executable.name?`
- `process.identity?`
- `discovery.identity_strength?`
- `discovery.identity_status?`

### 3.3 `container`

当前采用本地 runtime task root 扫描，不依赖 runtime API。

当前覆盖：

- `containerd`
- `docker runtime-runc`

每个容器当前会形成：

- 一个 `container` resource
- 一个 `container` target

当前常见字段：

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

---

## 4. 当前本地输出文件

当前运行一次后，discovery 及其直接下游通常会写出：

```text
state/discovery/resources.json
state/discovery/targets.json
state/discovery/meta.json

state/planner/host_metrics_candidates.json
state/planner/process_metrics_candidates.json
state/planner/container_metrics_candidates.json

state/telemetry/metrics_target_view.json
state/telemetry/metrics_runtime_snapshot.json
state/telemetry/metrics_samples.json
```

本文重点展示前四类：

- `resources.json`
- `targets.json`
- `meta.json`
- `*_metrics_candidates.json`

---

## 5. `resources.json` 当前样例

`resources.json` 当前是：

```text
DiscoveredResource[]
```

### 5.1 `host` resource 示例

```json
[
  {
    "resource_id": "hostname:demo-host",
    "kind": "host",
    "attributes": [
      { "key": "host.id", "value": "hostname:demo-host" },
      { "key": "host.name", "value": "demo-host" }
    ],
    "runtime_facts": [
      { "key": "host.name", "value": "demo-host" }
    ],
    "discovered_at": "2026-04-20T08:00:00Z",
    "last_seen_at": "2026-04-20T08:00:00Z",
    "health": "healthy",
    "source": "local_runtime"
  }
]
```

### 5.2 `process` resource 示例

```json
[
  {
    "resource_id": "host-1:pid:42:linux_proc_start:123",
    "kind": "process",
    "attributes": [
      { "key": "host.id", "value": "host-1" },
      { "key": "process.pid", "value": "42" },
      { "key": "process.executable.name", "value": "sshd" }
    ],
    "runtime_facts": [
      { "key": "process.pid", "value": "42" },
      { "key": "process.identity", "value": "linux_proc_start:123" }
    ],
    "discovered_at": "2026-04-20T08:00:00Z",
    "last_seen_at": "2026-04-20T08:00:00Z",
    "health": "healthy",
    "source": "local_runtime"
  }
]
```

当进程 identity 无法获取时，当前还可能出现：

- `discovery.identity_strength = weak`
- `discovery.identity_status = unavailable`

### 5.3 `container` resource 示例

```json
[
  {
    "resource_id": "3f2c9f3c1b8e",
    "kind": "container",
    "attributes": [
      { "key": "container.id", "value": "3f2c9f3c1b8e" },
      { "key": "container.name", "value": "nginx" },
      { "key": "container.runtime", "value": "containerd" },
      { "key": "host.id", "value": "host-1" },
      { "key": "k8s.namespace.name", "value": "default" },
      { "key": "k8s.pod.uid", "value": "pod-uid-1" },
      { "key": "k8s.pod.name", "value": "nginx-abc" },
      { "key": "k8s.container.name", "value": "nginx" }
    ],
    "runtime_facts": [
      { "key": "container.id", "value": "3f2c9f3c1b8e" },
      { "key": "container.runtime", "value": "containerd" },
      { "key": "container.runtime.namespace", "value": "k8s.io" },
      { "key": "pid", "value": "1234" },
      { "key": "cgroup.path", "value": "/kubepods/burstable/..." }
    ],
    "discovered_at": "2026-04-20T08:00:00Z",
    "last_seen_at": "2026-04-20T08:00:00Z",
    "health": "healthy",
    "source": "local_runtime"
  }
]
```

---

## 6. `targets.json` 当前样例

`targets.json` 当前是：

```text
DiscoveredTarget[]
```

### 6.1 `host` target 示例

```json
[
  {
    "target_id": "hostname:demo-host:host",
    "kind": "host",
    "resource_refs": ["hostname:demo-host"],
    "endpoint": null,
    "labels": [],
    "runtime_facts": [
      { "key": "host.name", "value": "demo-host" }
    ],
    "discovered_at": "2026-04-20T08:00:00Z",
    "last_seen_at": "2026-04-20T08:00:00Z",
    "state": "active",
    "source": "local_runtime"
  }
]
```

### 6.2 `process` target 示例

```json
[
  {
    "target_id": "host-1:pid:42:linux_proc_start:123:process",
    "kind": "process",
    "resource_refs": ["host-1:pid:42:linux_proc_start:123"],
    "endpoint": null,
    "labels": [],
    "runtime_facts": [
      { "key": "process.pid", "value": "42" },
      { "key": "process.identity", "value": "linux_proc_start:123" }
    ],
    "discovered_at": "2026-04-20T08:00:00Z",
    "last_seen_at": "2026-04-20T08:00:00Z",
    "state": "active",
    "source": "local_runtime"
  }
]
```

### 6.3 `container` target 示例

```json
[
  {
    "target_id": "3f2c9f3c1b8e",
    "kind": "container",
    "resource_refs": ["3f2c9f3c1b8e"],
    "endpoint": null,
    "labels": [],
    "runtime_facts": [
      { "key": "container.runtime", "value": "containerd" },
      { "key": "container.runtime.namespace", "value": "k8s.io" },
      { "key": "pid", "value": "1234" },
      { "key": "cgroup.path", "value": "/kubepods/burstable/..." },
      { "key": "k8s.pod.uid", "value": "pod-uid-1" }
    ],
    "discovered_at": "2026-04-20T08:00:00Z",
    "last_seen_at": "2026-04-20T08:00:00Z",
    "state": "active",
    "source": "local_runtime"
  }
]
```

---

## 7. `meta.json` 当前样例

`meta.json` 当前是：

```text
DiscoveryCacheMeta
```

示例：

```json
{
  "schema_version": "v1",
  "snapshot_id": "discovery:42:2026-04-20T08:00:00Z",
  "revision": 42,
  "generated_at": "2026-04-20T08:00:00Z",
  "last_success_at": "2026-04-20T08:00:00Z",
  "last_error": null
}
```

当前字段含义：

- `snapshot_id`
  本轮 discovery 快照 id
- `revision`
  当前快照 revision
- `generated_at`
  当前快照生成时间
- `last_success_at`
  最近一次成功 refresh 时间
- `last_error`
  最近一次 refresh / store 相关错误

---

## 8. planner candidate 当前样例

当前 planner bridge 会把 `DiscoveredTarget` 映射成：

- `host` -> `host_metrics`
- `process` -> `process_metrics`
- `container` -> `container_metrics`

### 8.1 `host_metrics_candidates.json`

```json
[
  {
    "candidate_id": "hostname:demo-host:host:host_metrics",
    "target_ref": "hostname:demo-host:host",
    "collection_kind": "host_metrics",
    "resource_refs": ["hostname:demo-host"],
    "execution_hints": [
      { "key": "discovery.source", "value": "local_runtime" }
    ],
    "generated_at": "2026-04-20T08:00:00Z"
  }
]
```

### 8.2 `process_metrics_candidates.json`

```json
[
  {
    "candidate_id": "host-1:pid:42:linux_proc_start:123:process:process_metrics",
    "target_ref": "host-1:pid:42:linux_proc_start:123:process",
    "collection_kind": "process_metrics",
    "resource_refs": ["host-1:pid:42:linux_proc_start:123"],
    "execution_hints": [
      { "key": "discovery.source", "value": "local_runtime" },
      { "key": "process.pid", "value": "42" },
      { "key": "process.identity", "value": "linux_proc_start:123" }
    ],
    "generated_at": "2026-04-20T08:00:00Z"
  }
]
```

### 8.3 `container_metrics_candidates.json`

```json
[
  {
    "candidate_id": "3f2c9f3c1b8e:container_metrics",
    "target_ref": "3f2c9f3c1b8e",
    "collection_kind": "container_metrics",
    "resource_refs": ["3f2c9f3c1b8e"],
    "execution_hints": [
      { "key": "discovery.source", "value": "local_runtime" },
      { "key": "container.runtime", "value": "containerd" },
      { "key": "container.runtime.namespace", "value": "k8s.io" },
      { "key": "pid", "value": "1234" },
      { "key": "cgroup.path", "value": "/kubepods/burstable/..." },
      { "key": "k8s.pod.uid", "value": "pod-uid-1" },
      { "key": "k8s.pod.name", "value": "nginx-abc" },
      { "key": "k8s.container.name", "value": "nginx" }
    ],
    "generated_at": "2026-04-20T08:00:00Z"
  }
]
```

---

## 9. 当前实现边界

当前这批样例描述的是“现在已经接上的 discovery 本地输出”，因此应明确：

- 这些示例是当前字段形态，不代表长期冻结 schema
- `host / process / container` 是当前唯一已接入主循环的 discovery 类型
- `k8s` 相关字段当前主要来自 container runtime hint，不来自独立 `k8s` probe
- discovery 已经接到 planner 和 metrics runtime
- discovery 结果目前仍是本地状态，没有上送到中心

---

## 10. 当前决定

当前阶段可把 discovery 现状收敛成一句话：

- `warp-insightd` 已经能在本地发现 `host / process / container`
- 已经能输出 `resources.json / targets.json / meta.json`
- 已经能把这些结果继续编译成 metrics candidate 和 metrics target view

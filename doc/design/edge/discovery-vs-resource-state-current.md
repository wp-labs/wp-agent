# warp-insightd Discovery 与资源状态当前边界

## 1. 文档目的

本文档只回答一个问题：

- 在当前 `warp-insightd` 设计里，“资源发现”和“资源状态”是如何分开的

这里的“资源状态”特指：

- 资源当前是否存在
- 当前能否被观测
- 当前 probe 采到了什么运行态事实或指标

本文档描述的是当前实现与当前设计口径，不展开未来架构扩展。

相关文档：

- [`resource-discovery-runtime.md`](./resource-discovery-runtime.md)
- [`discovery-runtime-current-state.md`](./discovery-runtime-current-state.md)
- [`discovery-output-examples-current.md`](./discovery-output-examples-current.md)
- [`../foundation/glossary.md`](../foundation/glossary.md)

---

## 2. 一句话边界

当前应把两种能力分开理解：

- `discovery`
  回答“本机有什么对象”
- `runtime state / metrics`
  回答“这些对象当前是什么状态”

也就是说：

- discovery 先建立对象身份和引用关系
- runtime state 再基于这些对象去补运行态事实

不应把两者混成：

- “只要能采到一点状态，就算 discovery”
- “discovery 结果本身就等于最终资源状态”

---

## 3. 当前对象分层

### 3.1 discovery 层对象

当前 discovery 层对象是：

- `DiscoveredResource`
- `DiscoveredTarget`
- `DiscoverySnapshot`

它们表达的是：

- 当前发现到了哪些 resource
- 当前发现到了哪些 target
- 这些对象的稳定 id、关联关系和最小本地事实

它们不表达：

- metrics collector 最终如何执行
- 当前指标采集是否成功
- 当前一轮 probe 的执行结果汇总

### 3.2 planning 层对象

当前 planning 层对象是：

- `CandidateCollectionTarget`

它表达的是：

- 对某个已发现 target，后续准备如何采

它是中间桥接层：

- 上接 discovery
- 下接 metrics runtime

### 3.3 runtime state 层对象

当前 runtime state 层对象主要体现在：

- `metrics_target_view`
- `metrics_runtime_snapshot`
- `metrics_samples`

它们表达的是：

- 实际投入 runtime 的 target 视图
- 当前一轮 metrics 执行摘要
- 当前一轮可消费 sample

它们不应被当作 discovery cache。

---

## 4. 当前文件边界

### 4.1 资源发现

当前 discovery 本地状态文件：

```text
state/discovery/resources.json
state/discovery/targets.json
state/discovery/meta.json
```

这些文件回答的是：

- 发现到了什么
- 对象 id 是什么
- resource 和 target 如何关联
- 最近一次 discovery snapshot 是什么

### 4.2 采集候选

当前 planning 本地状态文件：

```text
state/planner/host_metrics_candidates.json
state/planner/process_metrics_candidates.json
state/planner/container_metrics_candidates.json
```

这些文件回答的是：

- discovery 结果被编译成了哪些采集候选
- 每个候选准备采用哪种 `collection_kind`
- 执行需要哪些 `execution_hints`

### 4.3 运行态摘要与样本

当前 runtime state 本地状态文件：

```text
state/telemetry/metrics_target_view.json
state/telemetry/metrics_runtime_snapshot.json
state/telemetry/metrics_samples.json
```

这些文件回答的是：

- 当前 runtime 实际消费哪些 target
- 当前一轮 metrics probe 成功还是失败
- 当前采到了哪些 sample

---

## 5. 当前具体如何分层

### 5.1 `resources.json` / `targets.json`

当前这里保存的是“对象事实”。

例如：

- `host`
- `process`
- `container`

以及它们的：

- `resource_id`
- `target_id`
- `resource_ref`
- discovery 期间拿到的最小事实

当前这里不保存：

- `host.loadavg.1m`
- `process.cpu.user_ticks`
- `process.memory.rss_pages`
- 一轮采集成功/失败摘要

### 5.2 `*_metrics_candidates.json`

当前这里保存的是“准备怎么采”。

例如：

- `host` target -> `host_metrics`
- `process` target -> `process_metrics`
- `container` target -> `container_metrics`

这里不保存：

- 采集结果值
- 当前 probe 成败

### 5.3 `metrics_runtime_snapshot.json`

当前这里保存的是“当前一轮执行状态”。

例如：

- `attempted_targets`
- `succeeded_targets`
- `failed_targets`
- `status`
- `last_error`
- `runtime_facts`

它是 runtime summary，不是 discovery snapshot。

### 5.4 `metrics_samples.json`

当前这里保存的是“当前一轮采到的 sample”。

例如：

- `system.load_average.1m`
- `process.cpu.user_ticks`
- `process.memory.rss_pages`

这些是观测结果，不是 discovery 结果。

---

## 6. 当前系统里的接线关系

当前最小闭环是：

```text
discovery
  -> DiscoveredResource / DiscoveredTarget
  -> planner bridge
  -> CandidateCollectionTarget
  -> metrics target view
  -> metrics runtime snapshot
  -> metrics samples
```

因此应明确：

- discovery 输出的是“对象事实”和“执行入口事实”
- planner 输出的是“准备怎么采”
- metrics runtime 输出的是“当前采到了什么”

---

## 7. `resource.attributes` 与 `target.execution_hints` 边界

### 7.1 当前推荐模型

当前 discovery 输出对象模型是：

- `resource`
  - `resource_id`
  - `kind`
  - `origin_idx`
  - `attributes`
- `target`
  - `target_id`
  - `kind`
  - `origin_idx`
  - `resource_ref`
  - `execution_hints`

其中：

- `origin_idx`
  - 只在当前 snapshot 内有效
  - 用于索引 `meta.json.origins[]`
- `resource_ref`
  - 表示当前 target 主要绑定到哪个 resource
  - 当前不再表达多 resource target

### 7.2 `resource.attributes` 的语义

`resource.attributes` 表达的是：

- 资源身份事实
- 资源归类事实
- 资源相对稳定、适合被直接展示或检索的属性

它不应承担：

- metrics runtime 的执行参数集合
- 当前一轮 probe 的结果
- 仅在执行期才有意义的临时上下文

当前典型例子：

- `host`
  - `host.id`
  - `host.name`
- `process`
  - `process.pid`
  - `process.executable.name`
- `container`
  - `container.id`
  - `container.name`
  - `container.runtime`
  - `container.runtime.namespace?`
  - `k8s.*?`

### 7.3 `target.execution_hints` 的语义

`target.execution_hints` 表达的是：

- 后续 collector / runtime 真正执行时需要知道的结构化上下文
- 不要求稳定，但必须结构化可消费
- 不应要求下游再去解析 `target_id`

它不应承担：

- 资源发现来源元数据
- 当前一轮指标值
- snapshot 级公共上下文

当前典型例子：

- `host`
  - `host.name`
- `process`
  - `process.pid`
  - `process.identity?`
  - `discovery.identity_strength?`
  - `discovery.identity_status?`
- `container`
  - `container.runtime`
  - `container.runtime.namespace?`
  - `pid?`
  - `cgroup.path?`
  - `k8s.*?`

### 7.4 哪些字段允许两边同时出现

当前设计允许某些字段同时存在于：

- `resource.attributes`
- `target.execution_hints`

前提是它们同时承担两种职责：

- 既是资源事实
- 也是执行上下文

这不是简单重复，而是边界允许的双重语义复用。

当前可接受的例子：

- `process.pid`
  - 在 `resource.attributes` 中便于资源展示和检索
  - 在 `target.execution_hints` 中供 metrics runtime 直接执行 probe
- `container.runtime`
  - 在 `resource.attributes` 中表达容器归类
  - 在 `target.execution_hints` 中决定后续 runtime 行为
- `host.name`
  - 在 `resource.attributes` 中表达主机事实
  - 在 `target.execution_hints` 中供 host metrics runtime 直接复用

### 7.5 哪些字段不应放进 `execution_hints`

当前不建议放进 `execution_hints` 的是：

- `discovery.source`
- `generated_at`
- `observed_at`
- `health`
- `state`

原因是这些字段属于：

- snapshot 级上下文
- origin 元数据
- runtime 结果摘要

它们不应伪装成 target 执行提示。

当前这类信息统一放在：

- `state/discovery/meta.json`
- `origins[]`
- `metrics_runtime_snapshot.json`

### 7.6 设计判断准则

后续评估某个 discovery 字段应放在哪一层时，建议按下面顺序判断：

1. 这个字段是不是 snapshot 公共上下文
2. 这个字段是不是资源自身事实
3. 这个字段是不是执行期真正需要的结构化 hint
4. 如果两边都需要，它是否确实承担两种职责

如果一个字段只是：

- 来源说明
- 统一时间戳
- probe 执行摘要

那它通常不应进入 `execution_hints`。

- discovery 是上游事实层
- planning 是编译层
- runtime state 是执行结果层

三层是串起来的，但不是一个对象。

---

## 7. 当前容易混淆的点

### 7.1 `runtime_facts`

`DiscoveredResource.runtime_facts` 和 `DiscoveredTarget.runtime_facts` 当前允许存在。

但它们的定位是：

- discovery 期间就能拿到的最小本地事实
- 用于 identity、target hint、planner hint

它们不等于：

- metrics runtime 的执行摘要
- 最终可导出的 sample

### 7.2 `health` / `state`

当前 discovery 对象上也有：

- `DiscoveredResource.health`
- `DiscoveredTarget.state`

但这里的语义仍然是：

- discovery 本地视角下的轻量状态

它不等于：

- 完整运行态监控状态
- 中心侧资源生命周期状态机

### 7.3 `container` 上的 `k8s.*` 字段

当前 container discovery 里已经可能出现：

- `k8s.namespace.name`
- `k8s.pod.uid`
- `k8s.pod.name`
- `k8s.container.name`

这些当前只是 container discovery 的 hint / enrichment。

这不表示：

- 独立 `k8s_pod` discovery 已经完成
- 独立 `k8s_node` 资源对象已经建模完成

---

## 8. 当前结论

当前阶段应固定以下理解：

- discovery 是“对象发现能力”
- metrics runtime 是“对象运行态观测能力”
- 二者共享 resource/target 引用，但不共享同一语义层
- `state/discovery/*.json` 是发现结果
- `state/telemetry/*.json` 是运行态结果

一句话说：

- 先知道“有什么”
- 再知道“现在怎么样”

# warp-insight Host / Pod / Network Topology 子模型设计

## 1. 文档目的

本文档定义 `warp-insight` 中心侧 `host / pod / network` 之间的拓扑子模型。

目标不是另起一套新体系，而是在现有中心对象模型上补齐：

- 多个 `host` 上的 `pod` 如何表达
- 多个 `pod` 处于同一网络如何表达
- `host` 与 `network`、`pod` 与 `network` 的关系如何统一建模
- 这套关系如何与已有的 `HostInventory`、责任归属、软件与漏洞图谱对接

相关文档：

- [`host-inventory-and-runtime-state.md`](host-inventory-and-runtime-state.md)
- [`host-process-software-vulnerability-graph.md`](host-process-software-vulnerability-graph.md)
- [`host-responsibility-and-maintainer-model.md`](host-responsibility-and-maintainer-model.md)
- [`../../edge/resource-discovery-runtime.md`](../../edge/resource-discovery-runtime.md)

---

## 2. 核心结论

第一版固定以下结论：

- `host`、`pod`、`network` 都是独立对象，不应互相内嵌
- “多个 host 上的 pod 在同一网络”本质上是多个 attachment 指向同一网络对象
- `network` 不应作为 `host` 或 `pod` 的单字段属性
- `pod` 的调度归属与网络归属必须分开表达
- 这部分属于现有资源目录模型和关系图谱模型的扩展子模型，不是新体系

一句话说：

- `HostInventory` 回答“节点是谁”
- `PodInventory` 回答“运行对象是谁”
- `NetworkSegment` 回答“它们连在哪个网络里”

---

## 3. 为什么不是一个字段

如果把模型写成：

```text
HostInventory {
  pods[]
  network = "10.10.0.0/16"
}
```

或者：

```text
PodInventory {
  host_id
  network = "net-a"
}
```

会出现明显问题：

- 一个 `host` 可连接多个网络段
- 一个 `pod` 可挂多个网络附件，不一定只有一个网络
- `pod` 会迁移，调度到不同 `host`
- 同一网络可被很多 `host` 和很多 `pod` 共享
- `network` 有自己的身份、CIDR、域边界、生命周期

因此应明确：

- `host` 是计算承载对象
- `pod` 是运行对象
- `network` 是独立资源对象
- `attachment` 才是连接关系

---

## 4. 模型定位

这不是新的顶层模型，而是现有中心模型中的一个拓扑子模型。

它同时属于：

### 4.1 资源目录模型的扩展

以下对象都属于资源目录：

- `HostInventory`
- `PodInventory`
- `NetworkDomain`
- `NetworkSegment`

### 4.2 关系图谱模型的扩展

以下对象都属于关系边：

- `PodPlacement`
- `PodNetworkAttachment`
- `HostNetworkAttachment`

也就是说：

- 资源对象单独建模
- 动态状态单独建模
- 关系通过 attachment / placement 表达

---

## 5. 对象模型

### 5.1 `HostInventory`

沿用已有定义，表示主机/节点资产。

回答：

- 这台主机是谁

不直接表达：

- 完整 pod 列表
- 所有网络关系

### 5.2 `PodInventory`

表示稳定的 pod 目录对象。

建议结构：

```text
PodInventory {
  pod_id
  tenant_id
  environment_id
  cluster_id?
  namespace
  workload_id?
  pod_uid
  pod_name
  node_id?
  phase?
  first_seen_at
  last_seen_at
}
```

说明：

- `pod_id` 是中心内部稳定主键
- `pod_uid` 是外部 Kubernetes 语义下的稳定标识候选
- `node_id` 表示当前或最近一次已知调度节点，不应承担完整调度历史

### 5.3 `NetworkDomain`

表示更高层的网络边界。

建议结构：

```text
NetworkDomain {
  network_domain_id
  tenant_id
  environment_id
  kind
  name
  external_ref?
  metadata?
  created_at
  updated_at
}
```

`kind` 示例：

- `vpc`
- `vlan_fabric`
- `k8s_cluster_network`
- `cni_fabric`

### 5.4 `NetworkSegment`

表示具体的可连接网络段。

建议结构：

```text
NetworkSegment {
  network_segment_id
  network_domain_id
  segment_type
  name
  cidr?
  gateway_ip?
  metadata?
  created_at
  updated_at
}
```

`segment_type` 示例：

- `subnet`
- `overlay`
- `pod_network`
- `service_network`
- `namespace_network`

### 5.5 `PodPlacement`

表示 pod 与 host/node 的调度关系。

建议结构：

```text
PodPlacement {
  placement_id
  pod_id
  host_id
  source
  valid_from
  valid_to?
  created_at
  updated_at
}
```

说明：

- 一个 `pod` 在同一时刻应只有一个有效 placement
- 历史迁移通过多段 `valid_from / valid_to` 保存

### 5.6 `PodNetworkAttachment`

表示 pod 连到哪个网络段。

建议结构：

```text
PodNetworkAttachment {
  attachment_id
  pod_id
  network_segment_id
  interface_name?
  ip_addr?
  mac_addr?
  is_primary
  source
  valid_from
  valid_to?
  created_at
  updated_at
}
```

说明：

- 一个 `pod` 可有多个 attachment
- 一个 `network_segment` 可被很多 `pod` 共享
- `source` 可标记 `k8s_api`、`cni_sync`、`runtime_hint`

### 5.7 `HostNetworkAttachment`

表示 host 连到哪个网络段。

建议结构：

```text
HostNetworkAttachment {
  attachment_id
  host_id
  network_segment_id
  interface_name?
  ip_addr?
  mac_addr?
  is_primary
  source
  valid_from
  valid_to?
  created_at
  updated_at
}
```

说明：

- 它表达的是 host 自身与网络的关系
- 不应用 `PodPlacement` 替代这个关系

---

## 6. 关系图谱

第一版建议固定以下关系：

```text
HostInventory
  -> HostNetworkAttachment[]
  -> PodPlacement[]

PodInventory
  -> PodPlacement[]
  -> PodNetworkAttachment[]

PodNetworkAttachment
  -> NetworkSegment

HostNetworkAttachment
  -> NetworkSegment

NetworkSegment
  -> NetworkDomain
```

若处于 Kubernetes 环境，还可补充：

```text
K8sCluster
  -> K8sNamespace
  -> Workload
  -> PodInventory
```

这样可以同时回答：

- 这个 pod 跑在哪台 host 上
- 这个 pod 连接了哪些网络
- 这台 host 上有哪些 pod 与外部网络或 overlay 发生连接

---

## 7. 调度归属与网络归属必须分开

这是这个模型里最容易混淆的一点。

### 7.1 调度归属

调度归属回答：

- 这个 pod 当前落在哪个 host / node 上

它对应：

- `PodPlacement`

### 7.2 网络归属

网络归属回答：

- 这个 pod 接入了哪个网络段

它对应：

- `PodNetworkAttachment`

因此：

- `pod -> host` 是 placement 关系
- `pod -> network` 是 attachment 关系

两者不能合并。

---

## 8. 与现有模型的衔接

### 8.1 与 `HostInventory`

- `HostInventory` 继续作为节点/主机目录对象
- 网络和 pod 不直接内嵌进 `HostInventory`

### 8.2 与 `HostRuntimeState`

- `HostRuntimeState` 保存 host 当前资源水位和健康
- 不表达完整拓扑关系

### 8.3 与 `ProcessRuntimeState`

- 进程仍主要挂在 `host`
- 若后续容器/pod 内进程可观测，可增加 `process -> pod` 的可选关联

### 8.4 与软件和漏洞图谱

后续可形成：

```text
PodInventory
  -> SoftwareEvidence[]
  -> SoftwareEntity
  -> SoftwareVulnerabilityFinding[]
```

这样可以回答：

- 某个漏洞软件落在哪些 pod 上
- 这些 pod 分布在哪些 host 上
- 它们位于哪些网络段中

### 8.5 与责任归属模型

责任归属可继续独立建模：

- `host -> responsibility`
- 后续可增加 `cluster / namespace / workload -> responsibility`

不要把责任关系直接并进网络 attachment。

---

## 9. 第一版查询视图建议

### 9.1 主机拓扑视图

从 `HostInventory` 出发，展示：

- 主机基础信息
- 当前 pod 数量
- 连接的网络段
- 这些网络段中的 pod 分布摘要

### 9.2 Pod 拓扑视图

从 `PodInventory` 出发，展示：

- 所属 host / node
- 所属 namespace / workload
- 连接的网络段
- 相关软件与漏洞摘要

### 9.3 网络视图

从 `NetworkSegment` 出发，展示：

- 该网络段挂接的 host
- 该网络段挂接的 pod
- 关联的 cluster / namespace 分布

---

## 10. PostgreSQL 存储建议

第一版建议继续采用 PostgreSQL。

原因：

- 这部分是结构化 inventory + relation 数据
- 需要事务、一致性和 join 查询
- 与现有 `host inventory`、责任关系、软件图谱天然同库更简单

不建议第一版直接采用：

- 图数据库作为主存储
- 仅文档库存完整拓扑
- 把 attachment 关系塞进 JSON 字段

---

## 11. 第一版最小落地范围

当前建议固定为：

- 先支持 `PodInventory`
- 先支持 `NetworkDomain` 与 `NetworkSegment`
- 先支持 `PodPlacement`
- 先支持 `PodNetworkAttachment`
- `HostNetworkAttachment` 可与主机网络 inventory 一起推进

第一版不要一开始就做得过重：

- 不必先做全量 service mesh 拓扑
- 不必先做细粒度 east-west 实时流图
- 不必先做复杂网络策略推演

先把：

- pod 是谁
- pod 在哪台 host 上
- pod 连到哪个网络段
- 哪些 pod 共享同一网络段

四件事固定住。

---

## 12. 当前建议

当前建议固定为：

- 这是现有中心模型的拓扑扩展子模型，不是新体系
- `network` 是独立对象，不是 `host` 或 `pod` 的单字段属性
- `placement` 与 `attachment` 必须分开建模
- 最终形成 `host / pod / network / software / responsibility` 的统一关系图谱

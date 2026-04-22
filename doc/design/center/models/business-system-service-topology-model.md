# warp-insight Business / System / Service Topology 子模型设计

## 1. 文档目的

本文档定义 `warp-insight` 中心侧从业务到服务运行态的拓扑子模型。

目标是固定：

- 一个业务如何包含多个系统和子系统
- 系统如何包含多个逻辑服务
- 服务之间的依赖如何表达
- 服务如何实例化到 `pod`、`process` 和 `host`
- 服务地址与服务实例地址如何分层建模

相关文档：

- [`host-pod-network-topology-model.md`](host-pod-network-topology-model.md)
- [`host-inventory-and-runtime-state.md`](host-inventory-and-runtime-state.md)
- [`host-process-software-vulnerability-graph.md`](host-process-software-vulnerability-graph.md)
- [`host-responsibility-and-maintainer-model.md`](host-responsibility-and-maintainer-model.md)

---

## 2. 核心结论

第一版固定以下结论：

- `business`、`system`、`service`、`service instance` 不是同一个对象
- 服务依赖关系必须独立建模，不挂在 `host` 或 `network` 上
- 服务的逻辑入口地址与实例运行地址必须分开
- 服务实例运行在 `pod`、`container` 或 `process` 之上，最终落到 `host`
- 这部分属于现有中心模型的业务拓扑扩展子模型，不是新体系

一句话说：

- `BusinessDomain` 回答“这个业务是什么”
- `ServiceEntity` 回答“这个业务由哪些服务组成”
- `ServiceInstance` 回答“这些服务现在跑在哪里”

---

## 3. 为什么必须分层

如果把模型写成：

```text
Business {
  services[]
  dependencies[]
  addresses[]
}
```

或者：

```text
Service {
  business = "payment"
  host_id = "host-1"
  address = "10.0.0.12:8080"
}
```

会出现明显问题：

- 一个业务可包含多个系统与多个子系统
- 一个逻辑服务可有多个实例
- 一个服务可有多个入口地址
- 一个实例地址是动态的，可能随 pod 漂移
- 服务依赖是逻辑关系，不等同实例连通性

因此应明确：

- 业务架构层单独建模
- 运行部署层单独建模
- 地址层单独建模
- 依赖层单独建模

---

## 4. 模型定位

这不是新的顶层模型，而是现有中心模型中的一个业务拓扑子模型。

它同时属于：

### 4.1 业务目录模型的扩展

以下对象属于业务目录层：

- `BusinessDomain`
- `SystemBoundary`
- `Subsystem`
- `ServiceEntity`

### 4.2 运行关系图谱的扩展

以下对象属于运行关系层：

- `ServiceInstance`
- `ServiceDependency`
- `ServiceEndpoint`
- `ServiceInstanceEndpoint`

也就是说：

- 逻辑对象与运行对象分开
- 稳定地址与动态地址分开
- 服务依赖与网络 attachment 分开

---

## 5. 对象模型

### 5.1 `BusinessDomain`

表示业务域。

建议结构：

```text
BusinessDomain {
  business_id
  tenant_id
  name
  code?
  description?
  lifecycle_state?
  created_at
  updated_at
}
```

回答：

- 这个业务是谁

### 5.2 `SystemBoundary`

表示业务下的系统边界。

建议结构：

```text
SystemBoundary {
  system_id
  business_id
  tenant_id
  name
  code?
  description?
  parent_system_id?
  created_at
  updated_at
}
```

说明：

- 一个业务可以包含多个系统
- `parent_system_id` 可用于表达更粗或更细的系统边界

### 5.3 `Subsystem`

表示系统内部的子系统。

建议结构：

```text
Subsystem {
  subsystem_id
  system_id
  tenant_id
  name
  code?
  description?
  created_at
  updated_at
}
```

### 5.4 `ServiceEntity`

表示逻辑服务。

建议结构：

```text
ServiceEntity {
  service_id
  tenant_id
  business_id?
  system_id?
  subsystem_id?
  namespace?
  name
  service_type
  language?
  lifecycle_state?
  created_at
  updated_at
}
```

`service_type` 示例：

- `api`
- `worker`
- `scheduler`
- `gateway`
- `database_adapter`

说明：

- 这是逻辑定义，不是运行实例
- 一个服务可被部署为多个实例

### 5.5 `ServiceInstance`

表示服务的运行副本。

建议结构：

```text
ServiceInstance {
  service_instance_id
  service_id
  runtime_type
  pod_id?
  process_id?
  host_id?
  version?
  state?
  started_at?
  last_seen_at
}
```

`runtime_type` 示例：

- `pod`
- `container`
- `process`
- `vm`

说明：

- `ServiceInstance` 是业务服务与底层运行对象的桥
- 最终会落到 `PodInventory`、`ProcessRuntimeState` 和 `HostInventory`

### 5.6 `ServiceEndpoint`

表示服务的稳定入口地址。

建议结构：

```text
ServiceEndpoint {
  endpoint_id
  service_id
  endpoint_type
  address
  port?
  protocol
  exposure_scope
  valid_from
  valid_to?
  created_at
  updated_at
}
```

`endpoint_type` 示例：

- `dns`
- `vip`
- `ingress`
- `load_balancer`
- `nodeport`
- `external`

`exposure_scope` 示例：

- `cluster_internal`
- `vpc_internal`
- `public`

说明：

- 一个服务可有多个稳定入口
- 这些入口不等同具体实例地址

### 5.7 `ServiceInstanceEndpoint`

表示实例当前地址。

建议结构：

```text
ServiceInstanceEndpoint {
  instance_endpoint_id
  service_instance_id
  address
  port
  protocol
  source
  valid_from
  valid_to?
  created_at
  updated_at
}
```

说明：

- 这类地址通常是动态的
- 例如 pod IP:Port、host IP:Port、container IP:Port
- `source` 可标识 `k8s_api`、`discovery`、`runtime_probe`

### 5.8 `ServiceDependency`

表示服务之间的依赖关系。

建议结构：

```text
ServiceDependency {
  dependency_id
  upstream_service_id
  downstream_service_id
  dependency_type
  dependency_scope
  criticality?
  source
  valid_from
  valid_to?
  created_at
  updated_at
}
```

`dependency_type` 示例：

- `sync_rpc`
- `async_mq`
- `database`
- `cache`
- `external_api`

`dependency_scope` 建议区分：

- `declared`
- `observed`

说明：

- `declared` 是架构设计上的依赖
- `observed` 是运行时观测出来的依赖
- 这两类依赖不能混成一条

---

## 6. 关系图谱

第一版建议固定以下关系：

```text
BusinessDomain
  -> SystemBoundary[]

SystemBoundary
  -> Subsystem[]
  -> ServiceEntity[]

Subsystem
  -> ServiceEntity[]

ServiceEntity
  -> ServiceDependency[]
  -> ServiceEndpoint[]
  -> ServiceInstance[]

ServiceInstance
  -> ServiceInstanceEndpoint[]
  -> PodInventory / ProcessRuntimeState
  -> HostInventory
```

如果接入已有拓扑子模型，则进一步形成：

```text
ServiceInstance
  -> PodInventory
  -> PodNetworkAttachment
  -> NetworkSegment
  -> HostInventory
```

---

## 7. 服务地址与实例地址必须分开

这是本模型里必须固定的一条边界。

### 7.1 `ServiceEndpoint`

回答：

- 这个服务通过什么稳定入口被访问

例如：

- DNS
- Kubernetes Service DNS
- ClusterIP
- LoadBalancer VIP
- Ingress 域名

### 7.2 `ServiceInstanceEndpoint`

回答：

- 这个实例当前在哪个地址上运行

例如：

- Pod IP:Port
- Host IP:Port
- Container IP:Port

因此：

- 服务“有地址”，但通常是多个逻辑入口
- 实例“也有地址”，但通常是动态运行地址

两者不能合并成一个 `service.address` 字段。

---

## 8. 服务依赖与网络关系必须分开

### 8.1 服务依赖

回答：

- 上游服务在逻辑上依赖哪些下游服务

它对应：

- `ServiceDependency`

### 8.2 网络关系

回答：

- 服务实例所在 pod / host 接入了哪些网络段

它依赖：

- `PodNetworkAttachment`
- `HostNetworkAttachment`

因此：

- `service -> service` 是逻辑依赖
- `instance -> network` 是拓扑连接

两者不能混为一层。

---

## 9. 与现有模型的衔接

### 9.1 与 `HostInventory`

- `ServiceInstance` 最终运行在 `host`
- 但 `host` 不是服务定义本身

### 9.2 与 `PodInventory`

- 在 Kubernetes 环境里，`ServiceInstance` 常落到 `pod`
- `pod` 是服务实例的运行承载对象

### 9.3 与软件和漏洞图谱

后续可形成：

```text
ServiceEntity
  -> SoftwareEntity
  -> SoftwareVulnerabilityFinding[]
```

或：

```text
ServiceInstance
  -> SoftwareEvidence[]
  -> SoftwareEntity
```

这样可以回答：

- 哪个业务系统的哪个服务受某个漏洞影响
- 受影响的实例跑在哪些 pod / host 上

### 9.4 与责任归属模型

责任关系可继续独立建模：

- `business -> responsibility`
- `system -> responsibility`
- `service -> responsibility`
- `host -> responsibility`

不要把责任关系直接并进服务地址或依赖关系。

---

## 10. 第一版查询视图建议

### 10.1 业务视图

从 `BusinessDomain` 出发，展示：

- 包含哪些系统与子系统
- 包含哪些服务
- 服务依赖概览
- 关键服务运行健康摘要

### 10.2 服务视图

从 `ServiceEntity` 出发，展示：

- 服务所属业务与系统
- 服务依赖
- 服务入口地址
- 当前实例数量
- 实例分布在哪些 pod / host 上

### 10.3 实例视图

从 `ServiceInstance` 出发，展示：

- 实例属于哪个服务
- 当前实例地址
- 所在 pod / host
- 所在网络段

---

## 11. PostgreSQL 存储建议

第一版建议继续采用 PostgreSQL。

原因：

- 这部分是结构化目录对象与关系对象
- 需要事务、唯一约束和 join 查询
- 与已有 `host / pod / responsibility / software` 模型同库最简单

不建议第一版直接采用：

- 图数据库作为主存储
- 仅文档库承载服务拓扑
- 用 JSON 大字段保存整个业务树与依赖图

---

## 12. 第一版最小落地范围

当前建议固定为：

- 先支持 `BusinessDomain`
- 先支持 `SystemBoundary`
- 先支持 `ServiceEntity`
- 先支持 `ServiceInstance`
- 先支持 `ServiceEndpoint`
- 先支持 `ServiceDependency`

`Subsystem` 和 `ServiceInstanceEndpoint` 可在第一版一并支持，但如果要收敛，也可作为紧随其后的第二批对象。

第一版不要一开始就做得过重：

- 不必先做完整 APM 拓扑图
- 不必先做全量自动依赖推断
- 不必先做复杂流量权重和调用量模型

先把：

- 一个业务有哪些系统
- 一个系统有哪些服务
- 服务依赖谁
- 服务入口是什么
- 服务实例跑在哪里

五件事固定住。

---

## 13. 当前建议

当前建议固定为：

- 这是现有中心模型的业务拓扑扩展子模型，不是新体系
- `business / system / service / service instance` 必须分层
- `ServiceEndpoint` 与 `ServiceInstanceEndpoint` 必须分开
- `ServiceDependency` 与 `NetworkAttachment` 必须分开
- 最终形成 `business / system / service / pod / host / network / software / responsibility` 的统一关系图谱

# warp-insight Host Inventory 与 Runtime State 设计

## 1. 文档目的

本文档定义 `warp-insight` 中主机层面的两类核心对象：

- 主机资产库存
- 主机运行时状态

这里的目标是固定一个长期稳定的分层边界，避免把“变化很慢的主机事实”和“变化很快的主机运行态”混成一个对象。

本文重点回答：

- 哪些主机字段属于 inventory
- 哪些主机字段属于 runtime state
- 为什么两者必须分库存储
- 边缘和中心分别负责什么
- `host / process / software` 在中心模型中如何关联

相关文档：

- [`../../edge/resource-discovery-runtime.md`](../../edge/resource-discovery-runtime.md)
- [`../../edge/discovery-vs-resource-state-current.md`](../../edge/discovery-vs-resource-state-current.md)
- [`../control-center-architecture.md`](../control-center-architecture.md)
- [`software-normalization-and-vuln-enrichment.md`](software-normalization-and-vuln-enrichment.md)
- [`../../foundation/architecture.md`](../../foundation/architecture.md)
- [`../../foundation/glossary.md`](../../foundation/glossary.md)

---

## 2. 核心结论

第一版固定以下结论：

- 主机静态信息与主机动态状态必须拆成两套对象
- 主机 inventory 属于中心资源目录的一部分
- 主机 runtime state 属于动态状态层，不应直接覆盖 inventory 主对象
- `host inventory` 更新频率低，`host runtime state` 更新频率高
- `process runtime state`、`container runtime state` 不应直接塞进主机 inventory
- 两层通过稳定 `host_id` 关联

一句话说：

- `HostInventory` 回答“这台主机是什么”
- `HostRuntimeState` 回答“这台主机现在怎么样”

---

## 3. 为什么必须拆开

如果把主机静态事实和动态状态写进同一个对象，会出现明显问题：

- 主对象被高频无意义重写
- 审计 diff 失真
- 查询时难以区分“资产事实”与“当前状态”
- 热路径写放大明显
- 长期目录对象与时序/快照对象耦合

因此必须明确：

- inventory 是目录层
- runtime state 是状态层

两者相关，但不能合并成单表或单对象。

---

## 4. 分层边界

### 4.1 `HostInventory`

这是主机的慢变化事实。

它表达：

- 主机身份
- 硬件与平台信息
- OS 与版本信息
- 稳定网络与资产属性
- 初次发现与最近 inventory 刷新时间

它不表达：

- 当前 CPU 使用率
- 当前 load average
- 当前内存水位
- 当前磁盘压力
- 当前运行中的进程列表

### 4.2 `HostRuntimeState`

这是主机的快变化状态。

它表达：

- 当前运行时负载
- 当前资源水位
- 当前 agent 健康
- 当前运行态计数
- 当前保护/退化状态

它不表达：

- 资产主键
- 厂商/型号等慢变化资产事实
- 主机所有权等目录信息

---

## 5. 对象模型

### 5.1 `HostInventory`

建议结构：

```text
HostInventory {
  host_id
  tenant_id
  environment_id
  host_name
  machine_id?
  serial_number?
  cloud_instance_id?
  vendor?
  model?
  arch
  os_name
  os_version?
  kernel_version?
  cpu_model?
  cpu_core_count?
  memory_total_bytes?
  disk_inventory?
  network_interface_inventory?
  first_seen_at
  last_inventory_at
  inventory_revision
}
```

关键约束：

- `host_id` 是中心内部主键
- `host_name` 不是唯一主键
- `last_inventory_at` 表示最近一次 inventory 刷新完成时间

### 5.2 `HostRuntimeState`

建议结构：

```text
HostRuntimeState {
  host_id
  observed_at
  boot_id?
  uptime_seconds?
  current_ip_set?
  loadavg_1m?
  loadavg_5m?
  loadavg_15m?
  cpu_usage_pct?
  memory_used_bytes?
  memory_available_bytes?
  disk_used_bytes?
  disk_available_bytes?
  network_rx_bytes?
  network_tx_bytes?
  process_count?
  container_count?
  agent_health
  protection_state?
  last_error?
}
```

关键约束：

- `HostRuntimeState` 是时间点快照，不是目录对象
- 应允许频繁刷新
- 可做保留窗口、降采样和 TTL 清理

### 5.3 `HostRuntimeAggregate`

如果后续需要长期查询，也可保留聚合层：

```text
HostRuntimeAggregate {
  host_id
  time_bucket
  avg_cpu_usage_pct
  max_memory_used_bytes
  avg_loadavg_1m
  max_process_count
  protection_entered
}
```

这层是派生层，不是 source of truth。

---

## 6. 更新策略

### 6.1 inventory 更新

建议触发条件：

- 首次发现
- 主机重命名
- OS/内核升级
- 网络接口拓扑明显变化
- 云实例重新绑定
- 周期性低频 inventory refresh

特点：

- 低频
- 重视稳定性和幂等
- 允许人工审阅资产变更

### 6.2 runtime state 更新

建议触发条件：

- metrics probe 周期刷新
- agent heartbeat 周期刷新
- 保护模式状态切换
- 关键错误事件触发刷新

特点：

- 高频
- 可丢部分历史点
- 优先支持当前态和短期窗口查询

---

## 7. 边缘与中心分工

### 7.1 边缘负责什么

边缘负责：

- 发现 `host` 的基础事实
- 采集主机运行态指标
- 产出本地 `host` 资源引用
- 周期性上报 host runtime evidence

边缘不负责：

- 全局 inventory merge
- 多节点 host 去重
- 长期资产目录维护

### 7.2 中心负责什么

中心负责：

- 建立 `HostInventory`
- 维护 `HostRuntimeState`
- 做静态 inventory merge
- 做动态状态存储和过期清理
- 对 `host / process / software` 建立中心关联关系

---

## 8. 与 discovery / runtime state 的关系

现有边缘语义已经固定为：

- `discovery`
  回答“本机有什么对象”
- `runtime state`
  回答“这些对象当前是什么状态”

主机层落到中心后应继续保持同样边界：

- `DiscoveredResource(kind=host)` -> 主机对象身份与最小事实
- `HostInventory` -> 主机静态目录对象
- `HostRuntimeState` -> 主机动态快照对象

不应把 discovery cache 直接当作最终 inventory。

---

## 9. 与 `process` / `software` 的关系

建议中心侧固定如下关系：

```text
HostInventory
  -> ProcessRuntimeState[]
  -> ContainerRuntimeState[]
  -> SoftwareEvidence[]

SoftwareEvidence
  -> SoftwareEntity
  -> SoftwareVulnerabilityFindings[]
```

关键点：

- `HostInventory` 是宿主对象
- `ProcessRuntimeState` 是主机上的动态对象
- `SoftwareEntity` 不是主机字段，而是独立实体

因此不要把：

- 进程列表
- 软件列表
- 漏洞列表

直接塞进 `HostInventory` 主对象里。

---

## 10. 存储建议

第一版建议至少拆成：

- `host_inventory`
- `host_runtime_state`
- `process_runtime_state`
- `software_evidence`

如果后续需要时序和窗口聚合，再增加：

- `host_runtime_aggregate`

### 10.1 `host_inventory`

适合：

- 关系型元数据存储
- 资源目录库

### 10.2 `host_runtime_state`

适合：

- 状态快照表
- 时序库
- TTL 保留策略

---

## 11. 查询语义

建议对外查询明确分成两类：

### 11.1 inventory 查询

回答：

- 主机是谁
- 主机规格是什么
- 主机属于哪个环境
- 主机最近一次 inventory 是什么时候

### 11.2 runtime 查询

回答：

- 主机当前负载如何
- agent 当前状态如何
- 最近一段时间是否进入 `degraded/protect`
- 当前进程/容器数量是多少

不应把这两种查询混成同一个默认接口对象。

---

## 12. 第一版落地范围

第一版建议只做：

1. `HostInventory`
2. `HostRuntimeState`
3. `host_id` 关联
4. 与 `ProcessRuntimeState`、`SoftwareEvidence` 的最小关联

第一版不建议一开始就做：

- 完整 CMDB 替代
- 复杂历史版本对比 UI
- 大规模时间序列分析引擎

---

## 13. 验收标准

第一版至少应满足：

- 同一主机的 inventory 与 runtime state 可稳定通过 `host_id` 关联
- inventory 高频不被 runtime 写入污染
- runtime state 可独立周期刷新
- `process` / `software` 不直接污染 host inventory 主对象
- 查询时能明确区分静态资产与动态状态

---

## 14. 当前建议

当前建议固定为：

- 主机层必须采用“双层模型”
- 静态层是 `HostInventory`
- 动态层是 `HostRuntimeState`
- 两者通过 `host_id` 关联，但分别存储、分别刷新、分别查询

一句话总结：

`warp-insight` 应把主机设计成“一个稳定库存对象 + 多个运行态快照对象”，而不是一个不断被高频重写的大对象。

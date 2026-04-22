# warp-insight Host Inventory 与 Runtime State Schema 草案

## 1. 文档目的

本文档定义 `HostInventory` 和 `HostRuntimeState` 的字段级 schema 草案。

这里的目标是把：

- 主机静态资产事实
- 主机动态运行态快照

明确拆成两个对象，并给出第一版字段、约束和更新语义。

相关文档：

- [`host-inventory-and-runtime-state.md`](host-inventory-and-runtime-state.md)
- [`../../edge/resource-discovery-runtime.md`](../../edge/resource-discovery-runtime.md)
- [`../report-discovery-snapshot-schema.md`](../report-discovery-snapshot-schema.md)

---

## 2. 核心结论

第一版固定：

- `HostInventory` 是目录对象
- `HostRuntimeState` 是状态对象
- 两者通过 `host_id` 关联
- 两者不能合并成一个 schema

---

## 3. `HostInventory`

建议结构：

```text
HostInventory {
  api_version
  kind
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
  disks[]?
  network_interfaces[]?
  first_seen_at
  last_inventory_at
  inventory_revision
}
```

### 3.1 固定值

- `api_version = "v1"`
- `kind = "host_inventory"`

### 3.2 必选字段

- `api_version`
- `kind`
- `host_id`
- `tenant_id`
- `environment_id`
- `host_name`
- `arch`
- `os_name`
- `first_seen_at`
- `last_inventory_at`
- `inventory_revision`

### 3.3 字段要求

- `host_id`
  - 中心内部稳定主键
  - 不允许为空
- `inventory_revision`
  - 单主机单调递增
- `last_inventory_at`
  - 应不早于 `first_seen_at`

### 3.4 `disks[]`

建议结构：

```text
DiskInventoryItem {
  disk_id
  kind?
  capacity_bytes?
  mount_points[]?
}
```

### 3.5 `network_interfaces[]`

建议结构：

```text
NetworkInterfaceInventoryItem {
  interface_id
  name
  mac_address?
  addresses[]?
}
```

---

## 4. `HostRuntimeState`

建议结构：

```text
HostRuntimeState {
  api_version
  kind
  host_id
  observed_at
  boot_id?
  uptime_seconds?
  current_ip_set[]?
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
  degraded_reason?
  last_error?
}
```

### 4.1 固定值

- `api_version = "v1"`
- `kind = "host_runtime_state"`

### 4.2 必选字段

- `api_version`
- `kind`
- `host_id`
- `observed_at`
- `agent_health`

### 4.3 枚举字段

`agent_health` 建议取值：

- `healthy`
- `degraded`
- `protect`
- `unavailable`

`protection_state` 建议取值：

- `normal`
- `degraded`
- `protect`

### 4.4 时间语义

- `HostRuntimeState` 允许同一 `host_id` 存多条记录
- 主键不应只靠 `host_id`
- 建议唯一键：
  - `host_id + observed_at`

---

## 5. 与边缘对象的映射

### 5.1 `DiscoveredResource(kind=host)` 到 `HostInventory`

主要映射：

- `resource_id -> host_id` 候选
- `attributes -> inventory` 最小事实

### 5.2 `metrics_runtime_snapshot / samples` 到 `HostRuntimeState`

主要映射：

- `host metrics` 当前样本 -> runtime state 快照
- `agent health` / `protection state` -> runtime state 运行状态

---

## 6. 第一版限制

第一版不建议把以下内容直接并入 `HostInventory`：

- 当前 CPU 使用率
- 当前 load average
- 当前主机上运行的 process 细表
- 当前软件漏洞列表

这些内容应进入：

- `HostRuntimeState`
- `ProcessRuntimeState`
- `SoftwareVulnerabilityFinding`

---

## 7. 当前建议

当前建议固定为：

- `HostInventory` 只承载慢变化主机事实
- `HostRuntimeState` 只承载高频动态快照
- `host_id` 是两者的唯一关联键

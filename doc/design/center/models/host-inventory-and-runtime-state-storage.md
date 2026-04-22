# warp-insight Host Inventory 与 Runtime State 存储设计草案

## 1. 文档目的

本文档把 `HostInventory` 和 `HostRuntimeState` 进一步收敛到中心存储层。

重点回答：

- 哪些字段应进关系库
- 哪些状态应进快照表或时序存储
- 如何建主键、索引和保留策略
- 如何与 `process`、`software` 和漏洞 finding 关联

相关文档：

- [`host-inventory-and-runtime-state.md`](host-inventory-and-runtime-state.md)
- [`host-inventory-and-runtime-state-schema.md`](host-inventory-and-runtime-state-schema.md)
- [`../control-center-storage-schema.md`](../control-center-storage-schema.md)

---

## 2. 核心结论

第一版建议：

- `host_inventory` 进关系型元数据存储
- `host_runtime_state` 进动态快照表或时序存储
- `host_runtime_aggregate` 可选
- 不建议把 inventory 和 runtime state 合到一张表

---

## 3. `host_inventory`

用途：

- 保存主机稳定目录信息

建议字段：

- `host_id`
- `tenant_id`
- `environment_id`
- `host_name`
- `machine_id`
- `serial_number`
- `cloud_instance_id`
- `vendor`
- `model`
- `arch`
- `os_name`
- `os_version`
- `kernel_version`
- `cpu_model`
- `cpu_core_count`
- `memory_total_bytes`
- `inventory_blob_ref?`
- `first_seen_at`
- `last_inventory_at`
- `inventory_revision`

建议唯一约束：

- `host_inventory(host_id)`

建议索引：

- `(tenant_id, environment_id, host_name)`
- `(cloud_instance_id)`
- `(machine_id)`

---

## 4. `host_runtime_state`

用途：

- 保存主机动态状态快照

建议字段：

- `host_id`
- `observed_at`
- `boot_id`
- `uptime_seconds`
- `loadavg_1m`
- `loadavg_5m`
- `loadavg_15m`
- `cpu_usage_pct`
- `memory_used_bytes`
- `memory_available_bytes`
- `disk_used_bytes`
- `disk_available_bytes`
- `network_rx_bytes`
- `network_tx_bytes`
- `process_count`
- `container_count`
- `agent_health`
- `protection_state`
- `degraded_reason`
- `last_error`
- `runtime_blob_ref?`

建议唯一约束：

- `host_runtime_state(host_id, observed_at)`

建议索引：

- `(host_id, observed_at desc)`
- `(observed_at desc)`
- `(agent_health, observed_at desc)`

---

## 5. 保留与清理策略

### 5.1 inventory

- 长期保留
- revision 变更可审计
- 可只保留最新版本 + 变更历史摘要

### 5.2 runtime state

- 高频写入
- 建议 TTL 或冷热分层
- 例如只保留明细 `7-30` 天

### 5.3 aggregate

如需长期趋势分析，可增加聚合表：

- `1m`
- `5m`
- `1h`

---

## 6. 与其他对象的关系

### 6.1 与 `process_runtime_state`

建议：

- `process_runtime_state.host_id -> host_inventory.host_id`

### 6.2 与 `software_evidence`

建议：

- `software_evidence.host_id -> host_inventory.host_id`

### 6.3 与 `software_vulnerability_findings`

不直接挂在 `host_inventory` 表上，建议通过：

- `host_id -> software_evidence -> software_id -> findings`

---

## 7. 当前建议

当前建议固定为：

- 主机目录对象与主机动态状态必须分库存储
- inventory 走关系库主表
- runtime state 走快照/时序表

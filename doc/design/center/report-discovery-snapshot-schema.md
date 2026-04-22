# warp-insight ReportDiscoverySnapshot Schema 草案

## 1. 文档目的

本文档定义边缘 `warp-insightd` 向中心同步 discovery 本地快照时，通过 `warp-parse` discovery ingress receiver 投递的消息 envelope。

重点回答：

- `state/discovery/*.json` 是否应直接作为上送协议
- discovery 资源事实是否应复用 telemetry 数据面上送路径
- discovery 本地快照如何包装成控制面同步对象
- 中心如何做幂等接收、落库与后续资源目录归并

相关文档：

- [`../edge/resource-discovery-runtime.md`](../edge/resource-discovery-runtime.md)
- [`../edge/discovery-runtime-current-state.md`](../edge/discovery-runtime-current-state.md)
- [`discovery-sync-protocol.md`](discovery-sync-protocol.md)
- [`../telemetry/telemetry-uplink-and-warp-parse.md`](../telemetry/telemetry-uplink-and-warp-parse.md)
- [`control-center-storage-schema.md`](control-center-storage-schema.md)
- [`../foundation/glossary.md`](../foundation/glossary.md)

---

## 2. 核心结论

当前阶段固定以下结论：

- `DiscoverySnapshot` 是边缘本地事实快照，不是中心传输对象
- `state/discovery/resources.json` / `targets.json` / `meta.json` 是本地状态文件，不是对外协议
- discovery 资源事实同步应走 `warp-parse` 的 discovery ingress 协议
- discovery sync 不应默认复用 telemetry 数据面 uplink payload
- 但必须定义独立的 discovery envelope，而不是复用 `TelemetryRecord` 或 `ReportActionResult`
- 中心接收到的是边缘快照事实；中心资源目录仍需单独做归并、去重和生命周期管理

一句话说：

- discovery 要上送中心
- 但它上送的是 `ReportDiscoverySnapshot`
- 不是直接发送 `state/discovery/*.json`
- 也不是把资源事实伪装成 telemetry record

---

## 3. 角色边界

### 3.1 `DiscoverySnapshot`

`DiscoverySnapshot` 的统一定义见 [`../foundation/glossary.md`](../foundation/glossary.md)。

这里再次强调：

- 它是边缘 discovery 运行时产出的完整本地快照
- 它表达“本机当前发现到了哪些 resource / target”
- 它适用于边缘 cache、planner bridge 和本地只读查询

它不等于：

- 中心侧资源目录记录
- telemetry record
- capability report
- discovery sync transport envelope

### 3.2 `ReportDiscoverySnapshot`

本文定义的新对象：

- `ReportDiscoverySnapshot` 是控制面传输对象
- 用于把一轮边缘 `DiscoverySnapshot` 回报给中心

两者关系应固定为：

- `DiscoverySnapshot` 描述边缘本地事实
- `ReportDiscoverySnapshot` 描述边缘如何把该事实同步给中心

### 3.3 中心资源目录

中心侧最终资源目录仍是独立对象层：

- 它消费多个 agent 上送的 discovery 快照
- 它负责跨节点归并、去重、生命周期管理和关系补全

因此必须避免把：

- 边缘本地快照
- discovery 上送 envelope
- 中心资源目录记录

这三层混成一个对象。

---

## 4. 为什么不能复用 telemetry uplink

discovery sync 与 telemetry uplink 的对象语义不同。

### 4.1 telemetry uplink 解决的问题

telemetry uplink 解决的是：

- 日志
- 指标样本
- traces / security event
- 统一数据接入和下游路由

它传输的是：

- data-plane signal
- event / record / sample

### 4.2 discovery sync 解决的问题

discovery sync 解决的是：

- 边缘资源事实同步
- 控制面 target 选择参考
- 中心资源目录归并输入

它传输的是：

- control-plane resource fact snapshot

### 4.3 设计约束

因此第一版应固定：

- discovery sync 可以进入 `warp-parse`
- discovery sync 不复用 `TelemetryRecord`
- discovery sync 不把 `DiscoveredResource` / `DiscoveredTarget` 降级成一行文本 record

更准确地说：

- discovery sync 走 `warp-parse` 的专用 discovery receiver
- 但不走现有 telemetry record 接入语义

允许复用的包括：

- `warp-parse` ingress runtime
- 独立 discovery 端口
- 协议头校验
- 持久化接收
- ack 驱动的本地清理机制

---

## 5. 顶层结构

建议边缘向 `warp-parse` discovery receiver 上送以下对象：

```text
ReportDiscoverySnapshot {
  api_version
  kind
  report_id
  agent_id
  instance_id
  snapshot_id
  revision
  generated_at
  report_attempt
  report_mode
  reported_at
  snapshot
}
```

### 5.1 固定值

- `api_version = "v1"`
- `kind = "report_discovery_snapshot"`

### 5.2 必选字段

- `api_version`
- `kind`
- `report_id`
- `agent_id`
- `instance_id`
- `snapshot_id`
- `revision`
- `generated_at`
- `report_attempt`
- `report_mode`
- `reported_at`
- `snapshot`

---

## 6. 字段说明

### 6.1 `report_id`

建议类型：

- `string`

用途：

- 标识一次 discovery 快照回报尝试

注意：

- 它是传输尝试对象 id
- 不应拿它替代 `snapshot_id`

### 6.2 `agent_id`

建议类型：

- `string`

用途：

- 标识逻辑 agent 身份

### 6.3 `instance_id`

建议类型：

- `string`

用途：

- 标识本次 daemon 实例

说明：

- 中心可以借此区分同一 `agent_id` 的不同实例生命周期
- 但 discovery 快照幂等主语义仍应优先落在 `agent_id + snapshot_id` 或 `agent_id + revision`

### 6.4 `snapshot_id`

建议类型：

- `string`

说明：

- 直接引用边缘本轮 `DiscoverySnapshot.snapshot_id`
- 用于唯一标识某一轮边缘 discovery 快照

### 6.5 `revision`

建议类型：

- `uint64`

说明：

- 直接引用边缘本轮 `DiscoverySnapshot.revision`
- 用于表达同一 agent 本地 discovery 快照的推进顺序

### 6.6 `generated_at`

建议类型：

- RFC3339 UTC 时间

说明：

- 直接引用边缘本轮 `DiscoverySnapshot.generated_at`
- 表示边缘快照形成时间，不是消息发送时间

### 6.7 `report_attempt`

建议类型：

- `uint32`

说明：

- 表示边缘针对同一 `snapshot_id` 第几次回报
- 从 `1` 开始

### 6.8 `report_mode`

建议类型：

- `string`

第一版建议枚举：

- `full_snapshot`
- `snapshot_replace`

第一版建议固定：

- 先只实现 `full_snapshot`

含义说明：

- `full_snapshot`
  本次 payload 携带完整 `DiscoverySnapshot`
- `snapshot_replace`
  语义上表示用该完整快照替代中心对该 agent 最近一次已确认快照

说明：

- 第一版不建议设计部分 patch / delta 协议
- 先把“完整快照幂等替换”跑通

### 6.9 `reported_at`

建议类型：

- RFC3339 UTC 时间

说明：

- 表示本次 `ReportDiscoverySnapshot` 创建时间
- 它是传输时间，不等于 `generated_at`

### 6.10 `snapshot`

```text
snapshot: DiscoverySnapshot
```

说明：

- 直接复用 discovery 契约对象
- payload 内包含：
  - `resources[]`
  - `targets[]`
- `meta.json` 中的额外本地缓存字段不要求原样进入线上协议

---

## 7. payload 约束

### 7.1 允许携带的内容

第一版建议只上传：

- `DiscoverySnapshot` 中的 `resources[]`
- `DiscoverySnapshot` 中的 `targets[]`
- 与该快照强绑定的版本与时间字段

### 7.2 不建议直接上传的内容

第一版不建议把以下本地状态文件语义直接外露为协议字段：

- `meta.last_success_at`
- `meta.last_error`
- probe 局部故障细节
- 本地临时 cache 恢复原因

原因：

- 这些字段主要用于边缘运行时恢复与排障
- 不属于中心资源事实目录的核心主语义
- 若中心需要 agent 健康与故障信息，应走独立的 health / self-observability 通道

### 7.3 target 上送约束

第一版允许同步 `targets[]`，因为它们可服务：

- 控制面只读查询
- target 选择辅助
- 中心理解边缘“哪些对象可作为候选采集对象”

但必须明确：

- `DiscoveredTarget` 仍是 discovery 事实对象
- 它不是最终 `CandidateCollectionTarget`
- 中心不应把它直接等同于已下发采集配置

---

## 8. ack 与幂等

discovery sync 需要显式 ack。

建议中心返回：

```text
DiscoveryIngestAck {
  api_version
  kind
  report_id
  agent_id
  instance_id
  snapshot_id
  revision
  ack_status
  accepted_at
  reason_code?
  reason_message?
}
```

### 8.1 固定值

- `kind = "discovery_ingest_ack"`

### 8.2 `ack_status`

第一版建议枚举：

- `accepted`
- `duplicate`
- `stale`
- `rejected`

含义说明：

- `accepted`
  `warp-parse` 已可靠接收并完成当前快照落地
- `duplicate`
  相同 `snapshot_id` 或等价快照已处理
- `stale`
  收到的 `revision` 明显落后于中心已确认 revision
- `rejected`
  envelope 非法、schema 不匹配或身份不合法

### 8.3 幂等键建议

第一版建议 discovery ingress 至少支持以下幂等约束：

- `discovery_reports(report_id)` 唯一
- `agent_discovery_snapshots(agent_id, snapshot_id)` 唯一

同时建议中心维护：

- 每个 `agent_id` 当前最新已确认 `revision`

### 8.4 边缘重试原则

边缘应按如下原则处理：

1. 为同一 `snapshot_id` 保留本地待确认 reporting 状态
2. 未收到 `accepted` / `duplicate` 前允许重试
3. 收到 `duplicate` 后视为可安全清理本地待确认项
4. 收到 `stale` 时停止重试该快照，并记录控制面可见原因

---

## 9. 发送时机

第一版建议仅在以下场景发送：

### 9.1 首次成功形成快照后

- daemon 启动后首轮成功 discovery refresh

### 9.2 revision 前进后

- 仅当形成新的 `DiscoverySnapshot.revision` 时上送

### 9.3 会话恢复后补发未确认快照

- 若之前已形成本地 reporting 状态但未收到 ack
- 会话恢复后允许按原 `snapshot_id` 重试发送

### 9.4 不建议的发送时机

第一版不建议：

- 每次 tick 无条件重发最近快照
- 把 discovery sync 做成高频心跳负载
- 为单个 resource 变化设计即时碎片化单条上报

---

## 10. 中心落库分层

中心接收 `ReportDiscoverySnapshot` 后，建议拆成两层：

### 10.1 discovery ingress receipt

用途：

- 保存原始 ingress envelope 的接收事实
- 支撑幂等、审计、排障和重试判定

建议字段：

- `report_id`
- `agent_id`
- `instance_id`
- `snapshot_id`
- `revision`
- `reported_at`
- `received_at`
- `ack_status`
- `report_blob_ref`

### 10.2 agent scoped discovery snapshot

用途：

- 保存某个 agent 最近一次已确认的 discovery 快照索引

建议字段：

- `agent_id`
- `instance_id`
- `snapshot_id`
- `revision`
- `generated_at`
- `snapshot_blob_ref`
- `is_current`

### 10.3 center resource catalog

用途：

- 基于多个 agent 的 discovery snapshot 做归并后的中心资源目录

注意：

- 该层不是 `ReportDiscoverySnapshot` 的直接镜像表
- 需要独立的归并任务和生命周期规则

---

## 11. 与其它控制面对象的关系

### 11.1 与 telemetry uplink

- telemetry uplink 面向 logs / metrics / traces / security
- `ReportDiscoverySnapshot` 面向 resource fact snapshot

两者都可进入 `warp-parse`，但必须走不同 receiver 语义。

### 11.2 与 `CapabilityReport`

- `CapabilityReport`
  表达“本机能做什么”
- `ReportDiscoverySnapshot`
  表达“本机当前发现到了什么”

二者语义独立，不应混为一个对象。

### 11.3 与 `ReportActionResult`

- `ReportActionResult` 回报的是一次动作执行结果
- `ReportDiscoverySnapshot` 回报的是一轮 discovery 事实快照

两者都可复用 reporting / ack / retry 思路，但不应复用同一 payload。

---

## 12. 第一版实现建议

建议按以下顺序落地：

### 12.1 Step 1

先固定合同与网关消息：

- `report_discovery_snapshot`
- `discovery_ingest_ack`

### 12.2 Step 2

再补本地 reporting 状态：

- 待发送快照索引
- `report_attempt`
- ack 后清理

### 12.3 Step 3

再补中心接收与幂等：

- ingress receipt 落库
- snapshot 当前视图更新
- stale / duplicate 判定

### 12.4 Step 4

最后补中心资源目录归并：

- resource 去重
- 生命周期推进
- 跨 agent 关联

---

## 13. 最小示例

```json
{
  "api_version": "v1",
  "kind": "report_discovery_snapshot",
  "report_id": "disrep_01",
  "agent_id": "agent_prod_web_01",
  "instance_id": "inst_01",
  "snapshot_id": "discovery:42:2026-04-19T08:00:00Z",
  "revision": 42,
  "generated_at": "2026-04-19T08:00:00Z",
  "report_attempt": 1,
  "report_mode": "full_snapshot",
  "reported_at": "2026-04-19T08:00:02Z",
  "snapshot": {
    "schema_version": "v1",
    "snapshot_id": "discovery:42:2026-04-19T08:00:00Z",
    "revision": 42,
    "generated_at": "2026-04-19T08:00:00Z",
    "resources": [
      {
        "resource_id": "host:host-01",
        "kind": "host",
        "attributes": [
          {
            "key": "host.id",
            "value": "host-01"
          }
        ],
        "runtime_facts": [],
        "discovered_at": "2026-04-19T08:00:00Z",
        "last_seen_at": "2026-04-19T08:00:00Z",
        "health": "healthy",
        "source": "local_runtime"
      }
    ],
    "targets": [
      {
        "target_id": "host:host-01:host",
        "kind": "host",
        "resource_refs": [
          "host:host-01"
        ],
        "endpoint": null,
        "labels": [],
        "runtime_facts": [],
        "discovered_at": "2026-04-19T08:00:00Z",
        "last_seen_at": "2026-04-19T08:00:00Z",
        "state": "active",
        "source": "local_runtime"
      }
    ]
  }
}
```

---

## 14. 当前决定

当前阶段固定以下结论：

- discovery 上送中心是必要能力
- discovery 上送应定义为独立 ingress 协议对象
- 本地 `state/discovery/*.json` 不直接等于线上协议
- 第一版采用完整快照上送 + ack 幂等替换模型
- 中心按 ingress receipt、agent 当前快照、资源目录归并三层分离处理

# warp-insight 控制平面设计

## 1. 文档目的

本文档定义 `warp-insight` 的控制平面，重点回答以下问题：

- 中心节点与 `warp-insightd` 之间需要哪些控制对象
- 远程动作与升级编排在控制平面上如何建模
- `control` 与 `run` 如何绑定、审批、编译和下发
- `request_id / action_id / audit_id` 应如何贯穿控制链路
- 中心节点和边缘 agent 之间的状态机应如何设计

本文档建立在以下设计文档基础上：

- [`target.md`](../foundation/target.md)
- [`architecture.md`](../foundation/architecture.md)
- [`security-model.md`](../foundation/security-model.md)
- [`action-plan-ir.md`](../execution/action-plan-ir.md)
- [`action-dsl.md`](../execution/action-dsl.md)
- [`control-center-architecture.md`](control-center-architecture.md)
- [`control-center-storage-schema.md`](control-center-storage-schema.md)
- [`agent-gateway-protocol.md`](agent-gateway-protocol.md)
- [`dispatch-action-plan-schema.md`](dispatch-action-plan-schema.md)
- [`ack-action-plan-schema.md`](ack-action-plan-schema.md)
- [`report-action-result-schema.md`](report-action-result-schema.md)
- [`references.md`](../foundation/references.md)

---

## 2. 控制平面的角色

控制平面不是“发命令接口”的同义词，而是中心节点用来治理边缘 agent 的完整机制。

第一版控制平面至少承担以下职责：

- agent 注册与租约管理
- capability 协商
- 策略下发
- 动作请求受理
- 审批编排
- `control + run` 绑定
- ActionPlan IR 编译与签名
- 升级计划编排
- 执行状态跟踪
- 结果回传与审计归档

---

## 3. 控制平面边界

### 3.1 中心负责什么

中心节点负责：

- 定义策略
- 受理请求
- 做权限与审批校验
- 编译执行计划
- 分发给目标 agent
- 聚合状态与结果
- 归档审计

### 3.2 边缘负责什么

`warp-insightd` 负责：

- 维护与中心的受信连接
- 上报能力与状态
- 接收已签名的最终计划
- 校验来源、签名、过期时间和 capability
- 拉起 `warp-insight-exec` 或 `warp-insight-upgrader`
- 回传状态和结果

### 3.3 边界原则

- 边缘不负责编排审批
- 边缘不直接理解作者 DSL
- 中心不直接控制 `warp-insight-exec`
- 高风险动作必须经过控制平面完整链路

---

## 4. 控制对象总览

第一版建议引入以下核心对象。

### 4.1 AgentRegistryEntry

表示一个环境内 agent 的注册状态。

建议字段：

- `agent_id`
- `tenant_id`
- `environment_id`
- `node_id`
- `instance_id`
- `version`
- `capabilities`
- `health_state`
- `last_seen_at`
- `lease_expires_at`
- `policy_version`

### 4.2 CapabilityReport

表示 agent 当前可支持的 opcode、发现器、输入源和升级能力。

建议字段：

- `agent_id`
- `instance_id`
- `reported_at`
- `opcodes[]`
- `input_types[]`
- `discovery_types[]`
- `upgrade_features[]`
- `limits`

### 4.3 PolicySet

表示某个租户 / 环境 / 节点范围内生效的治理策略。

建议字段：

- `policy_id`
- `tenant_scope`
- `environment_scope`
- `node_selector`
- `disabled_opcodes[]`
- `risk_rules`
- `approval_rules`
- `limit_rules`
- `allow_rules`
- `effective_from`

### 4.4 ActionTemplate

表示一个可复用动作模板，由两部分组成：

- `ControlTemplate`
- `RunSpec`

建议字段：

- `template_id`
- `name`
- `version`
- `control_ref`
- `run_ref`
- `owner`
- `default_risk`
- `supported_targets`
- `required_capabilities`

### 4.5 ActionRequest

表示一次实际动作请求。

建议字段：

- `request_id`
- `tenant_id`
- `environment_id`
- `requested_by`
- `template_id`
- `target_selector`
- `input_args`
- `reason`
- `requested_at`
- `request_state`

### 4.6 ApprovalRecord

表示一次审批记录。

建议字段：

- `approval_ref`
- `request_id`
- `risk_level`
- `required_policy`
- `approved_by`
- `approved_at`
- `expires_at`
- `approval_state`
- `comment`

### 4.7 ActionPlan

表示中心编译完成、可下发到边缘的最终执行计划。

这是边缘真正消费的控制对象。

建议字段：

- `meta`
- `target`
- `constraints`
- `program`
- `attestation`
- `provenance`

### 4.8 DispatchRecord

表示中心把某个 `ActionPlan` 分发给某个目标 agent 的记录。

建议字段：

- `dispatch_id`
- `action_id`
- `plan_digest`
- `agent_id`
- `instance_id`
- `dispatched_at`
- `delivery_state`
- `delivery_attempt`
- `ack_at`

### 4.9 ActionExecution

表示边缘节点上的一次执行实例。

建议字段：

- `execution_id`
- `action_id`
- `plan_digest`
- `agent_id`
- `instance_id`
- `executor_instance_id`
- `started_at`
- `finished_at`
- `execution_state`
- `exit_reason`

### 4.10 ActionResult

表示执行结果。

建议字段：

- `action_id`
- `execution_id`
- `final_status`
- `step_records`
- `outputs`
- `resource_usage`
- `finished_at`

### 4.11 AuditRecord

表示控制平面中的审计记录。

建议字段：

- `audit_id`
- `request_id`
- `action_id`
- `execution_id`
- `actor`
- `event_type`
- `event_time`
- `event_detail`

---

## 5. 主键与关联关系

控制平面中至少要固定以下主键：

- `request_id`
- `approval_ref`
- `action_id`
- `plan_digest`
- `dispatch_id`
- `execution_id`
- `audit_id`

推荐关系如下：

```text
ActionTemplate --> ActionRequest --> ApprovalRecord --> ActionPlan --> DispatchRecord
                                                  \--> AuditRecord
ActionPlan --> ActionExecution --> ActionResult --> AuditRecord
```

建议约束：

- 一个 `ActionRequest` 可以生成多个 `ActionPlan`
  例如目标节点被展开为多台 agent
- 一个 `ActionPlan` 对应一个目标 agent 的一次下发对象
- 一个 `ActionPlan` 最终应对应零次或一次实际执行实例
- 同一 `action_id + plan_digest` 不应在同一目标 agent 上产生多次实际执行
- 所有高风险状态变化都必须产出 `AuditRecord`

---

## 6. `control + execution spec` 到 ActionPlan 的编译链

### 6.1 输入

控制平面在编译时至少读取以下输入：

- 平台基线策略
- 环境 / 租户策略
- `ControlTemplate`
- `ExecutionSpec`
- `ActionRequest`
- `ApprovalRecord`
- `CapabilityReport`

### 6.2 编译步骤

建议编译过程如下：

1. 加载动作模板
2. 解析 `control` 与 `execution spec`
3. 合成最终 `control`
4. 检查风险等级和审批要求是否满足
5. 检查目标范围是否合法
6. 根据目标 agent capability 做兼容性筛选
7. 把 `execution spec` 编译成 `program`
8. 生成目标节点级 `ActionPlan`
9. 对 `ActionPlan` 做签名与过期时间绑定

### 6.3 编译失败原因

以下情况建议直接拒绝生成 `ActionPlan`：

- 使用了禁用 opcode
- 审批缺失或已过期
- 目标节点 capability 不满足
- 请求参数越界
- `timeout` / `limits` 超出策略上限
- `control` 放宽了上层限制

---

## 7. 远程动作生命周期

### 7.1 ActionRequest 状态机

建议 `ActionRequest` 使用如下状态：

- `draft`
- `submitted`
- `under_policy_check`
- `waiting_approval`
- `approved`
- `compiling`
- `compiled`
- `dispatching`
- `running`
- `succeeded`
- `failed`
- `rejected`
- `cancelled`
- `expired`

建议流转如下：

```text
draft -> submitted -> under_policy_check
under_policy_check -> waiting_approval | approved | rejected
waiting_approval -> approved | rejected | expired
approved -> compiling -> compiled -> dispatching -> running
running -> succeeded | failed | cancelled
```

### 7.2 ActionPlan 状态机

建议 `ActionPlan` 使用如下状态：

- `created`
- `signed`
- `queued`
- `dispatched`
- `acked`
- `accepted`
- `running`
- `succeeded`
- `failed`
- `rejected`
- `expired`

这里的区别是：

- `ActionRequest` 是用户视角对象
- `ActionPlan` 是中心到边缘的执行视角对象

### 7.3 边缘执行状态机

建议边缘执行状态如下：

- `received`
- `validated`
- `spawned`
- `running`
- `collecting_result`
- `reported`
- `done`
- `failed`
- `timed_out`
- `denied`

---

## 8. 升级生命周期

升级在控制平面上建议与远程动作并行设计，但保留专门对象。

### 8.1 UpgradePlan

建议字段：

- `upgrade_id`
- `tenant_id`
- `environment_id`
- `target_selector`
- `channel`
- `target_version`
- `batch_policy`
- `health_gates`
- `rollback_policy`
- `requested_by`
- `approved_by`
- `upgrade_state`

### 8.2 Upgrade 状态机

建议状态：

- `draft`
- `validated`
- `approved`
- `batched`
- `dispatching`
- `running`
- `verifying`
- `completed`
- `paused`
- `rolling_back`
- `rolled_back`
- `failed`

### 8.3 与远程动作的区别

升级和远程动作都走控制平面，但区别在于：

- 升级更强调批次、健康门槛和回滚
- 远程动作更强调单次审批、风险等级和输出结果

---

## 9. 中心到边缘的协议对象

第一版控制平面协议建议围绕以下消息对象设计。

### 9.1 RegisterAgent

用途：

- agent 首次注册
- 上报版本与 capability

核心字段：

- `agent_id`
- `instance_id`
- `version`
- `capabilities`
- `boot_time`

### 9.2 AgentHeartbeat

用途：

- 周期上报健康与租约

核心字段：

- `agent_id`
- `instance_id`
- `health_state`
- `buffer_state`
- `protection_state`
- `lease_seq`

### 9.3 FetchPolicy / PolicySnapshot

用途：

- agent 拉取当前生效策略

核心字段：

- `policy_version`
- `limits`
- `disabled_opcodes`
- `allow_rules`

### 9.4 DispatchActionPlan

用途：

- 下发已签名 `ActionPlan`

核心字段：

- `action_id`
- `request_id`
- `constraints`
- `program`
- `attestation`
- `expires_at`
- `signature`

### 9.5 AckActionPlan

用途：

- agent 确认收到并校验通过或拒绝

核心字段：

- `action_id`
- `agent_id`
- `ack_state`
- `deny_reason`

### 9.6 ReportActionStatus

用途：

- 运行中状态上报

核心字段：

- `action_id`
- `execution_id`
- `execution_state`
- `progress`
- `resource_usage`

### 9.7 ReportActionResult

用途：

- 返回结构化结果

核心字段：

- `action_id`
- `plan_digest`
- `execution_id`
- `final_status`
- `step_records`
- `outputs`
- `result_attestation`
- `resource_usage`
- `reported_at`

### 9.8 DispatchUpgradePlan

用途：

- 下发升级计划

核心字段：

- `upgrade_id`
- `target_version`
- `batch_id`
- `health_gates`
- `signature`

---

## 10. `warp-insightd` 的控制面行为

`warp-insightd` 在控制平面里建议始终扮演“本地网关”角色。

### 10.1 接收动作计划时

`warp-insightd` 应按如下顺序处理：

1. 校验 `action_id / request_id`
2. 校验 `signature`
3. 校验 `expires_at`
4. 校验 `tenant / environment / target`
5. 校验本机 capability
6. 校验本地当前状态是否允许执行
7. 生成本地 `execution_id`
8. 拉起 `warp-insight-exec`

### 10.2 接收升级计划时

`warp-insightd` 应按如下顺序处理：

1. 校验 `upgrade_id`
2. 校验签名和版本来源
3. 校验本机状态、磁盘和 buffer 门槛
4. 生成本地升级执行上下文
5. 拉起 `warp-insight-upgrader`

### 10.3 本地拒绝条件

以下情况下，`warp-insightd` 应拒绝计划：

- 计划已过期
- 签名无效
- capability 不满足
- 本机处于保护模式且策略不允许
- 当前已有互斥执行任务
- 本地资源不足以安全执行

---

## 11. 审计与可追踪性

控制平面中的每一步关键状态变化都应写入审计链。

建议至少对以下事件记审计：

- 请求提交
- 策略校验通过 / 拒绝
- 审批通过 / 拒绝 / 过期
- 编译成功 / 失败
- 计划下发
- agent 接受 / 拒绝
- 开始执行
- 执行成功 / 失败 / 超时
- 升级开始 / 回滚

审计查询时至少应支持按以下维度检索：

- `request_id`
- `action_id`
- `execution_id`
- `upgrade_id`
- `agent_id`
- `tenant_id`
- `environment_id`

---

## 12. 幂等与重试

控制平面必须考虑中心重试和边缘断线重连。

建议原则：

- `request_id` 在用户语义层面幂等
- `action_id` 在单个目标 agent 维度幂等
- `dispatch_id` 在单次投递层面幂等
- agent 对相同 `action_id` 的重复投递必须可识别

也就是说：

- 中心可以重发
- 边缘不能重复执行同一个计划
- 结果回传允许重复，但必须可去重

---

## 13. 失败场景

第一版控制平面至少要处理以下失败场景：

- 审批通过后计划过期
- 编译成功但目标 agent 掉线
- 目标 agent 收到计划但 capability 不匹配
- 执行器启动失败
- 执行超时
- 结果已生成但回传失败
- 升级后健康检查失败，需要自动回滚

建议策略：

- 所有失败都要有明确状态
- 所有失败都要有审计记录
- 边缘拒绝必须带结构化原因
- 重试不能突破审批和过期边界

---

## 14. 第一阶段实现建议

建议按以下顺序落地控制平面：

### 14.1 Step 1

先实现远程动作最小闭环：

- `ActionTemplate`
- `ActionRequest`
- `ApprovalRecord`
- `ActionPlan`
- `ActionResult`

### 14.2 Step 2

再补齐 agent 协议：

- 注册
- 心跳
- capability 上报
- 计划下发
- 状态上报
- 结果回传

### 14.3 Step 3

再补齐升级编排：

- `UpgradePlan`
- 批次控制
- 健康门槛
- 回滚链路

### 14.4 Step 4

最后补齐治理和可视化：

- 审计检索
- 风险报表
- 审批看板
- 动作模板库

---

## 15. 当前结论

`warp-insight` 的控制平面应被明确设计为：

- 一个围绕 `request -> approval -> compile -> dispatch -> execute -> report -> audit` 的治理闭环
- 中心节点负责控制对象生成与状态机推进
- `warp-insightd` 只消费最终计划，不消费作者 DSL
- `warp-insight-exec` 与 `warp-insight-upgrader` 不直接挂接中心控制面
- 所有关键对象都必须有稳定主键、状态和审计关联

如果控制平面做不好，前面已经设计好的 `control/run` 分离、审批、风险分层、最小权限和边缘确定性都很难真正落地。

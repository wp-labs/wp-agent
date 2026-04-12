# wp-agent 控制中心存储设计草案

## 1. 文档目的

本文档把 [`control-center-architecture.md`](control-center-architecture.md) 中的控制中心模块进一步收敛到存储层。

重点回答：

- 控制中心需要哪些主存储对象
- 哪些对象应进关系库，哪些应进对象存储
- 各对象的主键、关联键和唯一约束是什么
- 哪些状态适合事件化，哪些状态必须事务落库

相关文档：

- [`control-center-architecture.md`](control-center-architecture.md)
- [`control-plane.md`](control-plane.md)
- [`dispatch-action-plan-schema.md`](dispatch-action-plan-schema.md)
- [`ack-action-plan-schema.md`](ack-action-plan-schema.md)
- [`report-action-result-schema.md`](report-action-result-schema.md)

---

## 2. 核心结论

控制中心第一版建议采用：

- 关系型元数据存储作为主库
- 对象存储保存 `ActionPlan` / `ActionResult` 原文
- outbox/event log 作为异步扩展基础

第一版不建议：

- 所有对象都只存 JSON 文档库
- 只靠消息总线维持状态机
- 让多个模块直接共享同一张“万能状态表”

---

## 3. 存储分层

### 3.1 关系库

建议存放：

- 身份与会话元数据
- 治理对象元数据
- 状态机摘要
- 检索索引
- 幂等键

### 3.2 对象存储

建议存放：

- `ActionPlan` 原文
- `ReportActionResult.result` 原文
- 审计附件
- 升级计划原文

### 3.3 事件日志 / outbox

建议承载：

- request lifecycle event
- dispatch lifecycle event
- execution lifecycle event
- audit event

---

## 4. 主键体系

第一版建议把以下键固定为一等主键：

- `tenant_id`
- `environment_id`
- `agent_id`
- `instance_id`
- `request_id`
- `approval_ref`
- `action_id`
- `plan_digest`
- `dispatch_id`
- `execution_id`
- `report_id`
- `audit_id`

### 4.1 关键唯一约束

建议至少固定以下唯一约束：

- `agent_registry(agent_id)`
- `agent_sessions(agent_id, instance_id)`
- `action_requests(request_id)`
- `approval_records(approval_ref)`
- `action_plans(action_id, plan_digest)`
- `dispatch_records(dispatch_id)`
- `action_executions(execution_id)`
- `result_reports(report_id)`

### 4.2 执行语义唯一约束

第一版建议显式约束：

- 在同一 `agent_id` 上，`action_id + plan_digest` 只能产生一个实际 execution 记录

原因：

- 这是控制中心侧对执行幂等的最终收口

---

## 5. 关系库对象建议

### 5.1 `agent_registry`

用途：

- 记录逻辑 agent 身份

建议字段：

- `agent_id`
- `tenant_id`
- `environment_id`
- `node_id`
- `registered_at`
- `last_known_version`
- `desired_policy_version?`
- `state`

### 5.2 `agent_sessions`

用途：

- 记录 agent 当前或最近实例会话

建议字段：

- `agent_id`
- `instance_id`
- `connected_at`
- `last_seen_at`
- `lease_expires_at`
- `health_state`
- `gateway_node_id?`

### 5.3 `capability_reports`

用途：

- 保存最近一次完整 capability 快照

建议字段：

- `agent_id`
- `instance_id`
- `reported_at`
- `report_blob_ref`
- `limits_summary_json`

说明：

- 完整 `CapabilityReport` 建议放对象存储
- 关系库存最常用索引字段和摘要

### 5.4 `policy_sets`

用途：

- 保存策略版本

建议字段：

- `policy_id`
- `tenant_scope`
- `environment_scope`
- `version`
- `effective_from`
- `state`
- `policy_blob_ref`

### 5.5 `action_templates`

用途：

- 保存模板和其版本

建议字段：

- `template_id`
- `version`
- `name`
- `owner`
- `default_risk`
- `supported_targets_json`
- `control_ref`
- `run_ref`
- `state`

### 5.6 `action_requests`

用途：

- 保存用户或自动化请求

建议字段：

- `request_id`
- `tenant_id`
- `environment_id`
- `template_id`
- `requested_by`
- `reason`
- `target_selector_json`
- `input_args_json`
- `request_state`
- `requested_at`

### 5.7 `approval_records`

用途：

- 保存审批过程与结果

建议字段：

- `approval_ref`
- `request_id`
- `risk_level`
- `approval_state`
- `approved_by?`
- `approved_at?`
- `expires_at?`
- `approval_detail_json`

### 5.8 `action_plans`

用途：

- 保存已编译计划的中心索引

建议字段：

- `action_id`
- `plan_digest`
- `request_id`
- `template_id?`
- `tenant_id`
- `environment_id`
- `agent_id`
- `instance_id?`
- `plan_version`
- `compiled_at`
- `expires_at`
- `plan_blob_ref`
- `signature_state`

建议唯一键：

- `(action_id, plan_digest)`

### 5.9 `dispatch_records`

用途：

- 保存计划投递状态

建议字段：

- `dispatch_id`
- `action_id`
- `plan_digest`
- `agent_id`
- `instance_id?`
- `delivery_attempt`
- `delivery_state`
- `dispatched_at`
- `ack_status?`
- `ack_at?`
- `execution_id?`

### 5.10 `action_executions`

用途：

- 保存边缘执行摘要

建议字段：

- `execution_id`
- `action_id`
- `plan_digest`
- `dispatch_id?`
- `agent_id`
- `instance_id`
- `execution_state`
- `final_status?`
- `started_at?`
- `finished_at?`
- `exit_reason?`

建议唯一键：

- `(agent_id, action_id, plan_digest)`

### 5.11 `result_reports`

用途：

- 保存结果上报 envelope 摘要

建议字段：

- `report_id`
- `execution_id`
- `action_id`
- `plan_digest`
- `agent_id`
- `report_attempt`
- `final_status`
- `result_digest`
- `result_signature`
- `reported_at`
- `result_blob_ref`

### 5.12 `audit_records`

用途：

- 保存统一审计索引

建议字段：

- `audit_id`
- `tenant_id`
- `environment_id`
- `agent_id?`
- `request_id?`
- `action_id?`
- `execution_id?`
- `event_type`
- `risk_level?`
- `actor`
- `event_time`
- `detail_blob_ref?`

---

## 6. 对象存储对象建议

第一版建议把以下原文保存到对象存储：

- `capability-reports/<agent_id>/<instance_id>/<ts>.json`
- `action-plans/<action_id>/<plan_digest>.json`
- `result-reports/<execution_id>/<report_id>.json`
- `audit-details/<audit_id>.json`
- `upgrade-plans/<upgrade_id>/<version>.json`

### 6.1 命名原则

建议遵循：

- 路径中带稳定主键
- 对象内容不可变
- 新版本写新对象，不原地覆盖

---

## 7. 状态机写入原则

### 7.1 强事务写入

以下更新建议在同一事务内完成：

- `ActionRequest` 创建
- `ApprovalRecord` 状态迁移
- `ActionPlan` 索引创建
- `DispatchRecord` 创建
- `ActionPlanAck` 摘要更新
- `ReportActionResult` 摘要更新

### 7.2 事件异步化

以下动作建议通过 outbox 触发：

- 审计事件扩散
- Webhook / 通知
- 搜索索引更新
- AI 摘要触发

---

## 8. 幂等与冲突处理

### 8.1 request 幂等

如需支持上层幂等提交，建议在 `action_requests` 额外支持：

- `idempotency_key`

### 8.2 dispatch 幂等

控制中心必须保证：

- `dispatch_id` 只代表一次投递尝试
- 不把新的 `dispatch_id` 误当成新的执行语义

### 8.3 result 幂等

控制中心处理 `ReportActionResult` 时建议按以下顺序去重：

1. `report_id`
2. `execution_id`
3. `action_id + plan_digest + result_digest`

---

## 9. 查询索引建议

第一版建议优先建立以下查询索引：

- agent 当前在线状态
- request 按租户 / 环境 / 时间检索
- execution 按 `agent_id` / `action_id` / `execution_id` 检索
- audit 按 actor / risk / time 检索
- dispatch backlog 检索

---

## 10. 第一版最小落地

第一版建议至少落下这些表或等价对象：

- `agent_registry`
- `agent_sessions`
- `capability_reports`
- `policy_sets`
- `action_templates`
- `action_requests`
- `approval_records`
- `action_plans`
- `dispatch_records`
- `action_executions`
- `result_reports`
- `audit_records`

---

## 11. 当前决定

当前阶段固定以下结论：

- 关系库是控制中心第一版主存储
- 大对象原文放对象存储
- `action_id + plan_digest` 是执行语义主索引之一
- 存储模型必须服务于控制闭环，而不是先追求通用平台大一统

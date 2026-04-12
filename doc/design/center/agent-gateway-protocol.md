# wp-agent Agent Gateway 协议设计

## 1. 文档目的

本文档定义控制中心 `Agent Gateway` 与边缘 `wp-agentd` 之间的南向协议。

重点回答：

- agent 与控制中心如何建立会话
- 控制中心如何向 agent 下发计划
- agent 如何 ack 和回报结果
- Gateway 应该承担哪些职责，不承担哪些职责

相关文档：

- [`control-center-architecture.md`](control-center-architecture.md)
- [`control-plane.md`](control-plane.md)
- [`capability-report-schema.md`](../edge/capability-report-schema.md)
- [`dispatch-action-plan-schema.md`](dispatch-action-plan-schema.md)
- [`ack-action-plan-schema.md`](ack-action-plan-schema.md)
- [`report-action-result-schema.md`](report-action-result-schema.md)
- [`security-model.md`](../foundation/security-model.md)

---

## 2. 核心结论

`Agent Gateway` 第一版建议提供一条中心到边缘的受信控制通路。

它至少要承载：

- register / resume
- heartbeat
- capability report
- `DispatchActionPlan`
- `ActionPlanAck`
- `ReportActionResult`

它不应承载：

- 作者 DSL
- 任意 shell
- 边缘本地调试 RPC

---

## 3. 设计原则

### 3.1 Gateway 是协议入口，不是编排器

Gateway 负责：

- 认证
- 会话
- 收发协议对象
- 投递状态更新

Gateway 不负责：

- 审批
- 编译
- 风险判断

### 3.2 一条受信控制流

第一版建议把下面这些消息都收敛到同一南向协议族：

- agent online messages
- plan dispatch messages
- ack/result messages

### 3.3 会话与对象分离

需要区分：

- 连接是否在线
- 计划是否已投递
- 计划是否已 ack
- 计划是否已执行完成

不能把“连接在线”误当成“动作已执行”。

---

## 4. 逻辑通道

第一版建议抽象成三类逻辑消息：

- `AgentHello`
- `AgentUpstreamMessage`
- `AgentDownstreamMessage`

### 4.1 `AgentHello`

用于建立或恢复 agent 会话。

### 4.2 `AgentUpstreamMessage`

边缘到中心的消息。

建议枚举：

- `heartbeat`
- `capability_report`
- `action_plan_ack`
- `report_action_result`
- `upgrade_ack`
- `upgrade_result`

### 4.3 `AgentDownstreamMessage`

中心到边缘的消息。

建议枚举：

- `dispatch_action_plan`
- `dispatch_upgrade_plan`
- `cancel_execution`
- `refresh_policy_hint`

说明：

- 第一版 `cancel_execution` 可以先保留占位
- 但不应阻塞 `DispatchActionPlan` 主链路

---

## 5. 会话建立

### 5.1 建连前提

`wp-agentd` 连接 Gateway 前必须具备：

- `agent_id`
- `instance_id`
- 双向身份认证材料

### 5.2 `AgentHello`

建议至少包含：

- `agent_id`
- `instance_id`
- `tenant_id`
- `environment_id`
- `node_id`
- `agent_version`
- `boot_id`
- `hello_time`
- `resume_token?`

### 5.3 Gateway 处理

Gateway 收到 `AgentHello` 后建议执行：

1. 完成双向认证
2. 校验 `agent_id / instance_id`
3. 创建或恢复 `agent_sessions`
4. 返回 `session_id` 或等价会话确认

### 5.4 会话恢复

第一版建议允许：

- 同一 `agent_id`
- 新 `instance_id`

被视为新实例上线，并使旧会话失效。

---

## 6. 心跳协议

### 6.1 用途

用于：

- 保持租约
- 刷新在线状态
- 返回最小运行摘要

### 6.2 建议字段

- `agent_id`
- `instance_id`
- `sent_at`
- `mode`
- `execution_queue_size`
- `running_executions`
- `reporting_executions`
- `agent_version`

### 6.3 处理原则

Gateway 不应因偶发一次心跳抖动就立即把 agent 标记永久离线。

建议区分：

- `online`
- `suspect`
- `offline`

---

## 7. CapabilityReport 上报

### 7.1 上报时机

第一版建议：

- 首次建连后立即上报
- agent 版本变化后上报
- 本地能力集变化后重上报

### 7.2 消息体

直接复用：

- [`capability-report-schema.md`](../edge/capability-report-schema.md)

### 7.3 Gateway 职责

Gateway 负责：

- 验证 envelope 基础字段
- 交给 Capability Catalog 落库与索引

Gateway 不负责：

- capability 语义决策

---

## 8. 计划下发协议

### 8.1 消息体

直接复用：

- [`dispatch-action-plan-schema.md`](dispatch-action-plan-schema.md)

### 8.2 下发前提

中心下发前应至少确认：

- agent 当前在线
- capability 预筛选通过
- `ActionPlan` 已签名
- 目标 `instance_id` 若绑定，则仍与当前会话一致

### 8.3 Gateway 职责

Gateway 负责：

- 从 Dispatch Service 接收待投递对象
- 投递到对应 agent 会话
- 更新投递尝试信息

Gateway 不负责：

- 重编译计划
- 修改计划内容

---

## 9. Ack 协议

### 9.1 消息体

直接复用：

- [`ack-action-plan-schema.md`](ack-action-plan-schema.md)

### 9.2 处理原则

Gateway 收到 ack 后建议：

1. 校验 `dispatch_id`
2. 校验 `action_id`
3. 校验 `plan_digest`
4. 更新 `dispatch_records`
5. 通知 Execution Tracker

### 9.3 关键区分

必须区分：

- `accepted`
- `queued`
- `duplicate`
- `rejected`

其中：

- `duplicate` 不等于执行失败
- `queued` 不等于已经开始执行

---

## 10. 结果回报协议

### 10.1 消息体

直接复用：

- [`report-action-result-schema.md`](report-action-result-schema.md)

### 10.2 Gateway 处理原则

Gateway 收到结果后建议：

1. 校验会话身份
2. 校验 `execution_id`
3. 校验 `action_id + plan_digest`
4. 校验 `result_attestation`
5. 交给 Result Ingestor 入库

### 10.3 幂等

Gateway 或 Result Ingestor 至少要支持以下幂等判断：

- `report_id`
- `execution_id`
- `action_id + plan_digest + result_attestation.result_digest`

---

## 11. 错误处理

第一版建议把 Gateway 错误分成四类：

- 认证错误
- 协议错误
- 路由错误
- 后端临时错误

### 11.1 认证错误

例如：

- mTLS 失败
- agent 身份不匹配

处理：

- 立即拒绝会话

### 11.2 协议错误

例如：

- 缺字段
- schema 不合法
- `dispatch_id` / `action_id` 不匹配

处理：

- 拒绝该消息
- 写审计

### 11.3 路由错误

例如：

- 会话不存在
- 目标 agent 不在线

处理：

- 交回 Dispatch Service 做重试或失败更新

### 11.4 后端临时错误

例如：

- 数据库短时不可写
- 对象存储抖动

处理：

- 优先返回可重试错误
- 不伪造成功 ack

---

## 12. 第一版传输建议

第一版建议满足以下要求：

- 双向认证
- 一条长连接控制通道
- 支持服务端主动下发
- 支持客户端主动上报

传输实现上可以是：

- gRPC bidirectional stream
- WebSocket over mTLS

当前更建议：

- gRPC bidirectional stream

原因：

- schema 化更自然
- 流式控制更清晰
- 后续扩展 cancel / upgrade / policy hint 更容易

---

## 13. Gateway 本地状态建议

Gateway 自身第一版建议维护：

- 当前在线会话表
- `agent_id -> session_id` 映射
- 最近心跳时间
- 未确认 dispatch 摘要

说明：

- 会话态允许主要在内存
- 关键投递状态必须落回主库

---

## 14. 第一版最小闭环

第一版 Gateway 至少要打通：

1. `AgentHello`
2. heartbeat
3. `CapabilityReport`
4. `DispatchActionPlan`
5. `ActionPlanAck`
6. `ReportActionResult`

如果这六步未打通，控制中心就还没有真正连上边缘。

---

## 15. 当前决定

当前阶段固定以下结论：

- `Agent Gateway` 是控制中心南向协议入口
- Gateway 负责认证、会话和消息转发，不负责编译和审批
- `DispatchActionPlan / ActionPlanAck / ReportActionResult` 是第一版核心协议对象
- 第一版优先采用双向流式长连接模型

# wp-agent DispatchActionPlan Schema 草案

## 1. 文档目的

本文档定义中心节点向边缘 `wp-agentd` 下发执行计划时使用的消息 envelope。

重点回答：

- `ActionPlan` 如何被包装成可投递对象
- 投递消息自身需要哪些最小字段
- 边缘 ack 的关联键应该是什么

相关文档：

- [`control-plane.md`](control-plane.md)
- [`action-plan-schema.md`](../execution/action-plan-schema.md)
- [`error-codes.md`](../edge/error-codes.md)

---

## 2. 核心结论

边缘不应接收作者侧源文件，也不应接收分离的 `control.wac` 和 `run.*`。

边缘接收的控制面投递对象应固定为：

- 一个 `DispatchActionPlan`
- 其中包含一个完整 `ActionPlan`

---

## 3. 顶层结构

```text
DispatchActionPlan {
  api_version
  kind
  dispatch_id
  action_id
  plan_digest
  agent_id
  instance_id?
  delivery
  plan
}
```

### 3.1 固定值

- `api_version = "v1alpha1"`
- `kind = "dispatch_action_plan"`

### 3.2 必选字段

- `api_version`
- `kind`
- `dispatch_id`
- `action_id`
- `plan_digest`
- `agent_id`
- `delivery`
- `plan`

### 3.3 字段约束

- `dispatch_id`: `string`, 必选
- `action_id`: `string`, 必选，必须与 `plan.meta.action_id` 一致
- `plan_digest`: `string`, 必选，必须与 `plan.attestation.plan_digest` 一致
- `agent_id`: `string`, 必选，必须与 `plan.target.agent_id` 一致
- `instance_id`: `string`, 可选，用于绑定具体 agent 实例

---

## 4. `delivery`

```text
DispatchDelivery {
  delivery_attempt
  priority?
  dispatched_at
  ack_deadline_at?
  expires_at?
  channel?
}
```

### 4.1 字段说明

- `delivery_attempt`
  第几次投递，从 `1` 开始
- `priority`
  第一版可选，整数值越小优先级越高
- `dispatched_at`
  中心真正下发时间
- `ack_deadline_at`
  期望边缘完成本地接收确认的截止时间
- `expires_at`
  整个 envelope 的投递过期时间
- `channel`
  可选，用于标记 `control_stream_v1` 一类的传输通道

### 4.2 第一版要求

- `ack_deadline_at` 不得晚于 `plan.meta.expires_at`
- 边缘不得仅凭 `delivery_attempt` 改变计划语义

---

## 5. `plan`

```text
plan: ActionPlan
```

说明：

- `plan` 内容直接复用 [`action-plan-schema.md`](../execution/action-plan-schema.md)
- `DispatchActionPlan` 只负责投递元信息，不重复定义 `ActionPlan` 内部语义

---

## 6. 边缘校验要求

`wp-agentd` 接收到 `DispatchActionPlan` 后，第一版至少要校验：

- `kind` / `api_version`
- `dispatch_id` 是否已见过
- `action_id + plan_digest` 是否已在本地 `execution_queue / running / reporting / history` 中出现
- `action_id` 与 `plan.meta.action_id` 是否一致
- `plan_digest` 与 `plan.attestation.plan_digest` 是否一致
- `agent_id` 与 `plan.target.agent_id` 是否一致
- `instance_id` 若存在，是否与当前实例匹配
- `delivery.expires_at` 与 `plan.meta.expires_at` 是否过期
- `ActionPlan` 签名、约束和能力匹配是否通过

其中：

- `dispatch_id` 只用于投递链路去重
- `action_id + plan_digest` 用于执行语义去重，避免同一计划因重投递被重复执行

---

## 7. 最小示例

```json
{
  "api_version": "v1alpha1",
  "kind": "dispatch_action_plan",
  "dispatch_id": "dsp_01",
  "action_id": "act_01",
  "plan_digest": "sha256:abc123",
  "agent_id": "agent_prod_web_01",
  "delivery": {
    "delivery_attempt": 1,
    "dispatched_at": "2026-04-12T10:00:00Z",
    "ack_deadline_at": "2026-04-12T10:00:05Z"
  },
  "plan": {
    "api_version": "v1alpha1",
    "kind": "action_plan",
    "meta": {
      "action_id": "act_01",
      "request_id": "req_01",
      "tenant_id": "tenant_01",
      "environment_id": "env_prod",
      "plan_version": 1,
      "compiled_at": "2026-04-12T10:00:00Z",
      "expires_at": "2026-04-12T10:05:00Z"
    },
    "target": {
      "agent_id": "agent_prod_web_01",
      "node_id": "prod-web-01",
      "platform": "linux",
      "arch": "amd64"
    },
    "constraints": {
      "risk_level": "R0",
      "approval_mode": "not_required",
      "requested_by": "ops@example",
      "max_total_duration_ms": 5000,
      "step_timeout_default_ms": 3000,
      "limits": {},
      "allow": {
        "paths": [
          "/etc/nginx/nginx.conf"
        ]
      },
      "required_capabilities": [
        "file.read_range"
      ],
      "execution_profile": "agent_exec_v1"
    },
    "program": {
      "entry": "s1",
      "steps": [
        {
          "step_id": "s1",
          "kind": "invoke",
          "op": "file.read_range",
          "args": {
            "path": {
              "literal": "/etc/nginx/nginx.conf"
            },
            "offset": {
              "literal": 0
            },
            "length": {
              "literal": 4096
            }
          },
          "bind": "part",
          "error_policy": "fail",
          "next": "s2"
        },
        {
          "step_id": "s2",
          "kind": "output",
          "items": [
            {
              "name": "part",
              "value": {
                "var": "part"
              }
            }
          ]
        }
      ]
    },
    "attestation": {
      "policy_version": "p1",
      "compiler_version": "c1",
      "plan_digest": "sha256:abc123",
      "signature": "sig_01",
      "issued_by": "control-plane"
    }
  }
}
```

---

## 8. 当前决定

当前阶段固定以下结论：

- 中心向边缘下发时必须使用 `DispatchActionPlan`
- 边缘只消费 `ActionPlan` 系列对象，不消费作者源文件
- `dispatch_id` 是投递链路主键
- `action_id + plan_digest` 是执行语义去重主键

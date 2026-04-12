# wp-agent ActionPlan IR 设计

## 1. 文档目的

本文档把 `ActionPlan IR` 定义为 `wp-agent` 远程执行体系中的唯一执行契约。

当前阶段先固定 IR，再讨论作者侧到底采用 `run.gxl`、`run.war`，还是其他 frontend。

本文档重点回答：

- `ActionPlan IR` 的边界是什么
- `ActionPlan` 与 `ActionRequest`、`ApprovalRecord`、`ActionResult` 的关系是什么
- IR 的最小对象模型应该如何拆分
- 边缘节点到底执行什么，不执行什么
- 哪些语义必须在编译期收敛，不能留到边缘解释

相关文档：

- [`action-dsl.md`](action-dsl.md)
- [`control-plane.md`](../center/control-plane.md)
- [`security-model.md`](../foundation/security-model.md)
- [`action-schema.md`](action-schema.md)

---

## 2. 核心结论

`ActionPlan IR` 不是“`run` 的一个序列化字段”，而是边缘节点唯一理解的受控指令计划。

必须明确：

- 作者输入可以有多种 frontend
- 中心节点只把 frontend 编译到统一 IR
- `wp-agentd` / `wp-agent-exec` 只接受 IR
- 边缘不解析 DSL，不执行脚本，不解释字符串表达式

一句话说：

`control + execution spec + request + approval + policy + capability -> ActionPlan IR`

---

## 3. 关键词评审结论

当前评审结论是：

- IR 关键词必须偏执行语义，不能直接沿用作者 DSL 语义
- 作者侧可以保留 `when / expect / emit / fail`
- IR 侧应收敛成 `branch / guard / output / abort`
- IR 应避免使用过于“源码输入导向”的字段名

建议采用如下口径：

### 3.1 建议保留的 IR 关键词

- `ActionPlan`
- `target`
- `program`
- `attestation`
- `step`
- `branch`
- `condition`
- `reason_code`

### 3.2 建议替换的 IR 关键词

- `control` -> `constraints`
- `source` -> `provenance`
- `call` -> `invoke`
- `assert` -> `guard`
- `emit` -> `output`
- `fail` -> `abort`

### 3.3 建议缩短的流程字段

- `entry_step` -> `entry`
- `next_step` -> `next`
- `then_step` -> `then`
- `else_step` -> `else`

### 3.4 建议改成机器友好单位的字段

- `max_total_duration` -> `max_total_duration_ms`
- `step_timeout_default` -> `step_timeout_default_ms`
- `timeout` -> `timeout_ms`
- `retry.delay` -> `delay_ms`

原因很直接：

- IR 主要给执行器消费，不是给作者直接写
- 字段名应优先表达执行含义，而不是 DSL 语法糖
- 边缘执行器更适合处理稳定、短小、机器友好的字段

---

## 4. 设计原则

### 3.1 IR 必须独立于作者语法

IR 不能和 `run.gxl`、`run.war` 强绑定。

原因是：

- frontend 仍未最终定型
- 未来可能同时存在多个 frontend
- 执行器不应跟随作者语法频繁演化

### 3.2 IR 必须是确定性的

边缘执行时不允许再做以下事情：

- 解析动态 DSL
- 下载额外代码
- 解析 shell 片段
- 根据字符串拼接决定 opcode
- 运行未审批的新分支

### 3.3 IR 必须可静态校验

边缘在执行前必须能本地校验：

- 版本
- 签名
- 过期时间
- 目标是否匹配本机
- capability 是否满足
- opcode 是否允许
- 路径 / 服务 / 资源范围是否在允许集内
- 每步超时和总超时是否有效

### 3.4 IR 必须把控制与执行放在同一份最终产物中

边缘不能接收分离的 `control.wac` 和 `run.*` 源文件。

边缘只接收：

- 一份已合成的控制约束
- 一份已编译的执行程序
- 一份已绑定审批、策略、签名的最终计划

---

## 5. 对象边界

### 4.1 `ActionRequest`

用户或自动化系统提交的请求对象。

它表达“想做什么”。

### 4.2 `ActionPlan`

中心节点编译出的、面向单个目标 agent 的最终执行对象。

它表达“在该节点上允许如何做”。

### 4.3 `ActionPlan IR`

`ActionPlan` 内部最核心的执行契约。

它表达“按什么指令序列执行”。

### 4.4 `ActionResult`

边缘执行完 `ActionPlan` 后回传的结果对象。

它表达“实际发生了什么”。

---

## 6. 顶层结构

建议把 `ActionPlan` 固定为以下四层：

- `meta`
- `target`
- `constraints`
- `program`

另有：

- `attestation`
- `provenance`

建议结构如下：

```text
ActionPlan {
  api_version
  kind = "action_plan"
  meta
  target
  constraints
  program
  attestation
  provenance
}
```

### 5.1 `meta`

表示控制平面视角的主键与关联。

建议字段：

- `action_id`
- `request_id`
- `template_id?`
- `tenant_id`
- `environment_id`
- `plan_version`
- `compiled_at`
- `expires_at`

### 5.2 `target`

表示该计划对应的唯一目标。

建议字段：

- `agent_id`
- `instance_id?`
- `node_id`
- `host_name?`
- `platform`
- `arch`
- `selectors`

约束：

- 一个 `ActionPlan` 只对应一个目标 agent
- 多目标请求必须在中心展开成多个 `ActionPlan`

### 6.3 `constraints`

表示已经合成完毕的最终安全控制。

建议字段：

- `risk_level`
- `approval_ref?`
- `approval_mode`
- `requested_by`
- `reason?`
- `max_total_duration_ms`
- `step_timeout_default_ms`
- `limits`
- `allow`
- `required_capabilities`
- `execution_profile`

其中：

- `limits` 用于约束资源和输出
- `allow` 用于约束路径、服务、资源、目标范围
- `execution_profile` 用于声明该计划只能由哪类执行器运行

### 6.4 `program`

表示可执行 IR。

建议字段：

- `entry`
- `steps[]`
- `failure_policy`

说明：

- 最终业务输出只通过 `kind = "output"` 的 step 进入 `ActionResult.outputs`
- 不再单独保留 `program.outputs` 并行机制

### 6.5 `attestation`

表示完整性与可追溯性信息。

建议字段：

- `policy_version`
- `compiler_version`
- `approval_digest`
- `plan_digest`
- `signature`
- `issued_by`

### 6.6 `provenance`

表示来源信息，只用于审计和回溯，不参与边缘执行决策。

建议字段：

- `frontend_kind`
- `frontend_ref`
- `frontend_digest`
- `control_ref?`
- `control_digest?`

这里的 `frontend_kind` 可以是：

- `run.gxl`
- `run.war`
- `native_json`

---

## 7. Program IR 模型

第一版建议把 `program` 做成显式步骤图，而不是把作者 AST 原样塞进去。

建议最小步骤类型如下：

- `invoke`
- `branch`
- `guard`
- `output`
- `abort`

不建议第一版支持：

- 循环
- 递归
- 动态 include
- 动态代码加载
- 任意脚本执行

### 7.1 `invoke`

表示一次受控 opcode 调用。

建议字段：

- `step_id`
- `kind = "invoke"`
- `op`
- `args`
- `bind?`
- `timeout_ms?`
- `retry_policy?`
- `error_policy`
- `next?`

说明：

- `bind` 是业务输出绑定名
- `retry_policy` 是该步级别的有限重试策略
- `error_policy` 是稳定的错误流转策略，不允许任意跳转脚本

### 7.2 `branch`

表示条件分支。

建议字段：

- `step_id`
- `kind = "branch"`
- `condition`
- `then`
- `else?`

### 7.3 `guard`

表示受控断言。

建议字段：

- `step_id`
- `kind = "guard"`
- `condition`
- `reason_code`
- `detail?`
- `next?`

### 7.4 `output`

表示结果输出选择，而不是 stdout 打印。

建议字段：

- `step_id`
- `kind = "output"`
- `items[]`
- `next?`

`items[]` 中每一项建议包含：

- `name`
- `value`
- `redaction?`

### 7.5 `abort`

表示显式失败。

建议字段：

- `step_id`
- `kind = "abort"`
- `reason_code`
- `detail?`

---

## 8. 表达式和值模型

边缘不应再解析 DSL 表达式字符串。

因此，`condition`、`args`、`emit.value` 都应在编译后落为结构化表达式树。

建议最小值类型：

- `null`
- `bool`
- `int`
- `float`
- `string`
- `bytes`
- `list`
- `object`

建议最小表达式节点：

- `literal`
- `var`
- `field`
- `compare`
- `logic`

示意：

```text
Expr::Compare {
  op: "eq",
  left: Expr::Field(Expr::Var("svc"), "state"),
  right: Expr::Literal("running")
}
```

这样做的原因是：

- 边缘不需要再带 DSL parser
- 条件求值更稳定
- 更容易做版本兼容
- 更容易在审计里回放真实决策路径

---

## 9. `invoke.args` 的编译原则

`invoke.args` 中不应出现“执行时再拼接 shell”的模型。

建议参数值只允许：

- 字面量
- 请求输入引用
- 前序步骤输出引用
- 受控字段选择

不允许：

- shell 片段
- 任意模板展开
- 下载后执行
- 把 opcode 名称本身做成动态值

---

## 10. 运行语义

### 9.1 执行单位

IR 中真正的最小执行单位是 `step`。

每个 `step` 至少要生成：

- 一条 `StepActionRecord`
- 零个或一个业务输出绑定

也就是说：

- `step` 不是结果值
- `step` 也不是 `ActionResult`
- `step` 执行后会同时产出“执行记录”和“业务输出”

### 9.2 业务输出

`bind` 绑定的是业务输出值。

例如：

- `process.list` 绑定 `procs`
- `service.status` 绑定 `svc`
- `file.read_range` 绑定 `part`

### 9.3 执行记录

每步应记录：

- `step_id`
- `attempt`
- `op?`
- `status`
- `started_at`
- `finished_at`
- `duration_ms`
- `error_code?`
- `stdout_summary?`
- `stderr_summary?`
- `resource_usage?`

这类记录建议命名为：

- `StepActionRecord`

不要直接让作者 DSL 面向 `Action` 这个运行时概念。

---

## 11. ActionResult 模型

建议边缘回传的 `ActionResult` 至少包含：

- `action_id`
- `execution_id`
- `final_status`
- `exit_reason`
- `step_records[]`
- `outputs`
- `resource_usage`
- `started_at`
- `finished_at`

其中：

- `step_records[]` 是执行审计
- `outputs` 是对外返回结果

建议明确区分：

- `step_records[]` 给审计和排障
- `outputs` 给控制平面和上层调用方消费

---

## 12. 校验规则

边缘在执行前至少要校验：

- `api_version` 是否支持
- `kind` 是否为 `action_plan`
- `target.agent_id` 是否匹配本机
- `expires_at` 是否未过期
- `signature` 是否有效
- `required_capabilities` 是否满足
- 每个 `invoke.op` 是否在本机支持集合中
- 每个 `invoke.args` 是否满足 schema
- 每个受限参数是否在 `allow` 集内
- 每个超时、重试、输出上限是否在 `limits` 内
- `program.steps[]` 是否图结构合法且无悬空引用

若任一失败，应直接拒绝执行，并回传 `rejected`。

---

## 13. 第一版收敛建议

为了尽快落地，第一版建议固定以下约束：

- 一个 `ActionPlan` 对应一个目标 agent
- `program` 只允许有限步骤图
- 只支持白名单 opcode
- 不支持循环
- 不支持动态 include
- 不支持任意 shell
- 不支持运行期下载并执行
- 所有表达式都编译成结构化节点

---

## 14. 一个最小示例

```text
ActionPlan {
  api_version: "v1alpha1"
  kind: "action_plan"
  meta: {
    action_id: "act_01"
    request_id: "req_01"
    tenant_id: "t1"
    environment_id: "prod"
    plan_version: 1
    compiled_at: "2026-04-12T10:00:00Z"
    expires_at: "2026-04-12T10:10:00Z"
  }
  target: {
    agent_id: "agent_prod_web_01"
    node_id: "prod-web-01"
    platform: "linux"
    arch: "amd64"
  }
  constraints: {
    risk_level: "R0"
    approval_mode: "not_required"
    max_total_duration_ms: 5000
    step_timeout_default_ms: 3000
    allow: {
      paths: ["/etc/nginx/nginx.conf"]
    }
    required_capabilities: ["file.read_range"]
  }
  program: {
    entry: "s1"
    steps: [
      {
        step_id: "s1"
        kind: "invoke"
        op: "file.read_range"
        args: {
          path: { literal: "/etc/nginx/nginx.conf" }
          offset: { literal: 0 }
          length: { literal: 4096 }
        }
        bind: "part"
        next: "s2"
      },
      {
        step_id: "s2"
        kind: "guard"
        condition: {
          compare: {
            op: "gt"
            left: { field: { base: { var: "part" }, name: "length" } }
            right: { literal: 0 }
          }
        }
        reason_code: "empty_result"
        next: "s3"
      },
      {
        step_id: "s3"
        kind: "output"
        items: [
          { name: "part", value: { var: "part" } }
        ]
      }
    ]
  }
}
```

---

## 15. 当前决定

当前先固定下面三件事：

- `ActionPlan IR` 是唯一执行契约
- frontend 暂不定死为 `run.gxl` 或 `run.war`
- 后续任何 frontend 都必须编译到本 IR，而不是绕过 IR 直接下发到边缘

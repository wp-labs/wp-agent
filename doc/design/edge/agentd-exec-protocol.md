# wp-agentd 与 wp-agent-exec 本地协议设计

## 1. 文档目的

本文档定义 `wp-agentd` 与 `wp-agent-exec` 之间的本地交互协议。

它主要解决以下问题：

- `wp-agentd` 如何把 `ActionPlan` 交给 `wp-agent-exec`
- `wp-agent-exec` 如何回传状态与结果
- 取消、超时、异常退出如何处理
- v1 应优先选用哪种本地协议，才能尽快落地并控制复杂度

相关文档：

- [`action-plan-ir.md`](../execution/action-plan-ir.md)
- [`architecture.md`](../foundation/architecture.md)
- [`roadmap.md`](../foundation/roadmap.md)

---

## 2. 核心结论

v1 建议采用：

- `工作目录 + 文件契约 + 进程信号`

不建议 v1 采用：

- `wp-agentd` 解析 `wp-agent-exec` 的 stdout/stderr 作为正式协议
- 自定义复杂本地 RPC
- 共享内存
- 一开始就引入 Unix Domain Socket 双向协议

一句话说：

- `wp-agentd` 负责创建执行工作目录
- `wp-agentd` 把输入文件写入工作目录
- `wp-agentd` 启动 `wp-agent-exec`
- `wp-agent-exec` 读取工作目录中的输入文件
- `wp-agent-exec` 原子写入状态文件和结果文件
- `wp-agentd` 用进程信号驱动取消和强制终止

这样做的主要原因是：

- 简单
- 可恢复
- 易调试
- 易审计
- 不依赖 stdout 文本解析
- crash 后容易保留现场

---

## 3. 设计原则

### 3.1 stdout/stderr 不是正式协议

`wp-agent-exec` 的 stdout/stderr 只能作为诊断日志，不作为结构化控制协议。

原因：

- 文本输出容易被实现细节污染
- 不利于版本兼容
- 不利于 crash 后恢复
- 不利于做稳定 schema

### 3.2 输入输出必须文件化

v1 要求所有正式输入输出都落到工作目录中的结构化文件。

正式契约包括：

- `plan.json`
- `runtime.json`
- `state.json`
- `result.json`

### 3.3 `wp-agentd` 是控制器

`wp-agentd` 负责：

- 创建工作目录
- 写入输入文件
- 启动执行器
- 监控进程
- 处理超时与取消
- 读取状态文件与结果文件
- 汇总并上报

`wp-agent-exec` 不负责：

- 排队
- 并发调度
- 本地多任务编排
- 审批判断
- 长连接控制面通信

### 3.4 协议必须支持 crash 后检查现场

执行失败后应能保留：

- 最终输入
- 最终状态
- 最终结果
- stdout/stderr 日志

便于本地排障和中心审计回放。

---

## 4. 工作目录模型

建议每次执行使用独立工作目录：

```text
<agent_run_root>/actions/<execution_id>/
  plan.json
  runtime.json
  state.json
  result.json
  stdout.log
  stderr.log
  meta.json
```

建议约束：

- 一个 `execution_id` 对应一个独立目录
- 目录权限默认仅允许 agent 用户访问
- 文件更新采用“写临时文件再原子 rename”方式

### 4.1 `plan.json`

由 `wp-agentd` 写入。

内容：

- 最终 `ActionPlan`

### 4.2 `runtime.json`

由 `wp-agentd` 写入。

内容建议包括：

- `execution_id`
- `spawned_at`
- `deadline_at`
- `agent_id`
- `node_id`
- `workdir`

`runtime.json` 不属于中心签名对象，它是本地执行上下文。

### 4.3 `state.json`

由 `wp-agent-exec` 更新。

表示当前本地执行状态。

### 4.4 `result.json`

由 `wp-agent-exec` 在结束时写入。

内容为最终 `ActionResult`。

说明：

- `result.json` 只包含执行语义对象，不直接包含中心回报 envelope
- 结果级摘要与签名由 `wp-agentd` 在读取 `result.json` 后生成

### 4.5 `stdout.log` / `stderr.log`

由 `wp-agentd` 重定向采集。

仅用于诊断，不属于正式协议字段。

### 4.6 `meta.json`

由 `wp-agentd` 维护。

用于记录本地调度元数据，例如：

- `pid`
- `spawn_attempt`
- `started_by`
- `cancel_requested_at`
- `kill_requested_at`

---

## 5. 进程启动协议

建议 `wp-agentd` 使用明确参数拉起：

```text
wp-agent-exec run --workdir <execution_workdir>
```

第一版不建议把 `ActionPlan` 直接通过命令行传递。

原因：

- 命令行长度有限
- 容易暴露敏感内容
- 不利于现场保留

### 5.1 `wp-agentd` 启动前步骤

1. 创建工作目录
2. 写入 `plan.json`
3. 写入 `runtime.json`
4. 初始化 `meta.json`
5. 打开 `stdout.log` / `stderr.log`
6. spawn `wp-agent-exec`

### 5.2 `wp-agent-exec` 启动后步骤

1. 读取 `runtime.json`
2. 读取 `plan.json`
3. 校验输入完整性
4. 写入 `state.json = validating`
5. 校验 `ActionPlan`
6. 进入执行阶段
7. 最终写入 `result.json`
8. 最终写入 `state.json = done|failed|timed_out|cancelled|rejected`

随后由 `wp-agentd`：

9. 读取 `result.json`
10. 计算 `result_digest`
11. 生成结果级签名
12. 组装 `ReportActionResult`

---

## 6. 本地状态协议

建议 `state.json` 至少包含：

- `execution_id`
- `action_id`
- `state`
- `updated_at`
- `step_id?`
- `attempt?`
- `reason_code?`
- `detail?`

建议第一版状态枚举：

- `spawned`
- `validating`
- `rejected`
- `running`
- `cancelling`
- `cancelled`
- `timed_out`
- `failed`
- `succeeded`

### 6.1 状态解释

- `spawned`
  进程已启动，尚未进入计划校验
- `validating`
  正在校验 `ActionPlan`
- `rejected`
  校验失败，不进入执行
- `running`
  正在执行 `program.steps[]`
- `cancelling`
  `wp-agentd` 已请求取消，等待执行器收敛
- `cancelled`
  执行器确认取消结束
- `timed_out`
  达到超时门限
- `failed`
  执行失败
- `succeeded`
  执行成功

---

## 7. 结果协议

`result.json` 应使用 [`action-plan-ir.md`](../execution/action-plan-ir.md) 中定义的 `ActionResult` 模型。

建议 `wp-agent-exec` 只在以下时机写入最终 `result.json`：

- 成功完成
- 校验拒绝
- 显式失败
- 被取消后完成清理
- 超时退出

写入要求：

- 先写临时文件
- fsync
- 原子 rename 到 `result.json`

`wp-agentd` 应只在检测到 `result.json` 完整落盘后，才认为结果可消费。

---

## 8. 取消与超时协议

### 8.1 超时由谁裁决

v1 建议：

- `wp-agentd` 负责总超时裁决
- `wp-agent-exec` 负责步骤超时执行

也就是说：

- `constraints.max_total_duration_ms` 由 `wp-agentd` 监控
- `program.steps[].timeout_ms` 或默认 step timeout 由 `wp-agent-exec` 监控

### 8.2 取消流程

建议流程：

1. `wp-agentd` 标记本地执行为 `cancel_requested`
2. `wp-agentd` 向 `wp-agent-exec` 发送 `SIGTERM`
3. `wp-agent-exec` 把 `state.json` 更新为 `cancelling`
4. `wp-agent-exec` 做有限清理并写最终 `result.json`
5. 若宽限时间后仍未退出，`wp-agentd` 发送 `SIGKILL`

### 8.3 强制终止

以下情况允许 `wp-agentd` 强制 kill：

- 取消宽限期超时
- 总超时已到且执行器未退出
- 执行器无响应且状态文件长期不更新

---

## 9. 失败语义

建议把失败拆成四类：

### 9.1 `rejected`

输入校验未通过。

典型原因：

- 签名无效
- 目标不匹配
- capability 不满足
- allow/limits 越界

### 9.2 `failed`

计划已进入执行，但执行失败。

典型原因：

- opcode 运行失败
- `guard` 不满足
- `abort` 被触发

### 9.3 `cancelled`

执行过程中被 `wp-agentd` 取消。

### 9.4 `timed_out`

达到步骤超时或总超时。

---

## 10. v1 协议字段草案

### 10.1 `runtime.json`

```json
{
  "execution_id": "exec_01",
  "action_id": "act_01",
  "agent_id": "agent_prod_web_01",
  "node_id": "prod-web-01",
  "workdir": "/var/lib/wp-agent/run/actions/exec_01",
  "spawned_at": "2026-04-12T10:00:00Z",
  "deadline_at": "2026-04-12T10:00:05Z"
}
```

### 10.2 `state.json`

```json
{
  "execution_id": "exec_01",
  "action_id": "act_01",
  "state": "running",
  "updated_at": "2026-04-12T10:00:01Z",
  "step_id": "s2",
  "attempt": 1
}
```

---

## 11. 为什么不优先选 Unix Domain Socket

不是说永远不用，而是 v1 不该先上。

原因：

- 双向消息协议要先定义 framing、重连、半关闭、进程异常处理
- 实现复杂度明显高于文件协议
- 对 M2 `Agentd Skeleton` 没有必要

后续如果需要：

- 流式 progress
- 细粒度 cancel ack
- 长任务心跳
- 交互式诊断

再考虑引入 UDS v2。

---

## 12. 当前决定

当前阶段固定以下结论：

- `wp-agentd` 与 `wp-agent-exec` 的 v1 正式协议不是 stdout/stderr
- v1 采用 `工作目录 + 文件契约 + 进程信号`
- `wp-agentd` 是本地控制器
- `wp-agent-exec` 是一次性受控执行器
- 未来若要增强流式交互，再引入 UDS v2，而不是一开始就做重协议

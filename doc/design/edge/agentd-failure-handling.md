# wp-agentd 故障处理与恢复设计

## 1. 文档目的

本文档定义 `wp-agentd` 在第一版里的故障分层、恢复语义和健康口径，重点回答三类问题：

- 什么故障只影响单个 execution
- 什么故障必须让 daemon 进入失败或保护态
- 本地状态文件损坏时，哪些对象可以重建，哪些对象必须保真

它直接服务于：

- `M3 Edge Runtime Skeleton`
- `M5 Controlled Action MVP`

相关文档：

- [`agentd-architecture.md`](agentd-architecture.md)
- [`agentd-state-and-boundaries.md`](agentd-state-and-boundaries.md)
- [`agentd-state-schema.md`](agentd-state-schema.md)
- [`self-observability.md`](self-observability.md)
- [`report-action-result-schema.md`](../center/report-action-result-schema.md)

---

## 2. 核心结论

`wp-agentd` 第一版必须固定以下结论：

1. 单个 execution 的坏数据，默认不能打停整个 daemon
2. 本地状态必须区分权威状态和可重建工件
3. crash 恢复与在线执行必须遵循同一套故障裁决规则
4. health 必须覆盖 `queue`、`running`、`reporting` 三类 backlog

一句话说：

- 单 execution 故障要隔离
- 全局状态故障才允许升级为 daemon 级故障

---

## 3. 故障分层

第一版把故障分成三层：

- execution-local failure
- runtime-degraded failure
- fatal daemon failure

### 3.1 execution-local failure

只影响单个 execution 的故障。

典型例子：

- 某个 `plan.json` 损坏
- 某个 `running/<execution_id>.json` 损坏
- 某个 `result.json` 损坏
- 某个 `reporting/<execution_id>.json` 损坏
- 某个 report envelope 损坏
- 某个 execution 的 workdir 不完整

默认处理规则：

- 不直接打停 `wp-agentd`
- 能重建就重建
- 不能重建就 quarantine 当前 execution
- 必须保留最小 history 记录

### 3.2 runtime-degraded failure

不会立刻终止 daemon，但说明当前运行环境退化，后续能力受限。

典型例子：

- 进程 identity 一度可读，后续暂时不可读
- `ps` 或 `/proc` 能力受限，无法可靠判断僵尸态
- 中心不可达，本地 reporting backlog 增长
- 非关键自观测输出失败

默认处理规则：

- daemon 继续运行
- 对相关 execution 采取保守阻塞或延迟恢复
- health / 指标必须能反映退化状态

### 3.3 fatal daemon failure

影响 daemon 全局一致性，无法安全继续推进后续 execution。

典型例子：

- 正式配置无效
- `state/` 根状态无法读写
- `agent_runtime.json` 无法持久化
- `execution_queue.json` 无法稳定持久化
- 统一 loader 无法完成 `parse -> env_eval -> path resolve -> validate`

默认处理规则：

- `run_once()` / `run_forever()` 返回错误
- daemon 由上层拉起或人工处理
- 不得假装继续正常服务

---

## 4. 本地状态权威性

`wp-agentd` 第一版必须明确以下 source-of-truth 关系。

### 4.1 execution 结果

- `run/actions/<execution_id>/result.json`
  是 execution 最终结果的权威来源

它保存：

- `ActionResult`
- 最终 `final_status`
- `step_records`
- `outputs`

如果它损坏：

- 不得信任其他派生工件替代它
- 当前 execution 应转入 quarantine 或失败恢复

### 4.2 reporting 状态

- `state/reporting/<execution_id>.json`
  是结果回报流程状态的权威来源

它保存：

- `final_state`
- `result_path`
- `report_attempt`
- `last_report_at`
- `last_report_error`
- `result_digest`
- `result_signature`

如果它缺失但 `result.json` 完整：

- 必须尝试重建
- 不应仅因 envelope 仍在就判定 reporting 完整

### 4.3 report envelope

- `state/reporting/<execution_id>.envelope.json`
  是对中心回报的传输工件

它不是权威状态，只是可重建产物。

如果它缺失或损坏，但 `result.json` 与 `reporting/<execution_id>.json` 完整：

- 必须重建
- 不得直接 quarantine 当前 execution

### 4.4 running 状态

- `state/running/<execution_id>.json`
  是 scheduler / controller 视角的运行控制状态

它不是最终结果来源，也不是 reporting 来源。

如果它损坏：

- 可以 quarantine 当前 execution
- 但必须同时阻止该 execution 被隐式重跑

---

## 5. 故障裁决规则

第一版建议按下表处理：

| 故障对象 | 是否可重建 | 默认动作 | 是否允许打停 daemon |
| --- | --- | --- | --- |
| `plan.json` | 否 | quarantine execution | 否 |
| `running/<execution_id>.json` | 否 | quarantine execution | 否 |
| `result.json` | 否 | quarantine execution 或恢复失败结果 | 否 |
| `reporting/<execution_id>.json` | 是 | 从 `result.json` 重建 | 否 |
| report envelope | 是 | 从 `result.json + reporting state` 重建 | 否 |
| `execution_queue.json` | 否 | daemon failure | 是 |
| `agent_runtime.json` | 否 | daemon failure | 是 |
| 正式配置 | 否 | daemon failure | 是 |

补充规则：

1. 若某 execution 已被 quarantine，不得在同一次调度 tick 内被重新执行
2. 若进程归属无法确认，只能保守阻塞，不能直接恢复为失败
3. 若 execution 级故障可以在本地确定隔离范围，不得升级成 daemon 级故障

---

## 6. crash 恢复与在线执行一致性

第一版必须保证：

- 在线执行路径
- daemon 启动恢复路径
- scheduler reconcile 路径

三者使用同一套判断标准。

### 6.1 一致性要求

如果某类损坏在线路径会 quarantine，那么恢复路径也应 quarantine。

如果某类工件在线路径允许重建，那么恢复路径也应重建。

不允许出现下面这种分裂：

- 恢复路径把损坏 envelope 当可重建
- 在线路径却把同类问题升级成 daemon 失败

### 6.2 进程 identity 退化规则

如果 `running` 记录里保存了 `process_identity`，但当前环境无法再次读取：

- 不应直接把该 execution 当成已结束
- 不应直接 kill 一个已无法确认归属的 pid
- 应保持 blocked / degraded 判断，等待下一轮恢复或人工介入

但如果 identity 明确不匹配：

- 应视为原 execution 已不再运行

---

## 7. health 与自观测口径

`wp-agentd` 的 runtime health 不能只看 queue 和 running。

第一版至少要覆盖：

- `execution_queue`
- `running`
- `reporting`

### 7.1 active 判定

满足任一条件时，health 应视为 `Active`：

- `execution_queue` 非空
- 存在 `running/*.json`
- 存在 `reporting/*.json`
- 当前 tick 刚完成一次 execution 或 reporting 推进

### 7.2 最小指标

第一版至少应暴露：

- `agent_execution_queue_size`
- `agent_running_executions`
- `agent_reporting_executions`
- `agent_plan_completed_total`
- `agent_plan_failed_total`
- `agent_plan_cancelled_total`
- `agent_plan_timed_out_total`
- `agent_execution_quarantined_total`

### 7.3 降级反映

如果出现以下情况，health 或事件中必须能体现：

- process identity 不可读
- reporting backlog 持续增长
- 本地状态重建发生
- execution 被 quarantine

---

## 8. 实现约束

实现时必须遵守以下约束：

1. 单 execution 的异常处理默认在 scheduler / recovery 层闭环
2. 可重建工件损坏时优先重建，不优先 quarantine
3. quarantine 必须同时清除会导致隐式重跑的调度入口
4. 正式配置必须只走统一 loader
5. `max_running_actions` 在未实现多并发前必须显式拒绝大于 `1`

---

## 9. 当前阶段决定

当前阶段固定以下结论：

1. `result.json` 是 execution 结果真相
2. `reporting/<execution_id>.json` 是 reporting 生命周期真相
3. report envelope 是可重建传输工件
4. execution-local 故障默认不得打停整个 daemon
5. health 必须把 reporting backlog 算入 active

后续若引入真正的 reporting worker、中心 ACK 清理或多 execution 并发，此文档应继续扩展，但不得推翻以上基础结论。

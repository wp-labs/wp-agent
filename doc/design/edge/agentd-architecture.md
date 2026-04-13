# wp-agentd 架构设计

## 1. 文档目的

本文档定义 `wp-agentd` 的职责边界、模块拆分、本地状态机和与 `wp-agent-exec` / `wp-agent-upgrader` 的关系。

它主要服务于 [`roadmap.md`](../foundation/roadmap.md) 中的：

- `M3 Edge Runtime Skeleton`
- `M5 Controlled Action MVP`

相关文档：

- [`architecture.md`](../foundation/architecture.md)
- [`action-plan-ir.md`](../execution/action-plan-ir.md)
- [`agentd-failure-handling.md`](agentd-failure-handling.md)
- [`agentd-exec-protocol.md`](agentd-exec-protocol.md)
- [`roadmap.md`](../foundation/roadmap.md)

---

## 2. 核心定位

`wp-agentd` 不是执行器，而是边缘控制器。

它的核心职责是：

- 常驻运行
- 接收中心侧下发对象
- 做本地校验
- 做本地 `execution_queue` 排队和并发控制
- 拉起 `wp-agent-exec`
- 拉起 `wp-agent-upgrader`
- 汇总本地执行结果
- 上报状态、健康和审计事件

一句话说：

- `wp-agentd` 负责控制
- `wp-agent-exec` 负责执行
- `wp-agent-upgrader` 负责升级辅助

### 2.1 运行模式

`wp-agentd` 第一版应明确支持两种运行模式：

- `standalone`
- `managed`

`standalone` 模式下：

- 没有中心控制节点也能启动和常驻运行
- 本地采集、发现、标准化、缓冲、上送、自观测继续工作
- 本地状态机和保护模式继续工作
- 不接收远程下发的 `ActionPlan`

`managed` 模式下：

- 在 `standalone` 模式能力基础上接入中心节点
- 启用会话、心跳、能力上报
- 启用远程任务与中心编排升级

这个区分必须是架构级约束，而不是实现阶段的临时兼容。

---

## 3. 明确不负责什么

`wp-agentd` 第一版不负责：

- 直接执行 `ActionPlan.program`
- 解析作者 DSL
- 本地审批判断
- 本地策略合成
- 复杂数据分析
- AI 推理

这几类能力都不应被塞回 `wp-agentd`。

---

## 4. 顶层模块

建议 `wp-agentd` 第一版拆成以下模块：

- `bootstrap`
- `config_runtime`
- `control_receiver`
- `plan_validator`
- `execution_scheduler`
- `executor_manager`
- `upgrade_manager`
- `result_aggregator`
- `audit_logger`
- `state_store`
- `self_observability`

---

## 5. 模块职责

### 5.1 `bootstrap`

负责：

- 启动参数解析
- 工作目录初始化
- 本地目录权限检查
- 本地身份与版本信息加载
- 子模块启动顺序编排

### 5.2 `config_runtime`

负责：

- 本地静态配置加载
- 判定当前是 `standalone` 还是 `managed`
- 策略版本记录
- 动态配置切换
- 运行时 feature gate

### 5.3 `control_receiver`

负责：

- 接收中心节点下发对象
- 反序列化 `ActionPlan`
- 做基础格式检查
- 交给 `plan_validator`

第一版它只需要支持动作计划，不必一开始就把所有控制对象都做全。

在 `standalone` 模式下它应表现为：

- 不建立中心会话
- 不接收远程计划
- 不影响其他本地模块启动

### 5.4 `plan_validator`

负责：

- 校验 `api_version`
- 校验 `kind`
- 校验目标是否匹配本机
- 校验过期时间
- 校验签名和 attestation
- 校验 capability
- 校验 constraints
- 校验 `program.steps[]` 图结构

它只负责“能不能执行”，不负责“怎么执行”。

### 5.5 `execution_scheduler`

负责：

- 本地 `execution_queue` 排队
- 并发控制
- 优先级控制
- 取消请求协调
- 总超时裁决

建议第一版约束：

- 同时执行的远程动作数必须有上限
- 升级与远程动作默认互斥

### 5.6 `executor_manager`

负责：

- 创建执行工作目录
- 写入 `plan.json` / `runtime.json`
- 拉起 `wp-agent-exec`
- 监控子进程生命周期
- 转发取消信号
- 回收退出状态

它不应理解 step 级别语义。

### 5.7 `upgrade_manager`

负责：

- 拉起 `wp-agent-upgrader`
- 维护升级互斥
- 接收升级结果

第一版应明确：

- 升级任务优先与远程动作任务互斥
- `managed` 模式下可接中心编排升级
- `standalone` 模式下至少应支持本地升级辅助，不应阻断 agent 正常运行

### 5.8 `result_aggregator`

负责：

- 读取 `state.json`
- 读取 `result.json`
- 汇总 stdout/stderr 摘要
- 生成本地统一执行结果对象
- 提交给控制面上报模块

### 5.9 `audit_logger`

负责：

- 记录计划接收
- 记录计划拒绝
- 记录进程启动
- 记录取消与 kill
- 记录结果归档

### 5.10 `state_store`

负责：

- 保存本地执行状态
- 保存 `execution_queue` 信息
- 保存运行中执行索引
- crash 后恢复时重建最小现场

### 5.11 `self_observability`

负责：

- 暴露 `wp-agentd` 自身状态
- 暴露 `execution_queue` 长度
- 暴露运行中任务数
- 暴露拒绝计数
- 暴露执行失败计数
- 暴露当前运行模式和中心连接状态

---

## 6. 建议的内部边界

建议内部边界固定如下：

- `control_receiver` 只接对象，不排队
- `plan_validator` 只做校验，不 spawn 进程
- `execution_scheduler` 只管调度，不执行 step
- `executor_manager` 只管子进程，不做审批
- `result_aggregator` 只管收敛结果，不决定重试策略

这几个边界不能在实现中重新耦合，否则后面会很快失控。

还要补一条运行约束：

- `control_receiver` 不在线时，不得影响数据面和本地守护主循环存活

---

## 7. 本地状态机

建议 `wp-agentd` 维护独立于控制平面的本地执行状态机。

第一版建议状态：

- `received`
- `validating`
- `rejected`
- `queued`
- `dispatching_local`
- `running`
- `cancelling`
- `cancelled`
- `timed_out`
- `failed`
- `succeeded`
- `reporting`
- `done`

### 7.1 状态解释

- `received`
  已从中心接收到对象
- `validating`
  正在做本地校验
- `rejected`
  本地校验失败
- `queued`
  已通过校验，等待调度
- `dispatching_local`
  正在准备工作目录并拉起 `wp-agent-exec`
- `running`
  子进程已进入执行
- `cancelling`
  已发出取消请求
- `cancelled`
  已取消完成
- `timed_out`
  总超时或本地调度超时
- `failed`
  执行失败
- `succeeded`
  执行成功
- `reporting`
  正在向中心回传
- `done`
  本地闭环结束

### 7.2 状态机约束

- `rejected` 不进入 `queued`
- `queued` 只能进入 `dispatching_local` 或 `cancelled`
- `running` 可以进入 `succeeded`、`failed`、`cancelled`、`timed_out`
- `reporting` 后必须进入 `done`

---

## 8. 本地数据与目录

建议 `wp-agentd` 管理三类本地数据：

### 8.1 运行目录

例如：

```text
<agent_root>/run/
  actions/
  upgrades/
```

### 8.2 状态目录

例如：

```text
<agent_root>/state/
  execution_queue.json
  running.json
  last_reported.json
```

### 8.3 日志目录

例如：

```text
<agent_root>/log/
  agentd.log
  actions/
  upgrades/
```

---

## 9. 并发与互斥

第一版建议固定以下原则：

- 远程动作执行并发数有硬上限
- 升级任务和远程动作默认互斥
- 高风险动作与升级任务默认互斥
- 同一 `action_id` 不允许在同一 agent 上并发执行

建议第一版先做最保守策略：

- `max_running_actions = 1`

等本地状态机稳定后，再扩展到更高并发。

---

## 10. 与 wp-agent-exec 的关系

`wp-agentd` 与 `wp-agent-exec` 的关系应固定为：

- `wp-agentd` 是父进程与控制器
- `wp-agent-exec` 是子进程与执行器
- 两者通过 [`agentd-exec-protocol.md`](agentd-exec-protocol.md) 中定义的本地协议交互

`wp-agentd` 不应：

- 直接在本进程中执行 opcode
- 直接解释 `program.steps[]`

否则三进程模型就失去意义。

---

## 11. 与 wp-agent-upgrader 的关系

`wp-agentd` 应是 `wp-agent-upgrader` 的调度入口，但不是升级执行体。

建议原则：

- `wp-agentd` 负责升级计划接收与互斥判断
- `wp-agent-upgrader` 负责升级下载、校验、切换、回滚
- `wp-agentd` 负责升级结果汇总与上报

---

## 12. 启动顺序

第一版建议 `wp-agentd` 启动顺序如下：

1. 初始化工作目录
2. 加载本地配置
3. 加载本地身份与版本信息
4. 恢复最小本地状态
5. 启动控制接收入口
6. 启动调度器
7. 启动自观测导出
8. 开始接收计划

---

## 13. crash 恢复原则

第一版不要求复杂恢复，但至少应做到：

- 启动时扫描 `run/actions/*`
- 识别孤儿执行目录
- 标记上次异常退出的执行
- 将未完成执行标记为 `failed` 或 `unknown`
- 避免重复上报同一结果

这对守护进程是必要能力，不应留到太后面。

---

## 14. M3 需要冻结的最小内容

为了真正启动 `M3 Edge Runtime Skeleton`，至少需要先冻结：

- `wp-agentd` 模块列表
- 本地状态机
- 工作目录布局
- 与 `wp-agent-exec` 的 v1 本地协议
- 并发与互斥基本原则

如果这几项不先冻结，`wp-agentd` 开发会很快陷入反复返工。

---

## 15. 当前决定

当前阶段固定以下结论：

- `wp-agentd` 必须先于 `wp-agent-exec` 进入开发主线
- `wp-agentd` 是边缘控制器，不是 step 执行器
- `wp-agentd` 必须持有本地状态机、队列和调度能力
- `wp-agentd` 与 `wp-agent-exec` 通过独立本地协议交互
- `wp-agentd` 与 `wp-agent-upgrader` 保持明确互斥与调度边界

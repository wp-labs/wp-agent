# wp-agent 多里程碑开发路线图

## 1. 文档目的

本文档把 `wp-agent` 当前设计讨论收敛为可执行的多里程碑开发计划。

路线图遵循以下原则：

- 先冻结执行契约，再做执行器
- 先打通边缘最小闭环，再扩展控制平面
- 先做只读、低风险能力，再做高风险控制能力
- 先保证执行确定性，再决定作者侧 frontend

当前阶段不应先押注 `run.gxl` 或 `run.war`。

当前阶段应先围绕 [`action-plan-ir.md`](../execution/action-plan-ir.md) 推进。

相关文档：

- [`target.md`](target.md)
- [`architecture.md`](architecture.md)
- [`security-model.md`](security-model.md)
- [`action-plan-ir.md`](../execution/action-plan-ir.md)
- [`action-schema.md`](../execution/action-schema.md)
- [`control-plane.md`](../center/control-plane.md)
- [`control-center-architecture.md`](../center/control-center-architecture.md)
- [`control-center-storage-schema.md`](../center/control-center-storage-schema.md)
- [`agent-gateway-protocol.md`](../center/agent-gateway-protocol.md)
- [`action-dsl.md`](../execution/action-dsl.md)

---

## 2. 总体推进顺序

建议按以下顺序推进：

1. 冻结 `ActionPlan IR`
2. 建立 IR schema 与本地校验器
3. 先启动 `wp-agentd` 最小守护进程骨架
4. 再启动 `wp-agent-exec` 最小执行器骨架
5. 打通 `wp-agentd -> wp-agent-exec` 本地闭环
6. 实现首批只读 opcode
7. 建立中心编译、下发、回传闭环
8. 强化签名、审批、审计、安全校验
9. 做非功能优化与退化保护
10. 最后再决定作者侧 frontend

---

## 3. 里程碑总览

| 里程碑 | 名称 | 目标 |
|---|---|---|
| M0 | 设计冻结 | 固定关键词、IR 分层、对象边界和第一版能力范围 |
| M1 | IR Schema | 把 `ActionPlan` / `ActionResult` 做成机器可校验契约 |
| M2 | Agentd Skeleton | 先建立 `wp-agentd` 常驻守护进程骨架 |
| M3 | Exec Skeleton | 建立 `wp-agent-exec` 最小执行器骨架 |
| M4 | Local Edge Loop | 打通 `wp-agentd -> wp-agent-exec -> ActionResult` 本地闭环 |
| M5 | Core Opcodes | 落地第一批只读诊断与读取 opcode |
| M6 | Control Plane MVP | 打通请求、审批、编译、下发、回传、归档闭环 |
| M7 | Security Hardening | 固化签名、审批绑定、审计追踪和拒绝机制 |
| M8 | 非功能达标 | 做资源占用、退化、backpressure、并发控制 |
| M9 | Authoring Frontend | 评估并落地 `run.gxl` 或 `run.war` 等 frontend |

---

## 4. 里程碑明细

### 4.1 M0 设计冻结

目标：

- 固定边缘唯一输入是 `ActionPlan IR`
- 固定 `constraints / program / attestation / provenance` 等关键词
- 固定单目标 `ActionPlan` 模型
- 固定第一版 opcode 白名单范围

子任务：

- 完成 `ActionPlan IR` 文档收敛
- 完成 `ActionResult` 与 `StepActionRecord` 概念收敛
- 完成第一版 opcode 范围确认
- 完成边缘进程边界确认：
  - `wp-agentd`
  - `wp-agent-exec`
  - `wp-agent-upgrader`
- 完成安全边界确认：
  - 边缘只执行 IR
  - 中心负责编译、审批、签名

验收标准：

- 核心设计文档不再在 IR 关键词上摇摆
- “边缘不接收 source files” 成为固定前提
- 第一版只支持白名单、非 shell、非动态下载执行

依赖关系：

- 无

### 4.2 M1 IR Schema

目标：

- 把 `ActionPlan` / `ActionResult` 变成机器可校验协议

子任务：

- 定义 `ActionPlan v1alpha1` schema
- 定义 `ActionResult v1alpha1` schema
- 定义 `StepActionRecord` schema
- 定义 `program.steps[]` 各 step type 的字段 schema
- 定义表达式节点 schema
- 实现本地 `plan validate` 校验器
- 准备正例、反例、越权样例、过期样例、图结构错误样例

验收标准：

- 可以静态校验版本、目标、签名字段、约束字段、step 图结构
- 非法 `invoke.op`、非法 `args`、非法 `allow` 越界都能被拒绝
- 样例集可作为后续回归基线

依赖关系：

- 依赖 M0

### 4.3 M2 Agentd Skeleton

目标：

- 先建立 `wp-agentd` 常驻守护进程骨架
- 明确 `wp-agentd` 是边缘控制器，而不是后置集成项

子任务：

- 建立常驻守护进程骨架
- 建立配置加载与进程生命周期管理
- 建立计划接收入口
- 建立本地校验入口
- 建立队列、并发控制和调度框架
- 建立 executor 拉起与回收框架
- 建立本地状态机与结果汇总骨架
- 预留与 `wp-agent-upgrader` 的协作边界

验收标准：

- `wp-agentd` 可以作为独立守护进程启动
- `wp-agentd` 的接收、校验、排队、调度骨架明确
- `wp-agentd` 不依赖 DSL parser
- `wp-agentd` 和 `wp-agent-exec` 的职责边界明确

依赖关系：

- 依赖 M1

### 4.4 M3 Exec Skeleton

目标：

- 建立 `wp-agent-exec` 最小执行器骨架
- 明确 `wp-agent-exec` 是受控执行内核，而不是边缘控制器

子任务：

- 建立 `ActionPlan` 加载与校验入口
- 建立执行上下文与变量绑定模型
- 实现 step 调度器
- 支持以下 step type：
  - `invoke`
  - `branch`
  - `guard`
  - `output`
  - `abort`
- 生成 `StepActionRecord`
- 生成 `ActionResult`
- 建立统一错误码与执行状态模型

验收标准：

- `wp-agent-exec` 可以独立执行最小单节点计划
- `wp-agent-exec` 不依赖 DSL parser
- `wp-agent-exec` 不允许 shell、脚本、动态下载
- 成功和失败路径都能产出结构化结果

依赖关系：

- 依赖 M1

### 4.5 M4 Local Edge Loop

目标：

- 打通边缘本地最小闭环
- 让 `wp-agentd` 成为计划接收、校验、调度、汇总入口

子任务：

- `wp-agentd` 接收 `ActionPlan`
- `wp-agentd` 做本地校验
- `wp-agentd` 做排队与并发控制
- `wp-agentd` 拉起 `wp-agent-exec`
- `wp-agent-exec` 执行最小计划
- `wp-agentd` 接收结果并汇总
- 建立本地执行状态机
- 支持取消、超时、拒绝执行

验收标准：

- 形成 `wp-agentd -> wp-agent-exec -> ActionResult` 闭环
- 非法计划在 `wp-agentd` 层就能被拒绝
- 成功和失败结果都可被 `wp-agentd` 汇总
- 有最小本地审计链路

依赖关系：

- 依赖 M2
- 依赖 M3

### 4.6 M5 Core Opcodes

目标：

- 先打通只读诊断与读取链路

建议首批 opcode：

- `process.list`
- `process.stat`
- `socket.check`
- `service.status`
- `file.read_range`
- `file.tail`
- `config.inspect`
- `agent.health_check`

子任务：

- 为每个 opcode 实现参数校验
- 为每个 opcode 实现返回结构
- 为每个 opcode 实现 allow/limits 检查
- 为每个 opcode 建立单测
- 为每个 opcode 建立错误场景测试

验收标准：

- 每个 opcode 都有 schema、实现、测试
- 读取类 opcode 强制走路径白名单
- 服务类 opcode 强制走服务白名单
- 返回结果全部结构化

依赖关系：

- 依赖 M4

### 4.7 M6 Control Plane MVP

目标：

- 形成从请求到边缘执行再到结果归档的最小控制平面闭环

子任务：

- 定义 `ActionRequest`
- 定义 `ApprovalRecord`
- 定义 `DispatchRecord`
- 实现 `ActionPlan` 编译器
- 实现目标展开逻辑
- 实现下发协议
- 实现 ACK 协议
- 实现结果回传与归档

验收标准：

- 单节点请求可完整跑通
- 多节点请求可展开为多个 `ActionPlan`
- 过期计划、审批缺失计划会被拒绝
- 控制平面能看到执行状态和结果

依赖关系：

- 依赖 M1
- 依赖 M4
- 依赖 M5

### 4.8 M7 Security Hardening

目标：

- 从“能跑”提升到“可控、可审计、可追责”

子任务：

- 实现计划签名与验签
- 绑定审批摘要到 `attestation`
- 建立 `request_id / action_id / execution_id` 贯穿链路
- 建立审计事件模型
- 建立拒绝执行原因码体系
- 建立篡改检测与来源校验

验收标准：

- 篡改计划会被拒绝
- 审批过期会被拒绝
- 不匹配目标 agent 的计划会被拒绝
- 控制平面可以回溯一次执行全过程

依赖关系：

- 依赖 M6

### 4.9 M8 非功能达标

目标：

- 把边缘执行能力拉到可上线水准

子任务：

- 定义空闲态 CPU / 内存目标
- 定义中等流量下 CPU / 内存目标
- 定义峰值流量下退化策略
- 定义 buffer 上限与 backpressure 策略
- 定义并发执行上限
- 做压测、故障注入、超时注入
- 做结果截断与资源保护

验收标准：

- 资源占用目标有量化指标
- 超时、取消、退化行为可预测
- 边缘执行不会拖垮业务节点
- 高压下仍能维持控制链路稳定

依赖关系：

- 依赖 M4
- 依赖 M6
- 依赖 M7

### 4.10 M9 Authoring Frontend

目标：

- 在 IR 稳定后再决定作者输入形态

候选项：

- `run.gxl`
- `run.war`
- `native_json`

子任务：

- 评估 frontend 可读性
- 评估 AI 生成稳定性
- 评估静态分析难度
- 评估与现有工具链复用程度
- 设计 lowering 到 `ActionPlan IR` 的编译器
- 建立 authoring 样例与 lint 规则

验收标准：

- 至少一个 frontend 能稳定编译到 `ActionPlan IR`
- frontend 不增加边缘复杂度
- frontend 选择不影响既有执行器稳定性

依赖关系：

- 依赖 M1
- 最好依赖 M6

---

## 5. 第一阶段推荐范围

如果目标是尽快做出可信的最小可运行版本，建议第一阶段只覆盖：

- M0
- M1
- M2
- M3
- M4
- M5

这样可以先得到：

- 固定的执行契约
- 可校验的 IR 协议
- 可运行的 `wp-agentd` 守护进程骨架
- 可运行的 `wp-agent-exec` 执行器骨架
- 打通的本地边缘执行闭环
- 第一批真实可用的只读能力

这会成为后续控制平面、审批、签名和 frontend 选择的稳定基础。

---

## 6. 建议的开发分组

建议按四条并行主线组织工作：

### 6.1 协议与模型线

负责：

- `ActionPlan`
- `ActionResult`
- 表达式节点
- schema
- 版本兼容策略

### 6.2 边缘执行器线

负责：

- `wp-agent-exec`
- step 调度
- opcode runtime
- 结果汇总
- 资源保护

### 6.3 Agent 集成线

负责：

- `wp-agentd`
- 接收与排队
- 本地状态机
- 与 exec / upgrader 进程交互

### 6.4 控制平面线

负责：

- 请求
- 审批
- 编译
- 下发
- 回传
- 归档

---

## 7. 当前建议

当前最值得立刻启动的是：

1. 把 `ActionPlan v1alpha1` schema 明确下来
2. 把 `ActionResult v1alpha1` schema 明确下来
3. 确定 `program.steps[]` 的字段级定义
4. 先开始 `wp-agentd` 最小守护进程骨架
5. 再开始 `wp-agent-exec` 最小 runtime 骨架

也就是说，下一步不再继续抽象讨论，而是进入：

- 协议定稿
- `wp-agentd` skeleton
- `wp-agent-exec` skeleton
- 首批 opcode 实现

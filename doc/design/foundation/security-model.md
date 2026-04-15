# warp-insight 安全模型设计

## 1. 文档目的

本文档在 [`target.md`](target.md) 和 [`architecture.md`](architecture.md) 的基础上，进一步定义 `warp-insight` 的安全模型，重点回答以下问题：

- 环境内 `warp-insightd`、`warp-insight-exec`、`warp-insight-upgrader` 之间的信任边界如何划分
- 中心节点与环境内 agent 之间如何建立可信控制链路
- 远程动作和自我升级如何做到可限权、可审批、可审计、可回滚
- 如何把“业务优先、故障隔离、资源封顶、可治理”落实到安全架构中

本文档优先定义第一版安全边界和控制原则，不展开到具体加密算法选型和最终 API 字段级别。

相关设计文档：

- [`architecture.md`](architecture.md)：环境内三进程模型和控制链路
- [`action-dsl.md`](../execution/action-dsl.md)：动作 DSL、执行 IR、opcode 白名单和审批绑定
- [`control-plane.md`](../center/control-plane.md)：请求、审批、编译、下发、执行、审计状态机

---

## 2. 安全目标

`warp-insight` 的安全目标不是简单“通信加密”，而是同时满足以下要求：

- 可信身份：
  中心节点、环境内 agent、升级包、远程动作请求都必须有明确身份
- 最小权限：
  采集、发现、升级、远程动作不共享不必要的高权限
- 强审计：
  高风险动作必须形成完整的请求、审批、执行、回传、归档链路
- 可回滚：
  升级失败、策略错误、执行异常后必须可以回退到稳定状态
- 可隔离：
  远程执行和升级能力不能反向污染采集热路径
- 可约束：
  中心控制面即使拥有调度权，也不能绕过审批和白名单直接下发任意动作

---

## 3. 安全原则

第一版设计建议遵循以下原则：

- 默认拒绝：
  未显式允许的动作、权限、目标范围默认拒绝
- 分层信任：
  中心节点、`warp-insightd`、`warp-insight-exec`、`warp-insight-upgrader` 之间不是同级全信任
- standalone 先收敛：
  第一阶段先验证 `standalone` 文件日志替代切片的本地安全边界，再叠加 `managed` 接入、远程动作和中心编排升级
- 最小暴露面：
  环境内只暴露必要通信接口，不引入不必要常驻监听面
- 高风险动作强治理：
  升级与远程执行必须受策略、审批、审计约束
- 身份与授权分离：
  能证明“你是谁”不代表“你能做什么”
- 资源与安全一体化：
  超时、并发、输出大小、CPU、内存上限也属于安全边界的一部分

---

## 4. 信任边界

### 4.1 核心边界

系统至少存在以下信任边界：

- 边界 A：
  中心节点与环境内 agent 之间
- 边界 B：
  `warp-insightd` 与 `warp-insight-exec` 之间
- 边界 C：
  `warp-insightd` 与 `warp-insight-upgrader` 之间
- 边界 D：
  agent 进程与宿主机操作系统 / 容器运行时 / Kubernetes 节点之间
- 边界 E：
  租户 A 与租户 B、环境 A 与环境 B 之间

### 4.2 信任结论

第一版应明确以下结论：

- 中心节点可信，但中心节点内部不同角色也要受 RBAC 和审批约束
- `warp-insightd` 是本地唯一协调入口，但不是“本地无限权限根”
- `warp-insight-exec` 是高风险受控执行器，不应默认继承 `warp-insightd` 的全部能力
- `warp-insight-upgrader` 是高风险切换器，不应承担采集和远程动作职责
- 宿主环境默认视为半可信，必须考虑本地篡改、环境异常和资源压力

---

## 5. 身份模型

### 5.1 身份主体

建议至少定义以下身份主体：

- `control-plane`
- `agentd`
- `agent-exec`
- `agent-upgrader`
- `human-operator`
- `automation-client`

### 5.2 agent 身份

每个环境内 agent 至少需要具备：

- `agent_id`
- `tenant_id`
- `environment_id`
- `node_id`
- `instance_id`
- `capabilities`
- `issued_at`
- `expires_at`

其中：

- `agent_id` 表示逻辑 agent 身份
- `instance_id` 表示具体运行实例，便于升级切换和异常追踪
- `capabilities` 表示该节点支持哪些输入、发现器、远程动作类型和升级能力

### 5.3 进程身份继承规则

环境内三类进程的身份关系不应等同：

- `warp-insightd` 持有与中心节点通信所需的主身份
- `warp-insight-exec` 不直接长期持有中心通信主身份
- `warp-insight-upgrader` 不直接长期持有中心通信主身份

建议模型是：

- `warp-insightd` 作为本地根协调者，接收中心指令
- `warp-insightd` 为每次执行动作生成一次性本地执行上下文
- `warp-insight-exec` / `warp-insight-upgrader` 只在该上下文内拥有临时执行权限

也就是说，执行器和升级器应基于“临时授权”，而不是“永久继承主权限”。

---

## 6. 认证与信任建立

### 6.1 中心与 agent 之间

中心节点与环境内 agent 之间必须建立双向可信认证。

第一版建议：

- 双向 TLS 或等价双向身份认证机制
- agent 首次注册需要引导式信任建立
- 证书或令牌必须有轮换机制
- 控制命令必须绑定发送者身份和请求上下文

### 6.2 升级包信任

升级包必须满足：

- 来源可信
- 版本可识别
- 签名或摘要可校验
- 与目标节点能力和版本兼容性可验证

未通过校验的升级包必须拒绝执行。

### 6.3 远程动作信任

远程动作计划必须至少包含：

- 请求人或调用方身份
- 目标范围
- 动作类型
- 参数摘要
- 风险等级
- 审批要求
- 请求时间和过期时间

如果上述信息不完整，`warp-insightd` 不应接受执行。

### 6.4 standalone 模式安全基线

在 `standalone` 模式下，第一版安全边界应固定为：

- 没有中心节点时，不建立远程动作信任链，也不开放依赖中心编排的升级链路
- `warp-insightd` 只承担本地采集、状态落盘、checkpoint 推进和数据上送 / 输出职责
- 文件日志输入链路的安全重点不是“谁下发了动作”，而是“本地状态是否被错误推进、篡改或误恢复”

因此 `standalone` 替代切片至少要满足：

- `checkpoint_offset` 只能在越过 `commit point` 后推进
- rotate / truncate / restart recovery 不得把未确认数据误标记为已提交
- 本地 `buffer / spool / checkpoint` 状态写入必须原子、可校验、可恢复
- `managed` 模式才启用的远程动作、中心升级和审批链路，不得在 `standalone` 中被隐式开启

---

## 7. 权限模型

### 7.1 权限域划分

建议把权限分成四层：

- 控制面权限：
  谁可以创建策略、升级计划、远程动作计划
- 审批权限：
  谁可以批准高风险动作
- 节点执行权限：
  哪些节点允许执行哪些动作
- 本地进程权限：
  `warp-insightd`、`warp-insight-exec`、`warp-insight-upgrader` 各自拥有哪些 OS / runtime 权限

### 7.2 本地最小权限

环境内应尽量做到：

- `warp-insightd` 只保留采集、发现、上送、状态管理所需权限
- `warp-insight-exec` 只在动作执行期间获得必要权限
- `warp-insight-upgrader` 只在升级窗口内获得安装、切换、回滚所需权限

不应接受以下设计：

- 用一个长期 root 进程同时承担采集、远程执行和升级
- 执行器默认可以访问全部采集配置和全部缓冲数据
- 升级器默认可执行任意运维动作

### 7.3 动作白名单

第一版远程动作必须基于白名单，而不是任意 shell。

建议白名单至少包含以下属性：

- `action_type`
- `allowed_targets`
- `required_role`
- `risk_level`
- `timeout_limit`
- `concurrency_limit`
- `stdout_limit`
- `stderr_limit`
- `requires_approval`

### 7.4 多租户与多环境隔离

中心节点必须保证：

- 租户 A 不能下发影响租户 B 的动作
- 环境 A 的高权限操作不能越界到环境 B
- 查询、审计、审批和回滚记录都必须按租户 / 环境隔离

---

## 8. 远程动作安全模型

### 8.1 动作分类

远程动作建议按风险分级：

- `R0` 只读诊断：
  如读取状态、查看版本、查看健康信息
- `R1` 低风险运维：
  如触发轻量自检、刷新某类缓存
- `R2` 中风险控制：
  如重载局部配置、重启局部组件
- `R3` 高风险动作：
  如服务切换、系统级操作、潜在破坏性动作

### 8.2 审批规则

建议默认规则：

- `R0` 可免审批，但必须审计
- `R1` 由有权限角色直接发起并审计
- `R2` 需要更高角色或双人确认
- `R3` 必须显式审批，并要求更严格时间窗、目标范围和并发限制

### 8.3 执行器约束

`warp-insight-exec` 必须具备以下硬约束：

- 明确超时
- 明确并发限制
- 明确输出大小上限
- 明确 CPU / 内存限制
- 明确可访问文件、网络、子进程范围
- 执行结束自动退出

### 8.4 本地调用链

远程动作本地调用链建议固定为：

1. `warp-insightd` 接收并校验 `DispatchActionPlan` / `ActionPlan`
2. `warp-insightd` 生成 `action_id` 和本地一次性执行上下文
3. `warp-insightd` 拉起 `warp-insight-exec`
4. `warp-insight-exec` 在受限上下文中执行动作
5. `warp-insight-exec` 输出 `ActionResult` 并退出
6. `warp-insightd` 对结果计算摘要并做结果级签名
7. `warp-insightd` 汇总结果、记录审计并上报中心

### 8.5 禁止事项

第一版明确禁止：

- 中心节点直接把任意 shell 文本透传到节点执行
- 执行器绕过 `warp-insightd` 直接接受中心编排
- 未带审批上下文的高风险动作直接执行
- 无超时、无输出上限、无并发限制的长时间执行任务

---

## 9. 升级安全模型

### 9.1 升级链路

升级建议采用如下受控链路：

1. 中心创建 `UpgradePlan`
2. 中心校验目标范围、版本兼容性和发布批次
3. `warp-insightd` 接收计划并做本地预检查
4. `warp-insightd` 拉起 `warp-insight-upgrader`
5. `warp-insight-upgrader` 下载、校验、切换、探活、必要时回滚
6. `warp-insightd` 汇总结果并上报中心

### 9.2 升级校验

升级至少应校验：

- 升级包签名或摘要
- 版本来源
- 与本机架构、OS、部署形态的兼容性
- 升级前磁盘与缓冲状态是否满足门槛
- 升级后健康检查是否通过

### 9.3 回滚安全

回滚必须满足：

- 可由本地自动触发
- 可由中心编排触发
- 回滚动作本身可审计
- 回滚后能恢复到上一个稳定版本和可用状态

### 9.4 升级器权限约束

`warp-insight-upgrader` 只应拥有升级所需最小权限，例如：

- 读取当前版本信息
- 写入目标安装目录
- 切换当前运行版本
- 触发健康检查
- 恢复旧版本

不应默认拥有：

- 任意远程动作执行能力
- 不受约束的系统级管理能力
- 长期常驻高权限会话

---

## 10. 审计模型

### 10.1 审计目标

以下行为必须形成完整审计链：

- agent 注册和身份变化
- 策略下发与版本切换
- 升级计划、执行、回滚
- 远程动作请求、审批、执行、结果
- 降级、限流、丢弃、保护模式切换

### 10.2 审计字段

建议关键审计事件至少包含：

- `audit_id`
- `request_id`
- `action_id`
- `tenant_id`
- `environment_id`
- `agent_id`
- `actor`
- `action_type`
- `risk_level`
- `approved_by`
- `started_at`
- `finished_at`
- `result`
- `reason`

### 10.3 审计链要求

审计链必须满足：

- 请求前后可关联
- 中心与边缘记录可关联
- 回滚、重试、超时、中断都有明确记录
- 审计记录不可被普通执行路径静默覆盖

---

## 11. 资源限制也是安全边界

对于 `warp-insight` 来说，资源失控本身就是安全问题和稳定性问题。

因此以下限制属于安全模型的一部分：

- 执行器并发上限
- 执行器 CPU / 内存上限
- 执行器输出上限
- 升级器运行时限
- agent 本地 buffer 上限
- 回传结果大小上限

如果这些边界缺失，即使认证和审批做得很好，系统仍可能因为资源耗尽而破坏业务优先原则。

---

## 12. 故障与攻击场景

第一版至少应考虑以下场景：

- 中心节点被误操作，下发错误高风险动作
- 低权限用户尝试越权对高价值节点执行远程动作
- 节点本地执行器卡死或输出失控
- 升级包损坏、来源错误或版本不兼容
- 节点本地环境被篡改，试图伪造执行结果
- 中心节点短时不可达，agent 仍需保持采集稳定
- `standalone` 文件日志链路在 rotate / truncate / restart 后错误推进 `checkpoint`

对应防线应包括：

- RBAC + 审批 + 白名单
- action 级约束与本地执行上下文限制
- 超时 / 并发 / 输出 / 资源限制
- 包签名与版本校验
- 双向身份认证、结果级完整性保护与结果链路审计
- 本地保护模式与中心端状态感知
- `commit point -> checkpoint_offset` 的单向推进和恢复校验

---

## 13. 第一阶段落地建议

第一阶段建议优先落地以下能力：

- `warp-insightd / warp-insight-exec / warp-insight-upgrader` 三进程角色固定
- `standalone` 替代切片的本地状态边界：
  `file input -> parser / multiline -> checkpoint / commit point -> buffer / spool -> warp-parse/file output`
- `standalone` 下默认关闭远程动作和中心编排升级入口
- 中心到 `warp-insightd` 的双向身份认证
- 远程动作白名单模型
- 风险等级与审批规则
- 升级包签名校验
- `action_id / request_id / audit_id` 审计主键体系
- 执行器超时、并发、输出、资源上限
- 回滚和失败状态回报

---

## 14. 当前结论

`warp-insight` 的第一版安全模型可以概括为：

- 第一安全验证门应先落在 `standalone` 替代切片，本地数据面状态推进和恢复边界必须先成立
- 中心节点负责策略、授权、审批、编排和归档，但不能绕过治理直接把任意动作塞进边缘节点
- `warp-insightd` 是本地唯一控制入口，但不应承载无限执行权
- `warp-insight-exec` 和 `warp-insight-upgrader` 是受临时授权驱动的高风险执行进程，不应长期继承主身份和高权限
- 远程动作和升级都必须走“身份 -> 授权 -> 审批 -> 执行 -> 回传 -> 审计 -> 必要时回滚”的完整闭环
- 在 `standalone` 模式下，默认只成立本地采集与状态恢复链路，不应隐式暴露 `managed` 才具备的控制能力
- 资源限制、超时、并发和输出上限不是附属优化，而是安全边界的一部分

如果这些边界不成立，`warp-insight` 即使功能完整，也不能算一个可投入生产环境的系统。

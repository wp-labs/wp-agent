# wp-agent 多里程碑开发路线图

## 1. 文档目的

本文档把 `wp-agent` 当前已经确认的目标、架构和边界，收敛成一份更细粒度的全项目开发路线图。

这里的“全项目”覆盖六条主线：

- 边缘身份与接入
- 边缘控制执行面
- 边缘数据面
- 中心控制平面
- 安装升级与发布
- 规模化与非功能达标

路线图的作用不是把所有事情都塞进第一阶段，而是明确：

- 哪些能力是前置依赖
- 哪些能力可以并行
- 哪些能力必须先做“基线版”，后做“强化版”
- 哪些能力不应过早投入

相关文档：

- [`target.md`](target.md)
- [`architecture.md`](architecture.md)
- [`security-model.md`](security-model.md)
- [`non-functional-targets.md`](non-functional-targets.md)
- [`action-plan-ir.md`](../execution/action-plan-ir.md)
- [`action-plan-schema.md`](../execution/action-plan-schema.md)
- [`action-result-schema.md`](../execution/action-result-schema.md)
- [`agentd-architecture.md`](../edge/agentd-architecture.md)
- [`agentd-exec-protocol.md`](../edge/agentd-exec-protocol.md)
- [`agent-config-schema.md`](../edge/agent-config-schema.md)
- [`control-plane.md`](../center/control-plane.md)
- [`control-center-architecture.md`](../center/control-center-architecture.md)
- [`agent-gateway-protocol.md`](../center/agent-gateway-protocol.md)
- [`metrics-integration-roadmap.md`](../telemetry/metrics-integration-roadmap.md)
- [`telemetry-uplink-and-warp-parse.md`](../telemetry/telemetry-uplink-and-warp-parse.md)
- [`action-dsl.md`](../execution/action-dsl.md)

---

## 2. 当前逻辑判断

当前路线的大方向是对的，但原版本还不够细，也有几个结构性缺口。

当前已经正确的地方：

1. 先冻结契约，再堆实现
2. 先做边缘最小闭环，再扩中心治理闭环
3. 先做常驻数据面和只读能力，再做高风险控制能力
4. 先做单级接入，再做多级树扩展
5. AI 放在中心，不放在边缘热路径

当前需要修正的地方：

1. 身份注册、证书、首次 enrollment 不能隐含在 `Gateway MVP` 里，必须单列前置里程碑
2. 资源发现不能只挂在 metrics 路线下面，必须是一条独立底盘能力
3. 日志、traces、security 不能长期缺位，否则和目标定义不一致
4. 安全不能全部放到后期 hardening，必须区分“基线安全”与“强化安全”
5. 安装 bootstrap 和中心编排升级不能混成一个里程碑
6. 验证、fixture、e2e harness 不应只在 GA 阶段才出现
7. `wp-agentd` 无中心节点独立运行能力必须前置，而不是做成受管模式的副产品
8. 数据面 `V1` 上报目标不能默认假设是中心 telemetry ingest，应先允许统一结构化文本 record -> `warp-parse`

因此，本路线图在逻辑上应调整为：

- 先补“身份接入基线”
- 先补“standalone 边缘运行基线”
- 先补“统一结构化 record -> warp-parse”的数据面成立路径
- 再补“边缘控制与数据双底盘”
- 再做“中心治理闭环”
- 再做“规模化、统一多信号和世界级非功能”

---

## 3. 核心推进原则

路线图固定遵循以下原则：

1. 先冻结协议、对象和状态机，再写运行时代码
2. 先做基线能力，再做强化能力
3. 先让边缘节点能独立稳定运行，再接入中心
4. 先让中心能治理单级 agent，再扩展到多级树
5. 先把 metrics 底盘做稳，再扩展 logs / traces / security
6. 先把安装和版本治理做成工程能力，再追求大规模自动化 rollout
7. 先达到“可上线”，再追求“世界第一”的非功能水准
8. 先保证 `standalone` 模式成立，再叠加 `managed` 模式能力
9. 先让数据面通过 `warp-parse` 成立，再决定是否补原生 metrics / traces receiver

当前阶段不应犯的错误：

- 把 `run.gxl` 或 `run.war` 当成最前置工作
- 先做 UI，再补控制闭环
- 先做多级树，再补单级稳定性
- 用远程 action 补 telemetry 采集主路径
- 把 AI 放进边缘热路径
- 把安全基线延后到功能全部跑通之后

---

## 4. 阶段总览

建议把全项目分成五个阶段推进。

| 阶段 | 范围 | 目标 |
|---|---|---|
| P0 | M0-M3 | 冻结契约，补齐 standalone 边缘运行基线与三进程基础骨架 |
| P1 | M4-M7 | 在 standalone 基础上叠加 managed 会话、执行闭环、资源发现和数据面底盘 |
| P2 | M8-M11 | 打通中心治理、安装升级和安全审计闭环 |
| P3 | M12-M14 | 打通多级树扩容和统一多信号能力 |
| P4 | M15-M16 | 落地 AI / authoring 与世界级非功能验收 |

---

## 5. 里程碑总览

| 里程碑 | 名称 | 目标 |
|---|---|---|
| M0 | 设计冻结 | 固定关键词、边界、协议对象和第一版范围 |
| M1 | 契约与 Schema | 冻结核心 schema、版本与校验器 |
| M2 | Identity / Enrollment Baseline | 固定首次注册、证书、agent 身份与实例语义 |
| M3 | Edge Runtime Skeleton | 建立支持 `standalone` 的 `wp-agentd` / `wp-agent-exec` / `wp-agent-upgrader` 基础骨架 |
| M4 | Gateway Session MVP | 打通 `hello`、heartbeat、capability 和会话管理 |
| M5 | Controlled Action MVP | 打通本地执行闭环和首批只读 opcode |
| M6 | Resource Discovery Foundation | 建立 host/process/container/pod/service 发现底盘 |
| M7 | Telemetry Core + WarpParse Uplink + Batch A | 建立统一结构化 record 数据面，并通过 `warp-parse` 打通 Batch A |
| M8 | Control Center MVP | 打通请求、审批、编译、签名、下发、ack、结果归档 |
| M9 | Install Bootstrap MVP | 打通安装、首启、注册、版本落盘和本地恢复 |
| M10 | Upgrade / Rollback MVP | 打通升级包、切换、健康检查、回滚和中心编排 |
| M11 | Security / Audit Baseline + Hardening | 固化基线安全、审批绑定、审计和拒绝机制 |
| M12 | Scale-Out Gateway | 做 gateway 分片、容量模型和 storm protection |
| M13 | Tree Topology | 做多级树、relay、hop 路由和分层接入 |
| M14 | Unified Signals Expansion | 扩展 logs / traces / security 与 Batch B/C telemetry |
| M15 | AI 与 Authoring | 落地中心 AI 辅助与作者侧 frontend |
| M16 | GA / World-Class NFR | 完成可靠性、压测、故障注入和上线门槛验收 |

---

## 6. 里程碑明细

### 6.1 M0 设计冻结

目标：

- 固定两层架构：
  - 边缘 agent
  - 中心节点
- 固定三进程边界：
  - `wp-agentd`
  - `wp-agent-exec`
  - `wp-agent-upgrader`
- 固定边缘唯一执行输入为 `ActionPlan`
- 固定南向逻辑协议对象
- 固定第一版 telemetry / discovery / remote action 的能力边界

子任务：

- 收敛术语词典
- 收敛 `ActionPlan` / `ActionResult` 概念
- 收敛控制平面对象
- 收敛 `Agent Gateway` 协议职责
- 收敛 telemetry 底盘目标与 exporter 策略

验收标准：

- 设计文档不再在核心关键词上反复摇摆
- “边缘不接收 source files” 成为固定前提
- “AI 不进入边缘热路径” 成为固定前提

依赖关系：

- 无

### 6.2 M1 契约与 Schema

目标：

- 把核心对象都变成机器可校验契约

子任务：

- 定义 `ActionPlan v1alpha1`
- 定义 `ActionResult v1alpha1`
- 定义 `CapabilityReport`
- 定义 `DispatchActionPlan / ActionPlanAck / ReportActionResult`
- 定义 `agent-config` 主配置骨架
- 定义 `agentd` 本地状态 schema
- 建立 schema 校验器
- 建立正例、反例、过期、篡改、越权样例集

验收标准：

- 核心对象都可静态校验
- 非法字段、非法状态组合、非法目标和非法约束都能被拒绝
- 样例集可作为 CI 回归基线

依赖关系：

- 依赖 M0

### 6.3 M2 Identity / Enrollment Baseline

目标：

- 固定 agent 首次注册和持续身份模型

子任务：

- 定义 `agent_id / instance_id / boot_id` 语义
- 定义首次 enrollment 流程
- 定义 mTLS 证书或等价身份材料发放方式
- 定义首次安装后的 identity 固化方式
- 定义证书轮转和实例替换基本规则
- 定义被吊销、被替换、重复实例的处理规则

验收标准：

- 新安装 agent 能完成首次注册
- 重启后实例语义和持久身份语义不混淆
- 重复实例和身份冲突能被中心识别

依赖关系：

- 依赖 M1

### 6.4 M3 Edge Runtime Skeleton

目标：

- 建立边缘三进程最小骨架
- 建立没有中心节点也能正常工作的 `standalone` 基线

子任务：

- 建立 `wp-agentd` 常驻进程骨架
- 建立 `wp-agent-exec` 最小 runtime 骨架
- 建立 `wp-agent-upgrader` 最小骨架
- 建立 `agentd <-> exec` 本地协议骨架
- 建立工作目录、状态落盘和生命周期管理
- 建立基础自观测、错误码和 panic/restart 框架
- 建立 `control_plane.enabled = false` 时的启动与运行路径
- 建立最小测试 harness

验收标准：

- 三个二进制具备独立启动与最小健康检查能力
- `wp-agentd` 能管理本地状态和子进程生命周期
- 没有中心节点时，`wp-agentd` 仍可稳定进入常驻运行
- 边缘实现不依赖 DSL parser

依赖关系：

- 依赖 M1

### 6.5 M4 Gateway Session MVP

目标：

- 建立边缘与中心之间的最小受信控制通道

子任务：

- 实现 `AgentHello`
- 实现 heartbeat
- 实现 `CapabilityReport`
- 建立 `WebSocket over mTLS` 南向长连接
- 建立 session / lease / reconnect 机制
- 建立 `AgentRegistry` 最小在线状态表
- 建立基础连接级审计

验收标准：

- 单个 `wp-agentd` 能稳定接入 `gateway`
- 首次 enrollment、正常重连、实例替换都能跑通
- 中心能看到在线状态和能力上报

依赖关系：

- 依赖 M2
- 最好依赖 M3

### 6.6 M5 Controlled Action MVP

目标：

- 打通受控远程执行最小闭环

子任务：

- `wp-agentd` 接收 `DispatchActionPlan`
- `wp-agentd` 做本地校验、排队与调度
- `wp-agentd` 拉起 `wp-agent-exec`
- `wp-agent-exec` 执行最小 step runtime
- `wp-agentd` 汇总结果并上报 `ActionPlanAck` / `ReportActionResult`
- 实现首批只读 opcode：
  - `process.list`
  - `process.stat`
  - `socket.check`
  - `service.status`
  - `file.read_range`
  - `file.tail`
  - `config.inspect`
  - `agent.health_check`
- 建立 opcode 参数校验、allow 校验和 fixture

验收标准：

- 单节点 action 可完整跑通
- 非法计划在边缘侧会被拒绝
- 成功、失败、取消、超时都能产出结构化结果
- 首批 opcode 有实现、有测试、有错误场景覆盖

依赖关系：

- 依赖 M3
- 依赖 M4

### 6.7 M6 Resource Discovery Foundation

目标：

- 建立资源发现底盘，而不是只在 metrics 内零散实现

子任务：

- 建立 host discovery
- 建立 process discovery
- 建立 container discovery
- 建立 pod / service 基础发现
- 建立 resource identity 与去重规则
- 建立 discovery cache 与 refresh 策略
- 建立 discovery 输出到 telemetry / action target selector 的共享模型

验收标准：

- 边缘节点可稳定产出统一 resource 清单
- 同一资源不会在不同模块中产生冲突 identity
- discovery 结果可同时服务 telemetry 和控制平面

依赖关系：

- 依赖 M3

### 6.8 M7 Telemetry Core + WarpParse Uplink + Batch A

目标：

- 建立边缘数据面底盘
- 建立统一结构化文本 record
- 通过 `warp-parse` 打通 `V1` 上报路径并落地 Batch A metrics

子任务：

- 建立 input / scheduler / normalize / buffer / exporter 基础流水线
- 定义统一 telemetry record envelope
- 建立 `warp_parse` exporter target
- 建立 `file / object_store` fallback target
- 建立 OTel 对齐与统一 resource 绑定
- 建立 telemetry budget 与 backpressure 基础实现
- 建立 Batch A metrics：
  - `host_metrics`
  - `process_metrics`
  - `container_metrics`
  - `k8s_node_pod_metrics`
  - `prom_scrape`
  - `otlp_metrics_receiver`
- 建立 `logs / metrics / traces / security` 到统一结构化 record 的编码规则
- 建立批量 fixture、回放和采样测试

验收标准：

- 边缘侧可稳定输出 Batch A 指标
- `wp-agentd` 可把多信号编码成统一结构化文本 record
- `warp-parse` 可作为 `V1` 统一数据接收器稳定接收这些 record
- 标准化结果可挂接统一 resource 语义
- 数据面拥塞不会直接打挂控制面

依赖关系：

- 依赖 M3
- 依赖 M6


### 6.9 M8 Control Center MVP

目标：

- 建立最小可用的中心控制闭环

子任务：

- 实现 `Northbound API`
- 实现 `ActionTemplate` / `ActionRequest`
- 实现 `ApprovalRecord`
- 实现 `Plan Compiler`
- 实现 `Signer`
- 实现 `Dispatch Service`
- 实现 `Execution Tracker`
- 实现 `Result Ingestor`
- 实现控制中心主库存储与对象存储落盘

验收标准：

- 请求可编译成单目标 `ActionPlan`
- 多目标请求可展开成多份 plan
- 中心可查看 dispatch、ack、execution、result 状态
- 中心与边缘形成端到端可观察闭环

依赖关系：

- 依赖 M1
- 依赖 M2
- 依赖 M4
- 依赖 M5

### 6.10 M9 Install Bootstrap MVP

目标：

- 打通“安装后能活起来”的基础工程能力

子任务：

- 设计安装包与目录布局
- 设计首次安装与首次启动流程
- 落地本地配置初始化
- 落地本地 identity / version / state 落盘
- 落地 systemd 或等价服务托管方式
- 落地 crash restart 与最小恢复

验收标准：

- 新节点可完成安装并稳定拉起 `wp-agentd`
- 重启后能恢复最小本地状态
- 安装阶段和升级阶段的职责边界明确

依赖关系：

- 依赖 M2
- 依赖 M3

### 6.11 M10 Upgrade / Rollback MVP

目标：

- 打通升级包、切换、健康检查和回滚闭环

子任务：

- 设计版本清单、签名校验与下载策略
- 打通 `wp-agent-upgrader`
- 打通升级前检查、切换、健康检查、回滚
- 建立升级编排最小中心对象与协议
- 建立升级与 action 的互斥规则
- 建立升级结果回报

验收标准：

- 单节点可完成升级与回滚
- 升级失败可回滚到上一个稳定版本
- 中心能看到升级分发与结果

依赖关系：

- 依赖 M8
- 依赖 M9

### 6.12 M11 Security / Audit Baseline + Hardening

目标：

- 把“基线安全”与“强化安全”合并成明确交付

基线安全范围：

- mTLS 接入
- 计划签名与验签
- opcode / path / service allow 控制
- 最小权限运行

强化安全范围：

- 审批摘要绑定到 `attestation`
- `request_id / action_id / dispatch_id / execution_id` 全链路贯通
- `result_attestation`
- 审计事件、审计查询和篡改检测
- 高风险动作审批门禁

验收标准：

- 篡改计划会被拒绝
- 审批缺失或过期会被拒绝
- 越权 opcode、越权路径、越权服务操作会被拒绝
- 一次执行的全链路都可审计回放

依赖关系：

- 依赖 M4
- 依赖 M5
- 依赖 M8
- 最好依赖 M10

### 6.13 M12 Scale-Out Gateway

目标：

- 让南向接入层具备横向扩展能力

子任务：

- 拆分独立 `Agent Gateway` 接入层
- 建立 gateway 水平分片
- 建立在线状态收敛与路由索引
- 建立 reconnect storm 保护
- 建立 dispatch 重试与回压策略
- 建立容量基线：
  - 单节点扇出
  - 心跳吞吐
  - 未确认 dispatch 水位

验收标准：

- 单级接入可稳定横向扩展
- 单点故障不会导致全局会话雪崩
- 控制中心主服务不被长连接接入层拖垮

依赖关系：

- 依赖 M8
- 依赖 M11

### 6.14 M13 Tree Topology

目标：

- 让系统从单级接入演进为多级树结构

子任务：

- 建立 relay / lower-gateway 节点模型
- 建立 hop 元数据与 relay 路由规则
- 建立父子会话与租约模型
- 建立树形路由与故障切换规则
- 建立树形审计链
- 建立万级叶子容量验证

验收标准：

- 可稳定承载万级叶子规模的分层接入
- 树形扩容不改变南向逻辑协议
- 审计系统能区分最终执行节点与中间转发节点

依赖关系：

- 依赖 M12

### 6.15 M14 Unified Signals Expansion

目标：

- 把系统从“metrics 底盘”扩展为“统一多信号底盘”

子任务：

- 扩展 logs 输入与标准化
- 评估并视需要补充原生 OTLP logs / traces receiver
- 扩展 security event 标准化
- 建立同一份原始事件下的多信号关联规则
- 落地 telemetry Batch B/C：
  - `StatsD`
  - `JMX`
  - `nginx`
  - `mysql`
  - `postgresql`
  - `redis`
  - `kafka`
  - `elasticsearch`
  - `rabbitmq`
  - `clickhouse`
  - `coredns`
  - `kube-apiserver`
  - `kubelet`
  - `etcd`

验收标准：

- 常见 target 大多可由 `wp-agentd` 直接采集
- logs / metrics / traces / security 能挂接统一 resource 上下文
- exporter compatibility mode 退居 fallback 角色

依赖关系：

- 依赖 M6
- 依赖 M7
- 最好依赖 M11

### 6.16 M15 AI 与 Authoring

目标：

- 在不破坏边缘确定性的前提下，把 AI 能力放到中心节点

子任务：

- 落地 AI 辅助 request 生成
- 落地风险解释与审批建议
- 落地失败原因摘要与回滚建议
- 评估并选择作者侧 frontend：
  - `run.gxl`
  - `run.war`
  - `native_json`
- 实现 frontend 到 `ActionPlan` 的 lowering
- 建立 lint、样例和 CI 校验

验收标准：

- AI 只产生建议，不直接拥有签发权限
- 至少一个 frontend 可稳定编译到 `ActionPlan`
- frontend 的引入不增加边缘复杂度

依赖关系：

- 依赖 M1
- 依赖 M8
- 最好依赖 M11

### 6.17 M16 GA / World-Class NFR

目标：

- 把系统从“功能可用”拉到“可大规模上线”

子任务：

- 按 [`non-functional-targets.md`](non-functional-targets.md) 做资源验收
- 做长稳压测、故障注入、网络抖动、中心不可达测试
- 做升级失败、结果回报失败、spool 填满、重连风暴测试
- 做 CPU / 内存 / fd / 线程 / buffer 预算收敛
- 做 protect / degraded 模式验收
- 建立 SLO、报警和发布门槛
- 建立全链路 e2e certification 套件

验收标准：

- `idle` / `moderate` / `peak` 资源目标达标
- 保护模式可预测且不会拖垮宿主业务
- action 结果、审计事件和关键控制状态具备保底能力
- 具备版本发布与回滚门槛

依赖关系：

- 依赖 M10
- 依赖 M11
- 依赖 M12
- 依赖 M13
- 依赖 M14

---

## 7. 推荐并行工作流

建议按七条主线并行组织开发。

### 7.1 契约与模型线

负责：

- schema
- 版本策略
- 错误码
- capability 定义
- 配置结构

主要覆盖：

- M0
- M1
- M11

### 7.2 身份与接入线

负责：

- enrollment
- 证书与身份
- gateway session
- online state

主要覆盖：

- M2
- M4
- M12
- M13

### 7.3 边缘执行线

负责：

- `wp-agent-exec`
- opcode runtime
- `ActionResult`
- 执行资源保护

主要覆盖：

- M3
- M5
- M11

### 7.4 边缘守护与升级线

负责：

- `wp-agentd`
- `wp-agent-upgrader`
- 本地状态机
- 安装与升级

主要覆盖：

- M3
- M9
- M10
- M16

### 7.5 边缘数据面线

负责：

- discovery
- inputs
- collectors
- normalize
- buffer / spool
- exporter

主要覆盖：

- M6
- M7
- M14
- M16

### 7.6 中心控制平面线

负责：

- request / approval / compile / sign / dispatch
- tracker / result / audit
- gateway 接入层扩展

主要覆盖：

- M8
- M11
- M12
- M13

### 7.7 验证与工具链线

负责：

- schema validator
- fixture
- replay
- integration tests
- e2e certification
- authoring lint / compile

主要覆盖：

- M1
- M5
- M7
- M14
- M15
- M16

---

## 8. 第一阶段建议范围

如果目标是尽快拿到可信的第一个可运行版本，建议第一阶段只承诺：

- M0
- M1
- M2
- M3
- M4
- M5

这样得到的是：

- 稳定的核心契约
- 稳定的 agent 身份和接入基线
- 可接入中心的 `wp-agentd`
- 可执行首批只读 action 的边缘闭环

---

## 9. 第二阶段建议范围

第二阶段建议承诺：

- M6
- M7
- M8
- M9
- M10
- M11

这样得到的是：

- 资源发现与 Batch A telemetry 底盘
- 最小可用的控制中心
- 安装与升级的工程闭环
- 基线安全与审计能力

---

## 10. 第三阶段建议范围

第三阶段建议承诺：

- M12
- M13
- M14
- M15
- M16

这样得到的是：

- 多级树拓扑和大规模接入能力
- 统一多信号底盘
- AI 辅助与作者工具链
- 接近正式 GA 的世界级非功能水准

---

## 11. 当前最值得立即启动的工作

当前最值得立刻启动的是：

1. 完成 M1 中尚未代码化的 schema 与校验器
2. 启动 M3 的 `standalone` `wp-agentd` / `wp-agent-exec` / `wp-agent-upgrader` skeleton
3. 启动 M6 与 M7 的本地 discovery / telemetry 底盘
4. 定义统一结构化 telemetry record，并先对接 `warp-parse`
5. 启动 M2 的 enrollment / identity 设计与实现
6. 启动 M4 的 gateway session MVP
7. 启动 M5 的首批只读 opcode 与执行闭环

如果资源还能再加一条并行主线，再启动：

8. M8 控制中心 MVP

也就是说，当前阶段最重要的不是继续抽象讨论，而是把：

- 契约
- standalone 边缘运行基线
- 本地 discovery / telemetry 底盘
- 统一结构化 record -> `warp-parse`
- 身份接入
- managed 接入
- 最小远程执行闭环

先代码化。

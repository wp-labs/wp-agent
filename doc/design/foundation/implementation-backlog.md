# wp-agent 首批实现 Backlog

## 1. 文档目的

本文档把 [`roadmap.md`](roadmap.md) 中已经拆分好的里程碑，进一步落成可执行的首批实现 backlog。

这里的目标不是定义所有长期工作，而是回答三件事：

- 现在可以立刻开哪些任务
- 这些任务建议落到哪些模块 / 文件
- 每项任务完成后，以什么结果算“做完”

当前仓库还没有代码目录，因此本文中的模块和文件布局属于建议实现骨架。

相关文档：

- [`roadmap.md`](roadmap.md)
- [`glossary.md`](glossary.md)
- [`../edge/agentd-architecture.md`](../edge/agentd-architecture.md)
- [`../edge/agentd-state-schema.md`](../edge/agentd-state-schema.md)
- [`../edge/log-file-state-schema.md`](../edge/log-file-state-schema.md)
- [`../execution/action-plan-schema.md`](../execution/action-plan-schema.md)
- [`../center/agent-gateway-protocol.md`](../center/agent-gateway-protocol.md)

---

## 2. 当前判断

基于当前设计文档，已经足够启动首批开发。

建议启动顺序：

1. `M1` 契约与校验器
2. `M3` `standalone` 三进程骨架
3. `M2` 身份与 enrollment 基线
4. `M4` gateway session
5. `M5` controlled action MVP
6. `M6/M7/M8` discovery + telemetry core + Batch A metrics
7. `M9/M10` control center core + dispatch 闭环

还不建议优先启动：

- `M15+` 的 scale-out gateway / tree topology
- `M18` 的 Batch B/C integrations
- `M19` 的 AI / authoring

---

## 3. 建议代码布局

当前仓库没有代码，可以按下面的最小布局起步：

```text
wp-agent/
  doc/
  crates/
    wp-agent-contracts/
    wp-agent-validate/
    wp-agent-shared/
    wp-agentd/
    wp-agent-exec/
    wp-agent-upgrader/
    wp-agent-gateway/
    wp-agent-control/
  fixtures/
    contracts/
    action-plans/
    action-results/
    telemetry/
  tests/
    e2e/
```

模块角色：

- `wp-agent-contracts/`
  所有 schema 对象、枚举、serde 类型、版本字段
- `wp-agent-validate/`
  独立校验器、约束检查器、负例测试
- `wp-agent-shared/`
  错误码、通用 paths、ids、时间工具、配置加载
- `wp-agentd/`
  边缘常驻 daemon、state store、scheduler、telemetry runtime
- `wp-agent-exec/`
  `ActionPlan` runtime、opcode dispatch、`ActionResult`
- `wp-agent-upgrader/`
  prepare / switch / health-check / rollback
- `wp-agent-gateway/`
  南向 session、hello、heartbeat、dispatch、ack/result 通道
- `wp-agent-control/`
  request / approval / compile / sign / dispatch / tracker

---

## 4. P0 Backlog

### 4.1 B001 合同类型定义

对应里程碑：

- `M1`

建议模块：

- `crates/wp-agent-contracts/src/action_plan.rs`
- `crates/wp-agent-contracts/src/action_result.rs`
- `crates/wp-agent-contracts/src/capability_report.rs`
- `crates/wp-agent-contracts/src/gateway.rs`
- `crates/wp-agent-contracts/src/agent_config.rs`
- `crates/wp-agent-contracts/src/state_exec.rs`
- `crates/wp-agent-contracts/src/state_logs.rs`

完成定义：

- 所有核心对象有可序列化类型
- `schema_version` / `api_version` / `kind` 固定值可表达
- 字段命名与设计文档一致

### 4.2 B002 合同校验器

对应里程碑：

- `M1`

建议模块：

- `crates/wp-agent-validate/src/action_plan.rs`
- `crates/wp-agent-validate/src/action_result.rs`
- `crates/wp-agent-validate/src/config.rs`
- `crates/wp-agent-validate/src/state.rs`

完成定义：

- 非法 `ActionPlan` 能被静态拒绝
- 非法目标、非法约束、过期时间、graph 结构错误都有明确错误码
- 校验器可被 CLI、`agentd`、control center 复用

### 4.3 B003 正反样例集

对应里程碑：

- `M1`

建议目录：

- `fixtures/contracts/action-plan/valid/`
- `fixtures/contracts/action-plan/invalid/`
- `fixtures/contracts/action-result/valid/`
- `fixtures/contracts/config/`

完成定义：

- 每类核心对象至少有一组 valid/invalid fixtures
- CI 可以批量回放校验

### 4.4 B004 `wp-agentd` skeleton

对应里程碑：

- `M3`

建议模块：

- `crates/wp-agentd/src/main.rs`
- `crates/wp-agentd/src/bootstrap.rs`
- `crates/wp-agentd/src/config_runtime.rs`
- `crates/wp-agentd/src/state_store/mod.rs`
- `crates/wp-agentd/src/self_observability.rs`

完成定义：

- `wp-agentd` 可独立启动
- 能初始化 `run/`、`state/`、`log/` 目录
- 能加载配置并进入 `standalone` 常驻主循环

### 4.5 B005 `wp-agent-exec` skeleton

对应里程碑：

- `M3`

建议模块：

- `crates/wp-agent-exec/src/main.rs`
- `crates/wp-agent-exec/src/runtime.rs`
- `crates/wp-agent-exec/src/workdir.rs`
- `crates/wp-agent-exec/src/result_writer.rs`

完成定义：

- 能读取 `plan.json` / `runtime.json`
- 能写出 `state.json` / `result.json`
- 暂无真实 opcode 也可完成最小空执行

### 4.6 B006 `wp-agent-upgrader` skeleton

对应里程碑：

- `M3`

建议模块：

- `crates/wp-agent-upgrader/src/main.rs`
- `crates/wp-agent-upgrader/src/prepare.rs`
- `crates/wp-agent-upgrader/src/switch.rs`
- `crates/wp-agent-upgrader/src/rollback.rs`

完成定义：

- 二进制可启动
- 预留 prepare / switch / rollback 命令面
- 与 `wp-agentd` 的工作目录和状态边界清晰

### 4.7 B007 execution state store

对应里程碑：

- `M1`
- `M3`

建议模块：

- `crates/wp-agentd/src/state_store/agent_runtime.rs`
- `crates/wp-agentd/src/state_store/execution_queue.rs`
- `crates/wp-agentd/src/state_store/running.rs`
- `crates/wp-agentd/src/state_store/reporting.rs`
- `crates/wp-agentd/src/state_store/history.rs`

完成定义：

- 对应 schema 的状态文件能原子落盘
- 唯一写入权边界在代码里明确

### 4.8 B008 logs checkpoint store

对应里程碑：

- `M1`
- `M3`
- `M7`

建议模块：

- `crates/wp-agentd/src/state_store/log_checkpoints.rs`

完成定义：

- 能读写 `state/logs/file_inputs/<input_id>/checkpoints.json`
- `checkpoint_offset` 与 `commit point` 语义被清晰编码

---

## 5. P1 Backlog

### 5.1 B101 identity runtime

对应里程碑：

- `M2`

建议模块：

- `crates/wp-agent-shared/src/identity.rs`
- `crates/wp-agentd/src/identity/mod.rs`

完成定义：

- `agent_id / instance_id / boot_id` 生成与持久化成立
- 首次启动与重启语义区分清楚

### 5.2 B102 enrollment client

对应里程碑：

- `M2`
- `M4`

建议模块：

- `crates/wp-agentd/src/identity/enroll.rs`
- `crates/wp-agent-gateway/src/enroll_api.rs`

完成定义：

- 新节点可完成首次 enrollment
- 重复实例可被识别

### 5.3 B103 gateway session client/server

对应里程碑：

- `M4`

建议模块：

- `crates/wp-agentd/src/gateway_client.rs`
- `crates/wp-agent-gateway/src/session.rs`
- `crates/wp-agent-gateway/src/lease.rs`

完成定义：

- `hello`、heartbeat、reconnect 跑通
- capability 上报可见

### 5.4 B104 capability report implementation

对应里程碑：

- `M4`

建议模块：

- `crates/wp-agentd/src/capability_report.rs`

完成定义：

- 边缘可上报 `exec` / `metrics` / `logs` / `upgrade`

### 5.5 B105 controlled action scheduler

对应里程碑：

- `M5`

建议模块：

- `crates/wp-agentd/src/control_receiver.rs`
- `crates/wp-agentd/src/plan_validator.rs`
- `crates/wp-agentd/src/execution_scheduler.rs`
- `crates/wp-agentd/src/executor_manager.rs`
- `crates/wp-agentd/src/result_aggregator.rs`

完成定义：

- `DispatchActionPlan -> queue -> exec -> ack/result` 跑通
- 成功、失败、取消、超时路径齐全

### 5.6 B106 opcode runtime v1

对应里程碑：

- `M5`

建议模块：

- `crates/wp-agent-exec/src/opcodes/process.rs`
- `crates/wp-agent-exec/src/opcodes/socket.rs`
- `crates/wp-agent-exec/src/opcodes/service.rs`
- `crates/wp-agent-exec/src/opcodes/file.rs`
- `crates/wp-agent-exec/src/opcodes/config.rs`
- `crates/wp-agent-exec/src/opcodes/agent.rs`

完成定义：

- 首批只读 opcode 都有实现和测试

### 5.7 B107 resource discovery foundation

对应里程碑：

- `M6`

建议模块：

- `crates/wp-agentd/src/discovery/host.rs`
- `crates/wp-agentd/src/discovery/process.rs`
- `crates/wp-agentd/src/discovery/container.rs`
- `crates/wp-agentd/src/discovery/k8s.rs`
- `crates/wp-agentd/src/discovery/cache.rs`

完成定义：

- 可稳定产出 host/process/container 基础资源视图
- resource identity 去重成立

### 5.8 B108 telemetry record envelope

对应里程碑：

- `M7`

建议模块：

- `crates/wp-agent-contracts/src/telemetry_record.rs`
- `crates/wp-agentd/src/telemetry/envelope.rs`
- `crates/wp-agentd/src/telemetry/normalize.rs`

完成定义：

- `logs / metrics / traces / security` 能编码到统一 record 骨架

### 5.9 B109 telemetry runtime core

对应里程碑：

- `M7`

建议模块：

- `crates/wp-agentd/src/telemetry/input_router.rs`
- `crates/wp-agentd/src/telemetry/buffer.rs`
- `crates/wp-agentd/src/telemetry/spool.rs`
- `crates/wp-agentd/src/telemetry/exporter.rs`
- `crates/wp-agentd/src/telemetry/warp_parse.rs`

完成定义：

- record 可进入本地 buffer/spool 并上送 `warp-parse`
- backpressure 不会拖垮控制面

### 5.10 B110 file input runtime

对应里程碑：

- `M7`
- `M17`

建议模块：

- `crates/wp-agentd/src/telemetry/logs/file_input.rs`
- `crates/wp-agentd/src/telemetry/logs/file_watcher.rs`
- `crates/wp-agentd/src/telemetry/logs/file_reader.rs`
- `crates/wp-agentd/src/telemetry/logs/multiline.rs`
- `crates/wp-agentd/src/telemetry/logs/parser.rs`

完成定义：

- `file input` 可按 `read offset` 持续读取
- `commit point` 与 `checkpoint_offset` 推进成立
- rotate / truncate / multiline 基线成立

### 5.11 B111 Batch A metrics integrations

对应里程碑：

- `M8`

建议模块：

- `crates/wp-agentd/src/telemetry/metrics/host_metrics.rs`
- `crates/wp-agentd/src/telemetry/metrics/process_metrics.rs`
- `crates/wp-agentd/src/telemetry/metrics/container_metrics.rs`
- `crates/wp-agentd/src/telemetry/metrics/k8s_node_pod_metrics.rs`
- `crates/wp-agentd/src/telemetry/metrics/prom_scrape.rs`
- `crates/wp-agentd/src/telemetry/metrics/otlp_metrics_receiver.rs`

完成定义：

- Batch A 指标可稳定采集并绑定 resource

### 5.12 B112 control center core

对应里程碑：

- `M9`

建议模块：

- `crates/wp-agent-control/src/api.rs`
- `crates/wp-agent-control/src/registry.rs`
- `crates/wp-agent-control/src/request_store.rs`
- `crates/wp-agent-control/src/query.rs`
- `crates/wp-agent-control/src/result_ingest.rs`

完成定义：

- request / tracker / result query 成立

### 5.13 B113 approval / signing / dispatch

对应里程碑：

- `M10`

建议模块：

- `crates/wp-agent-control/src/approval.rs`
- `crates/wp-agent-control/src/compiler.rs`
- `crates/wp-agent-control/src/signer.rs`
- `crates/wp-agent-control/src/dispatch.rs`

完成定义：

- approval -> compile -> sign -> dispatch -> ack/result 闭环成立

### 5.14 B114 install bootstrap

对应里程碑：

- `M11`

建议目录：

- `packaging/`
- `scripts/install/`

完成定义：

- 新节点可安装并拉起 `wp-agentd`

### 5.15 B115 upgrade / rollback

对应里程碑：

- `M12`

建议模块：

- `crates/wp-agent-upgrader/src/download.rs`
- `crates/wp-agent-upgrader/src/verify.rs`
- `crates/wp-agent-upgrader/src/health_check.rs`

完成定义：

- 单节点升级回滚成立

### 5.16 B116 security baseline

对应里程碑：

- `M13`

建议模块：

- `crates/wp-agent-validate/src/attestation.rs`
- `crates/wp-agent-exec/src/allow.rs`
- `crates/wp-agent-shared/src/error_codes.rs`

完成定义：

- 签名校验、allow 控制、最小权限约束成立

### 5.17 B117 audit hardening

对应里程碑：

- `M14`

建议模块：

- `crates/wp-agent-control/src/audit.rs`
- `crates/wp-agentd/src/audit_logger.rs`

完成定义：

- `request_id -> action_id -> dispatch_id -> execution_id` 全链路审计成立

---

## 6. 首批交付包建议

如果按最少风险切成三个交付包，建议：

### 6.1 Package A

- B001-B008
- B101-B104

结果：

- 可启动 `standalone` agent
- 可完成 enrollment 和 gateway session

### 6.2 Package B

- B105-B113

结果：

- 可执行只读 action
- telemetry core + Batch A metrics 成立
- control center 基础闭环成立

### 6.3 Package C

- B114-B117

结果：

- 安装、升级、安全、审计形成上线前基线

---

## 7. 当前最适合立刻创建的 issue

建议立刻建 12 个 issue：

1. contracts: 定义 `ActionPlan` / `ActionResult` / gateway envelopes
2. validate: 建立 schema validator 与 fixtures
3. agentd: 建立 daemon skeleton 与 state root
4. agent-exec: 建立 workdir 协议与空 runtime
5. upgrader: 建立 skeleton
6. identity: `agent_id / instance_id / boot_id` 与 enrollment
7. gateway: hello / heartbeat / reconnect
8. action: scheduler + first read-only opcodes
9. discovery: host/process/container foundation
10. telemetry: record envelope + buffer/spool + `warp-parse` uplink
11. metrics: Batch A integrations
12. control center: request/query/dispatch core

---

## 8. 当前决定

当前阶段固定以下结论：

- 可以启动开发，不需要等待更多抽象设计
- backlog 应按模块与交付包组织，而不是只按大里程碑名组织
- `M1-M8` 与 `M9-M14` 是当前最值得投入的主线

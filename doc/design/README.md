# wp-agent 设计文档索引

`doc/design` 目录按主题拆分为以下子目录：

- `foundation/`
  总体目标、总体架构、术语、路线图、非功能目标、安全模型、参考资料
- `execution/`
  动作 DSL、`ActionPlan IR`、执行 schema、opcode schema、`run.gxl` 相关文档
- `edge/`
  `wp-agentd` / `wp-agent-exec` / 本地状态 / logs file state / 配置 / capability / 自观测
- `center/`
  控制中心、控制平面、Gateway 协议、计划投递与结果回报 schema
- `telemetry/`
  metrics 集成、logs file input、discovery、resource mapping、uplink 设计与 Batch A 规格

建议阅读顺序：

1. [foundation/target.md](./foundation/target.md)
2. [foundation/architecture.md](./foundation/architecture.md)
3. [foundation/security-model.md](./foundation/security-model.md)
4. [execution/action-plan-ir.md](./execution/action-plan-ir.md)
5. [edge/agentd-architecture.md](./edge/agentd-architecture.md)
6. [center/control-center-architecture.md](./center/control-center-architecture.md)
7. [telemetry/metrics-integration-roadmap.md](./telemetry/metrics-integration-roadmap.md)
8. [telemetry/telemetry-uplink-and-warp-parse.md](./telemetry/telemetry-uplink-and-warp-parse.md)
9. [telemetry/log-file-input-spec.md](./telemetry/log-file-input-spec.md)
10. [edge/log-file-state-schema.md](./edge/log-file-state-schema.md)

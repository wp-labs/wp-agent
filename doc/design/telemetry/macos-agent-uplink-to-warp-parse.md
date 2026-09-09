# macOS P0 日志采集与数据面上送方案（warp-agentd → WarpGateway data-plane）

## 1. 文档目的

定义在 **macOS** 主机上让 `warp-agentd` 采集高价值（P0）安全/审计数据，并通过 **TCP** 上送到
**WarpGateway 数据平面（warp-parse）** 的端到端方案。

回答：

- P0 数据从哪些来源、用什么采集方式拿到（文件 tail / 统一日志 / OpenBSM / TCC 快照）；
- agent 侧如何结构化、缓冲、发送（复用哪些现有能力、需要新增哪些）；
- Gateway 数据平面如何接入（已就绪的 tcp/syslog 入站 + WPL 规则）；
- 分阶段落地路径、权限与合规约束、安全与可靠性设计。

相关文档：

- [macos-security-audit-log-sources.md](macos-security-audit-log-sources.md)（P0 源清单与位置）
- [telemetry-uplink-and-warp-parse.md](telemetry-uplink-and-warp-parse.md)（数据面角色边界）
- [log-file-input-spec.md](log-file-input-spec.md)（文件输入 tail/watcher/checkpoint 设计）
- [../edge/agent-config-schema.md](../edge/agent-config-schema.md)（agentd 配置 schema）
- [../edge/agentd-architecture.md](../edge/agentd-architecture.md)
- [../edge/log-file-state-schema.md](../edge/log-file-state-schema.md)
- [../foundation/security-model.md](../foundation/security-model.md)
- [../foundation/implementation-backlog.md](../foundation/implementation-backlog.md)

---

## 2. 结论

- `warp-agentd` 到 `warp-parse` 采用 **统一结构化 record（NDJSON）→ TCP(line framing)** 上送；
- 文件型 P0 子集**现有能力即可闭环**（几乎零代码，属配置 + WPL）；
- 统一日志 / OpenBSM / TCC 属于**采集器能力缺口**，需要 agentd 新增 source 与 root helper；
- 数据面只承担接入/解析/路由；控制面（注册、租约、任务、升级）仍走控制中心；
- “数据可以进 warp-parse，控制不能进 warp-parse”的原则不因本方案改变。

---

## 3. 现状盘点

### 3.1 warp-agentd 已有能力（可直接用）

- `[telemetry.logs.file_inputs]`：文件 tail，条目含
  `input_id / path / startup_position(head|tail) / multiline_mode`；
  带本地缓冲（`state/spool/logs`）与 checkpoint（见 log-file-input-spec）；
- `[telemetry.logs.output] kind = "file" | "tcp"`；
  - `tcp`：`addr / port / framing="line"`（行帧 NDJSON 发送已现成）；
  - `file`：`log/warp-parse-records.ndjson`（本地落盘/回放）。

### 3.2 Gateway 数据平面已就绪

- `wparse` 常驻（`crates/warp-gateway/bin/wparse` 0.25.21，工程在
  `crates/warp-gateway/data-plane`）；
- `tcp_1`：0.0.0.0:9000，`tcp_src`，`data_format=ndjson`、`framing=auto`（已实测可收）；
- `syslog_1`：1514（syslog 协议，可选）；
- WPL 素材：`models/wpl/mac/<类>/sample.dat`（真机样本）与逐步补齐的 `parse.wpl`/OML。

### 3.3 能力缺口（决定工作量）

| 采集面 | 说明 | 需要的增量 |
|---|---|---|
| 文件型 P0 | install.log / launchd / wifi / fsck / shutdown_monitor（追加型文本） | 基本为**配置** |
| `.ips` 崩溃 | 每次崩溃生成**新文件**（非 append），tail 语义不适用 | “目录新文件发现”（file watcher 扩展） |
| 统一日志 | 登录/防火墙/TCC/Gatekeeper 事件在 Unified Logging | 新 source：`log stream/show` 订阅器 |
| OpenBSM audit | exec/登录权威链在 `/var/audit`（二进制） | 新 source：`auditreduce` 导出器（root） |
| TCC / Gatekeeper | 授权 DB 与事件 | 周期快照 + 事件订阅 |

---

## 4. 目标架构

```text
 macOS 主机 (warp-agentd)
 ┌─────────────────────────────────────────────────────┐
 │ 文件 tail 源            统一日志源             audit 源 │
 │ install.log/launchd/    log stream 订阅       auditreduce│
 │ wifi/fsck/shutdown        (root+FDA)          (root)  │
 │      │                       │                    │    │
 │      └──────────┬────────────┴────────────────────┘    │
 │                 ▼                                      │
 │      normalize → 统一结构化 record（NDJSON）             │
 │                 ▼                                      │
 │      spool 本地缓冲（state/spool/logs，断连排队）        │
 │                 ▼                                      │
 │      output.kind=tcp / framing=line / addr=<data-plane>│
 └──────────────────────────┬────────────────────────────┘
                            │ TCP 9000
 ┌──────────────────────────▼────────────────────────────┐
 │ WarpGateway data-plane (wparse)                        │
 │  tcp_1 (ndjson)  [可选 syslog_1 1514]                  │
 │  → WPL mac/* 逐类解析 → 业务/security/miss/rescue sink │
 └────────────────────────────────────────────────────────┘
```

- 数据面只做：接入、解析、转换、路由。
- agent 只做：采集、结构化、缓冲、上送。
- 无论来源类型，统一先转结构化 record，便于数据面一套入口 + 按 `category` 路由。

### 4.1 上送帧格式（每行一条）

**JSON 信封 + 行尾 `RAW:<RAW>` 原文**，`raw` 不作为 JSON 字段、不做 JSON 转义：

```text
{JSON 信封} RAW:<原始单行>
```

示例（launchd 真实行）：

```text
{"agent_id":"node-uuid","instance_name":"mbp-ops","tenant_id":"tenant-default","env_id":"env-default","ts":"2026-09-09T08:56:22.956Z","category":"macos.launchd","host_ts":"2026-09-09 08:56:24.392429","event":{"level":"Warning","scope":"system","message":"failed lookup: name = com.apple.AppleLOM.Watchdog"}} RAW: 2026-09-09 08:56:24.392429 (system) <Warning>: failed lookup: name = com.apple.AppleLOM.Watchdog, flags = 0x1, requestor = watchdogd[551], error = 3: No such process
```

约定：

- JSON 信封只承载结构化字段（`agent_id/category/ts/host_ts/event...`），**不含 raw**；
- `RAW:` 是行尾固定前缀，之后到行尾为原始内容；原始内容**保持原样，不转义**，避免体积膨胀并便于审计核对；
- 因此上送行要求原始内容为**单行**；多行内容（如 `.ips`/panic 文本）由 agent 侧先归一为单行（换行转可视转义或取首行元数据）再上送，或该类走独立入口；
- 数据面处理：先 `json(...)` 解析信封字段路由；`RAW:` 原文按需由规则抓取，miss/rescue 保留原文供回放核对。

---

## 5. P0 采集映射

| P0 类 | 采集方式 | 权限 | 实现归属 |
|---|---|---|---|
| 软件安装/更新（install.log） | 文件 tail（head 起） | root/读许可 | 配置 |
| launchd 服务事件 | 文件 tail（launchd.log） | 读许可 | 配置 |
| 关机/重启、fsck | tail shutdown_monitor / fsck_apfs | root | 配置 |
| WiFi | tail wifi.log* | root | 配置（噪音过滤靠 WPL） |
| 崩溃/panic `.ips` | 目录新文件发现 | Full Disk Access | watcher 扩展 |
| 统一日志登录/防火墙/TCC/GK | log stream（或周期 log show） | root + `log config private_data:on` | 新 source：`unified_log` |
| OpenBSM exec/登录权威链 | auditreduce 周期导出 → 文本 | root（auditd flags 含 `ex`/argv） | 新 source：`audit_export` |
| TCC/隐私 | sqlite3 TCC.db 快照 + tccd 事件 | Full Disk Access | 新 source：`tcc_snapshot`（低频） |

> 权限红线：audit、TCC.db、`.ips`、统一日志脱敏解除均需要 **root / Full Disk Access**。
> agentd 需要独立的 **root helper（launchd daemon / SMJobBless 形态）** 来承担需要提权的采集，
> 普通用户态 daemon 采不完整。此项进 security-model 评审。

---

## 6. 数据面接入（Gateway 侧）

- 已就绪：`tcp_1`（9000，ndjson）与 `syslog_1`（1514）。
- 规则开发顺序：以 `models/wpl/mac/<类>/sample.dat`（真机实采）为准，逐类写 `parse.wpl` + OML；
  安全语义类路由到独立 sink 分组（参考 telemetry-uplink 的 security receiver 结论）。
- **未决风险：入站认证**。当前 wparse tcp/syslog 监听 0.0.0.0 且无认证：
  - V1 部署约束：数据面只监听可信网段/专用链路，Agent 指向该地址；
  - V2：为 `tcp_src` 增加 token/TLS 握手，或 agent 侧加密隧道后再进数据面。

---

## 7. 可靠性、安全与合规设计

| 主题 | 设计 |
|---|---|
| 缓冲/重试 | 复用 spool：断连本地排队、重连回放；TCP 指数退避；JSON 信封带 `seq` 序号，去重基于信封字段（不依赖 raw） |
| 隐私/合规 | 统一日志完整字段需 `private_data:on`——采集决策点；默认脱敏可用，仅对授权主机开完整；`RAW:` 原文按类裁剪 |
| 完整性 | audit 事件尽快离机；本地副本仅短期缓冲；数据面记录级去重兜底 |
| 时钟 | record 同时携带 `host_ts`（源时间）与 `ts`（采集/上送时间），WPL/OML 统一时区处理 |
| 权限治理 | root helper 提权面最小化（仅 audit/TCC/private log 采集），其余保持用户态 |
| 认证 | V1 内网约束；V2 agent↔wparse TCP 认证/TLS |

---

## 8. 落地阶段

### Phase 1 —— 最小闭环（低代码，先跑通链路）

1. agentd 配置：`file_inputs` 指向 `install.log`、`launchd.log`、`shutdown_monitor.log`、`fsck_apfs.log`；
2. `output.kind="tcp"`，`addr=<data-plane>`、`port=9000`、`framing="line"`；
3. data-plane 已收口（9000 实测可收，miss 落盘）；
4. 上述类写 `parse.wpl`（对照真机 sample）并 `wpl-check sample` 回放验证；
5. 验收：一次安装 / 一次 launchd 事件 → 本地 ndjson → TCP → wparse → 规则命中落业务/security sink。

### Phase 2 —— 覆盖剩余 P0

1. `.ips` 目录源（file watcher 扩展，崩溃新文件发现）；
2. 新 source `unified_log`（log stream 订阅器）；
3. 新 source `audit_export`（auditreduce 周期导出）；root helper 落地；
4. TCC/Gatekeeper 事件订阅与周期快照。

### Phase 3 —— 加固

1. agent↔wparse TCP 认证/TLS；
2. WPL security 语义路由 + 告警；
3. 容量与 SLA（rps、retention、miss/rescue 监控）。

---

## 9. 代码改动点清单

| 模块 | 改动 |
|---|---|
| `crates/warp-agentd` telemetry | ① file_inputs 支持目录新文件（.ips）；② 新增 source 类型 `unified_log / audit_export / tcc_snapshot`；③ root helper |
| agent 配置 schema | `telemetry.logs.sources` 扩展（unified_log predicate 集、audit flags/周期、快照源） |
| data-plane | 已就绪；后续补入站认证 |
| `models/wpl/mac/*` | 逐类补 `parse.wpl` + OML，真机样本校准 |

---

## 10. 未决问题

1. 统一日志/audit 是否要完整字段（开 `private_data:on`、引入 root helper），还是接受脱敏版？
2. Agent→数据面认证优先级（内网约束即可 vs TLS/token）。
3. `.ips` 采集形态：全量新文件直读 vs 仅元数据（首行）——影响 WPL 规则复杂度。
4. Phase 1 的 E2E 验收是否要现在就做（本机可闭环）。

---

## 11. 相关文档

- [macos-security-audit-log-sources.md](macos-security-audit-log-sources.md)
- [telemetry-uplink-and-warp-parse.md](telemetry-uplink-and-warp-parse.md)
- [log-file-input-spec.md](log-file-input-spec.md)
- [../edge/agent-config-schema.md](../edge/agent-config-schema.md)
- [../foundation/implementation-backlog.md](../foundation/implementation-backlog.md)

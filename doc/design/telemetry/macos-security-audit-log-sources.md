# macOS 主机安全与审计日志源清单（设计参考）

## 1. 文档目的

本文档整理在 **macOS** 操作系统上值得采集、可用于**安全与审计**的日志与事件位置，作为：

- `warp-agentd` telemetry `file_inputs` / 数据面接入（`warp-parse`）的**源清单**；
- 后续为 macOS 增加“统一日志 / OpenBSM audit”采集能力的**需求与设计输入**；
- SOC / 安全运维侧确定采集优先级与保留策略的参考。

它不是：

- 一套完整的 macOS 取证手册；
- 对某特定 macOS 小版本的穷举路径索引（路径与行为会随版本变化，见 §10 的本机核对）。

相关文档：

- [`telemetry-uplink-and-warp-parse.md`](telemetry-uplink-and-warp-parse.md)
- [`log-file-input-spec.md`](log-file-input-spec.md)
- [`../edge/log-file-state-schema.md`](../edge/log-file-state-schema.md)
- [`../edge/capability-report-schema.md`](../edge/capability-report-schema.md)
- [`../foundation/security-model.md`](../foundation/security-model.md)

---

## 2. 范围与前提

- 适用：macOS 11+；文档中的“本机核对”以 **macOS 26（本机实测）** 为准，其余按 macOS 通用事实描述。
- macOS 的日志体系与 Linux 差异很大，核心事实：

  1. **自 macOS 10.12 起默认日志入口是 Unified Logging（统一日志）**，历史 `/var/log/system.log` 仍然存在并持续写入部分事件，但**不是完整源**；
  2. **OpenBSM audit（`/var/audit`）** 是安全审计的“正统”事件源（登录、认证、可选命令执行），但默认仅开启登录/认证事件，需要 root 与调大审计策略才有完整价值；
  3. 大量高价值信息（安装、崩溃、防火墙、隐私授权）分散在 `/Library/Logs`、`/var/log` 与用户目录；
  4. **权限**：多数安全相关位置需要 `root` 或 **Full Disk Access (FDA)**，否则读取被 TCC 拦截或内容被脱敏；
  5. 部分内容默认只保留数日或自动清理（统一日志、`.ips` 崩溃、`.gz` 轮转），采集端要自行设计“近实时转发 + 周期归档”两条腿。

---

## 3. 采集面总览

| 采集面 | 载体 / 位置 | 价值 | 在 warp-insight 中的接入方式 |
|---|---|---|---|
| A. 文件型日志（file） | `/var/log/*`、`/Library/Logs/*`、`~/Library/Logs/*` | 安装、崩溃、WiFi、第三方 | `agentd telemetry.file_inputs` tail（现成能力，见 [log-file-input-spec.md](log-file-input-spec.md)） |
| B. 统一日志（Unified Logging） | `log` CLI 查询 `/var/db/diagnostics` 存储 | 登录/认证/防火墙/进程深度事件 | 需新增“统一日志采集器”（`log show` / `log stream`），当前是能力缺口 |
| C. OpenBSM audit | `/var/audit/*.log`（`praudit`/`auditreduce` 解码） | 登录/认证/可选命令执行的权威审计链 | 需新增“audit 导出器”（定期 `auditreduce` → 文本 → file input / 直发） |
| D. 隐私/安全数据库快照 | `TCC.db`、`~/Library/Preferences`、quarantine xattr 等 | 权限变更、下载来源（Gatekeeper） | 周期快照 + 差异上报（非实时日志） |

> 统一日志的“私有数据”默认会被打成 `{private}`；要拿到完整内容需 `sudo log config --mode "private_data:on"`。这不改变采集架构，但改变数据完整性与合规前提。

---

## 4. 推荐采集优先级总表

优先级含义：

- **P0**：安全/审计必采，第一时间要能看到；
- **P1**：高价值，建议开启（可能要 root 或扩审计策略）；
- **P2**：按场景选采。

| 优先级 | 类别 | 位置 / 查询 | 典型安全事件 |
|---|---|---|---|
| P0 | 登录/退出与会话 | OpenBSM `lo,la`；`/var/log/wtmp`（`last`）、`btmp` | 成功/失败登录、TTY 会话、sudo 会话 |
| P0 | 提权/关键命令 | OpenBSM `ex/pc` + `argv` 策略；`sudo` 事件 | root 执行、sudo、可疑命令行 |
| P0 | 软件安装与更新 | `/var/log/install.log`；统一日志 `softwareupdated` | 安装包、update、profile 变更 |
| P0 | 系统崩溃/内核 panic | `/Library/Logs/DiagnosticReports/*.panic*` | panic、异常重启（可能是被攻击征兆） |
| P0 | 防火墙（应用层） | 统一日志 `com.apple.alf` / 传统 `Application Firewall.log` | 入站拦截、进程首次联网放行 |
| P0 | 隐私授权（TCC） | 系统/用户 `TCC.db` + `tccd` 统一日志 | 摄像头/麦克风/磁盘/输入监控授权变更 |
| P1 | 进程执行 | OpenBSM `ex/pc`；统一日志进程事件 | 可疑二进制执行（配合规则） |
| P1 | 崩溃与异常进程 | `~/Library/Logs/DiagnosticReports`、`/Library/Logs/DiagnosticReports`（`.ips`） | 崩溃频率、异常进程画像 |
| P1 | 无线网络 | `/var/log/wifi.log*` | 关联攻击面/漫游异常 |
| P1 | launchd 服务 | `/var/log/com.apple.xpc.launchd/launchd.log*` | 服务反复崩溃、异常 launch |
| P1 | Gatekeeper/隔离 | `syspolicyd` 统一日志、`com.apple.quarantine` xattr | 运行被隔离/公证失败的应用 |
| P1 | 关机/重启 | `/var/log/shutdown_monitor.log`；统一日志 | 异常关机时间线 |
| P2 | 系统常规杂项 | `/var/log/fsck_apfs*`、`cups/`、`apache2/`、`mDNSResponder/` 等 | 磁盘/打印/服务异常 |
| P2 | 第三方 | `/Library/Logs/<vendor>`（如 MDM、VPN、EDR） | 依部署形态选采 |

---

## 5. 分项说明（路径 + 内容 + 采集方式）

### 5.1 登录、退出与会话

macOS 的登录链：图形登录（`loginwindow`）、SSH 远程登录（`sshd`）、控制台/`sudo`（OpenBSM 负责权威记录）。

- **OpenBSM（权威）**：`/var/audit/`
  - 事件：登录/退出（AUE_LOGIN / AUE_LOGOUT）、认证（AUE_AUTH_*）。
  - 读取（需 root）：`sudo praudit /var/audit/current`；按时间段：`sudo auditreduce -b <start> -e <end> | sudo praudit`。
  - 策略文件：`/etc/security/audit_control`。
    - 默认典型为 `flags:lo,la`（登录/退出 + 认证），**不足以覆盖命令审计**；
    - 如需执行审计建议 `flags:lo,la,ex,pc` 并确认 `policy:argv` 等（按部署安全策略调整）。
  - 注意：本机 `/var/audit` 默认可能为空（未产生事件或 auditd 未激活）；需 root 查看，审计文件按 `audit_control` 的 `filesz/expire` 自动轮转。
- **会话记录**：`/var/log/wtmp`（`last`）、`/var/log/btmp`（失败登录，按需创建）。
  - `last`、`last -t`、`lastb`（失败登录需 root）直接读取；
  - 采集端可直接 tail `/var/log/wtmp` 意义不大（二进制），建议定期跑 `last`/`lastb` 输出文本，或依赖 OpenBSM/统一日志。
- **统一日志**（补充上下文）：
  - 示例查询：
    ```
    log show --last 24h --predicate 'process == "loginwindow" OR process == "sshd"'
    ```
  - 注意 subsystem/进程名随版本变化，predicate 需要在本机验证（见 §10 方式）。

### 5.2 提权与命令执行

- **sudo**：sudo 事件进入统一日志与旧的 `system.log`；审计侧若开启 `ex` 事件则同时落在 `/var/audit`。
  - 示例：`log show --last 24h --predicate 'eventMessage CONTAINS[c] "sudo"'`（含提权到 root、用户切换）。
- **root / 关键命令执行**：以 OpenBSM `ex/pc` + argv 策略为主（见 5.1），文本日志只能做旁证。

### 5.3 系统与软件变更

- **安装/更新**：`/var/log/install.log`（文本，可 tail；含 `.pkg` 安装、`softwareupdated`、部分系统更新）。
- **launchd 服务**：`/var/log/com.apple.xpc.launchd/launchd.log*`（文本 + 数字轮转）。
- **配置描述文件 / MDM**：
  - `/Library/Managed Preferences/`（plist 状态）；`profiles` 命令可导出；
  - 变更事件主要走统一日志（`mdmclient` 等），无统一文本文件，建议按需用 `log` 采集或做配置快照差异。
- **关机/重启**：`/var/log/shutdown_monitor.log`；结合 OpenBSM 登录链与统一日志判断“异常重启还是 panic”。

### 5.4 进程与应用崩溃

- **系统级**：`/Library/Logs/DiagnosticReports/`
  - `.panic`、`.ips`（JSON 崩溃报告）、`.diag`、`Analytics-*.core_analytics`。
- **用户级**：`~/Library/Logs/DiagnosticReports/`（`.ips`）。
- 崩溃报告会自动轮转/清理（Retired 目录），**近实时转发**价值高于事后补采。

### 5.5 网络、防火墙与 WiFi

- **应用防火墙（ALF）**：现代 macOS 记入统一日志（`com.apple.alf` 相关），历史文件 `/var/log/appfirewall.log`（存在性随版本）；拦截/放行查询用 `log show` 以 “socketfilterfw / alf” 过滤。
- **PF 包过滤**：默认不落盘，需自行 `pflog` 采集，通常非首采目标。
- **WiFi**：`/var/log/wifi.log`（含 `.bz2` 轮转，实测存在）——信号、漫游、关联事件，WiFi 渗透/异常定位有用。
- **DNS/服务**：`/var/log/mDNSResponder/`（目录）；第三方网络客户端示例 `/var/log/netbird*`（VPN/网络审计可参考此模式采第三方日志）。

### 5.6 隐私授权（TCC）、Keychain 与 Gatekeeper

- **TCC 数据库（非实时日志，快照类）**：
  - 系统：`/Library/Application Support/com.apple.TCC/TCC.db`
  - 用户：`~/Library/Application Support/com.apple.TCC/TCC.db`
  - 变更事件可在统一日志按进程 `tccd` 过滤；建议**周期快照 + 差异上报**（相机/麦克风/屏幕录制/输入监控/全盘访问授权是典型内部风险信号）。
- **Keychain / securityd**：统一日志按 `securityd` / `com.apple.securityd` 过滤（解锁失败、项目访问异常）。
- **Gatekeeper / 隔离**：统一日志按 `syspolicyd` 过滤；下载来源看 xattr `com.apple.quarantine`（`xattr -p com.apple.quarantine <file>`），运行“被隔离/未公证”应用是 macOS 上的强信号。

### 5.7 其它 `/var/log` 杂项（P2）

`fsck_apfs.log*`、`cups/`、`apache2/`、`ppp/`、`powermanagement/`、`mDNSResponder/`、`DiagnosticMessages`、`asl/`（历史遗留，macOS 10.12 后基本不新增）等，按业务形态选采。

---

## 6. 统一日志（Unified Logging）采集建议

统一日志是 macOS 事件“最全”但“最需要技巧”的来源：

- **只读入口是 `log` 命令**（或 libsystem 的 OSLog API），不支持像 Linux 一样直接 tail 底层文件（`/var/db/diagnostics` 是私有格式）。
- 基本形态：
  ```
  log show    --last 24h --style compact --predicate '<查询>'
  log stream  --level info --predicate '<查询>'       # 实时
  log collect --last 1h --output /tmp/logarchive.gzip  # 归档包
  ```
- **私有数据**：默认脱敏为 `{private}`；完整内容需（root）：
  ```
  sudo log config --mode "private_data:on"
  ```
- **保留期**：默认只有最近数日（本地 `log` 存储），不能当长期审计库；长期方案 = 近实时转发到数据面 + 按需 `log collect` 归档。
- **落地建议**：不要全量 `log show`；按 §4 事件类别维护“predicate 集”，每个类别一个采集任务，输出结构化 record 进入数据面。subsystem/进程名需要**在目标 macOS 版本上验证**（见 §10）。

---

## 7. 权限、保留期与合规

| 关注点 | 说明 |
|---|---|
| root / FDA | `/var/audit`、统一日志脱敏解除、系统 `TCC.db`、部分 `/Library/Logs` 需要 root 或 Full Disk Access；agentd 需要单独说明授权（与安全模型一致） |
| 保留期 | 统一日志默认数日；`/var/log/*.gz`、`.ips`、audit 文件各自轮转；采集端必须“实时转发 + 长留归档”两段式 |
| 隐私合规 | 统一日志含大量私有/个人信息；采集前要过合规评审；按需仅采结构化安全事件，避免整库搬运 |
| 完整性 | audit 是 tamper-evident 的最佳载体；对本地 root 攻击者，日志应尽快转发离机（数据面落地），本机副本仅作短期缓冲 |

---

## 8. 接入 warp-insight 的落地路径（分级）

| 能力分级 | 采集对象 | 当前状态 | 做法 |
|---|---|---|---|
| A. 现成 | `/var/log/install.log`、`wifi.log`、launchd、崩溃 `.ips` 等**文本文件** | `agentd telemetry.file_inputs` 支持 tail/checkpoint/轮转（见 [log-file-input-spec.md](log-file-input-spec.md)） | 直接配置 glob + 规则解析 |
| B. 需“定时导出器” | OpenBSM audit、`last/lastb`、TCC 快照 | 二进制/DB，不能直接 tail | agentd 新增周期任务：`auditreduce`/`last`/`sqlite3` 导出文本 → 复用 file input 或直发 |
| C. 需“统一日志源” | Unified Logging（登录/防火墙/安全事件） | 需新 source 类型（`log stream` 订阅 + 结构化输出） | 按 predicate 集实现事件流 source；输出结构化 security record |
| D. 直发 | 已结构化的安全事件 | `agentd telemetry.output.kind = tcp`（NDJSON）→ data-plane `warp-parse` | 见 [telemetry-uplink-and-warp-parse.md](telemetry-uplink-and-warp-parse.md) |

推荐首版最小集（P0 且改动小）：

1. `file_inputs`：`/var/log/install.log`、`/var/log/shutdown_monitor.log`、`/var/log/com.apple.xpc.launchd/launchd.log`、`/var/log/wifi.log`、`/Library/Logs/DiagnosticReports/*.ips`；
2. 周期导出器：`last`/`lastb`、`auditreduce`（root）；
3. 数据面规则按 `security` 语义路由（`warp-parse` security receiver），与控制面互不耦合。

---

## 9. 相关文档

- [telemetry-uplink-and-warp-parse.md](telemetry-uplink-and-warp-parse.md)：数据面上报与 `warp-parse` 角色
- [log-file-input-spec.md](log-file-input-spec.md)：文件输入（tail/checkpoint/rotate）设计
- [../edge/log-file-state-schema.md](../edge/log-file-state-schema.md)：本地状态与 checkpoint
- [../edge/capability-report-schema.md](../edge/capability-report-schema.md)：agent 能力上报（file.tail 等）
- [../foundation/security-model.md](../foundation/security-model.md)：整体安全模型
- [../foundation/implementation-backlog.md](../foundation/implementation-backlog.md)：能力缺口登记（统一日志源 / audit 导出器）

---

## 10. 本机核对记录（macOS 26）

以下为本文档写作时在 macOS 26.6 实测确认的位置，供排期与测试脚本使用；部署前请在目标版本重跑核对：

```bash
# 目录/文件实测存在
ls /var/log/install.log /var/log/system.log /var/log/shutdown_monitor.log /var/log/wifi.log
ls /var/log/com.apple.xpc.launchd/launchd.log
ls /Library/Logs/DiagnosticReports ~/Library/Logs/DiagnosticReports
ls -d /Library/Application\ Support/com.apple.TCC ~/Library/Application\ Support/com.apple.TCC
ls -ld /var/audit          # root-only, 默认可能为空
# 工具存在性
command -v log auditreduce praudit last
```

- 本机 `/var/log` 另有 `fsck_apfs*.log`、`cups/`、`apache2/`、`mDNSResponder/`、`asl/`、`DiagnosticMessages`、`netbird*` 等（按 §5.7 选采）。
- `/var/audit` 为空属正常（未产生事件或 auditd 未激活）；启用前确认 `/etc/security/audit_control` 的 flags 是否覆盖目标事件。
- 统一日志 predicate 中的进程/subsystem 名（`loginwindow`、`sshd`、`securityd`、`tccd`、`syspolicyd`、`com.apple.alf` 等）在不同小版本可能不同，需按 §6 的“predicate 集”方式在目标版本上回归。

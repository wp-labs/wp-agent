# WarpInsight 通过 TCP 向 WarpParse 发送日志

本文说明如何把 `WarpInsight` 当前的 standalone 日志采集能力配置为通过 TCP 把日志发送到远端 `WarpParse`。

当前链路是：

`WarpInsight (wp-agentd) -> TCP -> WarpParse tcp_src`

## 1. 前提

需要先确认两件事：

1. `WarpParse` 侧已经启用 `tcp_src`。
2. `WarpInsight` 所在机器可以连通 `WarpParse` 的 TCP 监听地址和端口。

`WarpParse` 默认示例配置见：

- `warp-parse/docker/default_setting/connectors/source.d/04-tcp_src.toml`

默认值是：

```toml
[[connectors]]
id = "tcp_src"
type = "tcp"

[connectors.params]
addr = "0.0.0.0"
port = 9000
framing = "auto"
```

当前推荐分两种情况：

- 单行日志为主
  - `WarpInsight` 侧用 `framing = "line"`。
  - `WarpParse tcp_src` 保持 `framing = "auto"` 即可。
- 需要保留聚合后的多行事件
  - `WarpInsight` 侧改用 `framing = "len"`。
  - `WarpParse tcp_src` 可保持 `framing = "auto"`，也可显式设成 `len`。

原因是当前 `wp-agentd` 发送到 TCP 的内容是原始日志 `body`。如果 `body` 内部包含换行，而仍使用 `line` framing，接收端会按多行拆开。

## 2. 生成默认配置文件

在目标机器上执行：

```bash
wp-agentd init-config
```

默认会在当前目录创建：

```text
wp-agentd/agent.toml
```

如果只想先查看模板：

```bash
wp-agentd init-config --stdout
```

如果要放到指定目录：

```bash
wp-agentd init-config --config-dir /etc/warpinsight
```

## 3. 最小远程发送配置

把配置文件改成类似下面这样：

```toml
schema_version = "v1"

[telemetry.logs]
in_memory_buffer_bytes = 1048576
spool_dir = "state/spool/logs"

[telemetry.logs.output]
kind = "tcp"

[telemetry.logs.output.tcp]
addr = "10.0.0.25"
port = 9000
framing = "line"

[[telemetry.logs.file_inputs]]
input_id = "monitoring-app"
path = "/var/log/monitoring/app.log"
startup_position = "head"
multiline_mode = "none"
```

其中：

- `instance_name`
  - 可选。
  - 不写时会自动生成实例名。
  - 如果后续需要稳定区分主机来源，建议显式填写。
- `addr`
  - 远端 `WarpParse` 机器的 IP 或域名。
- `port`
  - 远端 `tcp_src` 监听端口，默认通常是 `9000`。
- `framing`
  - 单行日志建议用 `line`。
  - 如果要把聚合后的多行日志作为一条消息发送，必须改成 `len`。
- `input_id`
  - 本地输入源标识，需要在本机范围内唯一。
- `path`
  - 要采集的日志文件路径。
- `startup_position`
  - `head` 表示首次启动从文件开头读。
  - `tail` 表示首次启动从文件尾部开始，只采新增内容。
- `multiline_mode`
  - `none` 表示逐行采集。
  - `indented` 适合带缩进续行的堆栈日志。

## 4. Java / Go 堆栈日志示例

如果日志是这种形式：

```text
ERROR request failed
  at foo()
  at bar()
INFO next message
```

可以改成：

```toml
[telemetry.logs.output]
kind = "tcp"

[telemetry.logs.output.tcp]
addr = "10.0.0.25"
port = 9000
framing = "len"

[[telemetry.logs.file_inputs]]
input_id = "app-stack"
path = "/var/log/app/error.log"
startup_position = "tail"
multiline_mode = "indented"
```

这样缩进行会先被合并到上一条日志里，再作为一条 length-prefixed TCP 消息发送。

如果这里仍然使用 `framing = "line"`，那么聚合后的消息体里只要包含换行，`WarpParse` 侧仍会按多条来接收。

## 5. 默认目录说明

如果不显式写 `[paths]`，当前默认值是：

```toml
[paths]
root_dir = "."
run_dir = "run"
state_dir = "state"
log_dir = "log"
```

这表示相对路径都以配置目录为根来展开。

例如配置目录是：

```text
/opt/warpinsight/wp-agentd
```

那么默认会使用：

```text
/opt/warpinsight/wp-agentd/run
/opt/warpinsight/wp-agentd/state
/opt/warpinsight/wp-agentd/log
```

其中：

- `state/spool/logs`
  - 当远端 TCP 不可写时，日志会先落到这里。
- `state`
  - 也会保存日志读取 checkpoint。

## 6. 启动方式

使用默认配置目录：

```bash
wp-agentd
```

使用指定配置目录：

```bash
wp-agentd --config-dir /etc/warpinsight
```

只跑一轮，便于验证：

```bash
WP_AGENTD_RUN_ONCE=1 wp-agentd --config-dir /etc/warpinsight
```

## 7. 发送内容说明

当前通过 TCP 发给 `WarpParse` 的内容是：

- 原始日志 `body`

不是：

- `TelemetryRecordContract` JSON
- NDJSON telemetry envelope

这意味着 `WarpParse tcp_src` 收到的是原始日志文本，更适合作为后续规则解析输入。

还需要注意：

- `framing = "line"` 时，发送边界由换行决定，适合单行日志。
- `framing = "len"` 时，发送边界由长度前缀决定，可以保留消息体内部换行。

## 8. 故障与恢复

当前行为是：

1. `WarpParse` 不可达或 TCP 写失败时，新日志会先写入本地 `spool`。
2. 后续连接恢复后，`WarpInsight` 会先 replay 历史 `spool`，再继续发送新日志。
3. 即使原日志文件暂时不存在，只要该 input 仍有历史 `spool`，也会继续尝试补发。

## 9. 排障检查清单

如果远程发送没有成功，优先检查：

1. `WarpParse` 的 `tcp_src` 是否真的在监听目标端口。
2. 两台机器之间的防火墙、安全组、路由是否允许 TCP 连通。
3. `agent.toml` 里的 `addr`、`port` 是否写对。
4. `path` 指向的日志文件是否存在，进程是否有读取权限。
5. `state/spool/logs` 下是否出现 `.ndjson` 文件。
6. 如果启用了 `multiline_mode = "indented"`，是否同时把 TCP `framing` 配成了 `len`。

如果 `spool` 持续增长，通常表示：

- 远端不可达；
- 远端可达，但 `WarpParse` 没有正确接收；
- 本地持续发送失败。

## 10. 推荐的首轮验证步骤

建议按下面顺序做：

1. 在 `WarpParse` 机器确认 `9000/TCP` 已监听。
2. 在 `WarpInsight` 机器用一份最小配置只采集一个日志文件。
3. 先用 `WP_AGENTD_RUN_ONCE=1` 跑一次，确认无明显错误。
4. 再持续运行 `wp-agentd`。
5. 确认 `WarpParse` 已收到原始日志行。

## 11. 一个可直接改写的完整模板

```toml
schema_version = "v1"

[agent]
instance_name = "prod-host-01"

[telemetry.logs]
in_memory_buffer_bytes = 1048576
spool_dir = "state/spool/logs"

[telemetry.logs.output]
kind = "tcp"

[telemetry.logs.output.tcp]
addr = "192.168.10.20"
port = 9000
framing = "line"

[[telemetry.logs.file_inputs]]
input_id = "syslog"
path = "/var/log/system.log"
startup_position = "tail"
multiline_mode = "none"

[[telemetry.logs.file_inputs]]
input_id = "app-error"
path = "/var/log/myapp/error.log"
startup_position = "tail"
multiline_mode = "indented"
```

如果第二个输入 `app-error` 需要把聚合后的堆栈作为单条消息保留下来，请把上面的 `framing = "line"` 改成 `framing = "len"`。

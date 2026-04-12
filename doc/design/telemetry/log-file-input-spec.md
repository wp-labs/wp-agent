# wp-agent 文件日志输入设计

## 1. 文档目的

本文档定义 `wp-agentd` 对日志文件的常驻读取能力。

这里的“文件日志输入”特指：

- `wp-agentd` 作为数据面常驻组件持续监控和读取文本日志文件
- 将文件新增内容转换为统一 telemetry record
- 进入统一 `parse -> normalize -> resource binding -> buffer/spool -> export` 主线

本文档不讨论：

- 远程动作里的 `file.tail` / `file.read_range`
- `syslog` / `journald` / `winlog` 等非文件输入
- `warp-parse` 内部 receiver 的实现细节

相关文档：

- [`../foundation/architecture.md`](../foundation/architecture.md)
- [`../foundation/target.md`](../foundation/target.md)
- [`../foundation/non-functional-targets.md`](../foundation/non-functional-targets.md)
- [`telemetry-uplink-and-warp-parse.md`](telemetry-uplink-and-warp-parse.md)
- [`../edge/agent-config-schema.md`](../edge/agent-config-schema.md)
- [`../edge/capability-report-schema.md`](../edge/capability-report-schema.md)
- [`../edge/log-file-state-schema.md`](../edge/log-file-state-schema.md)

---

## 2. 核心结论

第一版固定以下结论：

- 文件日志读取是 `wp-agentd` 的一等数据面能力，不是远程动作替代方案
- 第一版必须明确对标 `Fluent Bit tail input`
- 对标对象是能力边界和工程行为，不是把 `wp-agent` 定义成 Fluent Bit 封装层
- 第一版必须覆盖：
  - 路径 glob 匹配与排除
  - 启动读头/读尾策略
  - 新发现文件的起始位置策略
  - `commit point`、本地 checkpoint 持久化与 crash 恢复
  - 文件 rotate / truncate 处理
  - `file watcher` 与轮询 fallback
  - 长行保护、多行拼装、buffer/backpressure
  - `source.path` / `source.offset` / 来源 metadata 注入
- 第一版不要求完全复刻 Fluent Bit 的全部历史兼容行为

一句话说：

- `wp-agentd` 需要做一个可对标 `Fluent Bit tail` 的文件日志输入器
- 但输出目标不是 Fluent Bit tag/chunk 模型，而是 `wp-agent` 的统一 record / resource / buffer 模型

---

## 3. 对标基线

### 3.1 参考对象

当前对标基线采用 Fluent Bit 官方 `Tail` 输入文档：

- `https://docs.fluentbit.io/manual/pipeline/inputs/tail`

按当前官方文档，Fluent Bit `tail` 已覆盖以下关键能力：

- `path` / `exclude_path`
- `read_from_head`
- `read_newly_discovered_files_from_head`
- `db` / `db.sync` / `db.compare_filename`
- `rotate_wait`
- `inotify_watcher`
- `buffer_chunk_size` / `buffer_max_size`
- `mem_buf_limit`
- `skip_long_lines` / `skip_empty_lines`
- `parser`
- `multiline.parser`
- `path_key` / `offset_key`
- `ignore_older`

### 3.2 对标口径

对标时要明确：

- 目标是达到同类成熟日志文件输入器应有的能力下限
- 不是要求配置名、内部状态文件格式、输出结构与 Fluent Bit 完全一致
- `wp-agent` 可以用更适合自身架构的对象模型替代 Fluent Bit 的 plugin/tag/chunk 习惯

### 3.3 我们应优于 Fluent Bit 的地方

第一版设计应明确争取在以下方面优于 Fluent Bit：

- `commit point`、checkpoint 与本地 spool / buffer 的关系更清晰
- resource binding 是一等语义，不依赖后置 filter 拼装
- 保护模式、退化模式和 drop 原因有统一状态机
- `standalone` / `managed` 模式下行为一致
- 自观测指标和审计事件更完整

---

## 4. 边界定义

### 4.1 它负责什么

文件日志输入负责：

- 发现匹配路径的目标文件
- 持续读取新增内容
- 进行按行切分
- 执行 parser / multiline 预处理
- 补充基础源信息和资源引用
- 把结果写入统一 telemetry pipeline
- 管理本地 checkpoint

### 4.2 它不负责什么

文件日志输入不负责：

- 远程文件内容读取
- 控制平面计划下发
- 复杂语义解析与 AI 推理
- 中心侧审计归档
- 取代 `warp-parse` 进行大规模规则解析

### 4.3 与 `file.tail` 的关系

必须明确区分：

- `logs.file_inputs[]`：
  常驻数据面输入
- `file.tail`：
  远程动作 opcode，用于临时诊断读取

两者都可能读取同一路径，但职责完全不同。

---

## 5. 第一版模块拆分

建议把文件日志输入拆成以下子模块：

- `file_target_discovery`
  负责 glob 展开、排除规则、文件初筛、目标变化检测
- `file_watcher`
  负责 `file watcher` 事件监听和轮询 fallback
- `tail_reader`
  负责按 `read offset` 增量读取、按行切分、长行处理
- `multiline_assembler`
  负责多行日志拼装与 flush
- `line_parser`
  负责 `raw/json/ndjson/cri/docker-json` 等轻量预解析
- `checkpoint_store`
  负责本地状态持久化与恢复
- `log_record_builder`
  负责补充源元数据、resource refs、统一 envelope

第一版不建议把这些模块暴露成独立进程。

它们应属于 `wp-agentd` 数据面内部模块，由统一 runtime supervisor 管理。

---

## 6. 运行模型

### 6.1 总体流水线

文件日志输入的推荐流水线应固定为：

```text
discover -> watch -> read delta -> split lines -> multiline
-> parse -> normalize -> attach resource refs
-> enqueue local telemetry buffer/spool -> reach commit point -> advance checkpoint
```

### 6.2 发现模型

第一版至少支持：

- `path_patterns[]`
- `exclude_path_patterns[]`
- 周期性 refresh
- 运行时新增文件发现

第一版建议支持的匹配语义：

- shell-style glob
- 多 pattern 并列
- exclude 在 include 之后生效

### 6.3 `file watcher` 模型

第一版建议支持两种 `file watcher` 模式：

- `native_notify`
  Linux 上优先使用 `inotify`
- `poll`
  基于 `stat` / `readdir` 的轮询 fallback

运行时建议：

- 默认 `auto`
- `auto` 优先选择 `native_notify`
- 原生 `file watcher` 不可用、配额不足或目标目录不适配时自动退回 `poll`

### 6.4 读取模型

每个被跟踪文件应维护独立 reader 状态：

- 当前 `file identity`
- 当前 `read offset`
- 最近读取时间
- 当前行缓冲
- multiline 暂存状态

读取语义固定为：

- 只读取追加内容
- 默认按 `\n` 切分
- 对未完成尾行可短暂缓存，直到补齐或 flush 超时

---

## 7. 配置骨架

第一版建议在 `AgentConfig.logs.file_inputs[]` 下固定如下结构：

```text
LogsSection {
  file_inputs[]?
}
```

```text
FileLogInput {
  id
  enabled
  path_patterns[]
  exclude_path_patterns[]?
  startup_position?
  discovered_file_position?
  ignore_older_ms?
  watcher_mode?
  refresh_interval_ms?
  rotate_wait_ms?
  parser?
  multiline?
  include_path_key?
  include_offset_key?
  include_file_id_key?
  line_buffer?
  checkpoint?
  buffering?
  resource_mapping?
  labels?
}
```

字段说明：

- `startup_position`
  - `tail`
  - `head`
- `discovered_file_position`
  表示启动完成后新发现文件在没有 checkpoint 时从哪里开始读
  - `tail`
  - `head`
- `watcher_mode`
  - `auto`
  - `native_notify`
  - `poll`

### 7.1 `parser`

```text
FileLogParser {
  mode
  time_key?
  time_format?
  body_key?
}
```

第一版建议：

- `mode`
  - `raw`
  - `json`
  - `ndjson`
  - `cri`
  - `docker_json`

### 7.2 `multiline`

```text
MultilineConfig {
  mode
  flush_timeout_ms?
  firstline_regex?
  continue_regex?
  max_lines?
  max_bytes?
}
```

第一版建议：

- `mode`
  - `off`
  - `docker`
  - `cri`
  - `java_stacktrace`
  - `python_traceback`
  - `go_panic`
  - `custom_regex`

### 7.3 `line_buffer`

```text
LineBufferConfig {
  initial_buffer_bytes?
  max_buffer_bytes?
  skip_long_lines?
  truncate_long_lines?
  skip_empty_lines?
}
```

这里的设计意图直接对标 Fluent Bit 的：

- `buffer_chunk_size`
- `buffer_max_size`
- `skip_long_lines`
- `skip_empty_lines`

### 7.4 `checkpoint`

```text
CheckpointConfig {
  enabled
  sync_mode?
  compare_filename?
  flush_interval_ms?
}
```

第一版建议：

- `enabled = true`
- `sync_mode`
  - `full`
  - `normal`
  - `off`
- `compare_filename = true`

这里的语义直接对标 Fluent Bit 的：

- `db`
- `db.sync`
- `db.compare_filename`

### 7.5 `buffering`

```text
FileLogBuffering {
  mem_buf_limit_bytes?
  static_batch_size_bytes?
  event_batch_size_bytes?
}
```

第一版建议保留这几个字段，用于对标 Fluent Bit 的：

- `mem_buf_limit`
- `static_batch_size`
- `event_batch_size`

---

## 8. 本地状态与 checkpoint

字段级 schema 独立定义在：

- [`../edge/log-file-state-schema.md`](../edge/log-file-state-schema.md)

本节只保留与运行语义直接相关的结论。

### 8.1 状态文件位置

第一版建议每个文件日志输入在本地维护：

- `state/logs/file_inputs/<id>/checkpoints.json`

### 8.2 `commit point` 与 checkpoint 推进规则

checkpoint 不能在“刚读到文件内容”时立即推进。

建议固定为：

- 读取内容并形成 record
- record 已成功进入本地 telemetry buffer
- 若启用了 spool，则以 durable spool 接纳成功为 `commit point`
- 若未启用 spool，则以 input 认可的本地 buffer 安全接纳点作为 `commit point`
- 之后再推进 checkpoint

这样可以保证：

- 正常运行与优雅退出时尽量不丢数据
- 异常崩溃时提供 at-least-once
- 允许小范围重复，不允许静默跳过

### 8.3 crash 恢复语义

第一版建议固定：

- 恢复时优先使用 checkpoint 中的最近已提交 `checkpoint offset`
- 如果 crash 发生在“已读取但未提交 checkpoint”窗口，允许重复少量记录
- 不允许因为 crash 把未持久化确认的数据视为已成功消费

---

## 9. rotate / truncate 语义

### 9.1 rotate

第一版必须支持最常见的 rename-rotate 场景：

1. 原路径文件被 rename 到新路径
2. 新文件在原路径重新创建
3. reader 继续读取旧文件剩余尾部
4. 同时开始跟踪新文件

建议提供：

- `rotate_wait_ms`

其语义与 Fluent Bit `rotate_wait` 对齐：

- 文件被 rotate 后，reader 继续保留一段时间，吸收尾部残留写入

### 9.2 truncate

第一版必须识别 truncate / copytruncate 这类场景。

建议规则：

- 同一 `file_id` 下，如果当前文件大小小于已提交 `checkpoint offset`
- 视为发生 truncate
- 记录一次 truncate 事件
- 将 `read offset` 重置到 `0`

### 9.3 inode 复用

inode 复用是 `file reader` / `tail reader` 的高风险边界。

第一版建议：

- 默认开启 `compare_filename`
- 必要时结合 `fingerprint`
- 当身份判断不可靠时，宁可保守重读少量内容，也不要静默跳过

---

## 10. 多行日志

第一版必须把 multiline 作为一等能力，而不是后期补丁。

原因很直接：

- Java / Python / Go stacktrace 很常见
- Docker / CRI 容器日志天然存在拆分与重组需求
- 没有 multiline，文件日志输入很难达到 Fluent Bit 同等级可用性

### 10.1 第一版最小模式

建议第一版至少支持：

- `docker`
- `cri`
- `java_stacktrace`
- `python_traceback`
- `go_panic`
- `custom_regex`

### 10.2 flush 规则

multiline 组装必须受以下限制：

- `flush_timeout_ms`
- `max_lines`
- `max_bytes`

任何一个限制触发时都必须：

- 立即结束当前组装
- 产生日志或指标
- 保留 `truncated` / `multiline_flush_reason` 等诊断字段

### 10.3 与 parser 的顺序

第一版建议固定顺序：

- 先按输入模式做必要的拆分或重组
- 再执行结构化 parser
- 最后进入 normalize / resource binding

不要让 parser 和 multiline 形成循环依赖。

---

## 11. 统一 record 与 resource 绑定

### 11.1 最小源字段

第一版建议每条日志 record 至少附带：

- `source_type = "file"`
- `source.path`
- `source.offset`
- `source.input_id`
- `observed_at`
- `body`

当启用对应开关时，还可补充：

- `source.file_id`
- `source.device_id`
- `source.inode`

### 11.2 resource binding

文件日志输入不能只把路径当字符串吐出去。

第一版应尽量在边缘建立：

- `host` 资源绑定
- 容器日志路径到 `container` / `k8s_pod` 的绑定
- 常见服务日志路径到 `service` 的绑定

必要时可结合：

- discovery cache
- 文件名 regex 提取
- 目录约定
- 运行时元数据

### 11.3 与 Fluent Bit 的差异

这里不要求照搬 Fluent Bit 的 `tag` / `tag_regex`。

`wp-agent` 更适合的做法是：

- 用显式 `resource_refs`
- 用结构化 `source.*`
- 把 filename 提取得到的字段放到 labels / attrs

---

## 12. 资源预算、backpressure 与保护模式

文件日志输入属于“不可反馈输入”。

这意味着：

- 无法像 HTTP / OTLP push 一样把背压直接传回上游
- 只能靠本地 queue、spool、限额和保护模式来吸收

### 12.1 第一版必须具备的保护手段

- 每文件独立读取 buffer 上限
- 每 input 级 `mem_buf_limit_bytes`
- 全局 telemetry queue / spool 上限
- 长行跳过或截断策略
- 当进入 `degraded` / `protect` 时降低扫描和读取强度

### 12.2 退化顺序

建议退化顺序：

1. 降低目录 refresh 频率
2. 暂停低优先级 input 的新文件发现
3. 减少单轮静态文件批处理量
4. 限制 multiline 暂存
5. 在达到硬上限时按 input 优先级丢弃，并记录原因

### 12.3 与 Fluent Bit 对标

这里至少要覆盖与 Fluent Bit 类似的两个层面：

- 读文件缓冲保护
- 输出拥塞时的内存保护

但 `wp-agent` 还应补充：

- 统一保护模式状态
- drop reason
- 控制面可见性

---

## 13. 自观测

第一版建议至少暴露以下指标：

- `agent_log_files_discovered`
- `agent_log_files_watched`
- `agent_log_lines_read_total`
- `agent_log_records_emitted_total`
- `agent_log_records_dropped_total`
- `agent_log_bytes_read_total`
- `agent_log_multiline_flush_total`
- `agent_log_checkpoint_commits_total`
- `agent_log_checkpoint_lag_bytes`
- `agent_log_rotate_events_total`
- `agent_log_truncate_events_total`
- `agent_log_reader_paused`

同时建议输出关键事件：

- `FileLogTargetDiscovered`
- `FileLogTargetDropped`
- `FileLogCheckpointRecovered`
- `FileLogRotated`
- `FileLogTruncated`
- `FileLogLongLineSkipped`
- `FileLogInputPaused`

---

## 14. 第一版验收标准

第一版建议至少满足以下验收：

1. 能稳定读取单文件和 glob 多文件输入。
2. 能在 restart 后基于 checkpoint 恢复，并满足 at-least-once。
3. 能正确处理 rename-rotate 和 truncate。
4. 能提供 `native_notify` 与 `poll` 两种 watcher 行为。
5. 能处理 `docker` / `cri` / `java_stacktrace` 三类常见 multiline。
6. 能在 backpressure 下维持资源硬边界，不因日志洪峰拖垮宿主机。
7. 能把 `source.path`、`source.offset`、`resource_refs` 稳定挂入统一 record。
8. 能用自观测指标证明与 Fluent Bit `tail` 同等级的关键能力已具备。

---

## 15. 当前决定

当前阶段固定以下结论：

- 文件日志输入必须进入 `wp-agentd` 第一版 logs 设计范围
- 其目标是对标 Fluent Bit `tail`，不是依赖 Fluent Bit
- 配置、checkpoint、rotate、multiline、budget 必须一起设计，不能拆成零散补丁
- `file.tail` 不能替代常驻文件日志采集

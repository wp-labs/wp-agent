# wp-agent 术语词典

## 1. 文档目的

本文档定义 `wp-agent` 设计文档中的统一术语。

目标是解决三类问题：

- 同一概念出现多个叫法
- 作者侧词汇与执行侧词汇混用
- 数据面、控制面、边缘本地状态词汇相互污染

使用原则：

- 新文档优先引用本词典
- 如与旧文档冲突，以本词典为准
- 发现新术语前，先判断能否复用现有词
- 术语定义尽量同时说明：作用域、统一定义、以及不应混用的近义词

---

## 2. 总体规则

### 2.1 作者侧与执行侧分开命名

- 作者侧可以使用更贴近 DSL 的词
- 执行侧必须使用更贴近 runtime / IR 的词
- 术语统一必须带作用域，不能把某一层的替换规则误推广到所有文档

例如：

- 作者侧：`when / expect / emit / fail`
- IR 侧：`branch / guard / output step / abort`

### 2.2 控制面与数据面分开命名

- 控制面对象不要借用数据面队列词
- 数据面 buffer/spool 不要和本地执行队列混用
- 同一个英文词如需跨层复用，必须带作用域限定语

例如：

- `IR provenance`
- `source.path`
- `execution_queue`

### 2.3 边缘本地状态优先用 execution 词根

在 `wp-agentd` 本地状态中，优先使用：

- `execution_queue`
- `running`
- `reporting`
- `history`

不再笼统使用 `queue`

### 2.4 对标成熟产品时，对齐能力，不对齐配置名

当 `wp-agent` 设计文档提到对标 `Fluent Bit`、`OpenTelemetry Collector` 等成熟产品时，默认含义是：

- 对齐能力边界
- 对齐关键运行时行为
- 对齐验收口径

不默认意味着：

- 配置字段名兼容
- 状态文件格式兼容
- 插件命名兼容
- tag/chunk/pipeline 对象模型兼容

例如：

- 可以说“文件日志输入对标 `Fluent Bit tail input`”
- 不建议说“`wp-agent` 的 logs 配置应兼容 `Fluent Bit tail` 配置”

只有在文档显式写出“兼容配置”或“兼容协议对象”时，才表示需要兼容对应产品的外部接口。

---

## 3. 角色术语

### 3.1 `wp-agent`

整个系统中的环境内代理产品名。

包含但不限于：

- `wp-agentd`
- `wp-agent-exec`
- `wp-agent-upgrader`

### 3.2 `wp-agentd`

边缘常驻 daemon。

统一定义：

- 边缘控制器
- 数据面主进程
- 本地调度入口

不应称为：

- executor
- step runner
- action engine

### 3.3 `wp-agent-exec`

边缘按需拉起的受控执行器。

统一定义：

- 一次性执行进程
- 只执行 `ActionPlan.program`

不应称为：

- daemon
- controller
- scheduler

### 3.4 `wp-agent-upgrader`

边缘按需拉起的升级辅助进程。

统一定义：

- upgrade helper
- upgrade executor

---

## 4. 执行契约术语

### 4.1 `ActionPlan`

中心编译完成、下发到边缘的最终执行对象。

统一定义：

- 单目标
- 已签名
- 已绑定约束
- 边缘真正消费的控制对象

### 4.2 `ActionPlan IR`

`ActionPlan` 内部的执行契约层。

统一定义：

- 边缘唯一理解的执行语义

### 4.3 `ActionResult`

边缘执行结束后回传的最终结果对象。

统一定义：

- 最终状态
- `step_records[]`
- `outputs`

### 4.4 `StepActionRecord`

单个 step 的执行记录。

统一定义：

- 审计与排障记录
- 不是业务输出值

不应称为：

- step result value
- output item

### 4.5 `step`

执行契约中的最小执行单位。

统一定义：

- runtime 概念
- 审计单位
- 超时/重试/错误处理单位

### 4.6 `output step`

IR 中 `kind = "output"` 的 step 类型。

统一定义：

- 负责从执行上下文中选择结果进入最终输出
- 是 `program.steps[]` 中的一种 step

不应直接拿它指代最终返回结果对象。

### 4.7 `outputs`

`ActionResult` 中对上暴露的最终业务结果集合。

统一定义：

- 来自 `program.output step` 的结果选择
- 给控制平面和调用方消费

不应与 `stdout` 混用。

---

## 5. IR 关键词

以下词是 IR 侧统一推荐用法。

它们的作用域仅限于：

- `ActionPlan`
- `ActionPlan IR`
- `ActionResult`
- `wp-agent-exec` runtime

不应把这张表直接套用到作者侧 DSL 文档。

| Scope | 推荐术语 | 含义 | 不再推荐 |
|---|---|---|---|
| `IR` | `constraints` | 最终执行约束 | `control` |
| `IR` | `provenance` | 来源追踪信息 | `source` |
| `IR` | `invoke` | opcode 调用 step | `call` |
| `IR` | `guard` | 受控断言 step | `assert` |
| `IR` | `output step` | 输出 step 类型 | `emit` |
| `IR` | `abort` | 显式失败 step | `fail` |
| `IR` | `entry` | 程序入口 step id | `entry_step` |
| `IR` | `next` | 顺序流转目标 | `next_step` |
| `IR` | `then` | 条件真分支 | `then_step` |
| `IR` | `else` | 条件假分支 | `else_step` |

---

## 6. 作者侧术语

### 6.1 `control.wac`

作者侧安全控制输入文件。

统一定义：

- 作者输入
- 中心侧治理输入

不是边缘执行输入。

### 6.2 `execution spec`

作者侧执行功能输入的通用称呼。

当前候选：

- `run.gxl`
- `run.war`

在 frontend 未定型前，统一使用：

- `execution spec`

### 6.3 `authoring frontend`

作者输入语言前端的统称。

当前候选：

- `run.gxl`
- `run.war`

注意：

- `authoring frontend` 指作者侧输入形式
- 不等于边缘执行协议
- 不等于 `ActionPlan`

### 6.4 `native_json`

一种结构化作者输入候选形式。

统一定义：

- 仍属于 authoring frontend candidate
- 可以更接近协议对象
- 但在进入边缘前仍是作者侧输入，不是最终执行契约

---

## 7. 边缘本地状态术语

### 7.1 `execution_queue`

统一定义：

- 已通过本地校验
- 尚未拉起 `wp-agent-exec`
- 等待 `execution_scheduler` 调度

不是：

- 数据面 buffer/spool
- 网络消息队列
- 观测事件队列

### 7.2 `running`

统一定义：

- 已被本地调度
- 正在本地执行中的 execution

### 7.3 `reporting`

统一定义：

- 已形成最终结果
- 正在等待或进行中心回传

### 7.4 `history`

统一定义：

- 已完成闭环
- 只保留最近摘要

---

## 8. 模块术语

### 8.1 `control_receiver`

接收中心对象的模块。

统一定义：

- 只接对象
- 不排队
- 不 spawn 进程

### 8.2 `plan_validator`

校验 `ActionPlan` 的模块。

统一定义：

- 只负责“能不能执行”
- 不负责调度

### 8.3 `execution_scheduler`

本地状态机和执行调度拥有者。

统一定义：

- 持有 `execution_queue`
- 持有 running 状态
- 决定 spawn / cancel / timeout

### 8.4 `executor_manager`

本地执行子进程管理模块。

统一定义：

- 只管 workdir
- 只管进程
- 不拥有调度状态

### 8.5 `result_aggregator`

本地结果汇总模块。

统一定义：

- 读取 `result.json`
- 形成 `ActionResult`
- 持有 reporting 状态

---

## 9. Logs 数据面术语

### 9.1 `file input`

统一定义：

- `wp-agentd` 数据面中的常驻文件日志输入能力
- 负责发现、跟踪、读取追加内容，并进入统一 telemetry pipeline

推荐使用场景：

- `logs.file_inputs[]`
- `file log input`

不应拿它指代：

- 远程动作 `file.tail`
- 一次性文件读取
- 实现备注中的内部 reader 实例

如需指实现内部对象，推荐使用：

- `file reader`
- `tail reader`

### 9.2 `file watcher`

统一定义：

- 文件目标变化监听机制
- 负责目录/文件变化感知，不直接负责 record 构建

第一版推荐枚举：

- `native_notify`
- `poll`
- `auto`

在文件日志输入语境下可简称：

- `watcher`

但离开该语境时，优先写全称 `file watcher`。

### 9.3 `read offset`

统一定义：

- reader 当前已读取到的位置
- 运行中内存态进度

注意：

- `read offset` 不等于 `checkpoint`
- `read offset` 可以先于 record 安全提交而前进

### 9.4 `commit point`

统一定义：

- file input 判定“可以推进 checkpoint”的最小安全条件
- `read offset` 与 `checkpoint offset` 之间的语义边界

注意：

- `commit point` 是语义边界，不一定对应单独持久化字段
- 第一版默认以“record 已进入本地 telemetry buffer/spool 的安全接纳点”作为 `commit point`

### 9.5 `checkpoint`

统一定义：

- 文件输入已安全提交的读取进度
- crash 恢复的起点

注意：

- `checkpoint` 不等于 `read offset`
- `checkpoint` 应在越过 `commit point` 后推进
- 新文档如需写明语义，优先使用 `checkpoint offset`

### 9.6 `file identity`

统一定义：

- 用于区分“这是哪个被跟踪文件”的稳定身份信息

第一版推荐来源：

- `device_id + inode`
- 或 `canonical_path + fingerprint`

注意：

- `path` 只表示当前路径
- `file identity` 用于 checkpoint 关联和 rotate/truncate 判断

### 9.7 `rotate`

统一定义：

- 文件因 rename-rotate 等机制发生身份切换，但 reader 需要继续处理旧尾部与新文件接续

推荐使用：

- `rotate`
- `rotate_wait_ms`

不建议混用：

- `reopen`
- `roll`

### 9.8 `truncate`

统一定义：

- 同一被跟踪文件内容长度回退，已提交 `checkpoint offset` 失效，需要从较小位置重新读取

推荐使用：

- `truncate`

不建议混用：

- `reset tail`
- `rewind`

### 9.9 `multiline`

统一定义：

- 多行日志拼装能力
- 在 record 进入 normalize 前完成事件边界合并

不应称为：

- parser chain
- post filter

### 9.10 `parser`

在文件日志输入语境下，统一定义：

- 输入侧轻量预解析器
- 负责把 `raw/json/ndjson/cri/docker_json` 等输入转换为统一字段骨架

注意：

- 它不等于中心侧规则解析
- 它不等于 `warp-parse` 的完整解析链路

### 9.11 `startup_position`

统一定义：

- agent 启动时，对“无 checkpoint 的已存在文件”从何处开始读取的策略

第一版推荐枚举：

- `head`
- `tail`

### 9.12 `discovered_file_position`

统一定义：

- agent 已启动后，对“运行时新发现且无 checkpoint 的文件”从何处开始读取的策略

第一版推荐枚举：

- `head`
- `tail`

注意：

- 不要与 `startup_position` 混用

### 9.13 `source.path` / `source.offset`

统一定义：

- 文件日志 record 的结构化来源字段
- 属于数据面 source 字段，不属于 IR `provenance`

推荐使用：

- `source.path`
- `source.offset`
- `source.input_id`

不建议退回到：

- 非结构化 path tag
- 仅靠文件名拼接字符串做来源表达

---

## 10. Metrics 数据面术语

### 10.1 `collector`

更偏本地采集器。

适用：

- `host_metrics`
- `process_metrics`
- `container_metrics`

### 10.2 `scraper`

更偏 pull 型目标抓取器。

适用：

- `prom_scrape`
- `jmx_scrape`

### 10.3 `receiver`

更偏 push 型输入接收器。

适用：

- `otlp_metrics_receiver`
- `statsd_receiver`

### 10.4 `exporter`

统一定义：

- 数据上送组件
- 或外部兼容目标中的现有 exporter

在 `wp-agent` 体系里：

- 外部 exporter 是 fallback
- 不是默认前提

---

## 11. 明确禁止混用的词

以下词对当前设计最容易造成歧义，建议避免混用：

- `queue`
  应改成 `execution_queue` 或 `buffer/spool`
- `control`
  在 IR 中应改成 `constraints`
- `source`
  在 IR 语境中应改成 `provenance`；在数据面语境中可保留 `source.path`、`source.offset` 等结构化来源字段
- `output`
  应按上下文改成 `stdout`、`output step` 或 `outputs`
- `action`
  不要拿来指代 step 或业务输出
- `result`
  必须区分 `ActionResult`、`StepActionRecord`、`outputs`
- `tail`
  在数据面文档中优先写 `file input` / `file log input`；仅在对标 `Fluent Bit tail input`、描述内部 `tail reader`，或远程动作 `file.tail` 时使用
- `offset`
  在文件日志输入文档中优先写 `checkpoint offset`、`read offset` 或 `source.offset`，避免单独使用造成歧义

---

## 12. 当前推荐用法速查

如果你要写新文档，优先使用：

- 边缘主进程：`wp-agentd`
- 边缘执行器：`wp-agent-exec`
- 最终执行对象：`ActionPlan`
- 最终回传对象：`ActionResult`
- 最小执行单位：`step`
- 单步执行记录：`StepActionRecord`
- 最终约束：`constraints`
- 来源追踪：`provenance`
- IR 输出 step：`output step`
- 最终业务结果：`outputs`
- 待调度 execution 队列：`execution_queue`
- 常驻文件日志输入：`file input`
- 文件变化监听：`file watcher`
- 运行态读取位置：`read offset`
- checkpoint 推进语义边界：`commit point`
- 文件进度提交点：`checkpoint`
- 多行日志拼装：`multiline`
- pull 型指标采集：`scraper`
- push 型指标接收：`receiver`
- 本地指标采集器：`collector`

---

## 13. 当前决定

当前阶段固定以下结论：

- 新文档优先引用本词典
- 作者侧词与 IR 词必须分开
- `execution_queue` 取代笼统的 `queue`
- 对标成熟产品时优先对齐能力，不对齐配置名
- `constraints` / `provenance` / `invoke` / `guard` / `output step` / `abort` 作为 IR 侧统一术语

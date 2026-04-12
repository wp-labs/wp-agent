# wp-agent 动作 DSL 与执行 IR 设计

## 1. 文档目的

本文档定义 `wp-agent` 的远程动作描述体系，重点回答以下问题：

- 是否需要 DSL 来表达远程执行能力
- 为什么不能直接用 YAML / TOML 作为最终表达形式
- 为什么不能让边缘 agent 直接解释一个功能过强的 DSL
- 中心节点的作者 DSL 与边缘节点的执行 IR 应如何分层
- 远程动作如何与审批、风险、审计、签名和最小权限模型结合

本文档默认建立在以下设计文档之上：

- [`target.md`](../foundation/target.md)
- [`architecture.md`](../foundation/architecture.md)
- [`action-plan-ir.md`](action-plan-ir.md)
- [`security-model.md`](../foundation/security-model.md)
- [`control-plane.md`](../center/control-plane.md)
- [`run-gxl-subset.md`](run-gxl-subset.md)
- [`run-gxl-construct-mapping.md`](run-gxl-construct-mapping.md)
- [`action-schema.md`](action-schema.md)
- [`references.md`](../foundation/references.md)

---

## 2. 核心结论

`wp-agent` 需要作者侧描述语言，但当前不应先把 frontend 定死为 `run.gxl` 或 `run.war`。

正确分层应为：

- 中心侧使用表达力更强、对人类和 AI 更友好的 Authoring Frontend
  该 Frontend 至少拆成两个输入面：
  一个 `control` 文件描述安全控制，一个 `execution spec` 文件描述执行功能
- 边缘侧不直接执行 Authoring DSL，而是只执行编译后的 ActionPlan IR

也就是说：

- `control DSL` 与 `execution spec` 共同构成写作输入
- `ActionPlan IR` 是执行语言

当前阶段先固定 `ActionPlan IR`。

至于 `execution spec` 最终采用 `run.gxl`、`run.war` 还是其他 frontend，后续再收敛，但都必须编译到同一 IR。

这是为了同时满足两类目标：

- 让动作描述具有足够表达力，便于写作、生成、复用和审批
- 让边缘执行保持可控、可审计、可限权、可静态校验

---

## 3. 为什么不能只用 YAML / TOML

YAML / TOML 适合做静态配置，但不适合作为远程动作的一线作者语言。

主要问题有：

- 表达层级弱：
  当动作包含条件、分支、前置检查、期望结果时，纯配置格式会迅速变得臃肿
- 可读性差：
  人写复杂动作时容易退化成嵌套键值结构，难以扫读
- 审批体验差：
  审批者需要理解“做什么”和“风险是什么”，而不是盯着一堆配置字段
- AI 生成不稳定：
  对 AI 来说，生成结构正确的 YAML 不难，但生成“结构正确且语义清晰可审”的 YAML 并不天然友好

因此，YAML / TOML 可以作为中间存储格式或测试夹具格式，但不建议作为主要作者语言。

---

## 4. 为什么边缘不能直接执行强 DSL

如果让 `wp-agent-exec` 直接解释一个功能很强的 DSL，会带来以下风险：

- 安全边界下沉：
  DSL 很容易逐步演化成远程脚本语言
- 审计困难：
  审批时难以准确判断最终执行路径
- 风险不可静态收敛：
  动态分支、循环、拼接命令会让风险分析复杂化
- 运行时变重：
  边缘需要解析、校验、计划和执行完整 DSL，违背轻量目标
- 版本兼容复杂：
  DSL 语义一旦增强，所有 agent 解释器都要随之升级

因此，边缘执行器应只认受控 IR，不直接认原始 DSL。

---

## 5. 两层模型

### 5.1 Authoring DSL

中心侧作者 DSL 不是一个混合文件，而是两个文件共同组成的作者模型：

- `*.control.*`
- `*.run.gxl`

它的目标是：

- 提高表达力
- 提高可读性
- 提高审批可理解性
- 提高 AI 辅助生成质量

其中：

- `control` 文件适合表达：
  动作目标、风险等级、审批要求、超时、资源限制、路径 / 服务白名单、capability 要求
- `run.gxl` 文件适合表达：
  前置检查、条件分支、步骤顺序、期望结果、输出摘要

### 5.2 ActionPlan IR

边缘侧执行 IR 的目标是：

- 可静态校验
- 可签名
- 可版本化
- 可限权
- 可确定性执行

它不追求“像语言一样好写”，只追求“像执行计划一样可控”。

### 5.3 编译边界

建议由中心节点承担以下职责：

- 解析 DSL
- 类型检查
- 风险分析
- 审批绑定
- capability 检查
- 编译 IR
- 对 IR 做签名或完整性保护

建议由边缘 agent 承担以下职责：

- 校验 IR 版本
- 校验签名与来源
- 校验本机 capability 是否满足
- 校验审批与过期时间是否满足
- 解释并执行有限 opcode
- 回传结构化结果

### 5.4 两文件作者模型

为了把“安全控制”和“执行功能”明确分离，作者侧采用两文件模型。

推荐命名如下：

- `action-name.control.wac`
- `action-name.run.gxl`

其中：

- `*.control.wac`
  描述安全控制、目标范围、风险等级、审批要求、超时、资源限制、路径 / 服务白名单、capability 要求
- `*.run.gxl`
  描述具体执行步骤，例如 `step`、`when`、`expect`、`emit`、`fail`、`retry`

这样拆分的好处是：

- 审批人优先看 `control`
- 动作作者优先写 `run.gxl`
- 中心节点可以先校验安全控制，再编译执行功能
- 同一个 `run.gxl` 可以在不同受控边界下复用

这里必须强调：

- 两文件模型是作者侧规范，不是可选补充
- 边缘侧不能直接执行两个分离文件
- 中心节点必须把 `control + run.gxl` 绑定、校验、审批并编译成一个最终 IR
- 同名不代表自动绑定，必须通过模板或显式元数据绑定

也就是说，两文件模型是作者模型，不是边缘执行模型。

建议目录结构例如：

```text
actions/
  nginx-status.control.wac
  nginx-status.run.gxl
  nginx-reload.control.wac
  nginx-reload.run.gxl
```

### 5.5 安全控制由谁设置

安全控制不应由动作作者、审批人或边缘 agent 中的任何一方单独决定。

正确模型应是中心节点按多层来源合成最终 `control`。

建议至少区分四层来源：

- 平台基线策略：
  由平台安全 / 平台治理团队设置，定义全局不可突破的硬边界
- 环境或租户策略：
  由环境管理员或租户管理员设置，定义某个环境、租户、节点组的额外限制
- 动作模板控制：
  由受信动作模板维护者设置，定义某类动作默认的风险、审批、白名单和限制
- 单次请求参数：
  由发起人提交，通常只允许在已有边界内收缩目标和参数，不允许突破上层限制

这四层的职责建议如下：

- 平台基线策略：
  决定哪些 opcode 全局禁用、哪些风险级别必须审批、哪些路径或服务永远不可操作
- 环境或租户策略：
  决定某个环境中哪些动作允许、哪些节点禁止、哪些审批要求更严格
- 动作模板控制：
  决定某个动作默认的 `risk`、`approval`、`timeout`、`limits`、`allow` 和 capability 要求
- 单次请求参数：
  决定这次具体针对哪个目标、使用哪些允许范围内的输入参数

正确的优先级关系应为：

- 下层不能放宽上层限制
- 单次请求只能收缩，不能扩权
- 边缘 agent 只校验最终 `control`，不自行生成安全控制

换句话说：

- `run.gxl` 主要由动作作者负责
- `control` 主要由中心治理系统生成和收敛

### 5.6 安全控制合成流程

建议中心节点按以下顺序合成最终 `control`：

1. 读取平台基线策略
2. 叠加环境 / 租户策略
3. 加载动作模板默认控制
4. 合并单次请求参数
5. 检查是否出现越权或越界
6. 绑定审批上下文
7. 生成最终 `control`
8. 与 `run.gxl` 一起编译成最终 IR

这意味着最终生效的 `control` 本质上是“治理结果”，而不是“作者输入原文”。

---

## 6. Authoring DSL 设计目标

### 6.1 应具备的能力

第一版 Authoring DSL 的能力应分别落在两个文件中。

`control` 文件建议具备以下表达能力：

- `target`
- `risk`
- `approval`
- `timeout`
- `limits`
- `allow`
- `require`
- `expires`

`run.gxl` 文件建议具备以下表达能力：

- `step`
- `when`
- `expect`
- `emit`
- `fail`
- `retry`

### 6.2 `run.gxl` 的定位

`run.gxl` 不是完整 GXL，也不是边缘执行语言。

它的正确定位应是：

- 基于 GXL 风格的中心侧作者 DSL
- 用于描述执行功能
- 由中心编译器解析并编译为 IR
- 不直接下发到边缘执行器

这意味着：

- 可以统一语法和工具链
- 可以尽量复用 `galaxy-flow` 的解析和编辑体验
- 不能把完整 `galaxy-flow` 运行时直接带到边缘

### 6.3 允许的 GXL 子集

`run.gxl` 建议只保留受控执行所需的最小表达能力。

建议允许：

- `step`
- `when`
- `expect`
- `emit`
- `fail`
- `retry`
- 受控变量引用
- 受控表达式
- 受控 opcode 调用，例如：
  `process.list`、`file.read_range`、`service.status`

这些能力本质上应映射到白名单 opcode，而不是映射到任意 shell。

### 6.4 禁止的 GXL 能力

为了避免 `run.gxl` 退化成通用脚本执行器，第一版建议明确禁止以下能力：

- `gx.cmd`
- `gx.shell`
- `gx.download`
- `gx.upload`
- 动态 include / 动态模块加载
- 任意外部脚本执行
- 运行时下载后执行
- 不受控制的系统命令拼接

### 6.5 不应具备的能力

第一版明确不建议支持：

- 任意循环
- 用户自定义函数
- 动态下载代码后执行
- 任意 shell 片段拼接
- 未经白名单约束的系统调用
- 跨节点直接跳转执行

这里的原则很明确：

第一版 DSL 要足够表达“受控动作计划”，但不要演化成通用编程语言。

---

## 7. Authoring DSL 示例

下面是建议风格示例。

`nginx-status.control.wac`

```txt
target node("prod-web-01")
risk R1
approval not_required
timeout 10s

limits {
  max_stdout 64kb
  max_stderr 32kb
  max_memory 64mb
}

require {
  capability "process.list"
  capability "socket.check"
}
```

`nginx-status.run.gxl`

```txt
step nginx = process.list(name = "nginx")
step port  = socket.check(port = 443)

when nginx.empty {
  emit "nginx process missing"
  fail "service_missing"
}

expect port.open == true
emit "nginx is running"
```

这个 DSL 的可读目标是：

- 审批人能快速看懂它做什么
- AI 能较稳定生成
- 编译器能较稳定做静态分析

---

## 8. 执行 IR 设计目标

### 8.1 IR 原则

执行 IR 应满足：

- 强类型
- 显式版本号
- 显式动作种类
- 显式风险等级
- 显式限制条件
- 显式审批引用
- 显式 opcode 序列
- 显式结果格式

### 8.2 IR 示例

```json
{
  "ir_version": "action-plan/v1",
  "action_id": "act_001",
  "request_id": "req_001",
  "tenant_id": "t1",
  "environment_id": "prod",
  "target": {
    "node_id": "prod-web-01"
  },
  "risk": "R1",
  "approval_ref": null,
  "timeout_sec": 10,
  "limits": {
    "max_stdout_bytes": 65536,
    "max_stderr_bytes": 32768,
    "max_memory_mb": 64
  },
  "steps": [
    {
      "id": "s1",
      "op": "process.list",
      "args": { "name": "nginx" }
    },
    {
      "id": "s2",
      "op": "socket.check",
      "args": { "port": 443 }
    }
  ],
  "expects": [
    {
      "expr": "s2.open == true"
    }
  ]
}
```

这里最关键的是：

- 不包含任意 shell 文本
- 不包含无限制脚本能力
- 不把复杂解释责任推给边缘
- `control` 与 `run.gxl` 在进入边缘前已经被绑定并编译

---

## 9. Opcode 白名单模型

### 9.1 为什么需要 opcode

边缘执行器不应接收“自由命令”，而应接收一组固定 `opcode`。

这样做的好处是：

- 能静态做风险分类
- 能做 capability 协商
- 能做最小权限映射
- 能做执行器资源限制
- 能做可预测的审计

### 9.2 第一版建议 opcode 分类

建议第一版按以下类别定义 opcode：

- 诊断类：
  `process.list`、`process.stat`、`socket.check`、`service.status`
- 读取类：
  `file.tail`、`file.read_range`、`config.inspect`
- agent 控制类：
  `agent.reload`、`agent.health_check`
- 服务控制类：
  `service.restart`、`service.reload`
- 升级辅助类：
  `upgrade.prepare`、`upgrade.verify`

### 9.3 第一版不建议 opcode

第一版不建议开放：

- `shell.exec`
- `script.eval`
- `download.and.run`
- `python.eval`
- `bash.inline`

这些能力一旦直接开放，系统很容易退化成“带审批外壳的远程 shell”。

### 9.4 能力设计示例

为了避免 opcode 只停留在名字层面，第一版建议为每个核心能力至少定义以下内容：

- 用途
- 输入参数
- 返回结构
- 建议风险等级
- 两文件示例

同时建议统一按两文件分工：

- `control` 文件共享字段：
  `target`、`risk`、`approval`、`timeout`、`limits`、`allow`、`require`
- `run.gxl` 文件共享字段：
  `step`、`when`、`expect`、`emit`、`fail`、`retry`

#### 9.4.1 诊断类

以下能力示例统一按两文件展示：

- `*.control.wac` 负责安全控制
- `*.run.gxl` 负责执行功能

`process.list`

- 用途：
  按名称、用户、命令行特征过滤进程列表
- 建议参数：
  `name`、`user`、`contains_cmdline`、`limit`
- 建议返回：
  `items[] { pid, ppid, name, user, cmdline, start_time }`
- 建议风险等级：
  `R0`
- 两文件示例：

`diag-nginx-process-list.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 5s

require {
  capability "process.list"
}
```

`diag-nginx-process-list.run.gxl`

```txt
step procs = process.list(name = "nginx", limit = 20)
emit procs
```

`process.stat`

- 用途：
  查看单个进程资源状态
- 建议参数：
  `pid`、`name`
- 建议返回：
  `{ pid, cpu_pct, rss_bytes, fd_count, thread_count, state }`
- 建议风险等级：
  `R0`
- 两文件示例：

`diag-agent-process-stat.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 5s

require {
  capability "process.stat"
}
```

`diag-agent-process-stat.run.gxl`

```txt
step p = process.stat(name = "wp-agentd")
expect p.pid > 0
emit p
```

`socket.check`

- 用途：
  检查端口监听或连通状态
- 建议参数：
  `port`、`protocol`、`state`
- 建议返回：
  `{ open, listeners[], protocol, port }`
- 建议风险等级：
  `R0`
- 两文件示例：

`diag-port-443.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 3s

require {
  capability "socket.check"
}
```

`diag-port-443.run.gxl`

```txt
step s = socket.check(port = 443, protocol = "tcp")
expect s.open == true
emit s
```

`service.status`

- 用途：
  查询服务运行状态
- 建议参数：
  `service`、`manager`
- 建议返回：
  `{ name, state, substate, enabled, pid }`
- 建议风险等级：
  `R0`
- 两文件示例：

`diag-service-status.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 5s

require {
  capability "service.status"
}
```

`diag-service-status.run.gxl`

```txt
step svc = service.status(service = "nginx", manager = "systemd")
emit svc
```

#### 9.4.2 读取类

`file.tail`

- 用途：
  读取日志尾部内容
- 建议参数：
  `path`、`lines`、`max_bytes`
- 建议返回：
  `{ path, lines[], truncated }`
- 建议风险等级：
  `R0`
- 额外约束：
  必须受路径白名单约束
- 两文件示例：

`read-nginx-error-tail.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 5s

limits {
  max_stdout 64kb
}

allow {
  paths ["/var/log/nginx/error.log"]
}

require {
  capability "file.tail"
}
```

`read-nginx-error-tail.run.gxl`

```txt
step tail = file.tail(path = "/var/log/nginx/error.log", lines = 100)
emit tail
```

`file.read_range`

- 用途：
  按偏移读取文件片段
- 建议参数：
  `path`、`offset`、`length`
- 建议返回：
  `{ path, offset, length, content, truncated }`
- 建议风险等级：
  `R0`
- 两文件示例：

`read-config-head.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 5s

allow {
  paths ["/etc/nginx/nginx.conf"]
}

require {
  capability "file.read_range"
}
```

`read-config-head.run.gxl`

```txt
step part = file.read_range(path = "/etc/nginx/nginx.conf", offset = 0, length = 4096)
emit part
```

`config.inspect`

- 用途：
  解析配置并抽取目标键
- 建议参数：
  `path`、`format`、`selectors[]`
- 建议返回：
  `{ matched: { key: value } }`
- 建议风险等级：
  `R0`
- 两文件示例：

`inspect-nginx-worker-config.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 5s

allow {
  paths ["/etc/nginx/nginx.conf"]
}

require {
  capability "config.inspect"
}
```

`inspect-nginx-worker-config.run.gxl`

```txt
step cfg = config.inspect(
  path = "/etc/nginx/nginx.conf",
  format = "nginx",
  selectors = ["worker_processes", "events.worker_connections"]
)
emit cfg
```

#### 9.4.3 agent 控制类

`agent.reload`

- 用途：
  重载 agent 配置或策略
- 建议参数：
  `scope`
- 建议返回：
  `{ reloaded, version_before, version_after }`
- 建议风险等级：
  `R1`
- 两文件示例：

`reload-agent-policy.control.wac`

```txt
target node("prod-web-01")
risk R1
approval role("ops")
timeout 10s

require {
  capability "agent.reload"
}
```

`reload-agent-policy.run.gxl`

```txt
step r = agent.reload(scope = "policy")
expect r.reloaded == true
emit r
```

`agent.health_check`

- 用途：
  触发 agent 自检
- 建议参数：
  `deep`、`include_buffer`、`include_exporter`
- 建议返回：
  `{ healthy, checks[], summary }`
- 建议风险等级：
  `R0` 或 `R1`
- 两文件示例：

`agent-health-check.control.wac`

```txt
target node("prod-web-01")
risk R0
approval not_required
timeout 8s

require {
  capability "agent.health_check"
}
```

`agent-health-check.run.gxl`

```txt
step hc = agent.health_check(deep = true, include_buffer = true)
expect hc.healthy == true
emit hc.summary
```

#### 9.4.4 服务控制类

`service.restart`

- 用途：
  重启服务
- 建议参数：
  `service`、`manager`、`graceful`、`wait_ready`
- 建议返回：
  `{ restarted, old_pid, new_pid, ready }`
- 建议风险等级：
  `R2`
- 两文件示例：

`restart-nginx.control.wac`

```txt
target node("prod-web-01")
risk R2
approval required("team-lead")
timeout 30s

allow {
  services ["nginx"]
}

require {
  capability "service.restart"
}
```

`restart-nginx.run.gxl`

```txt
step svc = service.restart(service = "nginx", manager = "systemd", graceful = true, wait_ready = true)
expect svc.ready == true
emit svc
```

`service.reload`

- 用途：
  平滑重载服务
- 建议参数：
  `service`、`manager`
- 建议返回：
  `{ reloaded, ready }`
- 建议风险等级：
  `R1`
- 两文件示例：

`reload-nginx.control.wac`

```txt
target node("prod-web-01")
risk R1
approval role("ops")
timeout 20s

allow {
  services ["nginx"]
}

require {
  capability "service.reload"
}
```

`reload-nginx.run.gxl`

```txt
step svc = service.reload(service = "nginx", manager = "systemd")
expect svc.ready == true
emit svc
```

#### 9.4.5 升级辅助类

`upgrade.prepare`

- 用途：
  下载并预检查升级包，但不执行切换
- 建议参数：
  `version`、`channel`、`artifact_ref`
- 建议返回：
  `{ prepared, package_verified, compatible, disk_ok }`
- 建议风险等级：
  `R1`
- 两文件示例：

`prepare-agent-upgrade.control.wac`

```txt
target node("prod-web-01")
risk R1
approval role("release")
timeout 60s

require {
  capability "upgrade.prepare"
}
```

`prepare-agent-upgrade.run.gxl`

```txt
step u = upgrade.prepare(version = "1.2.3", channel = "stable")
expect u.package_verified == true
expect u.compatible == true
emit u
```

`upgrade.verify`

- 用途：
  校验升级后的版本和健康状态
- 建议参数：
  `expected_version`、`include_buffer`、`include_exporter`
- 建议返回：
  `{ version_ok, healthy, checks[] }`
- 建议风险等级：
  `R1`
- 两文件示例：

`verify-agent-upgrade.control.wac`

```txt
target node("prod-web-01")
risk R1
approval role("release")
timeout 30s

require {
  capability "upgrade.verify"
}
```

`verify-agent-upgrade.run.gxl`

```txt
step v = upgrade.verify(expected_version = "1.2.3", include_buffer = true)
expect v.version_ok == true
expect v.healthy == true
emit v
```

#### 9.4.6 建议的统一限制模型

对于以上所有能力，建议统一支持以下限制字段：

- `max_stdout`
- `max_stderr`
- `max_memory`
- `timeout`
- `concurrency`
- `allowed_paths`
- `allowed_services`

#### 9.4.7 建议的统一风险分层

建议第一版先按以下口径收敛：

- `R0`：
  只读诊断、只读读取
- `R1`：
  agent 自检、reload、upgrade prepare/verify
- `R2`：
  service restart
- `R3`：
  第一版暂不开放

---

## 10. 风险与审批集成

### 10.1 DSL 中必须显式声明风险

每个动作都必须显式声明：

- `risk`
- `approval`
- `target`
- `timeout`
- `limits`

如果作者未填写，编译器不能默认忽略，而应：

- 自动补默认值
- 或拒绝编译

### 10.2 编译器的责任

中心编译器在生成 IR 前，至少要做以下检查：

- action 是否只使用允许的 opcode
- target 与租户 / 环境边界是否匹配
- risk 是否与 opcode 种类一致
- approval 是否满足风险等级要求
- timeout 和 limits 是否超出策略上限
- 目标 agent 是否具备相应 capability

### 10.3 IR 绑定审批

IR 中建议携带：

- `approval_ref`
- `approved_by`
- `approved_at`
- `expires_at`
- `policy_version`

边缘侧若发现审批上下文缺失或过期，应拒绝执行。

---

## 11. capability 协商

不同节点支持的动作并不完全一致，因此中心编译器不能假定所有节点都支持所有 opcode。

建议边缘 agent 上报 capability，例如：

- `process.list`
- `socket.check`
- `service.restart`
- `file.tail`
- `upgrade.prepare`

中心节点在编译或下发动作前，必须检查：

- 目标节点是否支持所需 opcode
- 所需权限是否在该节点当前策略范围内
- 所需动作是否被该环境禁用

---

## 12. 签名与完整性保护

由于边缘执行器不直接理解原始 DSL，因此真正需要保护的是 IR。

建议中心节点对编译后的 IR 做以下保护：

- 完整性摘要
- 来源签名
- 版本声明
- 过期时间
- 请求主键绑定

边缘侧收到 IR 后至少要检查：

- `ir_version`
- `action_id`
- `request_id`
- `expires_at`
- `signature`
- `tenant_id / environment_id / target`

---

## 13. 执行结果模型

边缘执行器回传的结果不应只是 stdout/stderr 文本，而应是结构化结果。

建议至少包括：

- `action_id`
- `request_id`
- `agent_id`
- `executor_instance_id`
- `started_at`
- `finished_at`
- `final_status`
- `exit_reason`
- `step_records`
- `outputs`
- `resource_usage`
- `audit_refs`

这样中心节点才能稳定做：

- 审计归档
- 风险复盘
- 自动摘要
- AI 辅助解释

---

## 14. AI 在 DSL 中的角色

AI 可以参与 DSL，但只能在中心节点侧。

AI 适合做：

- 根据用户意图生成 Authoring DSL 草稿
- 对 DSL 做可读性优化
- 给出风险提示
- 给出 opcode 替代建议
- 解释为什么某动作不能下发到某类节点

AI 不应直接做：

- 让边缘 agent 动态生成未审查动作
- 绕过审批直接构造执行 IR
- 在边缘侧运行时改写动作计划

---

## 15. 第一阶段实现建议

第一阶段建议分三步走：

### 15.1 Step 1

先不做完整文本 DSL，先定义：

- `ActionPlan IR`
- `Opcode` 白名单
- `RiskLevel`
- `ExecutionLimits`
- `ActionResult`

这样可以先把边缘执行器和中心治理闭环打通。

### 15.2 Step 2

再在中心节点上实现轻量 Authoring DSL：

- 面向动作计划编写
- 面向审批理解
- 面向 AI 生成

### 15.3 Step 3

最后补齐：

- DSL 到 IR 的编译器
- capability-aware 编译
- 风险分析器
- 审批绑定和签名链路

---

## 16. 当前结论

关于“是否需要设计一个 DSL 让 agent 具有可远程执行的能力”，当前结论如下：

1. 需要 DSL，但不应只有一层。
2. YAML / TOML 不适合作为主要作者语言。
3. 边缘 agent 不应直接解释高表达力 DSL。
4. 中心节点应使用更友好的 Authoring DSL。
5. 边缘执行器只应执行编译后的受控 IR。
6. IR 必须建立在 opcode 白名单、风险等级、审批绑定、签名校验和 capability 协商之上。
7. 这样才能同时满足表达力、可治理性和边缘确定性。

一句话概括：

`wp-agent` 应该做的是“中心强表达 DSL + 边缘受控执行 IR”，而不是“把一门强 DSL 直接下沉到边缘节点执行”。

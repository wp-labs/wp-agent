# warp-insight `run.gxl` 子集设计

## 1. 文档目的

本文档定义 `warp-insight` 中 `run.gxl` 的受限子集规格。

目标不是重新定义完整 GXL，而是明确：

- `run.gxl` 允许哪些语法
- `run.gxl` 禁止哪些语法
- `run.gxl` 如何映射到白名单 opcode
- `run.gxl` 的 AST 和编译边界如何设计

本文档是 [`action-dsl.md`](action-dsl.md) 的落地子文档。

相关设计文档：

- [`action-dsl.md`](action-dsl.md)
- [`run-gxl-construct-mapping.md`](run-gxl-construct-mapping.md)
- [`action-schema.md`](action-schema.md)
- [`control-plane.md`](../center/control-plane.md)

---

## 2. 定位

`run.gxl` 的定位必须严格固定为：

- 中心侧作者 DSL
- 用于描述执行功能
- 基于 GXL 风格，但不是完整 GXL
- 编译目标是 `ActionPlan IR`
- 不直接下发到边缘执行器

边缘侧只执行编译产物，不执行 `run.gxl` 源文件。

---

## 3. 设计目标

`run.gxl` 应同时满足以下目标：

- 对已有 GXL 用户足够熟悉
- 对 AI 生成足够稳定
- 对中心编译器足够容易做静态分析
- 对风险分析足够可收敛
- 对边缘执行器足够容易映射到 opcode

---

## 4. 最小语法面

第一版 `run.gxl` 建议只保留以下核心构件：

- `step`
- `when`
- `expect`
- `emit`
- `fail`
- `retry`

### 4.1 `step`

用于声明一个步骤和它的执行结果绑定名。

示例：

```txt
step procs = process.list(name = "nginx", limit = 20)
```

### 4.2 `when`

用于条件分支。

示例：

```txt
when procs.empty {
  emit "nginx process missing"
  fail "service_missing"
}
```

### 4.3 `expect`

用于结果断言。

示例：

```txt
expect port.open == true
```

### 4.4 `emit`

用于把结构化结果或文本摘要标记为输出。

示例：

```txt
emit procs
emit "nginx is running"
```

### 4.5 `fail`

用于显式失败并附带稳定原因码。

示例：

```txt
fail "service_missing"
```

### 4.6 `retry`

用于声明单步骤的有限重试。

示例：

```txt
retry 2 delay 1s on "temporary_error"
```

第一版建议：

- 只允许固定次数重试
- 不允许无限重试
- 不允许复杂重试策略语言

---

## 5. 建议语法骨架

第一版可以采用如下骨架：

```txt
step <ident> = <opcode-call>

when <expr> {
  <statement>*
}

expect <expr>
emit <expr-or-literal>
fail <reason-code>
retry <count> delay <duration> on <reason-code>
```

建议 statement 仅允许：

- `step`
- `when`
- `expect`
- `emit`
- `fail`
- `retry`

---

## 6. 允许的表达式

为了让静态分析保持简单，第一版表达式建议限制为：

- 变量引用：
  `procs`
- 属性访问：
  `procs.empty`
- 基本比较：
  `==`、`!=`、`>`、`>=`、`<`、`<=`
- 布尔组合：
  `&&`、`||`
- 字面量：
  `string`、`number`、`bool`

不建议第一版支持：

- 任意函数调用
- 动态字符串拼接
- 正则脚本片段
- 外部变量注入表达式求值

---

## 7. Opcode 调用形式

`run.gxl` 中的执行动作必须表现为受控 opcode 调用，而不是 shell 调用。

推荐形式：

```txt
step x = process.list(name = "nginx")
step y = file.read_range(path = "/etc/nginx/nginx.conf", offset = 0, length = 4096)
step z = service.status(service = "nginx", manager = "systemd")
```

调用约束：

- 只能调用白名单 opcode
- 参数名必须显式
- 参数值必须可静态检查
- 不允许通过字符串间接决定 opcode

不允许：

```txt
step x = gx.cmd("ps aux")
step y = gx.shell("cat /etc/passwd")
step z = dynamic_call(name = "process.list")
```

---

## 8. 最小 AST 设计

建议中心编译器把 `run.gxl` 解析为最小 AST：

- `RunSpec`
- `StepNode`
- `WhenNode`
- `ExpectNode`
- `EmitNode`
- `FailNode`
- `RetryNode`
- `OpcodeCall`
- `Expr`

建议结构如下：

```text
RunSpec
  statements[]

Statement
  - StepNode
  - WhenNode
  - ExpectNode
  - EmitNode
  - FailNode
  - RetryNode

StepNode
  name
  call: OpcodeCall

OpcodeCall
  op
  args
```

这样设计的目的，是让 `run.gxl` 很容易直接编译到 `ActionPlan IR.steps[]`。

---

## 9. 编译规则

中心编译器处理 `run.gxl` 时，建议按以下顺序：

1. 词法 / 语法解析
2. AST 构建
3. 语义校验
4. opcode 白名单检查
5. 参数静态校验
6. capability 需求提取
7. 风险辅助分析
8. 编译为 `run_ir`

### 9.1 语义校验

建议至少校验：

- `step` 名称唯一
- `when` 引用的对象已定义
- `expect` 引用的对象已定义
- `emit` 引用的对象已定义或是字面量
- `fail` 使用稳定 reason code

### 9.2 capability 提取

编译器应从 `run.gxl` 自动提取 capability 需求，例如：

- `process.list`
- `file.read_range`
- `service.reload`

然后与 `control.wac` 中显式 `require` 交叉校验：

- `run.gxl` 需要的 capability 必须被 `control` 覆盖
- `control` 不能声明与 `run.gxl` 完全无关的危险 capability

---

## 10. 允许的 opcode 映射

第一版建议 `run.gxl` 只允许映射到这些 opcode：

- `process.list`
- `process.stat`
- `socket.check`
- `service.status`
- `file.tail`
- `file.read_range`
- `config.inspect`
- `agent.reload`
- `agent.health_check`
- `service.restart`
- `service.reload`
- `upgrade.prepare`
- `upgrade.verify`

任何不在白名单中的调用，编译阶段直接失败。

---

## 11. 明确禁止的能力

第一版 `run.gxl` 明确禁止：

- `gx.cmd`
- `gx.shell`
- `gx.download`
- `gx.upload`
- `gx.patch_file`
- 动态 include
- 动态模块加载
- 用户自定义函数
- 任意循环
- 运行时外部脚本执行
- 下载后执行
- 任意 OS 命令拼接

这些禁止项的目标很明确：

- 防止 `run.gxl` 演化成通用自动化语言
- 防止 `run.gxl` 绕过 `control.wac`
- 防止中心侧 DSL 直接滑向边缘 shell

---

## 12. 示例

### 12.1 读取配置头部

`read-config-head.run.gxl`

```txt
step part = file.read_range(
  path = "/etc/nginx/nginx.conf",
  offset = 0,
  length = 4096
)

emit part
```

### 12.2 检查服务状态

`diag-nginx-status.run.gxl`

```txt
step svc = service.status(service = "nginx", manager = "systemd")
expect svc.state == "running"
emit svc
```

### 12.3 诊断端口异常

`diag-port-443.run.gxl`

```txt
step port = socket.check(port = 443, protocol = "tcp")

when port.open != true {
  emit "port 443 is not open"
  fail "port_not_open"
}

emit port
```

---

## 13. 与 `control.wac` 的关系

`run.gxl` 只描述“做什么”，不描述“允不允许做”。

因此：

- 文件路径白名单放在 `control.wac`
- 服务名白名单放在 `control.wac`
- 风险等级放在 `control.wac`
- 审批要求放在 `control.wac`
- timeout / limits 放在 `control.wac`

编译前，中心节点必须先完成：

- `control.wac` 合成
- `run.gxl` 解析
- 二者绑定

绑定完成后才能生成最终 `ActionPlan IR`。

---

## 14. 当前结论

`run.gxl` 应被定义为：

- 一个基于 GXL 风格的受限执行 DSL
- 一个只在中心节点侧存在的作者文件
- 一个只能调用白名单 opcode 的受控子集
- 一个必须与 `control.wac` 绑定后才能编译的输入

如果 `run.gxl` 不能保持在这个边界内，就不应被引入 `warp-insight` 体系。

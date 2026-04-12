# wp-agent `run.gxl` 构件映射设计

## 1. 文档目的

本文档把 `run.gxl` 中提出的核心构件：

- `step`
- `when`
- `expect`
- `emit`
- `fail`
- `retry`

映射到当前 `galaxy-flow` / GXL 已有能力和实现形态上，重点回答以下问题：

- 哪些构件可以直接复用当前 GXL 语义
- 哪些构件需要新增语法
- 哪些构件更适合作为编译期语义，而不是运行期语法

本文档建立在以下材料之上：

- [`run-gxl-subset.md`](./run-gxl-subset.md)
- [`action-dsl.md`](./action-dsl.md)
- [`galaxy-flow docs/gxl/syntax.md`](/Users/zuowenjian/devspace/wp-labs/tools/galaxy-flow/docs/gxl/syntax.md)

---

## 2. 当前 GXL 现状

根据 [`syntax.md`](/Users/zuowenjian/devspace/wp-labs/tools/galaxy-flow/docs/gxl/syntax.md)，当前 GXL 已有：

- `if`
- `for`
- `gx.assert`
- `gx.echo`
- `gx.cmd`
- `gx.shell`
- `gx.run`
- 其他 `gx.*` 内置能力

当前没有现成一级语法：

- `step`
- `when`
- `expect`
- `emit`
- `fail`
- `retry`

因此，这 6 个构件不能被误解为“当前 GXL 已支持的现有语法”，它们是 `run.gxl` 需要定义的受限执行层语义。

---

## 3. 总体映射结论

建议把这 6 个构件分成三类：

### 3.1 可复用现有 GXL 语义的构件

- `when`
- `expect`

### 3.2 需要新增受限语法或显式语义的构件

- `step`
- `emit`
- `fail`
- `retry`

### 3.3 实现策略建议

第一版建议：

- 尽量复用 GXL 现有表达式和 `if` 结构
- 不复用 `gx.cmd` / `gx.shell`
- 对 `step / emit / fail / retry` 增加最小新语法或编译层保留字
- 中心编译器把这些构件统一转换为 `ActionPlan IR`

---

## 4. 构件逐项映射

### 4.1 `step`

#### 4.1.1 语义目标

`step` 的作用是：

- 定义一个执行步骤
- 绑定步骤名
- 承载一个受控 opcode 调用
- 为后续 `when / expect / emit` 提供可引用结果

#### 4.1.2 当前 GXL 是否有直接对应物

没有直接对应物。

当前 GXL 有“调用能力”的概念，但没有“步骤名 + 结构化结果绑定”这一层固定模型。

#### 4.1.3 建议实现

建议新增 `step` 语法，原因是：

- 对动作 DSL 可读性最好
- 对编译器提取 `StepNode` 最直接
- 对 AI 生成最稳定

建议形式：

```txt
step svc = service.status(service = "nginx")
```

#### 4.1.4 实现代价

- 需要 parser 新增关键字级支持
- 需要 AST 新增 `StepNode`
- 需要把调用结果绑定到统一步骤上下文

#### 4.1.5 结论

`step` 不建议伪装成现有 GXL 调用形式，建议显式新增。

---

### 4.2 `when`

#### 4.2.1 语义目标

`when` 的作用是：

- 做受限条件分支
- 根据前面步骤结果决定后续行为

#### 4.2.2 当前 GXL 是否有直接对应物

有，最接近的是 `if`。

#### 4.2.3 建议实现

第一版有两种可行方式：

1. 直接沿用 `if`
2. 在 `run.gxl` 中引入 `when` 作为语法糖，并在编译时转换成 `if`

我更建议第一版直接沿用 `if`，原因是：

- 当前 parser 已支持
- 能直接复用表达式系统
- 实现代价最低

也就是说，`run.gxl` 的文档语义可以叫 `when`，但真正落地语法可以先用 `if`。

#### 4.2.4 结论

`when` 建议优先映射到现有 `if`，不急着新增关键字。

---

### 4.3 `expect`

#### 4.3.1 语义目标

`expect` 的作用是：

- 对步骤结果做断言
- 失败时产生稳定失败语义

#### 4.3.2 当前 GXL 是否有直接对应物

有，最接近的是 `gx.assert`。

参考：

- [`assert.md`](/Users/zuowenjian/devspace/wp-labs/tools/galaxy-flow/docs/gxl/inner/assert.md)

#### 4.3.3 建议实现

第一版可以不新增语法，而是把：

```txt
expect svc.state == "running"
```

在编译期转换成内部断言节点，底层语义等价于受限版 `gx.assert`。

如果需要最小实现代价，甚至可以先在作者侧直接写成 `gx.assert` 风格，再由编译器收敛。

但从 DSL 可读性看，`expect` 仍然是更好的作者语义。

#### 4.3.4 结论

`expect` 建议语义复用 `gx.assert`，可先做编译期糖，不一定先做 parser 新关键字。

---

### 4.4 `emit`

#### 4.4.1 语义目标

`emit` 的作用是：

- 标记哪些内容进入最终 `ActionResult`
- 输出结构化结果或摘要

#### 4.4.2 当前 GXL 是否有直接对应物

没有直接对应物。

`gx.echo` 只是 stdout 输出文本，不等于“纳入结构化结果”。

参考：

- [`echo.md`](/Users/zuowenjian/devspace/wp-labs/tools/galaxy-flow/docs/gxl/inner/echo.md)

#### 4.4.3 建议实现

建议新增 `emit` 语义。

可选实现策略：

- 作为 parser 新关键字
- 或在编译器里把某种受限 `gx.echo` 约定解释成 `emit`

但从长期设计看，最好单独有 `EmitNode`，不要复用 `gx.echo`。

#### 4.4.4 结论

`emit` 建议显式新增，不建议复用 `gx.echo`。

---

### 4.5 `fail`

#### 4.5.1 语义目标

`fail` 的作用是：

- 主动终止动作
- 附带稳定失败原因码
- 为控制平面和审计提供统一失败语义

#### 4.5.2 当前 GXL 是否有直接对应物

没有直接对应物。

当前更接近的是：

- `gx.assert` 失败
- 某个内置能力执行失败

但这两者都不是“显式失败原因码”。

#### 4.5.3 建议实现

建议新增 `fail` 语义，并要求：

- reason code 必须是稳定字符串
- 编译器收敛到统一 `FailNode`
- 最终映射到 `ActionResult.exit_reason`

#### 4.5.4 结论

`fail` 建议显式新增，不能依赖现有错误分支隐式表达。

---

### 4.6 `retry`

#### 4.6.1 语义目标

`retry` 的作用是：

- 对单步骤提供有限重试
- 由动作作者声明可容忍的暂时失败

#### 4.6.2 当前 GXL 是否有直接对应物

没有直接对应物。

#### 4.6.3 建议实现

建议第一版只支持极简形式：

```txt
retry 2 delay 1s on "temporary_error"
```

并且：

- 只作用于当前步骤或最近一步
- 不允许嵌套重试策略
- 不允许无限重试

编译器可把它编译成步骤元数据，而不是复杂控制流。

#### 4.6.4 结论

`retry` 建议先做编译期元数据语义，不急着做完整流程控制语法。

---

## 5. 推荐实现优先级

为了降低第一版实现成本，建议按以下优先级落地：

### 5.1 第一优先级

- `step`
- `if` 复用为 `when`
- `expect` 语义映射到断言

### 5.2 第二优先级

- `emit`
- `fail`

### 5.3 第三优先级

- `retry`

也就是说，第一版完全可以先把：

- `step`
- `if`
- `expect`
- `emit`

跑通，再补 `fail` 和 `retry` 的增强语义。

---

## 6. 推荐的第一版折中方案

如果目标是尽快落地，同时尽量复用当前 GXL，建议第一版采用以下折中：

- `step`
  新增
- `when`
  先直接使用现有 `if`
- `expect`
  编译期映射到断言
- `emit`
  新增
- `fail`
  新增
- `retry`
  先做编译期元数据

这样做的好处是：

- 复用现有 parser 和表达式系统
- 减少新关键字数量
- 不引入 `gx.cmd` / `gx.shell`
- 保持 `run.gxl` 的受控边界

---

## 7. 当前结论

这 6 个构件不应被理解为“当前 GXL 已有内建语法”。

更准确的结论是：

- `when` 最适合复用现有 `if`
- `expect` 最适合复用现有断言语义
- `step / emit / fail / retry` 需要 `run.gxl` 明确补出自己的受限执行层语义

因此，`run.gxl` 最合理的方向不是“直接复用完整 GXL”，而是“基于当前 GXL 能力做受控扩展”。 

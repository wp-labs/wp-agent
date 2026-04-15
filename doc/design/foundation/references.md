# warp-insight 设计参考

## 1. 文档目的

本文档记录 `warp-insight` 在目标定义、架构、安全模型、动作 DSL 与受控执行方面可借鉴的业界产品与设计模式。

这里的原则不是“照搬某个现成产品”，而是：

- 明确每个参考对象解决了哪一类问题
- 明确哪些点值得借鉴
- 明确哪些点不应直接复制

---

## 2. 总体结论

`warp-insight` 当前设计并没有一个可以完整照抄的单一参考对象，更合理的做法是组合借鉴以下四类模式：

- 动作编排与审批执行：
  参考 AWS Systems Manager Automation / Change Manager
- 安全控制与执行对象分离：
  参考 Kubernetes Admission Control + Kyverno / OPA
- 权限边界与作用域管理：
  参考 HashiCorp Boundary
- 只读诊断与受控查询：
  参考 Fleet / osquery

一句话概括：

`run` 的动作编排与执行面，可以参考 AWS SSM / Rundeck；`control` 的治理与策略面，可以参考 Kubernetes Admission + Kyverno；权限与会话边界可以参考 Boundary；只读诊断 opcode 设计可以参考 Fleet/osquery。

---

## 3. AWS Systems Manager Automation / Change Manager

### 3.1 为什么值得参考

AWS SSM 是目前最接近“动作定义 + 审批 + 执行控制”的参考之一。

它的几个关键点值得借鉴：

- 用文档或 runbook 描述动作步骤
- 支持审批步骤
- 支持分阶段执行
- 支持并发控制和错误阈值
- 支持对目标节点执行标准化动作

### 3.2 对 warp-insight 的借鉴点

适合借鉴：

- 把远程动作建模成 runbook / action plan，而不是任意 shell
- 审批和执行计划要绑定，而不是事后补审计
- 对高风险动作引入显式的执行窗口、批次和门槛
- 动作执行结果应标准化，而不是只返回文本

不应直接照搬：

- 直接继承其 DSL 或 AWS 资源模型
- 把所有动作都做成强工作流编排，导致边缘执行器过重

### 3.3 参考链接

- https://docs.aws.amazon.com/systems-manager/latest/userguide/systems-manager-automation.html
- https://docs.aws.amazon.com/systems-manager/latest/userguide/automation-documents.html
- https://docs.aws.amazon.com/systems-manager/latest/userguide/running-automations-require-approvals.html

---

## 4. Kubernetes Admission Control + Kyverno / OPA

### 4.1 为什么值得参考

这组模式最适合参考“安全控制”和“执行对象”分离的设计思想。

Kubernetes 的核心做法是：

- 用户先提交对象
- admission controller 决定是否接受、修改或拒绝
- policy 作为独立治理对象存在

Kyverno / OPA 则进一步强化了：

- policy 独立维护
- policy 与被执行对象分离
- 可以按环境、标签、命名空间、资源种类做策略约束

### 4.2 对 warp-insight 的借鉴点

适合借鉴：

- `run` 与 `control` 分离
- `control` 不由动作作者单独决定，而由治理层合成
- 平台基线策略、环境策略、模板策略分层叠加
- 动作提交后，中心节点先做校验、约束、审批绑定，再编译成 IR

不应直接照搬：

- 不需要引入完整 Kubernetes 风格资源 API
- 不需要把边缘 agent 变成通用 admission engine

### 4.3 参考链接

- https://kubernetes.io/docs/reference/access-authn-authz/admission-controllers/
- https://kubernetes.io/docs/reference/access-authn-authz/extensible-admission-controllers/
- https://kyverno.io/docs/introduction/how-kyverno-works/
- https://www.openpolicyagent.org/docs/latest/

---

## 5. Rundeck

### 5.1 为什么值得参考

Rundeck 是“作业定义”和“节点访问 / ACL / 执行权限”分离的典型产品。

它特别适合参考以下点：

- 作业模板化
- 节点范围控制
- ACL 独立管理
- 运维动作的组织和交付

### 5.2 对 warp-insight 的借鉴点

适合借鉴：

- 动作模板与执行权限分离
- 节点范围和角色控制要先于动作执行
- 运维动作需要稳定的作业模型，而不是 ad-hoc shell

不应直接照搬：

- 不应把边缘执行器做成传统运维作业平台
- 不应把 job engine 直接嵌入 agent 热路径

### 5.3 参考链接

- https://docs.rundeck.com/docs/learning/getting-started/acl-overview.html
- https://docs.rundeck.com/docs/learning/howto/acl_basic_examples.html

---

## 6. HashiCorp Boundary

### 6.1 为什么值得参考

Boundary 不解决动作 DSL，但非常适合参考：

- 作用域
- 会话
- 身份与授权分离
- 最小权限访问

这和 `warp-insight` 当前的租户、环境、目标节点、审批上下文、过期时间这些控制字段高度相关。

### 6.2 对 warp-insight 的借鉴点

适合借鉴：

- `tenant / environment / target` 的分层作用域模型
- 身份不等于授权
- 高风险动作应绑定短时有效的执行上下文
- 执行权限应是临时的、会话化的，而不是长期持有

不应直接照搬：

- `warp-insight` 不需要把自己做成访问代理产品
- 不需要引入完整 Boundary 控制面语义

### 6.3 参考链接

- https://developer.hashicorp.com/boundary/docs/domain-model/scopes
- https://developer.hashicorp.com/boundary/docs/concepts/iam
- https://developer.hashicorp.com/boundary/docs/concepts/domain-model/sessions

---

## 7. Fleet / osquery

### 7.1 为什么值得参考

Fleet / osquery 更接近 `R0` 诊断类和读取类能力的产品边界。

它们的核心思路是：

- 不开放任意 shell
- 提供受控的系统观测与查询能力
- 用角色和权限控制谁可以对哪些节点做哪些查询

### 7.2 对 warp-insight 的借鉴点

适合借鉴：

- 只读诊断能力优先做成结构化 opcode，而不是 shell
- `process.list`、`process.stat`、`socket.check`、`file.tail` 这类能力要优先结构化
- 查询类动作的输出应结构化、可索引、可审计

不应直接照搬：

- `warp-insight` 不只是终端查询系统
- 还要覆盖服务控制、升级辅助和 agent 控制类动作

### 7.3 参考链接

- https://fleetdm.com/guides/role-based-access
- https://osquery.readthedocs.io/en/stable/deployment/remote/

---

## 8. 对 warp-insight 的综合借鉴结论

建议把这些参考对象映射到 `warp-insight` 的不同设计部分：

- `target.md`：
  借鉴“边缘轻执行 + 中心治理”的总体思路，但不照搬某一产品
- `architecture.md`：
  借鉴 AWS SSM / Rundeck 的动作执行与编排骨架
- `security-model.md`：
  借鉴 Boundary 的作用域、最小权限、短时授权思路
- `action-dsl.md`：
  借鉴 Kubernetes Admission + Kyverno 的“对象与策略分离”，以及 Fleet/osquery 的“结构化诊断动作”

因此更准确的说法是：

`warp-insight` 不应被定义为某个现有产品的封装，而应有意识地组合多种成熟模式，形成更适合目标环境的统一设计。

---

## 9. 当前建议

当前阶段建议把参考基线明确为：

- 动作模型：
  AWS SSM Automation / Rundeck
- 安全控制分层：
  Kubernetes Admission + Kyverno / OPA
- 权限与作用域：
  HashiCorp Boundary
- 只读诊断 opcode：
  Fleet / osquery

如果后续继续细化协议和对象模型，应继续按“借鉴点”展开，而不是按“产品名”展开。

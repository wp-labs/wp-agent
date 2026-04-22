# warp-insight Host 责任关系外部系统同步设计

## 1. 文档目的

本文档定义 `warp-insight` 中心侧如何从外部系统同步主机责任关系。

这里的“外部系统”主要指：

- CMDB
- LDAP / IAM / HR 组织目录
- Oncall / 值班平台

目标是固定：

- 哪些数据应作为主数据输入
- 哪些对象由 `warp-insight` 自主维护
- 外部标识与内部主键如何映射
- 同步、幂等、冲突与失效如何处理

相关文档：

- [`host-responsibility-and-maintainer-model.md`](host-responsibility-and-maintainer-model.md)
- [`host-inventory-and-runtime-state.md`](host-inventory-and-runtime-state.md)
- [`../control-center-storage-schema.md`](../control-center-storage-schema.md)

---

## 2. 核心结论

第一版建议固定以下原则：

- `warp-insight` 不直接把外部系统当查询数据库使用
- 外部系统数据应先同步到中心侧规范化对象
- 内部统一使用稳定主键，不直接把外部字符串当外键
- 不同外部系统负责不同主数据域
- 冲突收敛必须有明确优先级和来源审计

一句话说：

- 外部系统提供事实
- `warp-insight` 负责归一、映射、继承和最终查询视图

---

## 3. 外部系统职责边界

建议按主数据域划分来源：

### 3.1 CMDB

负责提供：

- 主机基础归属
- 主机与应用/集群/业务组关系
- 默认 `owner` / `maintainer` 团队
- 环境、业务域、服务树关系

不建议由 CMDB 负责：

- 实时值班人
- 临时告警接收人
- 漏洞整改过程状态

### 3.2 LDAP / IAM / HR

负责提供：

- 用户身份
- 团队/组织结构
- 用户与团队成员关系
- 人员状态，如在职、离职、禁用

不建议由 LDAP / HR 直接负责：

- 主机责任分配本身
- 主机与业务系统归属

### 3.3 Oncall 平台

负责提供：

- 当前值班组
- 当前值班人
- 值班升级链
- 告警接收路由

不建议由 Oncall 平台负责：

- 最终业务归属
- 长期维护归属

---

## 4. 内部主数据模型与外部来源的关系

### 4.1 内部对象

内部仍以这些对象为准：

- `Subject`
- `HostGroup`
- `HostGroupMembership`
- `ResponsibilityAssignment`

### 4.2 外部来源对象

第一版建议补两类映射对象：

- `ExternalIdentityLink`
- `ExternalSyncCursor`

如有必要，可补第三类：

- `ExternalImportBatch`

---

## 5. `ExternalIdentityLink`

用途：

- 建立内部对象与外部对象的稳定映射

建议结构：

```text
ExternalIdentityLink {
  link_id
  tenant_id
  system_type
  object_type
  external_id
  external_key?
  internal_kind
  internal_id
  status
  first_seen_at
  last_seen_at
  last_synced_at
  metadata?
}
```

`system_type` 建议取值：

- `cmdb`
- `ldap`
- `iam`
- `hr`
- `oncall`

`object_type` 示例：

- `host`
- `host_group`
- `user`
- `team`
- `rotation`

`internal_kind` 示例：

- `subject`
- `host_group`
- `host`

设计原则：

- 外部 ID 与内部 ID 必须解耦
- 一个外部对象可映射到一个内部对象
- 同一内部对象可同时持有多个外部来源映射

---

## 6. `ExternalSyncCursor`

用途：

- 跟踪各同步源的增量进度

建议结构：

```text
ExternalSyncCursor {
  cursor_id
  tenant_id
  system_type
  scope_key
  cursor_value?
  full_sync_token?
  last_success_at?
  last_attempt_at?
  last_error?
  updated_at
}
```

说明：

- `scope_key` 可表示某个租户、环境、组织单元或接口分片
- `cursor_value` 可保存时间戳、水位 ID、分页 token
- 增量同步与全量校准都应可表达

---

## 7. 同步对象映射策略

### 7.1 主机归属来自 CMDB

建议映射：

- `cmdb host` -> `HostInventory`
- `cmdb app/service/cluster` -> `HostGroup`
- `cmdb host-group relation` -> `HostGroupMembership`
- `cmdb owner_team / maintainer_team` -> `ResponsibilityAssignment`

结论：

- CMDB 是主机归属与默认责任的重要来源
- 但同步后应落到内部统一模型，而不是查询时实时回查 CMDB

### 7.2 用户与团队来自 LDAP / IAM / HR

建议映射：

- `ldap user` -> `Subject(subject_type = user)`
- `ldap team/org group` -> `Subject(subject_type = team)`
- `team membership` 先进入 `subject.metadata` 或后续独立关系表

说明：

- 第一版若只关心“责任主体是谁”，团队成员关系不必一次做太深
- 但至少要能判断某个 `subject` 是否有效、是否停用

### 7.3 值班信息来自 Oncall

建议映射：

- `oncall rotation` -> `Subject(subject_type = team)` 或独立 `team` 映射
- `current oncall user/team` -> `ResponsibilityAssignment(role = oncall)`

说明：

- `oncall` 是动态责任关系
- 其生效时间较短，通常应带明确 `valid_from` / `valid_to`
- 这类 assignment 可以频繁刷新，但不应覆盖 `owner` / `maintainer`

---

## 8. 来源优先级与冲突收敛

不同系统可能对同一主机给出不同责任关系。

第一版建议明确来源优先级：

1. 手工治理配置 `manual`
2. CMDB 同步 `cmdb_sync`
3. Oncall 同步 `oncall_sync`
4. 规则推断 `rule_derived`

其中：

- `owner` / `maintainer` 主要由 `manual` 与 `cmdb_sync` 竞争
- `oncall` 主要由 `oncall_sync` 提供
- `security_owner` 可以先复用 `owner` 或 `maintainer`，后续再独立维护

### 8.1 冲突处理原则

- 不同来源的 assignment 不能直接互相覆盖原始记录
- 应保留来源字段与生效时间
- “最终展示值”通过优先级和继承规则计算

### 8.2 禁止事项

第一版应避免：

- 同步任务直接删除所有旧记录后全量重建
- 用外部用户名、邮箱、团队名直接当内部主键
- 查询时每次实时去外部系统拼装责任视图

---

## 9. 同步流程建议

第一版建议采用“全量校准 + 增量刷新”的方式。

### 9.1 周期性全量校准

适用于：

- 团队组织结构
- 主机与组关系
- 默认责任关系

流程：

1. 拉取外部对象分页数据
2. 建立或更新 `ExternalIdentityLink`
3. 归一到内部对象
4. 生成新的 assignment / membership 生效段
5. 对缺失对象做失活，而不是硬删除

### 9.2 高频增量刷新

适用于：

- Oncall 值班信息
- CMDB 的近期责任变更事件

流程：

1. 读取 `ExternalSyncCursor`
2. 拉取增量事件
3. 按外部对象映射到内部对象
4. 幂等 upsert 到 assignment
5. 提交 cursor

### 9.3 幂等原则

同步写入建议依赖以下天然幂等键：

- `system_type + object_type + external_id`
- `tenant_id + target_type + target_id + role + subject_id + valid_from + source`

说明：

- 不要依赖“本次任务生成的随机 ID”判断重复
- 应让同一外部事实多次同步得到相同内部结果

---

## 10. 失效与历史策略

同步对象必须支持失效，而不是只有新增。

### 10.1 主体失效

例如：

- 员工离职
- 团队解散
- 值班组停用

处理建议：

- `Subject.status = inactive`
- 保留历史 assignment，不做物理删除

### 10.2 关系失效

例如：

- 主机移出某个集群
- 维护团队切换
- 当前值班窗口结束

处理建议：

- 写 `valid_to`
- 更新有效责任视图
- 历史关系可继续审计

---

## 11. PostgreSQL 表建议

### 11.1 `external_identity_link`

```sql
create table external_identity_link (
  link_id uuid primary key,
  tenant_id uuid not null,
  system_type text not null,
  object_type text not null,
  external_id text not null,
  external_key text,
  internal_kind text not null,
  internal_id uuid not null,
  status text not null default 'active',
  first_seen_at timestamptz not null,
  last_seen_at timestamptz not null,
  last_synced_at timestamptz not null,
  metadata jsonb,
  unique (tenant_id, system_type, object_type, external_id)
);
```

建议索引：

- `(tenant_id, internal_kind, internal_id)`
- `(tenant_id, system_type, status)`

### 11.2 `external_sync_cursor`

```sql
create table external_sync_cursor (
  cursor_id uuid primary key,
  tenant_id uuid not null,
  system_type text not null,
  scope_key text not null,
  cursor_value text,
  full_sync_token text,
  last_success_at timestamptz,
  last_attempt_at timestamptz,
  last_error text,
  updated_at timestamptz not null,
  unique (tenant_id, system_type, scope_key)
);
```

---

## 12. 与责任关系模型的结合方式

建议责任同步遵循以下原则：

- `Subject`、`HostGroup`、`HostGroupMembership`、`ResponsibilityAssignment` 仍是中心 source of truth
- 外部系统只提供导入来源，不直接成为中心查询依赖
- 有效责任视图从内部表计算，不从外部接口实时拼装

这意味着：

- 断开外部系统短时可用性后，中心查询仍可工作
- 可对同步数据做审计、回滚和版本比较
- 不同来源系统可以统一归一到同一责任模型

---

## 13. 第一版落地建议

当前建议固定为：

- 先接 `CMDB + LDAP/IAM + Oncall` 三类源
- 先实现 `ExternalIdentityLink` 和 `ExternalSyncCursor`
- `owner` / `maintainer` 先由 `manual + cmdb_sync` 驱动
- `oncall` 先由 `oncall_sync` 驱动
- 查询只读内部归一后的责任视图

第一版不要一开始就做得过重：

- 不必先做双向写回外部系统
- 不必先做复杂主数据总线
- 不必先做所有来源之间的自动置信度融合

先把：

- 映射
- 同步
- 幂等
- 失效
- 冲突优先级

五件事固定住。

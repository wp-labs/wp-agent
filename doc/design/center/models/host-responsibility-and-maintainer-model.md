# warp-insight Host 责任者与维护者关系模型设计

## 1. 文档目的

本文档定义 `warp-insight` 中心侧针对主机责任归属的关系模型。

这里的目标是固定：

- 多台主机与责任主体之间的关系表达
- `owner`、`maintainer`、`oncall` 等角色边界
- 主机级与主机组级的责任继承规则
- 适合单机优先落地的 PostgreSQL 存储模型

相关文档：

- [`host-inventory-and-runtime-state.md`](host-inventory-and-runtime-state.md)
- [`host-inventory-and-runtime-state-storage.md`](host-inventory-and-runtime-state-storage.md)
- [`../control-center-storage-schema.md`](../control-center-storage-schema.md)
- [`host-responsibility-sync-from-external-systems.md`](host-responsibility-sync-from-external-systems.md)

---

## 2. 核心结论

第一版固定以下结论：

- 责任者与维护者不能直接塞进 `HostInventory` 的单字段
- 责任关系必须抽成独立对象
- 责任主体应优先绑定到 `team`，个人作为补充联系人
- 责任分配应支持 `host` 与 `host_group` 两级绑定
- 查询展示可提供“有效责任视图”，但源数据应保留分配与继承链路

一句话说：

- `HostInventory` 回答“这台主机是什么”
- `ResponsibilityAssignment` 回答“谁对这台主机负责”

---

## 3. 为什么不能直接写进 HostInventory

如果把责任者简单写成：

```text
HostInventory {
  owner = "team-a"
  maintainer = "zhangsan"
}
```

会出现明显问题：

- 一台主机往往有多个角色，不是单值字段
- 一个人或团队可能负责很多台主机，是多对多关系
- 责任常按应用、集群、环境批量继承，不是逐台配置
- 人员变更频繁，资产信息变更频率低，两者生命周期不同
- 需要保留责任历史与生效时间，单字段难以审计

因此必须明确：

- 主机资产是目录对象
- 责任归属是治理关系对象

两者相关，但不能混成一个表。

---

## 4. 对象模型

### 4.1 `Subject`

表示“谁”。

建议结构：

```text
Subject {
  subject_id
  tenant_id
  subject_type
  name
  external_ref?
  status
  metadata?
  created_at
  updated_at
}
```

`subject_type` 建议取值：

- `user`
- `team`
- `service_account`
- `vendor`

建议原则：

- 最终责任优先绑定 `team`
- `user` 更适合做主联系人、值班人、升级处理联系人

### 4.2 `HostGroup`

表示一批主机的归属边界。

建议结构：

```text
HostGroup {
  host_group_id
  tenant_id
  environment_id
  name
  group_type
  parent_group_id?
  description?
  created_at
  updated_at
}
```

`group_type` 可选值：

- `application`
- `business`
- `cluster`
- `environment`
- `ops_scope`

说明：

- 大多数责任不是逐台维护，而是按服务、业务、环境或集群继承
- `HostGroup` 是责任归属和批量治理的中间层，不应与运行时状态混用

### 4.3 `HostGroupMembership`

表示主机属于哪些组。

建议结构：

```text
HostGroupMembership {
  host_id
  host_group_id
  membership_role?
  valid_from
  valid_to?
  source
  created_at
  updated_at
}
```

说明：

- 一台主机可以属于多个组
- `membership_role` 可用于区分主集群、副集群、业务域等成员语义
- `source` 可标识 `manual`、`cmdb_sync`、`rule_derived`

### 4.4 `ResponsibilityAssignment`

这是核心关系对象。

建议结构：

```text
ResponsibilityAssignment {
  assignment_id
  tenant_id
  target_type
  target_id
  role
  subject_id
  is_primary
  priority
  source
  valid_from
  valid_to?
  remark?
  created_at
  updated_at
}
```

`target_type` 建议取值：

- `host`
- `host_group`

`role` 建议第一版固定：

- `owner`
- `maintainer`
- `operator`
- `oncall`
- `security_owner`

字段语义：

- `is_primary`
  - 同一角色下的主责任方
- `priority`
  - 多条候选责任关系的排序
- `source`
  - `manual` / `cmdb_sync` / `hr_sync` / `rule_derived`

---

## 5. 角色边界

第一版建议明确以下角色语义：

### 5.1 `owner`

回答：

- 这台主机最终归哪个团队负责

特点：

- 倾向绑定团队
- 用于归属统计、治理问责、变更审批归属

### 5.2 `maintainer`

回答：

- 谁负责日常维护、版本升级、配置变更和故障处理

特点：

- 可能与 `owner` 相同
- 也可能是共享平台团队或 SRE 团队

### 5.3 `oncall`

回答：

- 告警先通知谁

特点：

- 可以绑定团队值班组，也可以绑定个人
- 这是告警路由角色，不等同最终责任

### 5.4 `security_owner`

回答：

- 漏洞、基线、补丁整改由谁推进

特点：

- 在一些组织里会与 `maintainer` 相同
- 但不应在模型上强行合并

---

## 6. 继承与覆盖规则

责任关系建议采用以下优先级：

1. `host` 级显式分配
2. `host_group` 级分配
3. 环境或租户默认分配

解释：

- 单台主机的特殊归属可覆盖组归属
- 大部分主机通过组继承责任，降低维护成本
- 无显式配置时，允许落到默认团队，避免责任悬空

### 6.1 继承原则

- 同一角色可有多条 assignment
- `is_primary = true` 的 assignment 优先
- 若主标记冲突，则按 `priority` 排序
- 若仍冲突，则按最近 `updated_at` 决定最终展示值

### 6.2 禁止事项

第一版应避免：

- 把 `owner` 和 `maintainer` 强行合并
- 只支持单个人，不支持团队
- 把责任关系直接挂在 `HostRuntimeState`
- 在 UI 视图层跳过继承规则，直接读取任意一条原始 assignment

---

## 7. 查询视图

为了方便 UI 与 API 查询，建议增加一个派生视图：

```text
EffectiveHostResponsibility {
  host_id
  owner_subject_id?
  maintainer_subject_id?
  oncall_subject_id?
  security_owner_subject_id?
  owner_resolved_from?
  maintainer_resolved_from?
  oncall_resolved_from?
  security_owner_resolved_from?
  resolved_at
}
```

字段说明：

- `*_subject_id`
  - 该主机当前有效责任主体
- `*_resolved_from`
  - 标识责任来自 `host`、`host_group` 或默认层

说明：

- 这是查询加速层，不是 source of truth
- 原始分配关系仍应从 `ResponsibilityAssignment` 和 `HostGroupMembership` 计算

---

## 8. PostgreSQL 存储设计

### 8.1 `subject`

建议表结构：

```sql
create table subject (
  subject_id uuid primary key,
  tenant_id uuid not null,
  subject_type text not null,
  name text not null,
  external_ref text,
  status text not null default 'active',
  metadata jsonb,
  created_at timestamptz not null,
  updated_at timestamptz not null,
  check (subject_type in ('user', 'team', 'service_account', 'vendor'))
);
```

建议索引：

- `(tenant_id, subject_type, name)`
- `(tenant_id, external_ref)`

### 8.2 `host_group`

建议表结构：

```sql
create table host_group (
  host_group_id uuid primary key,
  tenant_id uuid not null,
  environment_id uuid not null,
  name text not null,
  group_type text not null,
  parent_group_id uuid references host_group(host_group_id),
  description text,
  created_at timestamptz not null,
  updated_at timestamptz not null
);
```

建议唯一约束：

- `(tenant_id, environment_id, name)`

建议索引：

- `(tenant_id, environment_id, group_type)`
- `(parent_group_id)`

### 8.3 `host_group_membership`

建议表结构：

```sql
create table host_group_membership (
  host_id uuid not null,
  host_group_id uuid not null references host_group(host_group_id),
  membership_role text,
  valid_from timestamptz not null,
  valid_to timestamptz,
  source text not null,
  created_at timestamptz not null,
  updated_at timestamptz not null,
  primary key (host_id, host_group_id, valid_from)
);
```

建议索引：

- `(host_id, valid_to)`
- `(host_group_id, valid_to)`

### 8.4 `responsibility_assignment`

建议表结构：

```sql
create table responsibility_assignment (
  assignment_id uuid primary key,
  tenant_id uuid not null,
  target_type text not null,
  target_id uuid not null,
  role text not null,
  subject_id uuid not null references subject(subject_id),
  is_primary boolean not null default false,
  priority integer not null default 100,
  source text not null,
  valid_from timestamptz not null,
  valid_to timestamptz,
  remark text,
  created_at timestamptz not null,
  updated_at timestamptz not null,
  check (target_type in ('host', 'host_group')),
  check (role in ('owner', 'maintainer', 'operator', 'oncall', 'security_owner'))
);
```

建议索引：

- `(tenant_id, target_type, target_id, role, valid_to)`
- `(subject_id, role, valid_to)`
- `(tenant_id, role, valid_to)`

### 8.5 约束建议

建议增加以下约束语义：

- 同一目标、同一角色、同一主体允许存在多段历史，不允许生效时间重叠
- 同一目标、同一角色允许多条记录，但 `is_primary = true` 应尽量控制为一条有效记录
- `valid_to` 为空表示当前生效

这类“时间段不重叠”约束在第一版可以先放到应用层完成，后续再逐步下沉为数据库约束。

---

## 9. 与现有中心对象的关系

### 9.1 与 `HostInventory`

- `HostInventory` 仍然只保存主机目录事实
- 不直接内嵌责任关系字段

### 9.2 与 `HostRuntimeState`

- `HostRuntimeState` 只保存动态运行态
- 不承载责任、维护、值班归属

### 9.3 与漏洞和告警治理

责任关系应作为其他治理对象的关联输入：

- 漏洞 finding 可通过 `host_id -> effective responsibility` 找到 `security_owner`
- 告警事件可通过 `host_id -> effective responsibility` 找到 `oncall`
- 变更审批可通过 `host_id -> effective responsibility` 找到 `owner` 或 `maintainer`

---

## 10. 第一版落地建议

当前建议固定为：

- 责任关系独立建模，不并入 `HostInventory`
- 优先支持 `team` 与 `user` 两类主体
- 先支持 `host` 与 `host_group` 两级目标
- 用 PostgreSQL 保存主体、分组、成员关系、责任分配
- UI 和查询层通过“有效责任视图”展示最终结果

第一版不要一开始就做得过重：

- 不必先上复杂图数据库
- 不必先做过深的多层组织树
- 不必先把 CMDB、HR、Oncall 平台全部强耦合接进来

先把：

- 责任主体
- 主机组
- 分配关系
- 继承规则

四件事固定住，后续再接外部系统同步。

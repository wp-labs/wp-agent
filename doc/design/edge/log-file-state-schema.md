# wp-agent 文件日志输入状态 Schema 草案

## 1. 文档目的

本文档定义 `wp-agentd` 文件日志输入的本地状态 schema。

这里的状态专指：

- `file input` 的 checkpoint state
- `file identity` 到 `checkpoint offset` 的持久化映射
- crash 恢复、rotate、truncate 判断所需的最小本地状态

本文档不讨论：

- `execution_queue` / `running` / `reporting` 这类远程执行状态
- `wp-agent-exec` workdir 状态
- parser / multiline 的运行时内存对象细节

相关文档：

- [`agentd-state-schema.md`](agentd-state-schema.md)
- [`agentd-state-and-boundaries.md`](agentd-state-and-boundaries.md)
- [`../telemetry/log-file-input-spec.md`](../telemetry/log-file-input-spec.md)
- [`../foundation/glossary.md`](../foundation/glossary.md)

---

## 2. 核心结论

第一版固定以下结论：

- 文件日志输入状态应独立于 execution 状态树
- 文件日志输入状态的唯一持久化目标是支撑 checkpoint 恢复，而不是复刻完整 runtime 内存对象
- `checkpoint offset` 只能在越过 `commit point` 后推进
- `file_id` 是 `file identity` 的持久化表示，不等于当前 `path`
- 第一版每个 `file input` 维护一份 `checkpoints.json`

---

## 3. 目录位置

建议目录结构：

```text
<agent_root>/
  state/
    logs/
      file_inputs/
        <input_id>/
          checkpoints.json
```

说明：

- `execution_queue` 等控制状态仍保留在 `state/` 根下
- 文件日志输入状态单独放入 `state/logs/`，避免与 execution 状态混写

---

## 4. 状态对象

### 4.1 `checkpoints.json`

```text
FileLogCheckpointState {
  schema_version
  input_id
  updated_at
  files[]
}
```

字段说明：

- `schema_version`
  第一版固定为 `v1`
- `input_id`
  对应 `logs.file_inputs[].id`
- `updated_at`
  本次状态文件成功落盘时间
- `files[]`
  当前仍保留 checkpoint 的文件状态集合

### 4.2 `TrackedFileCheckpoint`

```text
TrackedFileCheckpoint {
  file_id
  path
  device_id?
  inode?
  fingerprint?
  checkpoint_offset
  last_size?
  last_read_at?
  last_commit_point_at?
  rotated_from_path?
}
```

字段说明：

- `file_id`
  `file identity` 的持久化字段
- `path`
  最近一次确认的当前路径
- `device_id` / `inode`
  Unix-like 平台上的首选身份来源
- `fingerprint`
  inode 不可靠或需额外校验时使用
- `checkpoint_offset`
  最近一次已提交的文件读取进度
- `last_size`
  最近一次观测到的文件大小
- `last_read_at`
  最近一次 reader 实际读取到内容的时间
- `last_commit_point_at`
  最近一次越过 `commit point` 的时间
- `rotated_from_path`
  当前文件由哪个旧路径 rotate 而来；无则为空

---

## 5. `file_id` 与 `file identity`

第一版建议：

- `file_id` 不是自由字符串语义名，而是 `file identity` 的稳定持久化结果
- Linux / Unix 优先基于 `device_id + inode`
- `compare_filename = true` 时同时校验 `path`
- 当 inode 不可靠时，退化为 `canonical_path + fingerprint`

工程约束：

- `path` 改变不等于 `file identity` 改变
- `file identity` 改变时必须重新判断 rotate / truncate / 新文件接管

---

## 6. checkpoint 推进规则

### 6.1 `read offset`

`read offset` 属于 runtime 内存态，不要求直接持久化到 state file。

第一版建议：

- 运行中可维护 `read offset`
- 但持久化文件中只保存 `checkpoint_offset`

### 6.2 `commit point`

第一版建议把以下条件作为 `commit point`：

- record 已完成必要 parse / normalize / resource binding
- record 已成功进入本地 telemetry buffer
- 若启用了 durable spool，则必须成功进入 spool

只有在越过 `commit point` 后，才允许推进 `checkpoint_offset`。

### 6.3 推进步骤

建议固定为：

1. `file reader` 读取新增内容
2. 形成 record
3. record 进入本地 telemetry buffer / spool
4. 达到 `commit point`
5. 更新 `checkpoint_offset`
6. 原子落盘 `checkpoints.json`

---

## 7. rotate / truncate 规则

### 7.1 rotate

当发生 rename-rotate 时：

- 旧文件的 `file identity` 保持不变
- 原路径上的新文件形成新的 `file identity`
- 旧文件可在 `rotate_wait_ms` 窗口内继续读尾
- checkpoint 继续绑定到各自的 `file_id`

### 7.2 truncate

当满足以下条件时建议判定为 truncate：

- 同一 `file_id`
- 当前文件大小小于 `checkpoint_offset`

处理建议：

- 记录 `truncate` 事件
- 将 runtime `read offset` 重置到 `0`
- 后续从 `0` 重新建立新的 `checkpoint_offset`

---

## 8. 清理与保留策略

第一版建议：

- 已长期不存在且超过保留窗口的 `file_id` 可被清理
- 正在 `rotate_wait_ms` 窗口内的旧文件 checkpoint 不应过早删除
- 清理动作必须更新 `updated_at`

建议保留策略字段后续再进配置，不在第一版 schema 中编码。

---

## 9. 文件更新策略

`checkpoints.json` 建议沿用 `agentd-state-schema.md` 的统一规则：

- 先写临时文件
- `fsync`
- 原子 `rename`

第一版额外建议：

- 单次落盘应覆盖整个 `checkpoints.json`
- 不在同目录下维护增量 patch 文件

---

## 10. 最小示例

```json
{
  "schema_version": "v1",
  "input_id": "nginx_access",
  "updated_at": "2026-04-12T10:00:00Z",
  "files": [
    {
      "file_id": "dev_2049_ino_912345",
      "path": "/var/log/nginx/access.log",
      "device_id": "2049",
      "inode": "912345",
      "checkpoint_offset": 1839201,
      "last_size": 1839201,
      "last_read_at": "2026-04-12T09:59:58Z",
      "last_commit_point_at": "2026-04-12T09:59:58Z"
    }
  ]
}
```

---

## 11. 当前决定

当前阶段固定以下结论：

- 文件日志输入状态单独建模，不并入 execution state
- `checkpoint_offset` 是持久化真值，`read offset` 是运行时内存态
- `commit point` 是 checkpoint 推进的前置条件
- `file_id` 用于持久化 `file identity`

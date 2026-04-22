# warp-insight Software Normalization 与 Vulnerability Enrichment 设计

## 1. 文档目的

本文档定义 `warp-insight` 中“软件归一化”和“漏洞/元数据 enrichment”的独立设计。

这里的目标不是在边缘节点直接查漏洞，而是先把边缘发现到的进程、安装路径和包管理事实，收敛成中心侧可复用的软件实体，再挂载漏洞、生命周期和其他 meta 信息。

本文重点回答：

- `process` / `container` / package facts 如何归一到稳定的软件对象
- 软件知识库应采用什么标识体系
- `CPE`、`purl`、`SWID` 和内部 `software_id` 的关系是什么
- 漏洞 enrichment 应放在边缘还是中心
- enrichment 结果如何回挂到资源目录和多信号查询体系

相关文档：

- [`../edge/resource-discovery-runtime.md`](../edge/resource-discovery-runtime.md)
- [`control-center-architecture.md`](control-center-architecture.md)
- [`../foundation/architecture.md`](../foundation/architecture.md)
- [`../foundation/target.md`](../foundation/target.md)
- [`../foundation/glossary.md`](../foundation/glossary.md)

---

## 2. 核心结论

第一版固定以下结论：

- `warp-insightd` 只负责发现本地软件线索，不负责维护漏洞知识库
- 软件归一化和漏洞 enrichment 应放在中心侧
- 软件实体不能直接以 `CPE` 或 `purl` 作为唯一主键
- 中心必须维护内部稳定 `software_id`
- `purl` 和 `cpe_candidates[]` 都应支持，但作用不同
- enrichment 必须先完成 software normalization，再进入 vulnerability lookup

一句话说：

`process / package / file facts -> software normalization -> software entity -> vulnerability enrichment`

而不是：

`process.executable.name -> 直接查 CVE`

---

## 3. 为什么不能直接用进程名查漏洞

直接拿进程路径、可执行名或 `process.executable.name` 去查漏洞，会有明显问题：

- 同一软件在不同机器上的安装路径可能不同
- 同一软件可能有多个 helper / renderer / sidecar 进程
- 一个进程名并不稳定映射到一个发行物
- 进程路径本身通常既不是 `CPE`，也不是 `purl`
- 漏洞库的受影响对象通常按产品、版本、包生态建模，不按运行时进程建模

因此必须固定一个中间层：

- 先把运行时进程和安装线索归一成“软件实体”
- 再把软件实体映射到漏洞情报和生命周期知识

---

## 4. 分层边界

### 4.1 边缘负责什么

边缘负责：

- 发现 `process`
- 发现容器镜像、容器运行时信息
- 发现已安装 package / app bundle / binary path 等本地事实
- 提供 `process.pid`
- 提供 `process.executable.name`
- 提供安装路径、签名、版本线索、哈希等候选事实

边缘不负责：

- 维护完整软件目录
- 维护漏洞知识库
- 做跨主机、跨环境的软件归一化
- 直接把软件线索解释成最终 CVE 结论

### 4.2 中心负责什么

中心负责：

- 软件归一化
- 软件实体建模
- 软件目录存储
- `purl` / `CPE` / `SWID` 外部标识映射
- 漏洞 enrichment
- 生命周期 enrichment
- 许可证、供应商、维护状态等 meta enrichment

---

## 5. 对象模型

### 5.1 `SoftwareEvidence`

这是边缘上报或中心抽取后的“软件识别证据”。

建议结构：

```text
SoftwareEvidence {
  evidence_id
  source_kind
  environment_id
  agent_id
  resource_ref?
  observed_at
  executable_path?
  install_path?
  binary_name?
  display_name?
  version_text?
  package_name?
  package_manager?
  vendor_hint?
  signer?
  container_image_ref?
  file_hashes?
  raw_facts
}
```

作用：

- 只表达“看到了什么线索”
- 不表达“最终认定它是什么软件”

### 5.2 `SoftwareEntity`

这是中心归一后的软件实体。

建议结构：

```text
SoftwareEntity {
  software_id
  canonical_name
  vendor?
  normalized_version?
  edition?
  package_type?
  lifecycle_state?
  aliases[]
  homepage?
  created_at
  updated_at
}
```

关键约束：

- `software_id` 是中心内部稳定主键
- 任何外部标准标识都不直接替代 `software_id`

### 5.3 `SoftwareExternalId`

建议单独建模外部标识映射。

```text
SoftwareExternalId {
  software_id
  id_kind
  id_value
  confidence
  source
  observed_at
}
```

其中：

- `id_kind` 可取：
  - `purl`
  - `cpe`
  - `swid`
  - `vendor_product_id`
- `confidence` 表达匹配可信度

### 5.4 `SoftwareVulnerabilityFinding`

这是 enrichment 后的漏洞结果。

```text
SoftwareVulnerabilityFinding {
  finding_id
  software_id
  advisory_source
  vulnerability_id
  aliases[]
  severity
  affected_range?
  fixed_range?
  exploited_known?
  patch_available?
  published_at?
  updated_at?
  evidence
}
```

关键约束：

- 这层是 enrichment 结果，不是原始情报镜像
- `evidence` 应说明本次是基于 `purl`、`CPE` 还是 vendor feed 命中的

---

## 6. 标识体系

### 6.1 内部主键：`software_id`

必须明确：

- `software_id` 是系统内部唯一稳定主键
- 不能直接把 `CPE` 作为内部主键
- 不能直接把 `purl` 作为内部主键

原因：

- 一个软件实体可能对应多个 `CPE`
- 一个软件实体可能有多个 `purl`
- 同一产品不同来源可能存在别名和版本表达差异

### 6.2 `purl`

`purl` 更适合现代包生态。

适合场景：

- OS package
- Java / npm / PyPI / Go module
- 容器镜像中的 package
- 语言生态依赖

优势：

- 语义更现代
- 对 package ecosystem 更自然
- 适合和 SBOM 及包管理系统衔接

限制：

- 不覆盖所有传统桌面 app / 手工安装二进制
- 与 CVE/NVD 体系不是一一直接对齐

### 6.3 `CPE`

`CPE` 更适合漏洞情报映射。

适合场景：

- 对接 NVD
- 对接以产品/版本为核心的漏洞知识库
- 对接传统安全产品接口

优势：

- 在 CVE / NVD 场景中兼容性强

限制：

- 现代软件生态映射常常不自然
- 命名歧义较多
- 同一软件常常需要维护多个 `cpe_candidates[]`

### 6.4 `SWID`

`SWID` 可作为企业资产或安装清单补充标识，但不建议作为第一版主路径。

### 6.5 结论

第一版建议采用：

- 内部：`software_id`
- 现代生态映射：`purl`
- 漏洞情报映射：`cpe_candidates[]`
- 预留：`swid`

一句话说：

- 内部靠 `software_id`
- 包生态优先 `purl`
- 漏洞库兼容 `CPE`

---

## 7. 归一化流程

建议固定如下处理链：

```text
SoftwareEvidence ingest
-> evidence canonicalization
-> software candidate match
-> software entity merge / create
-> external id mapping
-> vulnerability enrichment
-> resource / process / package back-reference
```

### 7.1 evidence canonicalization

处理内容：

- 路径标准化
- 版本文本清洗
- 供应商别名统一
- 可执行名与 bundle 名归一
- package manager 名称统一

### 7.2 software candidate match

建议按以下优先级：

1. package manager + package name + version
2. signed bundle / product metadata
3. executable path + version metadata
4. hash + known catalog
5. heuristic alias mapping

### 7.3 merge / create

如果已有高置信匹配：

- 挂到现有 `software_id`

如果没有可靠匹配：

- 生成新的 `software_id`
- 标记 `normalization_status = provisional`

### 7.4 external id mapping

对同一软件实体：

- 生成或关联 `purl`
- 生成或关联一个或多个 `cpe_candidates`
- 记录匹配置信度和来源

### 7.5 vulnerability enrichment

只有在有足够可靠的外部标识后才进入漏洞 enrichment。

---

## 8. 漏洞 enrichment 设计

### 8.1 数据源分层

建议支持三类来源：

- `vendor advisory`
- `ecosystem advisory`
- `CVE/NVD`

优先级建议：

1. vendor advisory
2. ecosystem advisory
3. NVD / generic CVE feed

原因：

- vendor feed 通常更贴近真实产品版本
- ecosystem advisory 更适合语言包和依赖
- NVD 兼容性广，但匹配误差更大

### 8.2 enrichment 结果不直接等于最终风险结论

必须区分：

- 漏洞命中事实
- 运营风险结论

中间还需要结合：

- 软件是否真的安装
- 版本是否准确
- 是否在运行
- 是否暴露
- 是否有 exploit
- 是否有 patch

因此 `SoftwareVulnerabilityFinding` 只是 enrichment 事实，不直接等同“高风险告警”。

---

## 9. 与资源目录的关系

建议新增一个软件关联层，而不是直接把软件属性全部塞进 `DiscoveredResource`。

### 9.1 关联关系

```text
ProcessResource -> SoftwareEvidence -> SoftwareEntity -> VulnerabilityFindings
ContainerImage -> SoftwareEntity -> VulnerabilityFindings
InstalledPackage -> SoftwareEntity -> VulnerabilityFindings
```

### 9.2 回挂方式

中心查询时可为 `process` / `host` / `container` 附加：

- `software_ref`
- `software.name`
- `software.vendor`
- `software.version`
- `software.lifecycle_state`
- `software.vulnerability_summary`

但边缘本地 cache 不应直接变成漏洞数据库。

---

## 10. 边缘上报建议

第一版边缘只需补充足够的 `SoftwareEvidence` 候选事实，不必一开始就做完整 package inventory。

建议最小上报字段：

- `process.executable.name`
- `process.pid`
- `resource_id`
- 安装路径或 bundle 路径
- 版本候选文本（若能低成本获取）
- signer / package manager / image ref（若能低成本获取）

边缘原则：

- 低开销
- 不阻塞采集主路径
- 不因 enrichment 失败影响数据面

---

## 11. 存储建议

第一版中心存储建议拆成：

- `software_entities`
- `software_aliases`
- `software_external_ids`
- `software_evidence`
- `software_vulnerability_findings`
- `software_lifecycle_facts`

如果后续接 SBOM，可再增加：

- `software_components`
- `software_dependency_edges`

---

## 12. 第一版落地范围

第一版建议只做：

1. `process -> SoftwareEvidence`
2. 基于路径 / bundle / package name 的初步 software normalization
3. 内部 `software_id`
4. `purl` 与 `cpe_candidates[]` 的最小映射结构
5. 漏洞 enrichment 存储骨架

第一版不建议一开始就做：

- 全量 SBOM ingestion
- 软件依赖图
- 实时 exploit intelligence
- 复杂许可证合规分析

---

## 13. 验收标准

第一版至少应满足：

- 同一软件的多个 helper / renderer 进程能归到同一个 `software_id`
- 带空格路径、bundle app、package manager 安装项能稳定识别
- `software_id` 与 `purl` / `cpe_candidates[]` 可同时存在
- 漏洞 enrichment 结果能说明命中来源和置信度
- enrichment 失败不影响边缘 discovery 和 telemetry 主路径

---

## 14. 当前建议

当前建议固定为：

- 软件归一化采用“内部 `software_id` + `purl` + `cpe_candidates[]`”
- 不采用单一 `CPE` 作为唯一标准
- 边缘只提供 evidence
- 中心完成 software normalization 与 vulnerability enrichment

一句话总结：

`warp-insight` 应把软件知识库设计成中心侧软件实体目录，而不是把进程名直接当漏洞库查询键。

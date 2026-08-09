-- warp-insight-center 网关存储 schema（docker-entrypoint-initdb.d）
-- 由 docker-compose.yml 的 postgres 服务首次启动时执行。

CREATE TABLE IF NOT EXISTS gateways (
  gateway_id TEXT PRIMARY KEY,
  instance_id TEXT NOT NULL DEFAULT '',
  credential_token_hash TEXT NOT NULL,
  credential_status TEXT NOT NULL DEFAULT 'active',
  credential_expires_at TIMESTAMPTZ,
  version TEXT,
  status TEXT,
  health TEXT,
  memory_bytes BIGINT,
  cpu_percent DOUBLE PRECISION,
  lifecycle_state TEXT,
  initialized_at TIMESTAMPTZ,
  last_seen_at TIMESTAMPTZ
);

-- Gateway 上报的其下 Agent 状态（快照，agent_id 幂等 upsert）。
CREATE TABLE IF NOT EXISTS agent_status (
  agent_id TEXT PRIMARY KEY,
  gateway_id TEXT NOT NULL,
  instance_id TEXT NOT NULL DEFAULT '',
  version TEXT NOT NULL DEFAULT '',
  status TEXT NOT NULL DEFAULT 'online',
  health TEXT NOT NULL DEFAULT 'healthy',
  memory_bytes BIGINT,
  cpu_percent DOUBLE PRECISION,
  admin_latency_ms BIGINT,
  last_seen_at TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_agent_status_gateway ON agent_status (gateway_id);

-- 网关生命周期转变历史（append-only 过程记录）。
CREATE TABLE IF NOT EXISTS gateway_lifecycle_events (
  id BIGSERIAL PRIMARY KEY,
  gateway_id TEXT NOT NULL,
  from_state TEXT,
  to_state TEXT NOT NULL,
  at TIMESTAMPTZ NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_lifecycle_events_gateway ON gateway_lifecycle_events (gateway_id);

-- 版本发布记录（component = warp-agentd / warp-gateway）。
CREATE TABLE IF NOT EXISTS release_records (
  id BIGSERIAL PRIMARY KEY,
  component TEXT NOT NULL,
  version TEXT NOT NULL,
  artifact_url TEXT NOT NULL,
  status TEXT NOT NULL,
  published_at TIMESTAMPTZ NOT NULL
);

-- 升级计划（payload 为 JSON，含多目标/网关范围/多步执行）。
CREATE TABLE IF NOT EXISTS upgrade_plans (
  plan_id TEXT PRIMARY KEY,
  payload JSONB NOT NULL,
  status TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

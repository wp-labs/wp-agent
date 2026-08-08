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

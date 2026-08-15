export interface AgentRuntimeStatusView {
  agentId: string;
  instanceId: string;
  version: string;
  status: "online" | "offline" | "paused";
  health: "healthy" | "degraded" | "unhealthy";
  lastSeenAt: string;
}

export interface AgentOverviewMetrics {
  totalAgents: number;
  onlineAgents: number;
  unhealthyAgents: number;
  lastSeenLagSeconds: number;
}

export interface AgentMetricSample {
  at: string;
  memoryBytes?: number;
  cpuPercent?: number;
  adminLatencyMs?: number;
}

export interface RecentOnlineRegisteredAgent {
  agentId: string;
  instanceId: string;
  version: string;
  registeredAt: string;
  onlineSince: string;
  onlineDurationSeconds: number;
  source: "real" | "example";
  memoryBytes?: number;
  cpuPercent?: number;
  adminLatencyMs?: number;
  metricsHistory?: AgentMetricSample[];
}

export interface AgentOverview {
  metrics: AgentOverviewMetrics;
  recentOnlineAgents: RecentOnlineRegisteredAgent[];
  abnormalAgents: AgentRuntimeStatusView[];
}

export interface AgentInstallCode {
  x86LinuxInstallCode: string;
  armLinuxInstallCode: string;
  bootstrapEnrollmentToken: string;
}

/** 控制中心返回给 Gateway 的初始连接材料，字段与 Gateway 面接口契约一致。 */
export interface ControlCenterTrustBundle {
  trust_bundle_id: string;
  control_endpoint: string;
  ca_bundle: string;
  server_name: string;
  expected_san: string;
  issued_at: string | null;
  expires_at: string | null;
}

/** GET /api/v1/gateway/initial-config 的 config 载荷。 */
export interface GatewayInitialConfig {
  gateway_id: string;
  control_center_endpoint: string;
  trust_bundle: ControlCenterTrustBundle | null;
  server_tls_required: boolean;
  protocol_version: string;
  enrollment_token_id: string;
}

export interface DispatchReceipt {
  dispatchId: string;
  commandId: string;
  agentId: string;
  status: "accepted" | "rejected";
  createdAt: string;
}

export interface PauseAgentCommand {
  agentId: string;
  requestedBy: string;
}

export interface UpgradeAgentCommand {
  agentId: string;
  targetVersion: string;
  requestedBy: string;
}

export const ADMIN_AUTH_CHANGED_EVENT = "warpInsightAdminAuthChanged";
const ADMIN_API_TOKEN_STORAGE_KEY = "warpInsightAdminApiToken";

// Persist the admin token for the current browser session (survives page
// reloads, but is cleared when the tab/session closes).
let adminApiToken: string | null =
  typeof window !== "undefined"
    ? window.sessionStorage.getItem(ADMIN_API_TOKEN_STORAGE_KEY)
    : null;

export class ApiError extends Error {
  readonly status: number;
  /** Seconds until the per-IP rate-limit block expires, when status is 429. */
  readonly retryAfterSeconds?: number;

  constructor(status: number, path: string, retryAfterSeconds?: number) {
    super(`HTTP ${status} ${path}`);
    this.name = "ApiError";
    this.status = status;
    this.retryAfterSeconds = retryAfterSeconds;
  }
}

export function isRateLimitedError(error: unknown): error is ApiError {
  return error instanceof ApiError && error.status === 429;
}

async function requestJson<T>(path: string, init?: RequestInit): Promise<T> {
  const adminToken = getAdminApiToken();
  const response = await fetch(path, {
    ...init,
    headers: {
      "content-type": "application/json",
      ...(adminToken ? { authorization: `Bearer ${adminToken}` } : {}),
      ...(init?.headers ?? {}),
    },
  });
  if (!response.ok) {
    if (response.status === 429) {
      const retryAfter = Number.parseInt(response.headers.get("Retry-After") ?? "", 10);
      throw new ApiError(
        response.status,
        path,
        Number.isFinite(retryAfter) && retryAfter > 0 ? retryAfter : 60,
      );
    }
    throw new ApiError(response.status, path);
  }
  return (await response.json()) as T;
}

export function getAdminApiToken(): string | null {
  return adminApiToken;
}

export function setAdminApiToken(token: string): void {
  const trimmed = token.trim();
  adminApiToken = trimmed || null;
  if (typeof window !== "undefined") {
    if (adminApiToken) {
      window.sessionStorage.setItem(ADMIN_API_TOKEN_STORAGE_KEY, adminApiToken);
    } else {
      window.sessionStorage.removeItem(ADMIN_API_TOKEN_STORAGE_KEY);
    }
    window.dispatchEvent(new Event(ADMIN_AUTH_CHANGED_EVENT));
  }
}

export function clearAdminApiToken(): void {
  setAdminApiToken("");
}

function requiredString(value: unknown, fieldName: string): string {
  if (typeof value === "string") return value;
  throw new Error(`Invalid API response: missing ${fieldName}`);
}

function requiredNumber(value: unknown, fieldName: string): number {
  if (typeof value === "number") return value;
  throw new Error(`Invalid API response: missing ${fieldName}`);
}

function requiredBoolean(value: unknown, fieldName: string): boolean {
  if (typeof value === "boolean") return value;
  throw new Error(`Invalid API response: missing ${fieldName}`);
}

function requiredRecord(
  value: unknown,
  fieldName: string,
): Record<string, unknown> {
  if (typeof value === "object" && value !== null && !Array.isArray(value)) {
    return value as Record<string, unknown>;
  }
  throw new Error(`Invalid API response: missing ${fieldName}`);
}

function nullableString(value: unknown, fieldName: string): string | null {
  if (value === null) return null;
  return requiredString(value, fieldName);
}

function normalizeTrustBundle(value: unknown): ControlCenterTrustBundle | null {
  if (value === null) return null;
  const bundle = requiredRecord(value, "config.trust_bundle");
  return {
    trust_bundle_id: requiredString(
      bundle.trust_bundle_id,
      "config.trust_bundle.trust_bundle_id",
    ),
    control_endpoint: requiredString(
      bundle.control_endpoint,
      "config.trust_bundle.control_endpoint",
    ),
    ca_bundle: requiredString(bundle.ca_bundle, "config.trust_bundle.ca_bundle"),
    server_name: requiredString(
      bundle.server_name,
      "config.trust_bundle.server_name",
    ),
    expected_san: requiredString(
      bundle.expected_san,
      "config.trust_bundle.expected_san",
    ),
    issued_at: nullableString(
      bundle.issued_at,
      "config.trust_bundle.issued_at",
    ),
    expires_at: nullableString(
      bundle.expires_at,
      "config.trust_bundle.expires_at",
    ),
  };
}

/** 按当前 JSON 契约收敛初始配置，避免把服务端错误静默成空字段。 */
export function normalizeGatewayInitialConfig(
  payload: unknown,
): GatewayInitialConfig {
  const root = requiredRecord(payload, "response");
  const config = requiredRecord(root.config, "config");
  return {
    gateway_id: requiredString(config.gateway_id, "config.gateway_id"),
    control_center_endpoint: requiredString(
      config.control_center_endpoint,
      "config.control_center_endpoint",
    ),
    trust_bundle: normalizeTrustBundle(config.trust_bundle),
    server_tls_required: requiredBoolean(
      config.server_tls_required,
      "config.server_tls_required",
    ),
    protocol_version: requiredString(
      config.protocol_version,
      "config.protocol_version",
    ),
    enrollment_token_id: requiredString(
      config.enrollment_token_id,
      "config.enrollment_token_id",
    ),
  };
}

function requiredArray(value: unknown, fieldName: string): any[] {
  if (Array.isArray(value)) return value;
  throw new Error(`Invalid API response: missing ${fieldName}`);
}

function normalizeAgentStatus(
  value: unknown,
): AgentRuntimeStatusView["status"] {
  if (value === "online" || value === "offline" || value === "paused")
    return value;
  throw new Error("Invalid API response: invalid agent status");
}

function normalizeAgentHealth(
  value: unknown,
): AgentRuntimeStatusView["health"] {
  if (value === "healthy" || value === "degraded" || value === "unhealthy")
    return value;
  throw new Error("Invalid API response: invalid agent health");
}

function normalizeReceiptStatus(value: unknown): DispatchReceipt["status"] {
  if (value === "accepted" || value === "rejected") return value;
  throw new Error("Invalid API response: invalid dispatch receipt status");
}

function normalizeInstallCode(payload: any): AgentInstallCode {
  const installCode = payload.install_code ?? payload.installCode ?? payload;
  return {
    x86LinuxInstallCode: requiredString(
      installCode.x86_linux_install_code ?? installCode.x86LinuxInstallCode,
      "installCode.x86LinuxInstallCode",
    ),
    armLinuxInstallCode: requiredString(
      installCode.arm_linux_install_code ?? installCode.armLinuxInstallCode,
      "installCode.armLinuxInstallCode",
    ),
    bootstrapEnrollmentToken: requiredString(
      installCode.bootstrap_enrollment_token ??
        installCode.bootstrapEnrollmentToken,
      "installCode.bootstrapEnrollmentToken",
    ),
  };
}

function normalizeReceipt(payload: any): DispatchReceipt {
  const receipt = payload.result ?? payload;
  return {
    dispatchId: requiredString(
      receipt.dispatch_id ?? receipt.dispatchId,
      "receipt.dispatchId",
    ),
    commandId: requiredString(
      receipt.command_id ?? receipt.commandId,
      "receipt.commandId",
    ),
    agentId: requiredString(
      receipt.agent_id ?? receipt.agentId,
      "receipt.agentId",
    ),
    status: normalizeReceiptStatus(receipt.status),
    createdAt: requiredString(
      receipt.created_at ?? receipt.createdAt,
      "receipt.createdAt",
    ),
  };
}

function normalizeRuntimeStatus(payload: any): AgentRuntimeStatusView {
  return {
    agentId: requiredString(
      payload.agent_id ?? payload.agentId,
      "agent.agentId",
    ),
    instanceId: requiredString(
      payload.instance_id ?? payload.instanceId,
      "agent.instanceId",
    ),
    version: requiredString(payload.version, "agent.version"),
    status: normalizeAgentStatus(payload.status),
    health: normalizeAgentHealth(payload.health),
    lastSeenAt: requiredString(
      payload.last_seen_at ?? payload.lastSeenAt,
      "agent.lastSeenAt",
    ),
  };
}

function normalizeRecentOnlineAgent(payload: any): RecentOnlineRegisteredAgent {
  const source = payload.source ?? "real";
  if (source !== "real" && source !== "example") {
    throw new Error("Invalid API response: invalid recent online agent source");
  }
  const rawHistory = payload.metrics_history ?? payload.metricsHistory ?? [];
  return {
    agentId: requiredString(
      payload.agent_id ?? payload.agentId,
      "recentOnlineAgent.agentId",
    ),
    instanceId: requiredString(
      payload.instance_id ?? payload.instanceId,
      "recentOnlineAgent.instanceId",
    ),
    version: requiredString(payload.version, "recentOnlineAgent.version"),
    registeredAt: requiredString(
      payload.registered_at ?? payload.registeredAt,
      "recentOnlineAgent.registeredAt",
    ),
    onlineSince: requiredString(
      payload.online_since ?? payload.onlineSince,
      "recentOnlineAgent.onlineSince",
    ),
    onlineDurationSeconds: requiredNumber(
      payload.online_duration_seconds ?? payload.onlineDurationSeconds,
      "recentOnlineAgent.onlineDurationSeconds",
    ),
    source,
    memoryBytes: payload.memory_bytes ?? payload.memoryBytes,
    cpuPercent: payload.cpu_percent ?? payload.cpuPercent,
    adminLatencyMs: payload.admin_latency_ms ?? payload.adminLatencyMs,
    metricsHistory: rawHistory.map((sample: any) => ({
      at: sample.at,
      memoryBytes: sample.memory_bytes ?? sample.memoryBytes,
      cpuPercent: sample.cpu_percent ?? sample.cpuPercent,
      adminLatencyMs: sample.admin_latency_ms ?? sample.adminLatencyMs,
    })),
  };
}

export function normalizeOverview(payload: any): AgentOverview {
  const metrics = payload.metrics;
  const recentOnlineAgents =
    payload.recent_online_agents ?? payload.recentOnlineAgents;
  const abnormalAgents = payload.abnormal_agents ?? payload.abnormalAgents;
  return {
    metrics: {
      totalAgents: requiredNumber(
        metrics?.total_agents ?? metrics?.totalAgents,
        "metrics.totalAgents",
      ),
      onlineAgents: requiredNumber(
        metrics?.online_agents ?? metrics?.onlineAgents,
        "metrics.onlineAgents",
      ),
      unhealthyAgents: requiredNumber(
        metrics?.unhealthy_agents ?? metrics?.unhealthyAgents,
        "metrics.unhealthyAgents",
      ),
      lastSeenLagSeconds: requiredNumber(
        metrics?.last_seen_lag_seconds ?? metrics?.lastSeenLagSeconds,
        "metrics.lastSeenLagSeconds",
      ),
    },
    recentOnlineAgents: requiredArray(
      recentOnlineAgents,
      "overview.recentOnlineAgents",
    ).map(normalizeRecentOnlineAgent),
    abnormalAgents: requiredArray(
      abnormalAgents,
      "overview.abnormalAgents",
    ).map(normalizeRuntimeStatus),
  };
}

export async function fetchAgentOverview(): Promise<AgentOverview> {
  const payload = await requestJson<unknown>("/api/v1/admin/agents/overview");
  return normalizeOverview(payload);
}

export async function fetchAgentInstallCode(): Promise<AgentInstallCode> {
  const payload = await requestJson<unknown>("/api/v1/agent/install-code");
  return normalizeInstallCode(payload);
}

/**
 * 从 Gateway 页面调用控制中心的网关面初始化接口。
 * initUrl 由 Center 创建实例时下发，**不携带凭证**（token 不进 URL）；
 * 网关凭证由操作者单独输入，只放入 Authorization Header。
 * 返回 `application/toml` 的 config.toml 文本（置备时由 Center 生成）。
 */
export async function fetchGatewayInitialConfig(
  initUrl: string,
  token?: string,
): Promise<string> {
  // 去掉可能残留的 fragment（如手工复制带 # 的链接）。
  const path = initUrl.split("#", 1)[0];
  const response = await fetch(path, {
    headers: {
      accept: "application/toml",
      ...(token ? { authorization: `Bearer ${token}` } : {}),
    },
  });
  if (!response.ok) {
    throw new ApiError(response.status, path);
  }
  return response.text();
}

export async function pauseAgent(
  command: PauseAgentCommand,
): Promise<DispatchReceipt> {
  const payload = await requestJson<unknown>(
    `/api/v1/admin/agents/${encodeURIComponent(command.agentId)}/pause`,
    {
      method: "POST",
      body: JSON.stringify({ requested_by: command.requestedBy }),
    },
  );
  return normalizeReceipt(payload);
}

export async function upgradeAgent(
  command: UpgradeAgentCommand,
): Promise<DispatchReceipt> {
  const payload = await requestJson<unknown>(
    `/api/v1/admin/agents/${encodeURIComponent(command.agentId)}/upgrade`,
    {
      method: "POST",
      body: JSON.stringify({
        requested_by: command.requestedBy,
        target_version: command.targetVersion,
      }),
    },
  );
  return normalizeReceipt(payload);
}

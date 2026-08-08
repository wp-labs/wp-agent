// WarpInsightCenter 全局控制中心 Admin API client.
//
// 面向 AdminFacingInterface 的 HTTP 管理接口（Control.AdminFacingInterface）：
// 网关列表 / 状态视图 / 实例管理 / 初始配置 / 版本发布 / 升级计划。
// 后端接口尚未实现时，请求失败自动回退到 example 数据（source: "example"），
// 保证前端独立可渲染；接入真实后端后自动切换为 "real"。

export type GatewayStatus = "online" | "offline" | (string & {});
export type GatewayHealth = "healthy" | "degraded" | "unhealthy" | "unknown";

export interface GatewayStatusView {
  gatewayId: string;
  instanceId: string;
  version: string;
  status: GatewayStatus;
  health: GatewayHealth;
  lastSeenAt: string;
}

export interface GatewayListView {
  gatewayCount: number;
  onlineCount: number;
  degradedCount: number;
  offlineCount: number;
  updatedAt: string;
}

export interface GatewayInstance {
  gatewayId: string;
  instanceId: string;
  status: string;
  createdAt: string;
}

export interface GatewayCustomerBinding {
  gatewayId: string;
  customerId: string;
  status: string;
  boundAt: string;
}

export interface GatewayInitialConfig {
  controlCenterEndpoint: string;
  policyVersion: string;
  telemetryOutput: string;
}

export interface WarpAgentdRelease {
  version: string;
  artifactUrl: string;
  status: string;
  publishedAt: string;
}

export interface WarpGateWayRelease {
  version: string;
  artifactUrl: string;
  status: string;
  publishedAt: string;
}

export interface UpgradePlan {
  planId: string;
  component: string;
  targetVersion: string;
  targetCount: number;
  status: string;
  createdAt: string;
}

export interface UpgradePlanApproval {
  planId: string;
  status: string;
  approvedBy: string;
  approvedAt: string;
}

export interface GlobalPolicyDispatch {
  dispatchId: string;
  policyVersion: string;
  targetCount: number;
  status: string;
  dispatchedAt: string;
}

// ── 北向命令 ──

export interface CreateGatewayInstanceCommand {
  gatewayName: string;
  requestedBy: string;
}

export interface BindGatewayCustomerCommand {
  gatewayId: string;
  customerId: string;
  requestedBy: string;
}

export interface GetGatewayInitialConfigCommand {
  instanceId: string;
  requestedBy: string;
}

export interface PublishReleaseCommand {
  version: string;
  artifactUrl: string;
  requestedBy: string;
}

export interface CreateUpgradePlanCommand {
  component: string;
  targetVersion: string;
  gatewayIds: string[];
  requestedBy: string;
}

export interface ApproveUpgradePlanCommand {
  planId: string;
  approvedBy: string;
}

// ── 结果信封：数据 + 来源标记 ──

export interface ExampleResult<T> {
  data: T;
  source: "real" | "example";
}

// ── 认证与请求包装 ──

export const ADMIN_AUTH_CHANGED_EVENT = "warpInsightCenterAuthChanged";
const ADMIN_API_TOKEN_STORAGE_KEY = "warpInsightCenterApiToken";

let adminApiToken: string | null =
  typeof window !== "undefined"
    ? window.sessionStorage.getItem(ADMIN_API_TOKEN_STORAGE_KEY)
    : null;

export class ApiError extends Error {
  readonly status: number;
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
      const retryAfter = Number.parseInt(
        response.headers.get("Retry-After") ?? "",
        10,
      );
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

// ── 请求失败回退 example 数据 ──

async function fetchOrFallback<T>(
  path: string,
  fallback: () => T,
  init?: RequestInit,
): Promise<ExampleResult<T>> {
  try {
    const data = await requestJson<T>(path, init);
    return { data, source: "real" };
  } catch (error) {
    if (isRateLimitedError(error)) throw error;
    return { data: fallback(), source: "example" };
  }
}

// ── 类型归一化（snake_case / camelCase 容错） ──

function requiredString(value: unknown, fieldName: string): string {
  if (typeof value === "string") return value;
  throw new Error(`Invalid API response: missing ${fieldName}`);
}

function requiredNumber(value: unknown, fieldName: string): number {
  if (typeof value === "number") return value;
  throw new Error(`Invalid API response: missing ${fieldName}`);
}

function pick(obj: any, ...names: string[]): unknown {
  for (const name of names) {
    if (obj?.[name] !== undefined) return obj[name];
  }
  return undefined;
}

function normalizeGatewayStatus(value: unknown): GatewayStatus {
  // 模型 status 为 String，取值域未收紧；异常上报值不击穿整个列表，原样透传（徽标兜底显示离线）。
  if (typeof value === "string" && value.length > 0) return value as GatewayStatus;
  throw new Error("Invalid API response: invalid gateway status");
}

function normalizeGatewayHealth(value: unknown): GatewayHealth {
  if (value === "healthy" || value === "degraded" || value === "unhealthy")
    return value;
  // 未上报/异常健康值 → "unknown"，徽标显示「未知」。
  return "unknown";
}

function normalizeGatewayStatusView(payload: any): GatewayStatusView {
  return {
    gatewayId: requiredString(pick(payload, "gateway_id", "gatewayId"), "gateway.gatewayId"),
    instanceId: requiredString(pick(payload, "instance_id", "instanceId"), "gateway.instanceId"),
    version: requiredString(payload.version, "gateway.version"),
    status: normalizeGatewayStatus(payload.status),
    health: normalizeGatewayHealth(payload.health),
    lastSeenAt: requiredString(pick(payload, "last_seen_at", "lastSeenAt"), "gateway.lastSeenAt"),
  };
}

function normalizeGatewayListView(payload: any): GatewayListView {
  return {
    gatewayCount: requiredNumber(pick(payload, "gateway_count", "gatewayCount"), "list.gatewayCount"),
    onlineCount: requiredNumber(pick(payload, "online_count", "onlineCount"), "list.onlineCount"),
    degradedCount: requiredNumber(pick(payload, "degraded_count", "degradedCount"), "list.degradedCount"),
    offlineCount: requiredNumber(pick(payload, "offline_count", "offlineCount"), "list.offlineCount"),
    updatedAt: requiredString(pick(payload, "updated_at", "updatedAt"), "list.updatedAt"),
  };
}

function normalizeGatewayInstance(payload: any): GatewayInstance {
  return {
    gatewayId: requiredString(pick(payload, "gateway_id", "gatewayId"), "instance.gatewayId"),
    instanceId: requiredString(pick(payload, "instance_id", "instanceId"), "instance.instanceId"),
    status: requiredString(payload.status, "instance.status"),
    createdAt: requiredString(pick(payload, "created_at", "createdAt"), "instance.createdAt"),
  };
}

function normalizeGatewayCustomerBinding(payload: any): GatewayCustomerBinding {
  return {
    gatewayId: requiredString(pick(payload, "gateway_id", "gatewayId"), "binding.gatewayId"),
    customerId: requiredString(pick(payload, "customer_id", "customerId"), "binding.customerId"),
    status: requiredString(payload.status, "binding.status"),
    boundAt: requiredString(pick(payload, "bound_at", "boundAt"), "binding.boundAt"),
  };
}

function normalizeGatewayInitialConfig(payload: any): GatewayInitialConfig {
  return {
    controlCenterEndpoint: requiredString(
      pick(payload, "control_center_endpoint", "controlCenterEndpoint"),
      "config.controlCenterEndpoint",
    ),
    policyVersion: requiredString(pick(payload, "policy_version", "policyVersion"), "config.policyVersion"),
    telemetryOutput: requiredString(pick(payload, "telemetry_output", "telemetryOutput"), "config.telemetryOutput"),
  };
}

function normalizeRelease(payload: any): WarpAgentdRelease {
  return {
    version: requiredString(payload.version, "release.version"),
    artifactUrl: requiredString(pick(payload, "artifact_url", "artifactUrl"), "release.artifactUrl"),
    status: requiredString(payload.status, "release.status"),
    publishedAt: requiredString(pick(payload, "published_at", "publishedAt"), "release.publishedAt"),
  };
}

function normalizeUpgradePlan(payload: any): UpgradePlan {
  return {
    planId: requiredString(pick(payload, "plan_id", "planId"), "plan.planId"),
    component: requiredString(payload.component, "plan.component"),
    targetVersion: requiredString(pick(payload, "target_version", "targetVersion"), "plan.targetVersion"),
    targetCount: requiredNumber(pick(payload, "target_count", "targetCount"), "plan.targetCount"),
    status: requiredString(payload.status, "plan.status"),
    createdAt: requiredString(pick(payload, "created_at", "createdAt"), "plan.createdAt"),
  };
}

function normalizeUpgradePlanApproval(payload: any): UpgradePlanApproval {
  return {
    planId: requiredString(pick(payload, "plan_id", "planId"), "approval.planId"),
    status: requiredString(payload.status, "approval.status"),
    approvedBy: requiredString(pick(payload, "approved_by", "approvedBy"), "approval.approvedBy"),
    approvedAt: requiredString(pick(payload, "approved_at", "approvedAt"), "approval.approvedAt"),
  };
}

// ── example 数据 ──

function isoMinutesAgo(minutes: number): string {
  return new Date(Date.now() - minutes * 60_000).toISOString();
}

function exampleGatewayStatusView(): GatewayStatusView[] {
  return [
    { gatewayId: "gw-001", instanceId: "inst-7f2a", version: "v2.4.1", status: "online", health: "healthy", lastSeenAt: isoMinutesAgo(1) },
    { gatewayId: "gw-002", instanceId: "inst-9c31", version: "v2.4.1", status: "online", health: "degraded", lastSeenAt: isoMinutesAgo(4) },
    { gatewayId: "gw-003", instanceId: "inst-1d8b", version: "v2.3.0", status: "offline", health: "unhealthy", lastSeenAt: isoMinutesAgo(138) },
    { gatewayId: "gw-004", instanceId: "inst-4e77", version: "v2.4.0", status: "online", health: "healthy", lastSeenAt: isoMinutesAgo(2) },
    { gatewayId: "gw-005", instanceId: "inst-aa21", version: "v2.2.2", status: "offline", health: "unhealthy", lastSeenAt: isoMinutesAgo(420) },
    { gatewayId: "gw-006", instanceId: "inst-38c4", version: "v2.4.1", status: "online", health: "healthy", lastSeenAt: isoMinutesAgo(0) },
  ];
}

function exampleGatewayListView(): GatewayListView {
  return {
    gatewayCount: 6,
    onlineCount: 4,
    degradedCount: 1,
    offlineCount: 2,
    updatedAt: new Date().toISOString(),
  };
}

function exampleGatewayInstance(command: CreateGatewayInstanceCommand): GatewayInstance {
  return {
    gatewayId: `gw-${Math.random().toString(36).slice(2, 8)}`,
    instanceId: `inst-${Math.random().toString(36).slice(2, 6)}`,
    status: "provisioned",
    createdAt: new Date().toISOString(),
  };
}

function exampleBinding(command: BindGatewayCustomerCommand): GatewayCustomerBinding {
  return {
    gatewayId: command.gatewayId,
    customerId: command.customerId,
    status: "bound",
    boundAt: new Date().toISOString(),
  };
}

function exampleInitialConfig(command: GetGatewayInitialConfigCommand): GatewayInitialConfig {
  return {
    controlCenterEndpoint: "https://center.example.com",
    policyVersion: "policy-v12",
    telemetryOutput: "otlp://telemetry.example.com:4317",
  };
}

function exampleRelease(command: PublishReleaseCommand): WarpAgentdRelease {
  return {
    version: command.version,
    artifactUrl: command.artifactUrl,
    status: "published",
    publishedAt: new Date().toISOString(),
  };
}

function exampleUpgradePlan(command: CreateUpgradePlanCommand): UpgradePlan {
  return {
    planId: `plan-${Math.random().toString(36).slice(2, 8)}`,
    component: command.component,
    targetVersion: command.targetVersion,
    targetCount: command.gatewayIds.length,
    status: "pending",
    createdAt: new Date().toISOString(),
  };
}

function exampleApproval(command: ApproveUpgradePlanCommand): UpgradePlanApproval {
  return {
    planId: command.planId,
    status: "approved",
    approvedBy: command.approvedBy,
    approvedAt: new Date().toISOString(),
  };
}

// ── 接口调用 ──

export async function fetchGatewayStatusView(): Promise<ExampleResult<GatewayStatusView[]>> {
  return fetchOrFallback("/api/v1/admin/gateways/status", exampleGatewayStatusView).then(
    async (result) => {
      if (result.source !== "real") return result;
      const raw = result.data as any;
      const items = Array.isArray(raw)
        ? raw
        : Array.isArray(raw?.statuses)
          ? raw.statuses
          : Array.isArray(raw?.gateways)
            ? raw.gateways
            : raw?.status
              ? [raw.status]
              : [];
      return { ...result, data: items.map(normalizeGatewayStatusView) };
    },
  );
}

export async function fetchGatewayList(): Promise<ExampleResult<GatewayListView>> {
  return fetchOrFallback("/api/v1/admin/gateways", exampleGatewayListView).then(
    async (result) => {
      if (result.source !== "real") return result;
      const raw = result.data as any;
      return {
        ...result,
        data: normalizeGatewayListView(raw?.list ?? raw),
      };
    },
  );
}

export async function createGatewayInstance(
  command: CreateGatewayInstanceCommand,
): Promise<ExampleResult<GatewayInstance>> {
  return fetchOrFallback(
    "/api/v1/admin/gateways/instances",
    () => exampleGatewayInstance(command),
    {
      method: "POST",
      body: JSON.stringify({ gateway_name: command.gatewayName, requested_by: command.requestedBy }),
    },
  ).then(async (result) => {
    if (result.source === "real") {
      return { ...result, data: normalizeGatewayInstance(result.data) };
    }
    return result;
  });
}

export async function bindGatewayCustomer(
  command: BindGatewayCustomerCommand,
): Promise<ExampleResult<GatewayCustomerBinding>> {
  return fetchOrFallback(
    "/api/v1/admin/gateways/bind",
    () => exampleBinding(command),
    {
      method: "POST",
      body: JSON.stringify({
        gateway_id: command.gatewayId,
        customer_id: command.customerId,
        requested_by: command.requestedBy,
      }),
    },
  ).then(async (result) => {
    if (result.source === "real") {
      return { ...result, data: normalizeGatewayCustomerBinding(result.data) };
    }
    return result;
  });
}

export async function fetchGatewayInitialConfig(
  command: GetGatewayInitialConfigCommand,
): Promise<ExampleResult<GatewayInitialConfig>> {
  const path = `/api/v1/admin/gateways/instances/${encodeURIComponent(command.instanceId)}/config`;
  return fetchOrFallback(path, () => exampleInitialConfig(command)).then(
    async (result) => {
      if (result.source === "real") {
        return { ...result, data: normalizeGatewayInitialConfig(result.data) };
      }
      return result;
    },
  );
}

export async function publishWarpAgentd(
  command: PublishReleaseCommand,
): Promise<ExampleResult<WarpAgentdRelease>> {
  return fetchOrFallback(
    "/api/v1/admin/releases/warp-agentd",
    () => exampleRelease(command),
    {
      method: "POST",
      body: JSON.stringify({ version: command.version, artifact_url: command.artifactUrl, requested_by: command.requestedBy }),
    },
  ).then(async (result) => {
    if (result.source === "real") {
      return { ...result, data: normalizeRelease(result.data) };
    }
    return result;
  });
}

export async function publishWarpGateWay(
  command: PublishReleaseCommand,
): Promise<ExampleResult<WarpGateWayRelease>> {
  return fetchOrFallback(
    "/api/v1/admin/releases/warp-gateway",
    () => exampleRelease(command),
    {
      method: "POST",
      body: JSON.stringify({ version: command.version, artifact_url: command.artifactUrl, requested_by: command.requestedBy }),
    },
  ).then(async (result) => {
    if (result.source === "real") {
      return { ...result, data: normalizeRelease(result.data) };
    }
    return result;
  });
}

export async function createUpgradePlan(
  command: CreateUpgradePlanCommand,
): Promise<ExampleResult<UpgradePlan>> {
  return fetchOrFallback(
    "/api/v1/admin/upgrade-plans",
    () => exampleUpgradePlan(command),
    {
      method: "POST",
      body: JSON.stringify({
        component: command.component,
        target_version: command.targetVersion,
        gateway_ids: command.gatewayIds,
        requested_by: command.requestedBy,
      }),
    },
  ).then(async (result) => {
    if (result.source === "real") {
      return { ...result, data: normalizeUpgradePlan(result.data) };
    }
    return result;
  });
}

export async function approveUpgradePlan(
  command: ApproveUpgradePlanCommand,
): Promise<ExampleResult<UpgradePlanApproval>> {
  return fetchOrFallback(
    "/api/v1/admin/upgrade-plans/approve",
    () => exampleApproval(command),
    {
      method: "POST",
      body: JSON.stringify({ plan_id: command.planId, approved_by: command.approvedBy }),
    },
  ).then(async (result) => {
    if (result.source === "real") {
      return { ...result, data: normalizeUpgradePlanApproval(result.data) };
    }
    return result;
  });
}

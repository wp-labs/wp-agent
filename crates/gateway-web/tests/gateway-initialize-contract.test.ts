import {
  fetchGatewayInitialConfig,
  fetchGatewayInitializationStatus,
  GatewayAlreadyInitializedError,
  GatewayInitializationInputError,
  initializeGatewayViaUrl,
  normalizeGatewayInitialConfig,
  parseGatewayInitializationUrl,
} from "../src/api/admin";

const config = normalizeGatewayInitialConfig({
  config: {
    gateway_id: "gw-demo",
    control_center_endpoint: "https://center.example",
    trust_bundle: {
      trust_bundle_id: "trust-gw-demo-1",
      control_endpoint: "https://center.example",
      ca_bundle: "-----BEGIN CERTIFICATE-----\nCA\n-----END CERTIFICATE-----",
      server_name: "center.example",
      expected_san: "center.example",
      issued_at: null,
      expires_at: null,
    },
    server_tls_required: true,
    protocol_version: "v1",
    enrollment_token_id: "enroll-gw-demo-1",
  },
});

if (config.control_center_endpoint !== "https://center.example") {
  throw new Error("control center endpoint was not normalized");
}
if (config.gateway_id !== "gw-demo") {
  throw new Error("gateway id was not normalized");
}
if (config.trust_bundle?.trust_bundle_id !== "trust-gw-demo-1") {
  throw new Error("trust bundle was not normalized");
}
if (!config.server_tls_required || config.protocol_version !== "v1") {
  throw new Error("initial config contract fields were not preserved");
}

// 初始化 URL 契约：只允许 instance_id，状态地址从相同 Center 入口派生。
const target = parseGatewayInitializationUrl(
  "https://center.example/api/v1/gateway/initial-config?instance_id=gw-demo",
);
if (target.instanceId !== "gw-demo") {
  throw new Error("instance_id was not parsed from init URL");
}
if (
  target.statusUrl !==
  "https://center.example/api/v1/gateway/initialization-status?instance_id=gw-demo"
) {
  throw new Error("initialization status URL was not derived from init URL");
}

let rejectedCredentialUrl = false;
try {
  parseGatewayInitializationUrl(
    "https://center.example/api/v1/gateway/initial-config?instance_id=gw-demo&token=secret",
  );
} catch (error) {
  rejectedCredentialUrl = error instanceof GatewayInitializationInputError;
}
if (!rejectedCredentialUrl) {
  throw new Error("credentials in init URL must be rejected");
}

// 所有初始化请求中的显式 token 都只能放入 Authorization Header；initial-config 当前返回 JSON。
let fetchUrl = "";
let fetchAuth = "";
let fetchAccept = "";
globalThis.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
  fetchUrl = input.toString();
  const headers = new Headers(init?.headers);
  fetchAuth = headers.get("authorization") ?? "";
  fetchAccept = headers.get("accept") ?? "";
  return Response.json({
    config: {
      gateway_id: "gw-demo",
      control_center_endpoint: "https://center.example",
      trust_bundle: null,
      server_tls_required: false,
      protocol_version: "v1",
      enrollment_token_id: "enroll-gw-demo-1",
    },
    regist_token: "reg-token",
  });
};

const initialConfig = await fetchGatewayInitialConfig(
  target.initUrl,
  "tok demo",
);
if (fetchUrl.includes("token") || fetchUrl.includes("#")) {
  throw new Error("gateway credential must not be sent in URL");
}
if (fetchAuth !== "Bearer tok demo") {
  throw new Error("explicit token must be sent in the Authorization Header");
}
if (fetchAccept !== "application/json") {
  throw new Error("initial-config must be requested as application/json");
}
if (
  initialConfig.gateway_id !== "gw-demo" ||
  initialConfig.enrollment_token_id !== "enroll-gw-demo-1"
) {
  throw new Error("JSON GatewayInitialConfig was not returned");
}

globalThis.fetch = async (input: RequestInfo | URL, init?: RequestInit) => {
  fetchUrl = input.toString();
  fetchAuth = new Headers(init?.headers).get("authorization") ?? "";
  return Response.json({
    gateway_id: "gw-demo",
    instance_id: "gw-demo",
    lifecycle_state: "Provisioned",
    initialized: false,
  });
};
const status = await fetchGatewayInitializationStatus(
  target.statusUrl,
  "tok demo",
);
if (status.lifecycle_state !== "Provisioned" || status.initialized) {
  throw new Error("initialization status contract was not preserved");
}
if (fetchAuth !== "Bearer tok demo" || fetchUrl.includes("token")) {
  throw new Error(
    "status query must keep the credential in Authorization Header",
  );
}

let requestCount = 0;
globalThis.fetch = async (_input: RequestInfo | URL, init?: RequestInit) => {
  requestCount += 1;
  const authorization = new Headers(init?.headers).get("authorization");
  if (authorization !== "Bearer tok demo") {
    throw new Error("initialization service lost the Authorization Header");
  }
  if (requestCount === 1) {
    return Response.json({
      gateway_id: "gw-demo",
      instance_id: "gw-demo",
      lifecycle_state: "Provisioned",
      initialized: false,
    });
  }
  return Response.json({
    config: {
      gateway_id: "gw-demo",
      control_center_endpoint: "https://center.example",
      trust_bundle: null,
      server_tls_required: false,
      protocol_version: "v1",
      enrollment_token_id: "enroll-gw-demo-1",
    },
    regist_token: null,
  });
};
const initialized = await initializeGatewayViaUrl(target.initUrl, "tok demo");
if (requestCount !== 2 || initialized.config.gateway_id !== "gw-demo") {
  throw new Error(
    "initialization service must check status before fetching config",
  );
}

requestCount = 0;
globalThis.fetch = async () => {
  requestCount += 1;
  return Response.json({
    gateway_id: "gw-demo",
    instance_id: "gw-demo",
    lifecycle_state: "Running",
    initialized: true,
  });
};
let rejectedRepeat = false;
try {
  await initializeGatewayViaUrl(target.initUrl, "tok demo");
} catch (error) {
  rejectedRepeat = error instanceof GatewayAlreadyInitializedError;
}
if (!rejectedRepeat || requestCount !== 1) {
  throw new Error(
    "already initialized gateway must be rejected before config fetch",
  );
}

let rejected = false;
try {
  normalizeGatewayInitialConfig({
    config: {
      control_center_endpoint: "https://center.example",
      trust_bundle: null,
      server_tls_required: "true",
      protocol_version: "v1",
      enrollment_token_id: "enroll-gw-demo-1",
    },
  });
} catch {
  rejected = true;
}
if (!rejected) {
  throw new Error("invalid server_tls_required should be rejected");
}

console.log("gateway initialize contract test passed");

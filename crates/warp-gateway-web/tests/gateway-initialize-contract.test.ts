import {
  fetchGatewayInitialConfig,
  normalizeGatewayInitialConfig,
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

// fetchGatewayInitialConfig：显式 token 放 Authorization Header，
// 且请求地址必须剥离 fragment（token 不随 URL 发送）。
let fetchUrl = "";
let fetchAuth = "";
let fetchAccept = "";
(globalThis as any).fetch = async (url: string, init: any) => {
  fetchUrl = url;
  fetchAuth = init?.headers?.authorization ?? "";
  fetchAccept = init?.headers?.accept ?? "";
  return {
    ok: true,
    json: async () => ({
      config: {
        gateway_id: "gw-demo",
        control_center_endpoint: "https://center.example",
        trust_bundle: null,
        server_tls_required: false,
        protocol_version: "v1",
        enrollment_token_id: "enroll-gw-demo-1",
      },
    }),
    text: async () =>
      "version = 1\n[control_center]\nendpoint = \"https://center.example\"\n[enrollment]\ntoken_id = \"enroll-gw-demo-1\"\ntoken = \"reg-token\"\n",
  };
};

const toml = await fetchGatewayInitialConfig(
  "https://center.example/api/v1/gateway/initial-config?instance_id=gw-demo#gateway_token=tok%20demo",
  "tok demo",
);
if (fetchUrl.includes("gateway_token") || fetchUrl.includes("#")) {
  throw new Error("gateway token fragment must not be sent to Center");
}
if (fetchAuth !== "Bearer tok demo") {
  throw new Error("explicit token must be sent in the Authorization Header");
}
if (!toml.includes("version = 1") || !toml.includes("[enrollment]")) {
  throw new Error("config.toml text was not returned");
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

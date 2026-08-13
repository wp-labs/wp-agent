import {
  buildGatewayInitialConfigCurl,
  normalizeGatewayInitialConfig,
} from "../src/api/admin";

const config = normalizeGatewayInitialConfig({
  config: {
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
if (config.trust_bundle?.trust_bundle_id !== "trust-gw-demo-1") {
  throw new Error("trust bundle was not normalized");
}
if (!config.server_tls_required || config.protocol_version !== "v1") {
  throw new Error("initial config contract fields were not preserved");
}

const curl = buildGatewayInitialConfigCurl(
  "https://center.example/api/v1/gateway/initial-config?instance_id=gw-demo",
  "token-demo",
);
if (
  curl !==
  "curl -H 'Authorization: Bearer token-demo' 'https://center.example/api/v1/gateway/initial-config?instance_id=gw-demo'"
) {
  throw new Error(`unexpected init curl: ${curl}`);
}
if (curl.includes("?token=")) {
  throw new Error("gateway token must not be placed in the URL");
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

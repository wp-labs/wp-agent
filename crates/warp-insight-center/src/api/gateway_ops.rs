// ReceiveGatewayStatusReport 接收链路：POST /api/v1/gateway/status。
// 镜像 warp-gateway 的 submit_agent_status：Bearer 鉴权（sha256 常数时间比较）→
// store 落库最新状态 → 返回 GatewayStatusAcceptedReturned。

use std::net::SocketAddr;

use axum::{
    extract::{connect_info::ConnectInfo, Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use insight_control::{
    GatewayInitialConfig, GatewayInitialConfigReturned, GatewayStatusAccepted,
    GatewayStatusAcceptedReturned, ReportGatewayStatus,
};

use crate::infra::{
    sha256_hex, GatewayStatusUpdate, StoredAgent, StoredGateway, StoredGatewayCredentialStatus,
};

use super::{rate_limit, ApiState};

const GATEWAY_AUTH_SCOPE: &str = "gateway";

/// Gateway 上报其下 Agent 状态（POST /api/v1/gateway/agents/status）。
#[derive(serde::Deserialize)]
pub struct AgentStatusReportRequest {
    pub gateway_id: String,
    pub agents: Vec<AgentStatusEntry>,
}

#[derive(serde::Deserialize)]
pub struct AgentStatusEntry {
    pub agent_id: String,
    pub instance_id: String,
    pub version: String,
    pub status: String,
    pub health: String,
    #[serde(default)]
    pub memory_bytes: Option<i64>,
    #[serde(default)]
    pub cpu_percent: Option<f64>,
    #[serde(default)]
    pub admin_latency_ms: Option<i64>,
    pub last_seen_at: insight_control::types::DateTime,
}

/// 接收 Gateway 上报的 Agent 状态：Bearer 按 gateway_id 凭证鉴权 → store upsert → VM 推送。
pub async fn submit_agent_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Json(input): Json<AgentStatusReportRequest>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    match authenticate_gateway(&state, &headers, &input.gateway_id, &client_key).await {
        Ok(_) => {
            let stored: Vec<StoredAgent> = input
                .agents
                .iter()
                .map(|agent| StoredAgent {
                    agent_id: agent.agent_id.clone(),
                    gateway_id: input.gateway_id.clone(),
                    instance_id: agent.instance_id.clone(),
                    version: agent.version.clone(),
                    status: agent.status.clone(),
                    health: agent.health.clone(),
                    memory_bytes: agent.memory_bytes,
                    cpu_percent: agent.cpu_percent,
                    admin_latency_ms: agent.admin_latency_ms,
                    last_seen_at: agent.last_seen_at.clone(),
                })
                .collect();
            if let Err(err) = state
                .store
                .upsert_agent_status(&input.gateway_id, &stored)
                .await
            {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to update agent status: {err}"),
                )
                    .into_response();
            }
            if let Some(vm_url) = &state.config.victoriametrics_url {
                if let Err(err) = crate::infra::vm::push_agent_status(
                    vm_client(),
                    vm_url,
                    &input.gateway_id,
                    &stored,
                )
                .await
                {
                    eprintln!("warn agent_status vm push failed: {err}");
                }
            }
            (
                StatusCode::OK,
                Json(serde_json::json!({
                    "gateway_id": input.gateway_id,
                    "agents_accepted": stored.len(),
                })),
            )
                .into_response()
        }
        Err(response) => response,
    }
}

/// VM 推送全局 HTTP 客户端（进程内复用连接池）。
fn vm_client() -> &'static reqwest::Client {
    crate::infra::vm::shared_vm_client()
}

/// 下载本地镜像的制品：GET /api/v1/releases/artifact/:component/:version/:filename。
pub async fn download_release_artifact(
    State(state): State<ApiState>,
    Path((component, version, filename)): Path<(String, String, String)>,
) -> Response {
    let path = state
        .config
        .artifact_dir
        .join(&component)
        .join(&version)
        .join(&filename);
    match tokio::fs::read(&path).await {
        Ok(bytes) => (
            [(axum::http::header::CONTENT_TYPE, "application/octet-stream")],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "artifact not found").into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct InitialConfigQueryParams {
    pub instance_id: Option<String>,
}

/// 拉取网关初始配置：GET /api/v1/gateway/initial-config。
/// 对齐模型 `GetGatewayInitialConfig` entry；gateway 面 Bearer 鉴权
/// （init_url 里 instance_id = gateway_id，故按 instance_id 匹配凭证）。
pub async fn get_gateway_initial_config(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Query(params): Query<InitialConfigQueryParams>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    let Some(instance_id) = params.instance_id.as_deref() else {
        return (StatusCode::BAD_REQUEST, "missing instance_id").into_response();
    };
    match authenticate_gateway(&state, &headers, instance_id, &client_key).await {
        Ok(_) => {
            // 生命周期：Provisioned → Initializing（Gateway 首次拉取初始配置）。
            if let Err(err) = state.store.mark_gateway_initializing(instance_id).await {
                eprintln!("warn mark gateway initializing failed: {err}");
            }
            let host = state
                .config
                .public_url
                .trim_start_matches("http://")
                .trim_start_matches("https://")
                .split(':')
                .next()
                .unwrap_or("127.0.0.1");
            Json(GatewayInitialConfigReturned {
                config: GatewayInitialConfig {
                    control_center_endpoint: state.config.public_url.clone(),
                    policy_version: "policy-v1".to_string(),
                    telemetry_output: format!("otlp://{host}:4317"),
                },
            })
            .into_response()
        }
        Err(response) => response,
    }
}

pub async fn submit_gateway_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Json(input): Json<ReportGatewayStatus>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    match authenticate_gateway(&state, &headers, &input.gateway_id, &client_key).await {
        Ok(_) => {
            let accepted_at = input.reported_at.clone();
            let update = GatewayStatusUpdate {
                gateway_id: input.gateway_id.clone(),
                instance_id: input.instance_id.clone(),
                version: input.version.clone(),
                status: input.status.clone(),
                health: input.health.clone(),
                memory_bytes: input.memory_bytes,
                cpu_percent: input.cpu_percent,
                last_seen_at: accepted_at.clone(),
            };
            let update_result = state.store.upsert_gateway_status(&update).await;
            if let Err(err) = update_result {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to update gateway status: {err}"),
                )
                    .into_response();
            }
            // 时序历史：配置了 VictoriaMetrics 则推送指标（失败仅告警，不影响上报成功）。
            if let Some(vm_url) = &state.config.victoriametrics_url {
                if let Err(err) =
                    crate::infra::vm::push_gateway_status(vm_client(), vm_url, &update).await
                {
                    eprintln!("warn gateway_status vm push failed: {err}");
                }
            }
            (
                StatusCode::OK,
                Json(GatewayStatusAcceptedReturned {
                    receipt: GatewayStatusAccepted {
                        gateway_id: input.gateway_id,
                        instance_id: input.instance_id,
                        accepted_at,
                    },
                }),
            )
                .into_response()
        }
        Err(response) => response,
    }
}

/// 按 binding 的 `actor_identity WarpGateway.id from credential.gateway_id`：
/// bearer token → 匹配 store 中该 gateway 的凭证 hash（常数时间比较）→
/// Active 且未过期 → 与上报 gateway_id 一致。
async fn authenticate_gateway(
    state: &ApiState,
    headers: &HeaderMap,
    gateway_id: &str,
    client_key: &str,
) -> Result<StoredGateway, Response> {
    if let Some(response) = rate_limit::check_rate_limit(state, client_key, GATEWAY_AUTH_SCOPE) {
        return Err(response);
    }
    let Some(token) = bearer_token(headers) else {
        // 缺 token 属未认证请求，不计入暴力尝试。
        return Err((StatusCode::UNAUTHORIZED, "missing bearer credential").into_response());
    };
    let token_hash = sha256_hex(token);
    let gateway = match state.store.get_gateway(gateway_id).await {
        Ok(gateway) => gateway,
        Err(err) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway credential store: {err}"),
            )
                .into_response());
        }
    };
    let Some(gateway) = gateway else {
        rate_limit::record_auth_failure(state, client_key, GATEWAY_AUTH_SCOPE);
        return Err((StatusCode::UNAUTHORIZED, "unknown gateway credential").into_response());
    };
    // 网关身份由 `actor_identity WarpGateway.id from credential.gateway_id` 表达：
    // 这里按上报 gateway_id 查到的凭证做常数时间 token 比较，即身份一致性校验。
    if !constant_time_eq(gateway.credential_token_hash.as_bytes(), token_hash.as_bytes()) {
        rate_limit::record_auth_failure(state, client_key, GATEWAY_AUTH_SCOPE);
        return Err((StatusCode::UNAUTHORIZED, "invalid gateway credential").into_response());
    }
    if gateway.credential_status != StoredGatewayCredentialStatus::Active {
        return Err((StatusCode::UNAUTHORIZED, "gateway credential is not active").into_response());
    }
    if let Some(expires_at) = &gateway.credential_expires_at {
        if credential_is_expired(expires_at) {
            return Err((StatusCode::UNAUTHORIZED, "gateway credential is expired").into_response());
        }
    }
    rate_limit::clear_auth_failures(state, client_key, GATEWAY_AUTH_SCOPE);
    Ok(gateway)
}

fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(header::AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn credential_is_expired(expires_at: &str) -> bool {
    let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(expires_at) else {
        return true;
    };
    chrono::Utc::now() >= expires_at.with_timezone(&chrono::Utc)
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    let mut diff = left.len() ^ right.len();
    let max_len = left.len().max(right.len());
    for index in 0..max_len {
        let left_byte = left.get(index).copied().unwrap_or(0);
        let right_byte = right.get(index).copied().unwrap_or(0);
        diff |= (left_byte ^ right_byte) as usize;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request, routing::post, Router};
    use http_body_util::BodyExt;
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::ServiceExt;

    use crate::{
        api::ApiState,
        config::GatewayCredentialSeed,
        infra::FileStore,
    };

    fn test_state() -> ApiState {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-api-test-{nanos}.json"));
        let store = FileStore::new(path);
        store
            .seed(&[GatewayCredentialSeed {
                gateway_id: "gw-001".to_string(),
                token: "secret-token-1".to_string(),
                expires_at: None,
            }])
            .expect("seed");
        ApiState {
            config: crate::config::CenterConfig {
                listen_addr: "127.0.0.1:3100".to_string(),
                store_path: std::env::temp_dir().join(format!("wic-api-test-{nanos}.json")),
                gateway_credentials: Vec::new(),
                admin_token_hash: None,
                database_url: None,
                victoriametrics_url: None,
                public_url: "http://127.0.0.1:3100".to_string(),
                gateway_image: "warp-gateway:latest".to_string(),
                artifact_dir: std::env::temp_dir().join("wic-artifacts"),
                object_storage: None,
            },
            store: std::sync::Arc::new(store),
            artifact_store: std::sync::Arc::new(crate::infra::LocalArtifactStore::new(
                std::env::temp_dir().join("wic-artifacts"),
                "http://127.0.0.1:3100",
            )),
            rate_limits: std::sync::Arc::new(std::sync::Mutex::new(
                super::super::rate_limit::RateLimitState::default(),
            )),
        }
    }

    fn router() -> Router {
        let state = test_state();
        Router::new()
            .route("/api/v1/gateway/status", post(submit_gateway_status))
            .with_state(state)
    }

    fn status_payload(gateway_id: &str) -> String {
        format!(
            r#"{{"gateway_id":"{gateway_id}","instance_id":"inst-1","version":"v2.4.1","status":"online","health":"healthy","reported_at":"2026-08-07T12:00:00Z"}}"#
        )
    }

    #[tokio::test]
    async fn accepts_status_report_with_valid_credential() {
        let response = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/status")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer secret-token-1")
                    .body(Body::from(status_payload("gw-001")))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.expect("body").to_bytes();
        let returned: GatewayStatusAcceptedReturned = serde_json::from_slice(&body).expect("json");
        assert_eq!(returned.receipt.gateway_id, "gw-001");
        assert_eq!(returned.receipt.instance_id, "inst-1");
    }

    #[tokio::test]
    async fn rejects_report_with_invalid_credential() {
        let response = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/status")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer wrong-token")
                    .body(Body::from(status_payload("gw-001")))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn rejects_report_with_unknown_gateway() {
        let response = router()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/status")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer secret-token-1")
                    .body(Body::from(status_payload("gw-999")))
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

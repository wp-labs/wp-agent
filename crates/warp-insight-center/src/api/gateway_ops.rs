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
    GatewayEnrollmentResult, GatewayEnrollmentResultReturned, GatewayInitialConfig,
    GatewayInitialConfigReturned, GatewayStatusAccepted, GatewayStatusAcceptedReturned,
    RegisterGateway, ReportGatewayStatus,
};

use crate::infra::{
    sha256_hex, GatewayStatusUpdate, StoreError, StoredAgent, StoredGateway,
    StoredGatewayCredentialStatus,
};

use super::{build_control_center_trust_bundle, control_center_tls_required, rate_limit, ApiState};

const GATEWAY_AUTH_SCOPE: &str = "gateway";
/// 注册自携带 token 鉴权，无网关身份可查，独立限流桶防 token 暴力枚举。
const GATEWAY_REGISTER_SCOPE: &str = "gateway-register";

/// 允许 Gateway Web 以 Authorization Header 直接调用初始化端点。
/// 该端点不使用 Cookie，因此使用通配来源不会扩大用户会话权限；Token 仍只在 Header 中传输。
/// 处理浏览器对 Gateway 初始化请求的 Authorization 预检。
pub async fn options_gateway_initial_config() -> Response {
    StatusCode::NO_CONTENT.into_response()
}

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

/// 网关注册：POST /api/v1/gateway/register。
/// 对应模型 `RegisterGateway` + `RegisterGatewayFlow`：WarpGateway 持预共享
/// enrollment token 提交注册。消费 token（防重放/限量/吊销/过期）后签发注册回执；
/// 本迭代沿用"注册 token 即网关初始凭据"的简化，注册后同 token 作为 bearer
/// 访问 initial-config / status（后续按流程签发独立 GatewayCredentialBundle）。
pub async fn register_gateway(
    State(state): State<ApiState>,
    client: Option<ConnectInfo<SocketAddr>>,
    Json(input): Json<RegisterGateway>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    // 注册自携带 token 鉴权（无网关身份可查），独立限流桶防 token 暴力枚举。
    if let Some(response) =
        rate_limit::check_rate_limit(&state, &client_key, GATEWAY_REGISTER_SCOPE)
    {
        return response;
    }
    let consumed = match state
        .store
        .consume_enrollment_token(&input.enrollment_token)
        .await
    {
        Ok(token) => token,
        Err(StoreError::Enrollment(reason)) => {
            rate_limit::record_auth_failure(&state, &client_key, GATEWAY_REGISTER_SCOPE);
            return (
                StatusCode::UNAUTHORIZED,
                format!("enrollment token rejected: {reason}"),
            )
                .into_response();
        }
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to consume enrollment token: {err}"),
            )
                .into_response();
        }
    };
    // token 绑定的网关必须已创建。
    let gateway_exists = match state.store.get_gateway(&consumed.gateway_id).await {
        Ok(Some(_)) => true,
        Ok(None) => false,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway store: {err}"),
            )
                .into_response();
        }
    };
    if !gateway_exists {
        rate_limit::record_auth_failure(&state, &client_key, GATEWAY_REGISTER_SCOPE);
        return (
            StatusCode::UNAUTHORIZED,
            "enrollment token bound to unknown gateway".to_string(),
        )
            .into_response();
    }
    // 生命周期：Provisioned → Initializing（注册成功即进入初始化）。
    if let Err(err) = state
        .store
        .mark_gateway_initializing(&consumed.gateway_id)
        .await
    {
        eprintln!("warn mark gateway initializing failed: {err}");
    }
    rate_limit::clear_auth_failures(&state, &client_key, GATEWAY_REGISTER_SCOPE);
    Json(GatewayEnrollmentResultReturned {
        result: GatewayEnrollmentResult {
            status: "accepted".to_string(),
            gateway_id: consumed.gateway_id.clone(),
            instance_id: input.instance_id,
            credential_id: format!("cred-{}", consumed.gateway_id),
            initial_config: "v1".to_string(),
        },
    })
    .into_response()
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
            // 配置引用该网关最近签发的一个注册 Token（config.toml [enrollment] token_id）。
            let enrollment_token_id = state
                .store
                .get_enrollment_token_for_gateway(instance_id)
                .await
                .ok()
                .flatten()
                .map(|token| token.token_id)
                .unwrap_or_default();
            Json(GatewayInitialConfigReturned {
                config: GatewayInitialConfig {
                    control_center_endpoint: state.config.public_url.clone(),
                    trust_bundle: build_control_center_trust_bundle(&state.config, instance_id),
                    server_tls_required: control_center_tls_required(&state.config),
                    protocol_version: state.config.protocol_version.clone(),
                    enrollment_token_id,
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
    if !constant_time_eq(
        gateway.credential_token_hash.as_bytes(),
        token_hash.as_bytes(),
    ) {
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

    use crate::{api::ApiState, config::GatewayCredentialSeed, infra::FileStore};

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
                ca_cert: None,
                protocol_version: "1.0".to_string(),
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

    /// 构造带 seed 网关 + 注册 Token 的完整路由（register 端点走 router_for）。
    fn register_state(enrollment_token: &str) -> ApiState {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-register-{nanos}.json"));
        let store = FileStore::new(&path);
        store
            .seed(&[GatewayCredentialSeed {
                gateway_id: "gw-001".to_string(),
                token: "cred-tok".to_string(),
                expires_at: None,
            }])
            .expect("seed");
        store
            .create_enrollment_token(
                "gw-001",
                &crate::infra::EnrollmentTokenIssue {
                    token: enrollment_token.to_string(),
                    issued_by: "test".to_string(),
                    control_center_trust_bundle: None,
                },
            )
            .expect("enroll");
        ApiState {
            config: crate::config::CenterConfig {
                listen_addr: "127.0.0.1:3100".to_string(),
                store_path: std::env::temp_dir().join(format!("wic-register-{nanos}.json")),
                gateway_credentials: Vec::new(),
                admin_token_hash: None,
                database_url: None,
                victoriametrics_url: None,
                public_url: "http://127.0.0.1:3100".to_string(),
                gateway_image: "warp-gateway:latest".to_string(),
                artifact_dir: std::env::temp_dir().join("wic-artifacts"),
                object_storage: None,
                ca_cert: None,
                protocol_version: "1.0".to_string(),
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
        let body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
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

    fn register_payload(token: &str) -> String {
        format!(
            r#"{{"enrollment_token":"{token}","instance_id":"inst-1","requested_at":"2026-08-11T00:00:00Z"}}"#
        )
    }

    #[tokio::test]
    async fn register_consumes_token_once_then_rejects_replay() {
        let app = super::super::router_for(register_state("enroll-tok-a"));

        // 首次注册 → 200 accepted + 注册回执。
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/register")
                    .header("content-type", "application/json")
                    .body(Body::from(register_payload("enroll-tok-a")))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let response_body = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let returned: GatewayEnrollmentResultReturned =
            serde_json::from_slice(&response_body).expect("json");
        assert_eq!(returned.result.status, "accepted");
        assert_eq!(returned.result.gateway_id, "gw-001");
        assert_eq!(returned.result.instance_id, "inst-1");
        assert_eq!(returned.result.credential_id, "cred-gw-001");

        // 防重放：同一 token 二次注册 → 401（Exhausted）。
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/register")
                    .header("content-type", "application/json")
                    .body(Body::from(register_payload("enroll-tok-a")))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn initial_config_preflight_allows_gateway_web_authorization_header() {
        let response = super::super::router_for(register_state("enroll-tok-c"))
            .oneshot(
                Request::builder()
                    .method("OPTIONS")
                    .uri("/api/v1/gateway/initial-config?instance_id=gw-001")
                    .header("origin", "http://127.0.0.1:5174")
                    .header("access-control-request-method", "GET")
                    .header("access-control-request-headers", "authorization")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response
                .headers()
                .get("access-control-allow-origin")
                .and_then(|value| value.to_str().ok()),
            Some("*")
        );
        assert!(response
            .headers()
            .get("access-control-allow-headers")
            .expect("allow headers")
            .to_str()
            .expect("header value")
            .contains("authorization"));
    }

    #[tokio::test]
    async fn register_rejects_unknown_token() {
        let app = super::super::router_for(register_state("enroll-tok-a"));
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/register")
                    .header("content-type", "application/json")
                    .body(Body::from(register_payload("enroll-tok-unknown")))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn register_rejects_revoked_token() {
        // 构造 FileStore → seed 网关 + 签发 token → 吊销 → 再注册 → 401。
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-register-revoked-{nanos}.json"));
        let file_store = FileStore::new(&path);
        file_store
            .seed(&[GatewayCredentialSeed {
                gateway_id: "gw-001".to_string(),
                token: "cred-tok".to_string(),
                expires_at: None,
            }])
            .expect("seed");
        let token = file_store
            .create_enrollment_token(
                "gw-001",
                &crate::infra::EnrollmentTokenIssue {
                    token: "enroll-tok-b".to_string(),
                    issued_by: "test".to_string(),
                    control_center_trust_bundle: None,
                },
            )
            .expect("enroll");
        file_store
            .revoke_enrollment_token("gw-001", &token.token_id)
            .expect("revoke");
        let state = ApiState {
            config: crate::config::CenterConfig {
                listen_addr: "127.0.0.1:3100".to_string(),
                store_path: std::env::temp_dir().join(format!("wic-register-revoked-{nanos}.json")),
                gateway_credentials: Vec::new(),
                admin_token_hash: None,
                database_url: None,
                victoriametrics_url: None,
                public_url: "http://127.0.0.1:3100".to_string(),
                gateway_image: "warp-gateway:latest".to_string(),
                artifact_dir: std::env::temp_dir().join("wic-artifacts"),
                object_storage: None,
                ca_cert: None,
                protocol_version: "1.0".to_string(),
            },
            store: std::sync::Arc::new(file_store),
            artifact_store: std::sync::Arc::new(crate::infra::LocalArtifactStore::new(
                std::env::temp_dir().join("wic-artifacts"),
                "http://127.0.0.1:3100",
            )),
            rate_limits: std::sync::Arc::new(std::sync::Mutex::new(
                super::super::rate_limit::RateLimitState::default(),
            )),
        };
        let app = super::super::router_for(state);
        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/register")
                    .header("content-type", "application/json")
                    .body(Body::from(register_payload("enroll-tok-b")))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}

// 管理面接口：网关创建 + 列表聚合 / 状态卡片列表 / 单网关状态。
// 数据来自 center store（ReceiveGatewayStatusReport 落库的最新状态）。

use std::net::SocketAddr;

use axum::{
    extract::{connect_info::ConnectInfo, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use insight_control::{
    AdminGatewayInstanceReturned, AdminGatewayListReturned, AdminGatewayStatusListReturned,
    AdminGatewayStatusReturned, AgentRuntimeStatusView, GatewayInstance, GatewayListView,
    GatewayStatusView,
};
use insight_control::types::DateTime;

use crate::infra::{StoreError, StoredGateway};

use super::{admin_auth::require_admin_bearer, rate_limit, ApiState};

/// 创建网关实例请求体：对齐模型 `AdminCreateGatewayInstance`（gateway_name/requested_by），
/// 额外扩展可选 `token`（脚本传入；前端表单不传则创建无凭证网关，无法上报状态）。
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdminCreateGatewayInstanceRequest {
    pub gateway_name: String,
    pub requested_by: String,
    pub token: Option<String>,
}

/// 创建网关实例：POST /api/v1/admin/gateways/instances。
/// gateway_id 由 gateway_name 派生；重复 → 409。
pub async fn admin_create_gateway_instance(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Json(request): Json<AdminCreateGatewayInstanceRequest>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let gateway_id = request.gateway_name.trim();
    if gateway_id.is_empty() || request.requested_by.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            "gateway_name and requested_by must not be empty",
        )
            .into_response();
    }
    let token = request.token.as_deref().unwrap_or_default();
    match state.store.create_gateway(gateway_id, token).await {
        Ok(stored) => (
            StatusCode::CREATED,
            Json(AdminGatewayInstanceReturned {
                instance: GatewayInstance {
                    gateway_id: stored.gateway_id,
                    instance_id: stored.instance_id,
                    status: "active".to_string(),
                    created_at: DateTime::now(),
                },
            }),
        )
            .into_response(),
        Err(StoreError::Conflict(_)) => (
            StatusCode::CONFLICT,
            format!("gateway {gateway_id} already exists"),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to create gateway: {err}"),
        )
            .into_response(),
    }
}

/// 网关在线率查询参数（默认 1h 窗口）。
#[derive(serde::Deserialize)]
pub struct GatewayUptimeQueryParams {
    pub window: Option<String>,
}

/// 网关在线率返回（0..1；无历史数据或 VM 不可达 → None）。
#[derive(serde::Serialize)]
pub struct GatewayUptimeReturned {
    pub gateway_id: String,
    pub window: String,
    pub uptime: Option<f64>,
}

/// 网关历史查询参数；当前控制台使用 1h，保留 6h/24h 供后续切换。
#[derive(serde::Deserialize)]
pub struct GatewayHistoryQueryParams {
    pub window: Option<String>,
}

/// 网关历史接口返回，采样来自 VictoriaMetrics range query。
#[derive(serde::Serialize)]
pub struct GatewayHistoryReturned {
    pub gateway_id: String,
    pub window: String,
    pub step_seconds: i64,
    pub samples: Vec<crate::infra::vm::GatewayMetricSample>,
}

/// 单个 Agent 历史接口返回。
#[derive(serde::Serialize)]
pub struct AgentHistoryReturned {
    pub gateway_id: String,
    pub agent_id: String,
    pub window: String,
    pub step_seconds: i64,
    pub samples: Vec<crate::infra::vm::AgentMetricSample>,
}

/// 查询网关在线率：GET /api/v1/admin/gateways/:gateway_id/status/uptime。
/// 转发 VM `avg_over_time(gateway_up{gateway_id="X"}[window])`；
/// VM 未配置 / 查询失败 / 无历史样本 → uptime None（HTTP 200，列表页平滑显示"—"）。
pub async fn admin_get_gateway_uptime(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Path(gateway_id): Path<String>,
    Query(params): Query<GatewayUptimeQueryParams>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let window = params.window.as_deref().unwrap_or("1h");
    let uptime = match &state.config.victoriametrics_url {
        Some(vm_url) => {
            match crate::infra::vm::query_uptime(
                crate::infra::vm::shared_vm_client(),
                vm_url,
                &gateway_id,
                window,
            )
            .await
            {
                Ok(uptime) => uptime,
                Err(err) => {
                    eprintln!("warn gateway uptime vm query failed: {err}");
                    None
                }
            }
        }
        None => None,
    };
    Json(GatewayUptimeReturned {
        gateway_id,
        window: window.to_string(),
        uptime,
    })
    .into_response()
}

/// 查询网关最近一段时间的在线、内存和 CPU 历史。
///
/// VM 未配置或暂时不可用时返回空 samples，页面保留快照并显示历史空态。
pub async fn admin_get_gateway_history(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Path(gateway_id): Path<String>,
    Query(params): Query<GatewayHistoryQueryParams>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let window = params.window.as_deref().unwrap_or("1h");
    let Some((window_seconds, step_seconds)) = history_window_config(window) else {
        return (
            StatusCode::BAD_REQUEST,
            "window must be one of: 1h, 6h, 24h",
        )
            .into_response();
    };
    let now = chrono::Utc::now().timestamp();
    let end = now - now.rem_euclid(step_seconds);
    let start = end - window_seconds;
    let samples = match &state.config.victoriametrics_url {
        Some(vm_url) => match crate::infra::vm::query_gateway_history(
            crate::infra::vm::shared_vm_client(),
            vm_url,
            &gateway_id,
            start,
            end,
            step_seconds,
        )
        .await
        {
            Ok(samples) => samples,
            Err(err) => {
                eprintln!("warn gateway history vm query failed: {err}");
                Vec::new()
            }
        },
        None => Vec::new(),
    };
    Json(GatewayHistoryReturned {
        gateway_id,
        window: window.to_string(),
        step_seconds,
        samples,
    })
    .into_response()
}

/// 查询单个 Agent 最近一段时间的在线、内存、CPU 和管理时延历史。
pub async fn admin_get_agent_history(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Path((gateway_id, agent_id)): Path<(String, String)>,
    Query(params): Query<GatewayHistoryQueryParams>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let window = params.window.as_deref().unwrap_or("1h");
    let Some((window_seconds, step_seconds)) = history_window_config(window) else {
        return (
            StatusCode::BAD_REQUEST,
            "window must be one of: 1h, 6h, 24h",
        )
            .into_response();
    };
    let now = chrono::Utc::now().timestamp();
    let end = now - now.rem_euclid(step_seconds);
    let start = end - window_seconds;
    let samples = match &state.config.victoriametrics_url {
        Some(vm_url) => match crate::infra::vm::query_agent_history(
            crate::infra::vm::shared_vm_client(),
            vm_url,
            &gateway_id,
            &agent_id,
            start,
            end,
            step_seconds,
        )
        .await
        {
            Ok(samples) => samples,
            Err(err) => {
                eprintln!("warn agent history vm query failed: {err}");
                Vec::new()
            }
        },
        None => Vec::new(),
    };
    Json(AgentHistoryReturned {
        gateway_id,
        agent_id,
        window: window.to_string(),
        step_seconds,
        samples,
    })
    .into_response()
}

/// 将公开窗口收敛为固定秒数和采样步长，避免把任意字符串带入 PromQL。
fn history_window_config(window: &str) -> Option<(i64, i64)> {
    match window {
        "1h" => Some((60 * 60, 60)),
        "6h" => Some((6 * 60 * 60, 5 * 60)),
        "24h" => Some((24 * 60 * 60, 15 * 60)),
        _ => None,
    }
}

/// 查询某 gateway 下的 Agent 状态：GET /api/v1/admin/gateways/:gateway_id/agents。
pub async fn admin_list_gateway_agents(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Path(gateway_id): Path<String>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let agents = match state.store.list_agents_by_gateway(&gateway_id).await {
        Ok(agents) => agents,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway agents: {err}"),
            )
                .into_response();
        }
    };
    let views: Vec<AgentRuntimeStatusView> = agents
        .into_iter()
        .map(|agent| AgentRuntimeStatusView {
            agent_id: agent.agent_id,
            instance_id: agent.instance_id,
            version: agent.version,
            status: agent.status,
            health: agent.health,
            memory_bytes: agent.memory_bytes,
            cpu_percent: agent.cpu_percent,
            admin_latency_ms: agent.admin_latency_ms,
            last_seen_at: agent.last_seen_at,
        })
        .collect();
    Json(views).into_response()
}

pub async fn admin_view_gateway_list(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let gateways = match state.store.list_gateways().await {
        Ok(gateways) => gateways,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway store: {err}"),
            )
                .into_response();
        }
    };
    let (online_count, offline_count, degraded_count) = gateways.iter().fold(
        (0_i64, 0_i64, 0_i64),
        |(online, offline, degraded), stored| {
            let online = online + i64::from(stored.status.as_deref() == Some("online"));
            // 离线 = 不在线（含已上报离线与从未上报的已接入网关），保证 online + offline = gateway_count。
            let offline = offline + i64::from(stored.status.as_deref() != Some("online"));
            let degraded =
                degraded + i64::from(stored.health.as_deref() == Some("degraded"));
            (online, offline, degraded)
        },
    );
    Json(AdminGatewayListReturned {
        list: GatewayListView {
            gateway_count: gateways.len() as i64,
            online_count,
            offline_count,
            degraded_count,
            updated_at: DateTime::now(),
        },
    })
    .into_response()
}

pub async fn admin_list_gateway_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let gateways = match state.store.list_gateways().await {
        Ok(gateways) => gateways,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway store: {err}"),
            )
                .into_response();
        }
    };
    // 状态视图只展示「已上报过」的网关；从未上报的网关无状态可展示（聚合 gateway_count 仍计入）。
    let mut views: Vec<GatewayStatusView> = gateways
        .iter()
        .filter(|stored| stored.last_seen_at.is_some())
        .map(gateway_status_view)
        .collect();
    views.sort_by(|left, right| left.gateway_id.cmp(&right.gateway_id));
    Json(AdminGatewayStatusListReturned { statuses: views }).into_response()
}

pub async fn admin_show_gateway_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Path(gateway_id): Path<String>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let stored = match state.store.get_gateway(&gateway_id).await {
        Ok(stored) => stored,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway store: {err}"),
            )
                .into_response();
        }
    };
    let Some(stored) = stored else {
        return (
            StatusCode::NOT_FOUND,
            format!("unknown gateway {gateway_id}"),
        )
            .into_response();
    };
    if stored.last_seen_at.is_none() {
        // 已接入但从未上报：无状态可返回（与列表端点一致）。
        return (
            StatusCode::NOT_FOUND,
            format!("gateway {gateway_id} has not reported yet"),
        )
            .into_response();
    }
    Json(AdminGatewayStatusReturned {
        status: gateway_status_view(&stored),
    })
    .into_response()
}

/// StoredGateway → GatewayStatusView。仅对已上报网关调用（列表/单查已过滤）；
/// Option 缺省值保留为防御性兜底。
fn gateway_status_view(stored: &StoredGateway) -> GatewayStatusView {
    GatewayStatusView {
        gateway_id: stored.gateway_id.clone(),
        instance_id: stored.instance_id.clone(),
        version: stored.version.clone().unwrap_or_default(),
        status: stored.status.clone().unwrap_or_else(|| "offline".to_string()),
        health: stored.health.clone().unwrap_or_else(|| "unknown".to_string()),
        memory_bytes: stored.memory_bytes,
        cpu_percent: stored.cpu_percent,
        last_seen_at: stored
            .last_seen_at
            .clone()
            .unwrap_or_else(DateTime::now),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::{
        config::{CenterConfig, GatewayCredentialSeed},
        infra::FileStore,
    };

    fn test_store() -> FileStore {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-admin-test-{nanos}.json"));
        let store = FileStore::new(&path);
        store
            .seed(&[
                GatewayCredentialSeed {
                    gateway_id: "gw-001".to_string(),
                    token: "tok-a".to_string(),
                    expires_at: None,
                },
                GatewayCredentialSeed {
                    gateway_id: "gw-002".to_string(),
                    token: "tok-b".to_string(),
                    expires_at: None,
                },
            ])
            .expect("seed");
        store
            .update(|snapshot| {
                if let Some(gw) = snapshot.gateways.get_mut("gw-001") {
                    gw.instance_id = "inst-1".to_string();
                    gw.version = Some("v2.4.1".to_string());
                    gw.status = Some("online".to_string());
                    gw.health = Some("healthy".to_string());
                    gw.last_seen_at = Some(DateTime::now());
                }
                if let Some(gw) = snapshot.gateways.get_mut("gw-002") {
                    gw.status = Some("offline".to_string());
                    gw.health = Some("degraded".to_string());
                    gw.last_seen_at = Some(DateTime::now());
                }
            })
            .expect("status update");
        store
    }

    fn test_state() -> ApiState {
        ApiState {
            config: CenterConfig {
                listen_addr: "127.0.0.1:3100".to_string(),
                store_path: std::env::temp_dir().join("unused.json"),
                gateway_credentials: Vec::new(),
                admin_token_hash: Some(super::super::super::infra::sha256_hex("admin-tok")),
                database_url: None,
                victoriametrics_url: None,
            },
            store: std::sync::Arc::new(test_store()),
            rate_limits: std::sync::Arc::new(std::sync::Mutex::new(
                super::super::rate_limit::RateLimitState::default(),
            )),
        }
    }

    #[test]
    fn history_windows_use_bounded_steps() {
        assert_eq!(history_window_config("1h"), Some((3600, 60)));
        assert_eq!(history_window_config("6h"), Some((21_600, 300)));
        assert_eq!(history_window_config("24h"), Some((86_400, 900)));
        assert_eq!(history_window_config("7d"), None);
    }

    #[tokio::test]
    async fn history_without_vm_returns_empty_samples() {
        use axum::{body::Body, http::Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app = super::super::router_for(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/gateways/gw-001/status/history?window=1h")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.expect("body").to_bytes();
        let returned: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(returned["gateway_id"], "gw-001");
        assert_eq!(returned["window"], "1h");
        assert_eq!(returned["step_seconds"], 60);
        assert_eq!(returned["samples"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn agent_history_without_vm_returns_empty_samples() {
        use axum::{body::Body, http::Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let app = super::super::router_for(test_state());
        let response = app
            .oneshot(
                Request::builder()
                    .uri(
                        "/api/v1/admin/gateways/gw-001/agents/agent-1/history?window=1h",
                    )
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
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
        let returned: serde_json::Value = serde_json::from_slice(&body).expect("json");
        assert_eq!(returned["gateway_id"], "gw-001");
        assert_eq!(returned["agent_id"], "agent-1");
        assert_eq!(returned["samples"], serde_json::json!([]));
    }

    #[tokio::test]
    async fn never_reported_gateway_is_offline_not_listed() {
        use axum::{body::Body, http::Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-never-{nanos}.json"));
        let store = FileStore::new(&path);
        store
            .seed(&[
                GatewayCredentialSeed {
                    gateway_id: "gw-001".to_string(),
                    token: "tok-a".to_string(),
                    expires_at: None,
                },
                // gw-002 从未上报
                GatewayCredentialSeed {
                    gateway_id: "gw-002".to_string(),
                    token: "tok-b".to_string(),
                    expires_at: None,
                },
            ])
            .expect("seed");
        store
            .update(|snapshot| {
                if let Some(gw) = snapshot.gateways.get_mut("gw-001") {
                    gw.status = Some("online".to_string());
                    gw.health = Some("healthy".to_string());
                    gw.last_seen_at = Some(DateTime::now());
                }
            })
            .expect("update");
        let state = ApiState {
            config: CenterConfig {
                listen_addr: "127.0.0.1:3100".to_string(),
                store_path: std::env::temp_dir().join(format!("wic-never-{nanos}.json")),
                gateway_credentials: Vec::new(),
                admin_token_hash: Some(super::super::super::infra::sha256_hex("admin-tok")),
                database_url: None,
                victoriametrics_url: None,
            },
            store: std::sync::Arc::new(store),
            rate_limits: std::sync::Arc::new(std::sync::Mutex::new(
                super::super::rate_limit::RateLimitState::default(),
            )),
        };
        let app = super::super::router_for(state);

        // 聚合：gw-002（未上报）计入 offline，online + offline = gateway_count。
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/gateways")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let body = response.into_body().collect().await.expect("body").to_bytes();
        let list: AdminGatewayListReturned = serde_json::from_slice(&body).expect("json");
        assert_eq!(list.list.gateway_count, 2);
        assert_eq!(list.list.online_count, 1);
        assert_eq!(list.list.offline_count, 1);

        // 列表：只含已上报的 gw-001。
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/gateways/status")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        let body = response.into_body().collect().await.expect("body").to_bytes();
        let list: AdminGatewayStatusListReturned = serde_json::from_slice(&body).expect("json");
        assert_eq!(list.statuses.len(), 1);
        assert_eq!(list.statuses[0].gateway_id, "gw-001");

        // 单查：未上报的 gw-002 → 404。
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/gateways/gw-002/status")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn list_aggregates_from_store() {
        use axum::{body::Body, http::Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let state = test_state();
        let app = super::super::router_for(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/gateways")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);
        let body = response.into_body().collect().await.expect("body").to_bytes();
        let returned: AdminGatewayListReturned = serde_json::from_slice(&body).expect("json");
        assert_eq!(returned.list.gateway_count, 2);
        assert_eq!(returned.list.online_count, 1);
        assert_eq!(returned.list.offline_count, 1);
        assert_eq!(returned.list.degraded_count, 1);
    }

    #[tokio::test]
    async fn admin_auth_failures_are_rate_limited() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let state = test_state();
        let app = super::super::router_for(state);
        for _ in 0..5 {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri("/api/v1/admin/gateways/status")
                        .header("authorization", "Bearer wrong-token")
                        .body(Body::empty())
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        }
        // 第 6 次即使带正确 token 也被限流（测试环境共享 "unknown" 桶）。
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/gateways/status")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[tokio::test]
    async fn removed_list_alias_route_is_gone() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let state = test_state();
        let app = super::super::router_for(state);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/v1/admin/gateways/list")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    fn create_state_with_store(store: FileStore) -> ApiState {
        ApiState {
            config: CenterConfig {
                listen_addr: "127.0.0.1:3100".to_string(),
                store_path: std::env::temp_dir().join("unused.json"),
                gateway_credentials: Vec::new(),
                admin_token_hash: Some(super::super::super::infra::sha256_hex("admin-tok")),
                database_url: None,
                victoriametrics_url: None,
            },
            store: std::sync::Arc::new(store),
            rate_limits: std::sync::Arc::new(std::sync::Mutex::new(
                super::super::rate_limit::RateLimitState::default(),
            )),
        }
    }

    fn create_payload(gateway_name: &str) -> String {
        format!(
            r#"{{"gateway_name":"{gateway_name}","requested_by":"test","token":"tok-create"}}"#
        )
    }

    #[tokio::test]
    async fn create_gateway_instance_creates_and_conflicts() {
        use axum::{body::Body, http::Request};
        use http_body_util::BodyExt;
        use tower::ServiceExt;

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-create-{nanos}.json"));
        let state = create_state_with_store(FileStore::new(&path));
        let app = super::super::router_for(state);

        // 创建 → 201 + instance。
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/gateways/instances")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::from(create_payload("gw-create")))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::CREATED);
        let body = response.into_body().collect().await.expect("body").to_bytes();
        let returned: AdminGatewayInstanceReturned = serde_json::from_slice(&body).expect("json");
        assert_eq!(returned.instance.gateway_id, "gw-create");
        assert_eq!(returned.instance.status, "active");

        // 重复创建同一 gateway_id → 409。
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/admin/gateways/instances")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer admin-tok")
                    .body(Body::from(create_payload("gw-create")))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::CONFLICT);

        // 用创建时的 token 上报状态 → 200（创建凭证立即可用）。
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/v1/gateway/status")
                    .header("content-type", "application/json")
                    .header("authorization", "Bearer tok-create")
                    .body(Body::from(
                        r#"{"gateway_id":"gw-create","instance_id":"inst-1","version":"v2.4.1","status":"online","health":"healthy","reported_at":"2026-08-08T12:00:00Z"}"#,
                    ))
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(response.status(), StatusCode::OK);

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn create_gateway_instance_rejects_empty_name() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-create-empty-{nanos}.json"));
        let state = create_state_with_store(FileStore::new(&path));
        let app = super::super::router_for(state);

        for payload in [
            r#"{"gateway_name":"  ","requested_by":"test","token":"tok"}"#,
            r#"{"gateway_name":"gw-x","requested_by":"  ","token":"tok"}"#,
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/api/v1/admin/gateways/instances")
                        .header("content-type", "application/json")
                        .header("authorization", "Bearer admin-tok")
                        .body(Body::from(payload))
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        }

        let _ = std::fs::remove_file(path);
    }

    #[tokio::test]
    async fn read_endpoints_require_admin_token() {
        use axum::{body::Body, http::Request};
        use tower::ServiceExt;

        let state = test_state();
        let app = super::super::router_for(state);
        for uri in [
            "/api/v1/admin/gateways",
            "/api/v1/admin/gateways/status",
            "/api/v1/admin/gateways/gw-001/status",
        ] {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .uri(uri)
                        .body(Body::empty())
                        .expect("request"),
                )
                .await
                .expect("response");
            assert_eq!(
                response.status(),
                StatusCode::UNAUTHORIZED,
                "uri {uri} should require admin token"
            );
        }
    }
}

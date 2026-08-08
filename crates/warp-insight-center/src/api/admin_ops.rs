// 管理面读取接口：网关列表聚合 / 状态卡片列表 / 单网关状态。
// 数据来自 center store（ReceiveGatewayStatusReport 落库的最新状态）。

use std::net::SocketAddr;

use axum::{
    extract::{connect_info::ConnectInfo, Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use insight_control::{
    AdminGatewayListReturned, AdminGatewayStatusListReturned, AdminGatewayStatusReturned,
    GatewayListView, GatewayStatusView,
};
use insight_control::types::DateTime;

use crate::infra::StoredGateway;

use super::{admin_auth::require_admin_bearer, rate_limit, ApiState};

pub async fn admin_view_gateway_list(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    if let Err(response) = require_admin_bearer(&state, &headers, &client_key) {
        return response;
    }
    let snapshot = match state.store.load() {
        Ok(snapshot) => snapshot,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway store: {err}"),
            )
                .into_response();
        }
    };
    let gateways = snapshot.gateways;
    let (online_count, offline_count, degraded_count) = gateways.values().fold(
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
    let snapshot = match state.store.load() {
        Ok(snapshot) => snapshot,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway store: {err}"),
            )
                .into_response();
        }
    };
    // 状态视图只展示「已上报过」的网关；从未上报的网关无状态可展示（聚合 gateway_count 仍计入）。
    let mut views: Vec<GatewayStatusView> = snapshot
        .gateways
        .values()
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
    let snapshot = match state.store.load() {
        Ok(snapshot) => snapshot,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("failed to load gateway store: {err}"),
            )
                .into_response();
        }
    };
    let Some(stored) = snapshot.gateways.get(&gateway_id) else {
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
        status: gateway_status_view(stored),
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
        infra::CenterStore,
    };

    fn test_store() -> CenterStore {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-admin-test-{nanos}.json"));
        let store = CenterStore::new(&path);
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
            },
            store: test_store(),
            rate_limits: std::sync::Arc::new(std::sync::Mutex::new(
                super::super::rate_limit::RateLimitState::default(),
            )),
        }
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
        let store = CenterStore::new(&path);
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
            },
            store,
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

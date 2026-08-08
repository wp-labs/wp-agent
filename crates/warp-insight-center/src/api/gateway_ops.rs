// ReceiveGatewayStatusReport 接收链路：POST /api/v1/gateway/status。
// 镜像 warp-gateway 的 submit_agent_status：Bearer 鉴权（sha256 常数时间比较）→
// store 落库最新状态 → 返回 GatewayStatusAcceptedReturned。

use std::net::SocketAddr;

use axum::{
    extract::{connect_info::ConnectInfo, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};

use insight_control::{
    GatewayStatusAccepted, GatewayStatusAcceptedReturned, ReportGatewayStatus,
};

use crate::infra::{sha256_hex, StoredGateway, StoredGatewayCredentialStatus};

use super::{rate_limit, ApiState};

const GATEWAY_AUTH_SCOPE: &str = "gateway";

pub async fn submit_gateway_status(
    State(state): State<ApiState>,
    headers: HeaderMap,
    client: Option<ConnectInfo<SocketAddr>>,
    Json(input): Json<ReportGatewayStatus>,
) -> Response {
    let client_key = rate_limit::client_key(client);
    match authenticate_gateway(&state, &headers, &input.gateway_id, &client_key) {
        Ok(_) => {
            let accepted_at = input.reported_at.clone();
            let instance_id = input.instance_id.clone();
            let update_result = state.store.update(|snapshot| {
                if let Some(stored) = snapshot.gateways.get_mut(&input.gateway_id) {
                    stored.instance_id = instance_id;
                    stored.version = Some(input.version.clone());
                    stored.status = Some(input.status.clone());
                    stored.health = Some(input.health.clone());
                    stored.last_seen_at = Some(accepted_at.clone());
                }
            });
            if let Err(err) = update_result {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("failed to update gateway status: {err}"),
                )
                    .into_response();
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
fn authenticate_gateway(
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
    let snapshot = state.store.load().map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("failed to load gateway credential store: {err}"),
        )
            .into_response()
    })?;
    let Some(gateway) = snapshot.gateways.get(gateway_id) else {
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
    Ok(gateway.clone())
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
        infra::CenterStore,
    };

    fn test_state() -> ApiState {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("wic-api-test-{nanos}.json"));
        let store = CenterStore::new(path);
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
            },
            store,
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

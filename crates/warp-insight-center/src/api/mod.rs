// WarpInsightCenter 管理面 / 网关面 HTTP API。

use std::sync::{Arc, Mutex};

use axum::{routing::get, routing::post, Router};

use crate::{config::CenterConfig, infra::CenterStore};

mod admin_auth;
mod admin_ops;
mod gateway_ops;
mod rate_limit;

use admin_ops::{
    admin_list_gateway_status, admin_show_gateway_status, admin_view_gateway_list,
};
use gateway_ops::submit_gateway_status;

#[derive(Debug, Clone)]
pub struct ApiState {
    pub config: CenterConfig,
    pub store: CenterStore,
    pub rate_limits: Arc<Mutex<rate_limit::RateLimitState>>,
}

pub fn router(config: CenterConfig, store: CenterStore) -> Router {
    router_for(ApiState {
        config,
        store,
        rate_limits: Arc::new(Mutex::new(rate_limit::RateLimitState::default())),
    })
}

/// 以给定 state 构建路由（测试与运行共用）。
pub fn router_for(state: ApiState) -> Router {
    Router::new()
        // 网关面：ReceiveGatewayStatusReport
        .route("/api/v1/gateway/status", post(submit_gateway_status))
        // 管理面：网关列表聚合（AdminViewGatewayList，GET /api/v1/admin/gateways）
        .route("/api/v1/admin/gateways", get(admin_view_gateway_list))
        // 管理面：状态卡片列表（AdminListGatewayStatus）
        .route(
            "/api/v1/admin/gateways/status",
            get(admin_list_gateway_status),
        )
        // 管理面：单网关状态（AdminShowGatewayStatus）
        .route(
            "/api/v1/admin/gateways/:gateway_id/status",
            get(admin_show_gateway_status),
        )
        .with_state(state)
}

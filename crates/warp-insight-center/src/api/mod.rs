// WarpInsightCenter 管理面 / 网关面 HTTP API。

use std::sync::{Arc, Mutex};

use axum::{routing::get, routing::post, Router};

use crate::{config::CenterConfig, infra::Store};

mod admin_auth;
mod admin_ops;
mod gateway_ops;
mod rate_limit;

use admin_ops::{
    admin_create_gateway_instance, admin_get_agent_history, admin_get_gateway_history,
    admin_get_gateway_uptime, admin_list_gateway_agents, admin_list_gateway_status,
    admin_show_gateway_status, admin_view_gateway_list,
};
use gateway_ops::{submit_agent_status, submit_gateway_status};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub config: CenterConfig,
    pub store: Arc<dyn Store>,
    pub rate_limits: Arc<Mutex<rate_limit::RateLimitState>>,
}

pub fn router(config: CenterConfig, store: Arc<dyn Store>) -> Router {
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
        // 网关面：Gateway 上报其下 Agent 状态
        .route("/api/v1/gateway/agents/status", post(submit_agent_status))
        // 管理面：创建网关实例（AdminCreateGatewayInstance，POST /api/v1/admin/gateways/instances）
        .route(
            "/api/v1/admin/gateways/instances",
            post(admin_create_gateway_instance),
        )
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
        // 管理面：网关在线率（转发 VM avg_over_time）
        .route(
            "/api/v1/admin/gateways/:gateway_id/status/uptime",
            get(admin_get_gateway_uptime),
        )
        // 管理面：网关历史趋势（转发 VM query_range）
        .route(
            "/api/v1/admin/gateways/:gateway_id/status/history",
            get(admin_get_gateway_history),
        )
        // 管理面：单 Agent 历史趋势（转发 VM query_range）
        .route(
            "/api/v1/admin/gateways/:gateway_id/agents/:agent_id/history",
            get(admin_get_agent_history),
        )
        // 管理面：某 gateway 下的 Agent 状态列表
        .route(
            "/api/v1/admin/gateways/:gateway_id/agents",
            get(admin_list_gateway_agents),
        )
        .with_state(state)
}

// WarpInsightCenter 管理面 / 网关面 HTTP API。

use std::sync::{Arc, Mutex};

use axum::{routing::get, routing::post, Router};

use insight_control::ControlCenterTrustBundle;

use crate::{config::CenterConfig, infra::ArtifactStore, infra::Store};

mod admin_auth;
mod admin_ops;
mod gateway_ops;
mod rate_limit;

use admin_ops::{
    admin_create_gateway_instance, admin_get_agent_history, admin_get_gateway_history,
    admin_get_gateway_uptime, admin_list_gateway_agents, admin_list_gateway_instances,
    admin_approve_upgrade_plan, admin_bind_gateway_customer, admin_create_upgrade_plan,
    admin_get_gateway_initial_config, admin_list_gateway_lifecycle, admin_list_gateway_status,
    admin_list_releases, admin_list_upgrade_plans, admin_publish_release,
    admin_revoke_gateway_enrollment_token, admin_show_gateway_status, admin_view_gateway_list,
};
use gateway_ops::{
    download_release_artifact, get_gateway_initial_config, register_gateway, submit_agent_status,
    submit_gateway_status,
};

#[derive(Debug, Clone)]
pub struct ApiState {
    pub config: CenterConfig,
    pub store: Arc<dyn Store>,
    pub artifact_store: Arc<dyn ArtifactStore>,
    pub rate_limits: Arc<Mutex<rate_limit::RateLimitState>>,
}

/// 从 center 配置构造控制中心信任包：`ca_cert`（PEM）→ `ca_bundle`（公钥），
/// `public_url` → 端点 / server_name / expected_san。未配置 CA → None。
pub(crate) fn build_control_center_trust_bundle(
    config: &CenterConfig,
    gateway_id: &str,
) -> Option<ControlCenterTrustBundle> {
    let ca_bundle = config.ca_cert.clone()?;
    let host = config
        .public_url
        .trim_start_matches("http://")
        .trim_start_matches("https://")
        .split(':')
        .next()
        .unwrap_or_default()
        .to_string();
    Some(ControlCenterTrustBundle {
        trust_bundle_id: format!("trust-{gateway_id}-1"),
        control_endpoint: config.public_url.clone(),
        ca_bundle,
        server_name: host.clone(),
        expected_san: host,
        issued_at: None,
        expires_at: None,
    })
}

pub fn router(config: CenterConfig, store: Arc<dyn Store>) -> Router {
    let artifact_store = crate::infra::build_artifact_store(&config);
    router_for(ApiState {
        config,
        store,
        artifact_store,
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
        // 网关面：WarpGateway 持注册 Token 注册（RegisterGatewayFlow）
        .route("/api/v1/gateway/register", post(register_gateway))
        // 网关面：Gateway 拉取初始配置（初始化 URL 指向此端点）
        .route(
            "/api/v1/gateway/initial-config",
            get(get_gateway_initial_config),
        )
        // 管理面：创建网关实例（AdminCreateGatewayInstance，POST /api/v1/admin/gateways/instances）
        .route(
            "/api/v1/admin/gateways/instances",
            post(admin_create_gateway_instance),
        )
        // 管理面：实例列表（含生命周期状态）
        .route(
            "/api/v1/admin/gateways/instances",
            get(admin_list_gateway_instances),
        )
        // 管理面：绑定网关到客户
        .route(
            "/api/v1/admin/gateways/bind",
            post(admin_bind_gateway_customer),
        )
        // 管理面：吊销网关注册 Token（GatewayEnrollmentTokenStatus.Revoked）
        .route(
            "/api/v1/admin/gateways/:gateway_id/enrollment-tokens/:token_id/revoke",
            post(admin_revoke_gateway_enrollment_token),
        )
        // 管理面：实例初始配置（admin 侧查询）
        .route(
            "/api/v1/admin/gateways/instances/:instance_id/config",
            get(admin_get_gateway_initial_config),
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
        // 管理面：某 gateway 生命周期转变历史
        .route(
            "/api/v1/admin/gateways/:gateway_id/lifecycle",
            get(admin_list_gateway_lifecycle),
        )
        // 管理面：版本发布（warp-agentd / warp-gateway，镜像外部制品）
        .route(
            "/api/v1/admin/releases/:component",
            post(admin_publish_release).get(admin_list_releases),
        )
        // 制品下载（本地镜像）
        .route(
            "/api/v1/releases/artifact/:component/:version/:filename",
            get(download_release_artifact),
        )
        // 管理面：升级计划（创建/列表/批准，多目标+范围+多步执行）
        .route(
            "/api/v1/admin/upgrade-plans",
            post(admin_create_upgrade_plan).get(admin_list_upgrade_plans),
        )
        .route(
            "/api/v1/admin/upgrade-plans/approve",
            post(admin_approve_upgrade_plan),
        )
        .with_state(state)
}

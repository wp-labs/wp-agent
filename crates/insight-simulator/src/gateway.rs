// Gateway 角色：向 WarpInsightCenter 上报状态 / 拉取初始配置。

use insight_control::{
    GatewayInitialConfigReturned, GetGatewayInitialConfig, ReportGatewayStatus,
};
use insight_control::types::DateTime;
use reqwest::StatusCode;

use crate::{client, config::SimConfig};

/// 上报网关状态：POST {center}/api/v1/gateway/status。
pub async fn report_gateway_status(
    client: &reqwest::Client,
    config: &SimConfig,
) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/gateway/status",
        config.upstream_url.trim_end_matches('/')
    );
    let report = ReportGatewayStatus {
        gateway_id: config.id.clone(),
        instance_id: config.instance_id.clone(),
        version: config.version.clone(),
        status: config
            .status
            .clone()
            .unwrap_or_else(|| "online".to_string()),
        health: config
            .health
            .clone()
            .unwrap_or_else(|| "healthy".to_string()),
        // 模拟 gateway 自身运行指标（2GB 内存、~35% CPU）。
        memory_bytes: Some(2 * 1024 * 1024 * 1024),
        cpu_percent: Some(35.0),
        reported_at: DateTime::now(),
    };
    let response = client::send_json(|| client.post(&url).json(&report), &config.token).await?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "gateway status report rejected: HTTP {status} body={}",
            response.text().await.unwrap_or_default()
        ));
    }
    println!(
        "event=GatewayStatusReported status={status} gateway_id={} instance_id={}",
        config.id, config.instance_id
    );
    Ok(())
}

/// 上报其下 2 个模拟 Agent 状态：POST {center}/api/v1/gateway/agents/status。
pub async fn report_agents_status(
    client: &reqwest::Client,
    config: &SimConfig,
) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/gateway/agents/status",
        config.upstream_url.trim_end_matches('/')
    );
    let now = DateTime::now().to_chrono().to_rfc3339();
    let request = serde_json::json!({
        "gateway_id": config.id,
        "agents": [
            {
                "agent_id": format!("{}-agent-1", config.id),
                "instance_id": format!("inst-{}-a1", config.id),
                "version": "v0.3.2",
                "status": "online",
                "health": "healthy",
                "memory_bytes": 512 * 1024 * 1024,
                "cpu_percent": 20.0,
                "admin_latency_ms": 8,
                "last_seen_at": now,
            },
            {
                "agent_id": format!("{}-agent-2", config.id),
                "instance_id": format!("inst-{}-a2", config.id),
                "version": "v0.3.0",
                "status": "online",
                "health": "degraded",
                "memory_bytes": 384 * 1024 * 1024,
                "cpu_percent": 45.0,
                "admin_latency_ms": 15,
                "last_seen_at": now,
            },
        ],
    });
    let response = client::send_json(|| client.post(&url).json(&request), &config.token).await?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!(
            "agent status report rejected: HTTP {status} body={}",
            response.text().await.unwrap_or_default()
        ));
    }
    println!("event=AgentsReported status={status} gateway_id={}", config.id);
    Ok(())
}

/// 拉取网关初始配置：GET {center}/api/v1/gateway/initial-config。
/// 注意：中心暂未实现该端点，会返回 404（调用方自行处理）。
pub async fn fetch_initial_config(
    client: &reqwest::Client,
    config: &SimConfig,
) -> Result<(), String> {
    let url = format!(
        "{}/api/v1/gateway/initial-config",
        config.upstream_url.trim_end_matches('/')
    );
    // initial-config 按 gateway 凭证鉴权，instance_id 参数用 gateway_id（与创建时生成的 init_url 一致）。
    let request = GetGatewayInitialConfig {
        instance_id: config.id.clone(),
        requested_at: DateTime::now(),
    };
    let response = client::send_json(
        || client.get(&url).query(&[("instance_id", request.instance_id.as_str())]),
        &config.token,
    )
    .await?;
    if response.status() == StatusCode::NOT_FOUND {
        return Err("initial config endpoint not implemented on center (HTTP 404)".to_string());
    }
    if !response.status().is_success() {
        return Err(format!(
            "initial config fetch rejected: HTTP {}",
            response.status()
        ));
    }
    let returned: GatewayInitialConfigReturned = response
        .json()
        .await
        .map_err(|err| format!("failed to decode initial config response: {err}"))?;
    println!(
        "event=InitialConfigFetched endpoint={} tls_required={} protocol={} trust_bundle_id={} ca_bundle_len={}",
        returned.config.control_center_endpoint,
        returned.config.server_tls_required,
        returned.config.protocol_version,
        returned
            .config
            .trust_bundle
            .as_ref()
            .map(|bundle| bundle.trust_bundle_id.as_str())
            .unwrap_or("<none>"),
        returned
            .config
            .trust_bundle
            .as_ref()
            .map(|bundle| bundle.ca_bundle.len())
            .unwrap_or(0),
    );
    Ok(())
}

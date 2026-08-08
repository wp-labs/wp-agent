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
    let request = GetGatewayInitialConfig {
        instance_id: config.instance_id.clone(),
        requested_at: DateTime::now(),
    };
    let response = client::send_json(|| client.get(&url).json(&request), &config.token).await?;
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
        "event=InitialConfigFetched endpoint={} policy={} telemetry={}",
        returned.config.control_center_endpoint,
        returned.config.policy_version,
        returned.config.telemetry_output
    );
    Ok(())
}

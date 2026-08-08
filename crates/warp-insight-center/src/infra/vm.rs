// VictoriaMetrics 时序推送：把网关状态上报转成 Prometheus 文本写 /api/v1/import/prometheus。
// 属于独立外部系统（不属于 Store 抽象）：push 失败仅记日志，不阻塞上报主流程。

use std::time::Duration;

use reqwest::Client;

use super::{GatewayStatusUpdate, StoredAgent};

/// VM 推送 HTTP 客户端：短超时，push 失败快速返回。
pub fn build_vm_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(2))
        .connect_timeout(Duration::from_secs(1))
        .build()
        .unwrap_or_else(|_| Client::new())
}

/// 进程内共享 VM 客户端（连接池复用）。
pub fn shared_vm_client() -> &'static Client {
    static VM_CLIENT: std::sync::OnceLock<Client> = std::sync::OnceLock::new();
    VM_CLIENT.get_or_init(build_vm_client)
}

/// 推送一次状态上报到 VictoriaMetrics（三行 Prometheus 文本，时间戳 = 上报时间毫秒）。
/// 非 2xx 视为失败；调用方只用于日志。
pub async fn push_gateway_status(
    client: &Client,
    base_url: &str,
    update: &GatewayStatusUpdate,
) -> Result<(), String> {
    let body = render_prometheus_lines(update);
    let url = format!(
        "{}/api/v1/import/prometheus",
        base_url.trim_end_matches('/')
    );
    let response = client
        .post(&url)
        .header("content-type", "text/plain")
        .body(body)
        .send()
        .await
        .map_err(|err| format!("vm push request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "vm push rejected: HTTP {} body={}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }
    Ok(())
}

/// 推送 Gateway 上报的 Agent 状态到 VM（每 agent 三行：agent_up / agent_info / agent_health）。
pub async fn push_agent_status(
    client: &Client,
    base_url: &str,
    gateway_id: &str,
    agents: &[StoredAgent],
) -> Result<(), String> {
    let body = agents
        .iter()
        .map(|agent| render_agent_lines(gateway_id, agent))
        .collect::<String>();
    let url = format!(
        "{}/api/v1/import/prometheus",
        base_url.trim_end_matches('/')
    );
    let response = client
        .post(&url)
        .header("content-type", "text/plain")
        .body(body)
        .send()
        .await
        .map_err(|err| format!("vm agent push request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "vm agent push rejected: HTTP {} body={}",
            response.status(),
            response.text().await.unwrap_or_default()
        ));
    }
    Ok(())
}

/// 查询网关在线率：转发 VM `avg_over_time(gateway_up{gateway_id="X"}[window])`。
/// 无数据 / VM 不可达 → None（列表页平滑显示"—"）；多个 instance series 取平均。
pub async fn query_uptime(
    client: &Client,
    base_url: &str,
    gateway_id: &str,
    window: &str,
) -> Result<Option<f64>, String> {
    let url = format!("{}/api/v1/query", base_url.trim_end_matches('/'));
    let query = format!(
        "avg_over_time(gateway_up{{gateway_id=\"{}\"}}[{window}])",
        escape_label(gateway_id)
    );
    let response = client
        .get(&url)
        .query(&[("query", query)])
        .send()
        .await
        .map_err(|err| format!("vm query request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!("vm query rejected: HTTP {}", response.status()));
    }
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|err| format!("failed to decode vm query response: {err}"))?;
    let results = payload
        .get("data")
        .and_then(|data| data.get("result"))
        .and_then(serde_json::Value::as_array);
    let Some(results) = results else {
        return Ok(None);
    };
    let mut total = 0.0;
    let mut count = 0_usize;
    for series in results {
        let value = series
            .get("value")
            .and_then(|value| value.as_array())
            .and_then(|pair| pair.get(1))
            .and_then(serde_json::Value::as_str);
        if let Some(value) = value.and_then(|text| text.parse::<f64>().ok()) {
            total += value;
            count += 1;
        }
    }
    if count == 0 {
        return Ok(None);
    }
    Ok(Some(total / count as f64))
}

/// 构造 Prometheus 文本：
/// - gateway_up：status=online → 1，否则 0（可算 uptime、告警）；
/// - gateway_info / gateway_health：info-style，version/health 作为 label。
fn render_prometheus_lines(update: &GatewayStatusUpdate) -> String {
    let ts = update.last_seen_at.to_chrono().timestamp_millis();
    let up = if update.status == "online" { 1 } else { 0 };
    let gateway_id = escape_label(&update.gateway_id);
    let instance_id = escape_label(&update.instance_id);
    let version = escape_label(&update.version);
    let health = escape_label(&update.health);
    format!(
        "gateway_up{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\"}} {up} {ts}\n\
         gateway_info{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\",version=\"{version}\"}} 1 {ts}\n\
         gateway_health{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\",health=\"{health}\"}} 1 {ts}\n"
    )
}

/// 构造 agent 的 Prometheus 文本三行（agent_up=online→1 / agent_info / agent_health）。
fn render_agent_lines(gateway_id: &str, agent: &StoredAgent) -> String {
    let ts = agent.last_seen_at.to_chrono().timestamp_millis();
    let up = if agent.status == "online" { 1 } else { 0 };
    let gateway_id = escape_label(gateway_id);
    let agent_id = escape_label(&agent.agent_id);
    let version = escape_label(&agent.version);
    let health = escape_label(&agent.health);
    format!(
        "agent_up{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\"}} {up} {ts}\n\
         agent_info{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\",version=\"{version}\"}} 1 {ts}\n\
         agent_health{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\",health=\"{health}\"}} 1 {ts}\n"
    )
}

/// Prometheus 文本 label 值转义：`\` → `\\`，`"` → `\"`，换行 → `\n`。
fn escape_label(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use insight_control::types::DateTime;

    fn update_with(status: &str, health: &str) -> GatewayStatusUpdate {
        GatewayStatusUpdate {
            gateway_id: "gw-001".to_string(),
            instance_id: "inst-1".to_string(),
            version: "v2.4.1".to_string(),
            status: status.to_string(),
            health: health.to_string(),
            last_seen_at: DateTime::from_rfc3339("2026-08-08T12:00:00Z").expect("ts"),
        }
    }

    #[test]
    fn renders_up_info_health_lines() {
        let update = update_with("online", "healthy");
        let text = render_prometheus_lines(&update);
        let ts = update.last_seen_at.to_chrono().timestamp_millis();

        assert!(text.starts_with(&format!(
            "gateway_up{{gateway_id=\"gw-001\",instance_id=\"inst-1\"}} 1 {ts}"
        )));
        assert!(text.contains(&format!(
            "gateway_info{{gateway_id=\"gw-001\",instance_id=\"inst-1\",version=\"v2.4.1\"}} 1 {ts}"
        )));
        assert!(text.contains(&format!(
            "gateway_health{{gateway_id=\"gw-001\",instance_id=\"inst-1\",health=\"healthy\"}} 1 {ts}"
        )));
    }

    #[test]
    fn offline_maps_to_zero() {
        let text = render_prometheus_lines(&update_with("offline", "degraded"));
        assert!(text.starts_with(
            "gateway_up{gateway_id=\"gw-001\",instance_id=\"inst-1\"} 0 "
        ));
        assert!(text.contains("gateway_health{gateway_id=\"gw-001\",instance_id=\"inst-1\",health=\"degraded\"}"));
    }

    #[test]
    fn escapes_label_values() {
        assert_eq!(escape_label(r#"a"b\c"#), r#"a\"b\\c"#);
        assert_eq!(escape_label("a\nb"), "a\\nb");
    }

    #[test]
    fn renders_agent_lines() {
        let agent = StoredAgent {
            agent_id: "agent-1".to_string(),
            gateway_id: "gw-x".to_string(),
            instance_id: "inst-a".to_string(),
            version: "v0.3.2".to_string(),
            status: "online".to_string(),
            health: "healthy".to_string(),
            last_seen_at: DateTime::from_rfc3339("2026-08-08T12:00:00Z").expect("ts"),
        };
        let text = render_agent_lines("gw-x", &agent);
        let ts = agent.last_seen_at.to_chrono().timestamp_millis();
        assert!(text.starts_with(&format!(
            "agent_up{{agent_id=\"agent-1\",gateway_id=\"gw-x\"}} 1 {ts}"
        )));
        assert!(text.contains(&format!(
            "agent_info{{agent_id=\"agent-1\",gateway_id=\"gw-x\",version=\"v0.3.2\"}} 1 {ts}"
        )));
        assert!(text.contains(&format!(
            "agent_health{{agent_id=\"agent-1\",gateway_id=\"gw-x\",health=\"healthy\"}} 1 {ts}"
        )));

        // offline → agent_up 0。
        let offline = StoredAgent { status: "offline".to_string(), ..agent };
        assert!(render_agent_lines("gw-x", &offline).starts_with(
            "agent_up{agent_id=\"agent-1\",gateway_id=\"gw-x\"} 0 "
        ));
    }
}

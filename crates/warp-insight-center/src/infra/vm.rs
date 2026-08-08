// VictoriaMetrics 时序推送：把网关状态上报转成 Prometheus 文本写 /api/v1/import/prometheus。
// 属于独立外部系统（不属于 Store 抽象）：push 失败仅记日志，不阻塞上报主流程。

use std::{collections::BTreeMap, time::Duration};

use reqwest::Client;

use super::{GatewayStatusUpdate, StoredAgent};

/// 网关在一个采样时刻的历史指标；字段缺失表示该指标在 VM 中没有样本。
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct GatewayMetricSample {
    pub at: i64,
    pub online: Option<f64>,
    pub memory_bytes: Option<f64>,
    pub cpu_percent: Option<f64>,
}

/// Agent 在一个采样时刻的历史指标。
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AgentMetricSample {
    pub at: i64,
    pub online: Option<f64>,
    pub memory_bytes: Option<f64>,
    pub cpu_percent: Option<f64>,
    pub admin_latency_ms: Option<f64>,
}

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

/// 查询网关在指定时间范围内的在线、内存和 CPU 序列，并按时间戳合并。
///
/// PromQL 先按 gateway 聚合多个 instance series，确保前端收到一条稳定时间线。
pub async fn query_gateway_history(
    client: &Client,
    base_url: &str,
    gateway_id: &str,
    start: i64,
    end: i64,
    step_seconds: i64,
) -> Result<Vec<GatewayMetricSample>, String> {
    let gateway_id = escape_label(gateway_id);
    let online_query = format!("avg(gateway_up{{gateway_id=\"{gateway_id}\"}})");
    let memory_query = format!("avg(gateway_memory_bytes{{gateway_id=\"{gateway_id}\"}})");
    let cpu_query = format!("avg(gateway_cpu_percent{{gateway_id=\"{gateway_id}\"}})");

    let (online, memory, cpu) = tokio::try_join!(
        query_range_metric(client, base_url, &online_query, start, end, step_seconds),
        query_range_metric(client, base_url, &memory_query, start, end, step_seconds),
        query_range_metric(client, base_url, &cpu_query, start, end, step_seconds),
    )?;

    Ok(merge_gateway_history(online, memory, cpu))
}

/// 查询单个 Agent 的在线、内存、CPU 和管理时延历史。
pub async fn query_agent_history(
    client: &Client,
    base_url: &str,
    gateway_id: &str,
    agent_id: &str,
    start: i64,
    end: i64,
    step_seconds: i64,
) -> Result<Vec<AgentMetricSample>, String> {
    let gateway_id = escape_label(gateway_id);
    let agent_id = escape_label(agent_id);
    let online_query = format!(
        "avg(agent_up{{gateway_id=\"{gateway_id}\",agent_id=\"{agent_id}\"}})"
    );
    let memory_query = format!(
        "avg(agent_memory_bytes{{gateway_id=\"{gateway_id}\",agent_id=\"{agent_id}\"}})"
    );
    let cpu_query = format!(
        "avg(agent_cpu_percent{{gateway_id=\"{gateway_id}\",agent_id=\"{agent_id}\"}})"
    );
    let latency_query = format!(
        "avg(agent_admin_latency_ms{{gateway_id=\"{gateway_id}\",agent_id=\"{agent_id}\"}})"
    );

    let (online, memory, cpu, latency) = tokio::try_join!(
        query_range_metric(client, base_url, &online_query, start, end, step_seconds),
        query_range_metric(client, base_url, &memory_query, start, end, step_seconds),
        query_range_metric(client, base_url, &cpu_query, start, end, step_seconds),
        query_range_metric(client, base_url, &latency_query, start, end, step_seconds),
    )?;

    Ok(merge_agent_history(online, memory, cpu, latency))
}

/// 执行单条 VM range query，并收敛为时间戳和值的序列。
async fn query_range_metric(
    client: &Client,
    base_url: &str,
    query: &str,
    start: i64,
    end: i64,
    step_seconds: i64,
) -> Result<Vec<(i64, f64)>, String> {
    let url = format!("{}/api/v1/query_range", base_url.trim_end_matches('/'));
    let response = client
        .get(&url)
        .query(&[
            ("query", query.to_string()),
            ("start", start.to_string()),
            ("end", end.to_string()),
            ("step", step_seconds.to_string()),
        ])
        .send()
        .await
        .map_err(|err| format!("vm range query request failed: {err}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "vm range query rejected: HTTP {}",
            response.status()
        ));
    }
    let payload: serde_json::Value = response
        .json()
        .await
        .map_err(|err| format!("failed to decode vm range query response: {err}"))?;
    Ok(parse_range_values(&payload))
}

/// 解析 Prometheus matrix 响应；异常点按缺失处理，不影响同批其他样本。
fn parse_range_values(payload: &serde_json::Value) -> Vec<(i64, f64)> {
    let mut values = Vec::new();
    let Some(results) = payload
        .get("data")
        .and_then(|data| data.get("result"))
        .and_then(serde_json::Value::as_array)
    else {
        return values;
    };
    for series in results {
        let Some(samples) = series.get("values").and_then(serde_json::Value::as_array) else {
            continue;
        };
        for sample in samples {
            let Some(pair) = sample.as_array() else {
                continue;
            };
            let timestamp = pair.first().and_then(serde_json::Value::as_f64);
            let value = pair
                .get(1)
                .and_then(serde_json::Value::as_str)
                .and_then(|text| text.parse::<f64>().ok());
            if let (Some(timestamp), Some(value)) = (timestamp, value) {
                if value.is_finite() {
                    values.push((timestamp.round() as i64, value));
                }
            }
        }
    }
    values
}

/// 把三条独立指标序列合并成按时间升序的历史采样。
fn merge_gateway_history(
    online: Vec<(i64, f64)>,
    memory: Vec<(i64, f64)>,
    cpu: Vec<(i64, f64)>,
) -> Vec<GatewayMetricSample> {
    let mut samples = BTreeMap::<i64, GatewayMetricSample>::new();
    for (at, value) in online {
        samples
            .entry(at)
            .or_insert_with(|| empty_gateway_sample(at))
            .online = Some(value);
    }
    for (at, value) in memory {
        samples
            .entry(at)
            .or_insert_with(|| empty_gateway_sample(at))
            .memory_bytes = Some(value);
    }
    for (at, value) in cpu {
        samples
            .entry(at)
            .or_insert_with(|| empty_gateway_sample(at))
            .cpu_percent = Some(value);
    }
    samples.into_values().collect()
}

fn merge_agent_history(
    online: Vec<(i64, f64)>,
    memory: Vec<(i64, f64)>,
    cpu: Vec<(i64, f64)>,
    latency: Vec<(i64, f64)>,
) -> Vec<AgentMetricSample> {
    let mut samples = BTreeMap::<i64, AgentMetricSample>::new();
    for (at, value) in online {
        samples
            .entry(at)
            .or_insert_with(|| empty_agent_sample(at))
            .online = Some(value);
    }
    for (at, value) in memory {
        samples
            .entry(at)
            .or_insert_with(|| empty_agent_sample(at))
            .memory_bytes = Some(value);
    }
    for (at, value) in cpu {
        samples
            .entry(at)
            .or_insert_with(|| empty_agent_sample(at))
            .cpu_percent = Some(value);
    }
    for (at, value) in latency {
        samples
            .entry(at)
            .or_insert_with(|| empty_agent_sample(at))
            .admin_latency_ms = Some(value);
    }
    samples.into_values().collect()
}

fn empty_gateway_sample(at: i64) -> GatewayMetricSample {
    GatewayMetricSample {
        at,
        online: None,
        memory_bytes: None,
        cpu_percent: None,
    }
}

fn empty_agent_sample(at: i64) -> AgentMetricSample {
    AgentMetricSample {
        at,
        online: None,
        memory_bytes: None,
        cpu_percent: None,
        admin_latency_ms: None,
    }
}

/// 构造 Prometheus 文本：
/// - gateway_up：status=online → 1，否则 0（可算 uptime、告警）；
/// - gateway_info / gateway_health：info-style，version/health 作为 label；
/// - gateway_memory_bytes / gateway_cpu_percent：指标（可选，有值才推）。
fn render_prometheus_lines(update: &GatewayStatusUpdate) -> String {
    let ts = update.last_seen_at.to_chrono().timestamp_millis();
    let up = if update.status == "online" { 1 } else { 0 };
    let gateway_id = escape_label(&update.gateway_id);
    let instance_id = escape_label(&update.instance_id);
    let version = escape_label(&update.version);
    let health = escape_label(&update.health);
    let mut text = format!(
        "gateway_up{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\"}} {up} {ts}\n\
         gateway_info{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\",version=\"{version}\"}} 1 {ts}\n\
         gateway_health{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\",health=\"{health}\"}} 1 {ts}\n"
    );
    if let Some(memory) = update.memory_bytes {
        text.push_str(&format!(
            "gateway_memory_bytes{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\"}} {memory} {ts}\n"
        ));
    }
    if let Some(cpu) = update.cpu_percent {
        text.push_str(&format!(
            "gateway_cpu_percent{{gateway_id=\"{gateway_id}\",instance_id=\"{instance_id}\"}} {cpu} {ts}\n"
        ));
    }
    text
}

/// 构造 agent 的 Prometheus 文本（agent_up=online→1 / agent_info / agent_health / 指标可选）。
fn render_agent_lines(gateway_id: &str, agent: &StoredAgent) -> String {
    let ts = agent.last_seen_at.to_chrono().timestamp_millis();
    let up = if agent.status == "online" { 1 } else { 0 };
    let gateway_id = escape_label(gateway_id);
    let agent_id = escape_label(&agent.agent_id);
    let version = escape_label(&agent.version);
    let health = escape_label(&agent.health);
    let mut text = format!(
        "agent_up{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\"}} {up} {ts}\n\
         agent_info{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\",version=\"{version}\"}} 1 {ts}\n\
         agent_health{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\",health=\"{health}\"}} 1 {ts}\n"
    );
    if let Some(memory) = agent.memory_bytes {
        text.push_str(&format!(
            "agent_memory_bytes{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\"}} {memory} {ts}\n"
        ));
    }
    if let Some(cpu) = agent.cpu_percent {
        text.push_str(&format!(
            "agent_cpu_percent{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\"}} {cpu} {ts}\n"
        ));
    }
    if let Some(latency) = agent.admin_latency_ms {
        text.push_str(&format!(
            "agent_admin_latency_ms{{agent_id=\"{agent_id}\",gateway_id=\"{gateway_id}\"}} {latency} {ts}\n"
        ));
    }
    text
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
            memory_bytes: Some(536_870_912),
            cpu_percent: Some(21.5),
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
            memory_bytes: Some(268_435_456),
            cpu_percent: Some(33.0),
            admin_latency_ms: Some(12),
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

    #[test]
    fn parses_and_merges_range_samples_by_timestamp() {
        let payload = serde_json::json!({
            "data": {
                "result": [{
                    "values": [[1000.0, "1"], [1060.0, "0.5"], [1120.0, "NaN"]]
                }]
            }
        });
        let online = parse_range_values(&payload);
        assert_eq!(online, vec![(1000, 1.0), (1060, 0.5)]);

        let samples = merge_gateway_history(
            online,
            vec![(1000, 536_870_912.0), (1120, 545_259_520.0)],
            vec![(1060, 32.5)],
        );
        assert_eq!(samples.len(), 3);
        assert_eq!(samples[0].at, 1000);
        assert_eq!(samples[0].online, Some(1.0));
        assert_eq!(samples[0].memory_bytes, Some(536_870_912.0));
        assert_eq!(samples[1].cpu_percent, Some(32.5));
        assert_eq!(samples[2].memory_bytes, Some(545_259_520.0));
    }

    #[test]
    fn merges_agent_history_with_latency_series() {
        let samples = merge_agent_history(
            vec![(1000, 1.0)],
            vec![(1000, 268_435_456.0)],
            vec![(1000, 20.0)],
            vec![(1000, 8.0)],
        );
        assert_eq!(samples, vec![AgentMetricSample {
            at: 1000,
            online: Some(1.0),
            memory_bytes: Some(268_435_456.0),
            cpu_percent: Some(20.0),
            admin_latency_ms: Some(8.0),
        }]);
    }
}

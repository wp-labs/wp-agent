//! Minimal metrics sample view built from runtime snapshot outcomes.

use std::io;
use std::path::Path;

use serde::{Deserialize, Serialize};
use warp_insight_contracts::discovery::StringKeyValue;
use warp_insight_shared::fs::write_json_atomic;

use super::runtime::{MetricsCollectionOutcome, MetricsRuntimeSnapshot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsSamplesSnapshot {
    pub generated_at: String,
    #[serde(default)]
    pub samples: Vec<MetricsSampleRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsSampleRecord {
    pub metric_name: String,
    pub value: MetricsSampleValue,
    pub value_type: String,
    pub unit: String,
    pub collection_kind: String,
    pub target_ref: String,
    pub resource_ref: Option<String>,
    #[serde(default)]
    pub metric_attributes: Vec<StringKeyValue>,
    #[serde(default)]
    pub resource_attributes: Vec<StringKeyValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum MetricsSampleValue {
    I64(i64),
    F64(String),
    Text(String),
}

pub fn build_samples_snapshot(runtime: &MetricsRuntimeSnapshot) -> MetricsSamplesSnapshot {
    let mut samples = Vec::new();

    for outcome in &runtime.outcomes {
        samples.extend(build_outcome_samples(outcome));
    }

    MetricsSamplesSnapshot {
        generated_at: runtime.generated_at.clone(),
        samples,
    }
}

pub fn path_for(state_dir: &Path) -> std::path::PathBuf {
    state_dir.join("telemetry").join("metrics_samples.json")
}

pub fn store(path: &Path, snapshot: &MetricsSamplesSnapshot) -> io::Result<()> {
    write_json_atomic(path, snapshot)
}

fn build_outcome_samples(outcome: &MetricsCollectionOutcome) -> Vec<MetricsSampleRecord> {
    let mut samples = Vec::new();

    for target in &outcome.sample_targets {
        let resource_ref = Some(target.resource_ref.clone());
        let mut metric_attributes = vec![StringKeyValue::new("sample.status", &target.status)];
        if let Some(error) = &target.last_error {
            metric_attributes.push(StringKeyValue::new("sample.last_error", error));
        }
        let resource_attributes = build_resource_attributes(outcome, target);

        for fact in &target.runtime_facts {
            let Some((metric_name, unit, value_type)) =
                map_runtime_fact_to_metric(&outcome.collection_kind, fact.key.as_str())
            else {
                continue;
            };
            let mut attrs = metric_attributes.clone();
            attrs.extend(build_metric_attributes(target));
            samples.push(MetricsSampleRecord {
                metric_name: metric_name.to_string(),
                value: sample_value(value_type, &fact.value),
                value_type: value_type.to_string(),
                unit: unit.to_string(),
                collection_kind: outcome.collection_kind.clone(),
                target_ref: target.target_ref.clone(),
                resource_ref: resource_ref.clone(),
                metric_attributes: attrs,
                resource_attributes: resource_attributes.clone(),
            });
        }
    }

    samples
}

fn sample_value(value_type: &str, raw: &str) -> MetricsSampleValue {
    match value_type {
        "gauge_i64" => raw
            .parse::<i64>()
            .map(MetricsSampleValue::I64)
            .unwrap_or_else(|_| MetricsSampleValue::Text(raw.to_string())),
        "gauge_f64" => {
            if raw.parse::<f64>().is_ok() {
                MetricsSampleValue::F64(raw.to_string())
            } else {
                MetricsSampleValue::Text(raw.to_string())
            }
        }
        _ => MetricsSampleValue::Text(raw.to_string()),
    }
}

fn build_metric_attributes(
    target: &super::runtime::MetricsCollectionTargetSample,
) -> Vec<StringKeyValue> {
    target
        .execution_hints
        .iter()
        .filter(|hint| {
            matches!(
                hint.key.as_str(),
                "process.identity" | "container.runtime" | "container.runtime.namespace"
            )
        })
        .cloned()
        .collect()
}

fn build_resource_attributes(
    outcome: &MetricsCollectionOutcome,
    target: &super::runtime::MetricsCollectionTargetSample,
) -> Vec<StringKeyValue> {
    let mut attrs = vec![StringKeyValue::new("resource.id", &target.resource_ref)];
    attrs.push(StringKeyValue::new(
        "collection.kind",
        &outcome.collection_kind,
    ));
    for hint in &target.execution_hints {
        if matches!(
            hint.key.as_str(),
            "process.pid"
                | "pid"
                | "k8s.namespace.name"
                | "k8s.pod.uid"
                | "k8s.pod.name"
                | "k8s.container.name"
        ) {
            attrs.push(hint.clone());
        }
    }
    attrs
}

fn map_runtime_fact_to_metric(
    collection_kind: &str,
    key: &str,
) -> Option<(&'static str, &'static str, &'static str)> {
    match (collection_kind, key) {
        ("host_metrics", "host.target.count") => Some(("system.target.count", "1", "gauge_i64")),
        ("host_metrics", "host.loadavg.1m") => Some(("system.load_average.1m", "1", "gauge_f64")),
        ("host_metrics", "host.loadavg.5m") => Some(("system.load_average.5m", "1", "gauge_f64")),
        ("host_metrics", "host.loadavg.15m") => Some(("system.load_average.15m", "1", "gauge_f64")),
        ("host_metrics", "host.uptime.seconds") => Some(("system.uptime", "s", "gauge_f64")),
        ("host_metrics", "host.memory.total_kb") => {
            Some(("system.memory.total", "KiBy", "gauge_i64"))
        }
        ("host_metrics", "host.memory.available_kb") => {
            Some(("system.memory.available", "KiBy", "gauge_i64"))
        }
        ("process_metrics", "process.cpu.user_ticks") => {
            Some(("process.cpu.time.user", "ticks", "gauge_i64"))
        }
        ("process_metrics", "process.cpu.system_ticks") => {
            Some(("process.cpu.time.system", "ticks", "gauge_i64"))
        }
        ("process_metrics", "process.memory.rss_pages") => {
            Some(("process.memory.rss", "pages", "gauge_i64"))
        }
        ("process_metrics", "process.memory.rss_kb") => {
            Some(("process.memory.rss", "KiBy", "gauge_i64"))
        }
        ("process_metrics", "process.state") => Some(("process.state", "state", "gauge_string")),
        ("container_metrics", "process.cpu.user_ticks") => {
            Some(("container.cpu.time.user", "ticks", "gauge_i64"))
        }
        ("container_metrics", "process.cpu.system_ticks") => {
            Some(("container.cpu.time.system", "ticks", "gauge_i64"))
        }
        ("container_metrics", "process.memory.rss_pages") => {
            Some(("container.memory.rss", "pages", "gauge_i64"))
        }
        ("container_metrics", "process.memory.rss_kb") => {
            Some(("container.memory.rss", "KiBy", "gauge_i64"))
        }
        ("container_metrics", "process.state") => {
            Some(("container.state", "state", "gauge_string"))
        }
        ("container_metrics", "container.pid") => Some(("container.pid", "1", "gauge_i64")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use warp_insight_contracts::discovery::StringKeyValue;
    use warp_insight_shared::fs::read_json;

    use super::{
        MetricsSampleRecord, MetricsSampleValue, MetricsSamplesSnapshot, build_samples_snapshot,
        path_for, store,
    };
    use crate::telemetry::metrics::runtime::{
        MetricsCollectionOutcome, MetricsCollectionTargetSample, MetricsRuntimeSnapshot,
    };

    fn temp_dir(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("duration")
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("warp-insight-metrics-samples-{name}-{suffix}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn build_samples_snapshot_maps_runtime_facts_into_sample_records() {
        let runtime = MetricsRuntimeSnapshot {
            generated_at: "2026-04-19T00:00:00Z".to_string(),
            total_targets: 1,
            host_targets: 1,
            process_targets: 0,
            container_targets: 0,
            outcomes: vec![MetricsCollectionOutcome {
                collection_kind: "host_metrics".to_string(),
                status: "succeeded".to_string(),
                attempted_targets: 1,
                succeeded_targets: 1,
                failed_targets: 0,
                last_error: None,
                runtime_facts: vec![StringKeyValue::new("host.loadavg.1m", "0.25")],
                sample_targets: vec![MetricsCollectionTargetSample {
                    candidate_id: "host-1".to_string(),
                    target_ref: "host-1:host".to_string(),
                    status: "succeeded".to_string(),
                    last_error: None,
                    resource_ref: "host-1".to_string(),
                    execution_hints: vec![StringKeyValue::new("host.name", "local-host")],
                    runtime_facts: vec![
                        StringKeyValue::new("host.loadavg.1m", "0.25"),
                        StringKeyValue::new("host.uptime.seconds", "3600"),
                    ],
                }],
            }],
        };

        let snapshot = build_samples_snapshot(&runtime);

        assert_eq!(snapshot.generated_at, "2026-04-19T00:00:00Z");
        assert_eq!(snapshot.samples.len(), 2);
        assert!(snapshot.samples.iter().any(|sample| {
            sample.metric_name == "system.load_average.1m"
                && sample.value == MetricsSampleValue::F64("0.25".to_string())
                && sample.value_type == "gauge_f64"
                && sample.target_ref == "host-1:host"
        }));
        assert!(snapshot.samples.iter().any(|sample| {
            sample.metric_name == "system.uptime"
                && sample.unit == "s"
                && sample
                    .resource_attributes
                    .iter()
                    .any(|attr| attr.key == "resource.id" && attr.value == "host-1")
        }));
    }

    #[test]
    fn store_samples_snapshot_round_trip() {
        let state_dir = temp_dir("store");
        let snapshot = MetricsSamplesSnapshot {
            generated_at: "2026-04-19T00:00:00Z".to_string(),
            samples: vec![MetricsSampleRecord {
                metric_name: "system.uptime".to_string(),
                value: MetricsSampleValue::F64("42".to_string()),
                value_type: "gauge_f64".to_string(),
                unit: "s".to_string(),
                collection_kind: "host_metrics".to_string(),
                target_ref: "host-1:host".to_string(),
                resource_ref: Some("host-1".to_string()),
                metric_attributes: vec![StringKeyValue::new("sample.status", "succeeded")],
                resource_attributes: vec![StringKeyValue::new("resource.id", "host-1")],
            }],
        };
        let samples_path = path_for(&state_dir);

        store(&samples_path, &snapshot).expect("store samples snapshot");
        let loaded: MetricsSamplesSnapshot =
            read_json(&samples_path).expect("load samples snapshot");

        assert_eq!(loaded, snapshot);
    }
}

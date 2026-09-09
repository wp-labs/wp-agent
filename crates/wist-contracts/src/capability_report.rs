//! `CapabilityReport` contract types.

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityReportContract {
    pub schema_version: String,
    pub agent_id: String,
    pub instance_id: String,
    pub reported_at: String,
    pub exec: ExecCapabilities,
    pub metrics: MetricsCapabilities,
    pub logs: Option<LogsCapabilities>,
    pub upgrade: UpgradeCapabilities,
    pub limits: CapabilityLimits,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityReportSections {
    pub agent_id: String,
    pub instance_id: String,
    pub reported_at: String,
    pub exec: ExecCapabilities,
    pub metrics: MetricsCapabilities,
    pub logs: Option<LogsCapabilities>,
    pub upgrade: UpgradeCapabilities,
    pub limits: CapabilityLimits,
}

impl CapabilityReportContract {
    pub fn new(sections: CapabilityReportSections) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            agent_id: sections.agent_id,
            instance_id: sections.instance_id,
            reported_at: sections.reported_at,
            exec: sections.exec,
            metrics: sections.metrics,
            logs: sections.logs,
            upgrade: sections.upgrade,
            limits: sections.limits,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecCapabilities {
    #[serde(default)]
    pub opcodes: Vec<String>,
    #[serde(default)]
    pub execution_profiles: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricsCapabilities {
    #[serde(default)]
    pub collectors: Vec<String>,
    #[serde(default)]
    pub scrapers: Vec<String>,
    #[serde(default)]
    pub receivers: Vec<String>,
    #[serde(default)]
    pub discovery_modes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogsCapabilities {
    #[serde(default)]
    pub file_inputs: Vec<String>,
    #[serde(default)]
    pub parsers: Vec<String>,
    #[serde(default)]
    pub multiline_modes: Vec<String>,
    #[serde(default)]
    pub watcher_modes: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpgradeCapabilities {
    pub supported: bool,
    #[serde(default)]
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityLimits {
    pub max_running_actions: Option<u32>,
    pub max_stdout_bytes: Option<u64>,
    pub max_stderr_bytes: Option<u64>,
    pub max_memory_bytes: Option<u64>,
    pub max_metrics_targets: Option<u32>,
}

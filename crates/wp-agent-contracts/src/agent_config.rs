//! `AgentConfig` contract types.

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfigContract {
    pub schema_version: String,
    pub agent: AgentSection,
    pub control_plane: ControlPlaneSection,
    pub paths: PathsSection,
    pub execution: ExecutionSection,
    #[serde(default)]
    pub telemetry: TelemetrySection,
}

impl AgentConfigContract {
    pub fn new(
        agent: AgentSection,
        control_plane: ControlPlaneSection,
        paths: PathsSection,
        execution: ExecutionSection,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            agent,
            control_plane,
            paths,
            execution,
            telemetry: TelemetrySection::default(),
        }
    }

    pub fn with_telemetry(mut self, telemetry: TelemetrySection) -> Self {
        self.telemetry = telemetry;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSection {
    pub agent_id: Option<String>,
    pub environment_id: Option<String>,
    pub instance_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlPlaneSection {
    pub enabled: bool,
    pub endpoint: Option<String>,
    pub tls_mode: Option<String>,
    pub auth_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathsSection {
    pub root_dir: String,
    pub run_dir: String,
    pub state_dir: String,
    pub log_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSection {
    pub max_running_actions: u32,
    pub cancel_grace_ms: u64,
    pub default_stdout_limit_bytes: u64,
    pub default_stderr_limit_bytes: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TelemetrySection {
    #[serde(default)]
    pub logs: LogsSection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogsSection {
    #[serde(default)]
    pub file_inputs: Vec<LogFileInputSection>,
    #[serde(default = "default_logs_buffer_bytes")]
    pub in_memory_buffer_bytes: u64,
    #[serde(default = "default_logs_spool_dir")]
    pub spool_dir: String,
    #[serde(default = "default_logs_output_file")]
    pub output_file: String,
}

impl Default for LogsSection {
    fn default() -> Self {
        Self {
            file_inputs: Vec::new(),
            in_memory_buffer_bytes: default_logs_buffer_bytes(),
            spool_dir: default_logs_spool_dir(),
            output_file: default_logs_output_file(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LogFileInputSection {
    pub input_id: String,
    pub path: String,
    #[serde(default = "default_startup_position")]
    pub startup_position: String,
    #[serde(default = "default_multiline_mode")]
    pub multiline_mode: String,
}

fn default_logs_buffer_bytes() -> u64 {
    1_048_576
}

fn default_logs_spool_dir() -> String {
    "state/spool/logs".to_string()
}

fn default_logs_output_file() -> String {
    "log/warp-parse-records.ndjson".to_string()
}

fn default_multiline_mode() -> String {
    "none".to_string()
}

fn default_startup_position() -> String {
    "head".to_string()
}

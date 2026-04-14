//! `AgentConfig` contract types.

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfigContract {
    pub schema_version: String,
    #[serde(default)]
    pub agent: AgentSection,
    #[serde(default)]
    pub control_plane: ControlPlaneSection,
    #[serde(default)]
    pub paths: PathsSection,
    #[serde(default)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentSection {
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub environment_id: Option<String>,
    #[serde(default)]
    pub instance_name: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ControlPlaneSection {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub endpoint: Option<String>,
    #[serde(default)]
    pub tls_mode: Option<String>,
    #[serde(default)]
    pub auth_mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PathsSection {
    #[serde(default = "default_root_dir")]
    pub root_dir: String,
    #[serde(default = "default_run_dir")]
    pub run_dir: String,
    #[serde(default = "default_state_dir")]
    pub state_dir: String,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
}

impl Default for PathsSection {
    fn default() -> Self {
        Self {
            root_dir: default_root_dir(),
            run_dir: default_run_dir(),
            state_dir: default_state_dir(),
            log_dir: default_log_dir(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSection {
    #[serde(default = "default_max_running_actions")]
    pub max_running_actions: u32,
    #[serde(default = "default_cancel_grace_ms")]
    pub cancel_grace_ms: u64,
    #[serde(default = "default_stdout_limit_bytes")]
    pub default_stdout_limit_bytes: u64,
    #[serde(default = "default_stderr_limit_bytes")]
    pub default_stderr_limit_bytes: u64,
}

impl Default for ExecutionSection {
    fn default() -> Self {
        Self {
            max_running_actions: default_max_running_actions(),
            cancel_grace_ms: default_cancel_grace_ms(),
            default_stdout_limit_bytes: default_stdout_limit_bytes(),
            default_stderr_limit_bytes: default_stderr_limit_bytes(),
        }
    }
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

fn default_root_dir() -> String {
    ".".to_string()
}

fn default_run_dir() -> String {
    "run".to_string()
}

fn default_state_dir() -> String {
    "state".to_string()
}

fn default_log_dir() -> String {
    "log".to_string()
}

fn default_max_running_actions() -> u32 {
    1
}

fn default_cancel_grace_ms() -> u64 {
    5_000
}

fn default_stdout_limit_bytes() -> u64 {
    1_048_576
}

fn default_stderr_limit_bytes() -> u64 {
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

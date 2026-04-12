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
        }
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

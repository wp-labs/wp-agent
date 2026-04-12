//! `AgentConfig` contract types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentConfigContract {
    pub schema_version: String,
    pub agent: AgentSection,
    pub control_plane: ControlPlaneSection,
    pub paths: PathsSection,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentSection {
    pub agent_id: Option<String>,
    pub environment_id: Option<String>,
    pub instance_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlPlaneSection {
    pub enabled: bool,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathsSection {
    pub root_dir: String,
    pub run_dir: String,
    pub state_dir: String,
    pub log_dir: String,
}

//! Execution-related local state contract types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRuntimeState {
    pub schema_version: String,
    pub agent_id: String,
    pub instance_id: String,
    pub version: String,
    pub mode: RuntimeMode,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    Normal,
    Degraded,
    Protect,
    UpgradeInProgress,
}

//! `CapabilityReport` contract types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CapabilityReportContract {
    pub schema_version: String,
    pub agent_id: String,
    pub instance_id: String,
    pub exec: ExecCapabilities,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecCapabilities {
    pub opcodes: Vec<String>,
    pub execution_profiles: Vec<String>,
}

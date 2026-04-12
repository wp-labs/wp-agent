//! `CapabilityReport` contract types.

use serde::{Deserialize, Serialize};

use crate::SCHEMA_VERSION_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CapabilityReportContract {
    pub schema_version: String,
    pub agent_id: String,
    pub instance_id: String,
    pub exec: ExecCapabilities,
}

impl CapabilityReportContract {
    pub fn new(agent_id: String, instance_id: String, exec: ExecCapabilities) -> Self {
        Self {
            schema_version: SCHEMA_VERSION_V1.to_string(),
            agent_id,
            instance_id,
            exec,
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

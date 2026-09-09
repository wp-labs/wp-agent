// @jumo generated
// @jumo hash=b046700e50f48336

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Registry")]
pub struct AgentInitialConfig {
    pub gateway_endpoint: String,
    pub schema_version: String,
    pub telemetry_output: String,
    pub policy_version: String,
    pub mode: String,
}

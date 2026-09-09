// @jumo generated
// @jumo hash=002fd7b00bba6934

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Registry")]
pub struct AgentPolicyBinding {
    pub bound_at: crate::control::types::DateTime,
    pub policy_id: String,
    pub agent_id: String,
    pub policy_version: String,
}

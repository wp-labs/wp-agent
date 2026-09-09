// @jumo generated
// @jumo hash=cfb5641c937ad6f8

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentCredentialVerificationResult {
    pub agent_id: String,
    pub instance_id: String,
    pub reason_code: String,
    pub status: crate::control::types::AgentCredentialVerificationStatus,
    pub verified_at: crate::control::types::DateTime,
}

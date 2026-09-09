// @jumo generated
// @jumo hash=4c52643461d17585

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentCredentialVerified {
    pub agent_id: String,
    pub instance_id: String,
    pub verified_at: String,
}

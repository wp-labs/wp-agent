// @jumo generated
// @jumo hash=a1b7cd54c5a25543

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentCredentialRejected {
    pub agent_id: String,
    pub instance_id: String,
    pub reason_code: String,
    pub rejected_at: String,
}

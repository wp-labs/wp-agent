// @jumo generated
// @jumo hash=3b97dd40e692bd73

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentCredentialRevoked {
    pub agent_id: String,
    pub instance_id: String,
    pub reason_code: String,
    pub revoked_at: String,
}

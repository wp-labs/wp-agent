// @jumo generated
// @jumo hash=2ddd5bd704f3783b

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentIdentity {
    pub tenant_id: String,
    pub expires_at: crate::control::types::DateTime,
    pub status: crate::control::types::AgentIdentityStatus,
    pub instance_id: String,
    pub environment_id: String,
    #[jumo(unique)]
    pub agent_id: String,
    pub issued_at: crate::control::types::DateTime,
    pub node_id: String,
}

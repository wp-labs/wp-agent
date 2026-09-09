// @jumo generated
// @jumo hash=cec6e68eee826022

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentCredentialBundle {
    pub not_before: crate::control::types::DateTime,
    pub not_after: crate::control::types::DateTime,
    pub certificate: String,
    pub auth_scheme: String,
    pub bearer_token: String,
    pub ca_bundle: String,
    pub instance_id: String,
    pub issued_at: crate::control::types::DateTime,
    #[jumo(unique)]
    pub credential_id: String,
    pub private_key_ref: String,
    pub agent_id: String,
}

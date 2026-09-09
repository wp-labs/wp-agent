// @jumo generated
// @jumo hash=762a98664f991f1c

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Command")]
pub struct DispatchReceipt {
    pub created_at: crate::control::types::DateTime,
    pub command_id: String,
    pub dispatch_id: String,
    pub status: String,
    pub agent_id: String,
}

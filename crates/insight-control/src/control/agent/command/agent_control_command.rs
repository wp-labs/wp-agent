// @jumo generated
// @jumo hash=3176176b86aa633d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Command")]
pub struct AgentControlCommand {
    pub requested_by: String,
    pub command_kind: String,
    pub issued_at: crate::control::types::DateTime,
    pub target_version: String,
    pub sequence: i64,
    pub expires_at: crate::control::types::DateTime,
    #[jumo(unique)]
    pub command_id: String,
    pub payload: String,
    pub agent_id: String,
}

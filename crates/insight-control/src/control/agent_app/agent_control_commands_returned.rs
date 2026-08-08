// @moju generated
// @moju hash=14428160952fb811

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "response", domain = "Control", module = "Control.AgentApp.FacingInterface")]
pub struct AgentControlCommandsReturned {
    pub messages: Vec<crate::control::types::AgentControlCommand>,
    pub next_sequence: i64,
    pub agent_id: String,
    pub returned_at: crate::control::types::DateTime,
    pub instance_id: String,
}

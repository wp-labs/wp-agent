// @jumo generated
// @jumo hash=ec6451692d1963f3

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Status")]
pub struct AgentInstance {
    pub started_at: crate::control::types::DateTime,
    pub last_seen_at: crate::control::types::DateTime,
    pub agent_id: String,
    #[jumo(unique)]
    pub instance_id: String,
    pub version: String,
    pub boot_id: String,
}

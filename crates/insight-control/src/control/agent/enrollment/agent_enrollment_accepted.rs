// @jumo generated
// @jumo hash=7b44dd21b2a1e06d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentEnrollmentAccepted {
    pub agent_id: String,
    pub instance_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub node_id: String,
    pub accepted_at: String,
}

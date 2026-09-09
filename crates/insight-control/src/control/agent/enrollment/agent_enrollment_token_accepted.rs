// @jumo generated
// @jumo hash=49d79d5f2eecbd47

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentEnrollmentTokenAccepted {
    pub token_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub node_id: String,
    pub accepted_at: String,
}

//! Gateway envelope contract types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentHello {
    pub agent_id: String,
    pub instance_id: String,
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionPlanAck {
    pub action_id: String,
    pub execution_id: String,
    pub ack_status: AckStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AckStatus {
    Accepted,
    Rejected,
    Queued,
}

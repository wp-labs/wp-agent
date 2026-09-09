// @jumo generated
// @jumo hash=d8862d2eaef92bea

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentEnrollmentTokenRejected {
    pub token_id: String,
    pub node_id: String,
    pub reason_code: String,
    pub rejected_at: String,
}

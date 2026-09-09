// @jumo generated
// @jumo hash=0941aef4756feb6b

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "event", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentEnrollmentRejected {
    pub token_id: String,
    pub node_id: String,
    pub reason_code: String,
    pub rejected_at: String,
}

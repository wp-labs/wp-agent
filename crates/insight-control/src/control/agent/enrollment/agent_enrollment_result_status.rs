// @jumo generated
// @jumo hash=ab50a295bb1e44bf

#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Agent.Enrollment")]
pub enum AgentEnrollmentResultStatus {
    Accepted,
    Rejected,
    PendingReview,
}

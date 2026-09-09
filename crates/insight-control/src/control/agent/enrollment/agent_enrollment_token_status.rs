// @jumo generated
// @jumo hash=29d2bb84737948e5

#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Agent.Enrollment")]
pub enum AgentEnrollmentTokenStatus {
    Active,
    Expired,
    Revoked,
    Exhausted,
}

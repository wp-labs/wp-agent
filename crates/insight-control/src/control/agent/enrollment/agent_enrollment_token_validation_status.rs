// @jumo generated
// @jumo hash=552bb414a3656898

#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Agent.Enrollment")]
pub enum AgentEnrollmentTokenValidationStatus {
    Valid,
    HashMismatch,
    Expired,
    Revoked,
    Exhausted,
    EnvironmentMismatch,
    HostNotAllowed,
}

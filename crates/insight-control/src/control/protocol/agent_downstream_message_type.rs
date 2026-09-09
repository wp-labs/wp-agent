// @jumo generated
// @jumo hash=20efa995a3fe7976

#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Protocol")]
pub enum AgentDownstreamMessageType {
    EnrollmentResult,
    ControlCommands,
    PolicyRefreshHint,
    IdentityRotationHint,
}

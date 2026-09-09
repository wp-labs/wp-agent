// @jumo generated
// @jumo hash=08f280facbcbf272

#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Protocol")]
pub enum AgentUpstreamMessageType {
    EnrollmentRequest,
    StatusReport,
    CommandPoll,
    ActionResult,
}

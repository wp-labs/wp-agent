// @jumo generated
// @jumo hash=bce5b039157990bd

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Control", module = "Control.AgentApp.FacingInterface")]
pub struct AgentEnrollmentResultReturned {
    pub result: crate::control::types::AgentEnrollmentResult,
}

// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Control", module = "Control.GatewayApp.FacingInterface")]
pub struct GatewayEnrollmentResultReturned {
    pub result: crate::GatewayEnrollmentResult,
}

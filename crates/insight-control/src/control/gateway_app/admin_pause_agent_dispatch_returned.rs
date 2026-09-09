// @jumo generated
// @jumo hash=b0e3c3c00014a16d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminPauseAgentDispatchReturned {
    pub result: crate::control::types::DispatchReceipt,
}

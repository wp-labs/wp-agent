// @moju generated
// @moju hash=b0e3c3c00014a16d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "response", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminPauseAgentDispatchReturned {
    pub result: crate::control::types::DispatchReceipt,
}

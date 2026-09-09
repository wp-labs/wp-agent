// @jumo generated
// @jumo hash=93f24d584f2af3ab

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminUpgradeAgentDispatchReturned {
    pub result: crate::control::types::DispatchReceipt,
}

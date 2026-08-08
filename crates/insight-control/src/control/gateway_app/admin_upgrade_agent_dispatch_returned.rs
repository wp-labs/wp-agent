// @moju generated
// @moju hash=93f24d584f2af3ab

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "response", domain = "Control", module = "Control.GatewayApp.UserFacingInterface")]
pub struct AdminUpgradeAgentDispatchReturned {
    pub result: crate::control::types::DispatchReceipt,
}

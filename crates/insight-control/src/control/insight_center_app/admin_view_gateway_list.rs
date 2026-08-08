// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenterApp.AdminFacingInterface")]
pub struct AdminViewGatewayList {
    pub requested_by: String,
}

// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenterApp.AdminFacingInterface")]
pub struct AdminShowGatewayStatus {
    pub gateway_id: String,
    pub requested_by: String,
}

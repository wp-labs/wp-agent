// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.AdminFacingInterface")]
pub struct AdminCreateGatewayInstance {
    pub gateway_name: String,
    pub requested_by: String,
}

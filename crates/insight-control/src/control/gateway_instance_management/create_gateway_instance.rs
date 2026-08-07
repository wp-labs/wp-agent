// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.GatewayInstanceManagement")]
pub struct CreateGatewayInstance {
    pub gateway_name: String,
    pub requested_by: String,
    pub requested_at: crate::DateTime,
}

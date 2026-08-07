// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GatewaySupervision")]
pub struct GatewayInstance {
    pub gateway_id: String,
    pub instance_id: String,
    pub status: String,
    pub created_at: crate::DateTime,
}

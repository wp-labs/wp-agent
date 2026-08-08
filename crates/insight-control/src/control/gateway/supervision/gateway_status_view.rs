// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GatewayStatusView {
    pub gateway_id: String,
    pub instance_id: String,
    pub version: String,
    pub status: String,
    pub health: String,
    pub last_seen_at: crate::DateTime,
}

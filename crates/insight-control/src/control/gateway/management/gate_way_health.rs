// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.Gateway.Management")]
pub struct GateWayHealth {
    pub gateway_id: String,
    pub status: String,
    pub reported_at: crate::DateTime,
}

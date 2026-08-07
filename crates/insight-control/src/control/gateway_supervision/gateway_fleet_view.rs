// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GatewaySupervision")]
pub struct GatewayFleetView {
    pub gateway_count: i64,
    pub online_count: i64,
    pub degraded_count: i64,
    pub offline_count: i64,
    pub updated_at: crate::DateTime,
}

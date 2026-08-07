// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GateWayManagement")]
pub struct GateWayManagementStateView {
    pub gateway_id: String,
    pub instance_id: String,
    pub version: String,
    pub status: String,
    pub config: crate::GateWayControlConfig,
    pub health: crate::GateWayHealth,
    pub last_seen_at: crate::DateTime,
}

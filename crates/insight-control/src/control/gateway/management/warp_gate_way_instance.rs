// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Management")]
pub struct WarpGateWayInstance {
    #[jumo(unique)]
    pub gateway_id: String,
    pub instance_id: String,
    pub boot_id: String,
    pub version: String,
    pub started_at: crate::DateTime,
    pub last_seen_at: crate::DateTime,
}
